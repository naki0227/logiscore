use crate::error::LogiscoreError;
use crate::protocol::{Header, HarmonicByte};
use crate::protocol::scales::SCALES;
use midly::{
    MetaMessage, MidiMessage, Smf,
    TrackEventKind,
};

/// 最小密度 (安全性のため)
const MIN_BYTES_PER_TICK: usize = 1;

#[derive(Clone, Copy)]
struct NoteEvent {
    abs_tick: u64,
    channel: u8,
    order: usize,
    note: u8,
    velocity: u8,
}

/// プロジェクト全体のファイル群を順次再生（メドレー形式）の MIDI (Type 0) としてエンコードする。
pub fn encode_project_to_midi(
    files: &[(String, Vec<u8>, Header)],
    global_header: &Header,
) -> Result<Vec<u8>, LogiscoreError> {
    let mut midi_binary: Vec<u8> = Vec::new();

    // 1. MThd (Type 0, Single Track)
    midi_binary.extend_from_slice(b"MThd");
    midi_binary.extend_from_slice(&6u32.to_be_bytes()); // length
    midi_binary.extend_from_slice(&0u16.to_be_bytes()); // format 0
    midi_binary.extend_from_slice(&1u16.to_be_bytes()); // num tracks
    midi_binary.extend_from_slice(&480u16.to_be_bytes()); // PPQ

    let mut track_data: Vec<u8> = Vec::new();

    // --- Global Track Setup ---
    // Tempo
    write_vlq(&mut track_data, 0);
    track_data.extend_from_slice(&[0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]);
    // Global Header
    let meta_strings = global_header.to_full_meta_strings();
    for text in &meta_strings {
        write_vlq(&mut track_data, 0);
        track_data.push(0xFF);
        track_data.push(0x01);
        let bytes = text.as_bytes();
        write_vlq(&mut track_data, bytes.len() as u32);
        track_data.extend_from_slice(bytes);
    }

    // ファイル数に応じて1ファイルあたりの目標時間を調整 (45s ~ 2.5s)
    let num_files = files.len() as u32;
    let target_total_ticks = if num_files <= 1 {
        43200 // 約45秒
    } else if num_files <= 10 {
        28800 // 約30秒
    } else if num_files <= 100 {
        14400 // 約15秒
    } else if num_files <= 500 {
        7200  // 約7.5秒
    } else {
        2400  // 約2.5秒
    };

    // --- Append each file sequentially ---
    for (i, (name, data, header)) in files.iter().enumerate() {
        let channel = (i % 15) as u8;
        let real_channel = if channel >= 9 { channel + 1 } else { channel };
        
        append_file_to_track(&mut track_data, name, data, header, real_channel, target_total_ticks)?;
    }

    // End of Track
    write_vlq(&mut track_data, 0);
    track_data.extend_from_slice(&[0xFF, 0x2F, 0x00]);

    // 2. MTrk
    midi_binary.extend_from_slice(b"MTrk");
    midi_binary.extend_from_slice(&(track_data.len() as u32).to_be_bytes());
    midi_binary.extend_from_slice(&track_data);

    Ok(midi_binary)
}

fn append_file_to_track(
    track_data: &mut Vec<u8>,
    name: &str,
    data: &[u8],
    header: &Header,
    channel: u8,
    target_total_ticks: u32,
) -> Result<(), LogiscoreError> {
    // Marker (Boundary)
    write_vlq(track_data, 0);
    track_data.push(0xFF);
    track_data.push(0x06); // Marker event
    let marker_text = format!("FILE:{}", name);
    let marker_bytes = marker_text.as_bytes();
    write_vlq(track_data, marker_bytes.len() as u32);
    track_data.extend_from_slice(marker_bytes);

    // Local Metadata (Minimal: Only length)
    let meta_strings = header.to_minimal_meta_strings(data.len());
    for text in &meta_strings {
        write_vlq(track_data, 0);
        track_data.push(0xFF);
        track_data.push(0x01);
        let bytes = text.as_bytes();
        write_vlq(track_data, bytes.len() as u32);
        track_data.extend_from_slice(bytes);
    }

    // Instrument (Logiscore Philharmonic Orchestra Mapping)
    let lower_name = name.to_lowercase();
    let ext = name.split('.').last().unwrap_or("").to_lowercase();
    
    let program = match ext.as_str() {
        "rs" | "cpp" | "c" | "h" | "hpp" => 48, // Strings Ensemble 1
        "py" | "rb" | "dart" | "swift" => 42, // Cello
        "java" | "kt" | "kts" => 45, // Pizzicato Strings
        "go" => 60, // French Horn
        "sh" | "bash" | "zsh" => 56, // Trumpet
        "ts" | "js" | "tsx" | "jsx" => 71, // Clarinet
        "css" | "scss" | "sass" | "less" | "html" | "svg" => 73, // Flute
        "md" | "json" | "yaml" | "yml" | "toml" | "xml" | "txt" | "env" => 46, // Harp
        "sql" => 47, // Timpani
        _ => {
            // Check for build/infra files (Tubular Bells)
            if lower_name.contains("dockerfile") || 
               lower_name.contains("makefile") || 
               lower_name.contains("gemfile") || 
               lower_name.contains("cargo.toml") || 
               lower_name.contains("package.json") ||
               lower_name.contains("go.mod") {
                14 // Tubular Bells
            } else {
                0 // Grand Piano
            }
        }
    };
    write_vlq(track_data, 0);
    track_data.push(0xC0 | (channel & 0x0F));
    track_data.push(program & 0x7F);

    // Notes
    let scale = &SCALES[header.scale_id as usize];
    let bpt = header.bytes_per_tick as usize;
    let num_ticks = if data.is_empty() { 0 } else { data.len().div_ceil(bpt) };
    
    let delta_per_tick = if num_ticks > 0 {
        (target_total_ticks / num_ticks as u32).clamp(4, 127)
    } else {
        127
    };
    let note_duration = (delta_per_tick as f32 * 0.8) as u32;
    let rest_duration = delta_per_tick - note_duration;

    let mut data_idx = 0;
    let mut last_status: u8 = 0;
    for tick in 0..num_ticks {
        // NoteOn
        for note_in_tick in 0..bpt {
            let byte = if data_idx < data.len() { data[data_idx] } else { 0x00 };
            data_idx += 1;
            let hb = HarmonicByte::from_byte(byte);
            let note = hb.to_midi_note(header.root_key, scale, tick as u64);
            let velocity = hb.to_midi_velocity().max(1); // 0は NoteOff 用なので最低 1 を確保

            let delta = if note_in_tick == 0 {
                if tick == 0 { 0 } else { rest_duration }
            } else {
                0
            };
            write_vlq(track_data, delta);
            
            // Running Status: 0x90 | channel (Note On)
            let status = 0x90 | (channel & 0x0F);
            if status != last_status {
                track_data.push(status);
                last_status = status;
            }
            track_data.push(note & 0x7F);
            track_data.push(velocity & 0x7F);
        }
        // NoteOff (Instead of 0x80, use 0x90 with Vel=0 to keep Running Status)
        for note_in_tick in 0..bpt {
            let delta = if note_in_tick == 0 { note_duration } else { 0 };
            
            let idx = (data_idx - bpt) + note_in_tick;
            let byte = if idx < data.len() { data[idx] } else { 0x00 };
            let hb = HarmonicByte::from_byte(byte);
            let note = hb.to_midi_note(header.root_key, scale, tick as u64);

            write_vlq(track_data, delta);
            
            // Status remains 0x90. If last_status is already 0x90 | channel, we PUSH NOTHING.
            let status = 0x90 | (channel & 0x0F);
            if status != last_status {
                track_data.push(status);
                last_status = status;
            }
            track_data.push(note & 0x7F);
            track_data.push(0x00); // Velocity 0 = Note Off
        }
    }

    Ok(())
}
/// 圧縮済みバイト列から MIDI バイナリを生成する。
pub fn encode_to_midi(data: &[u8], header: &Header) -> Result<Vec<u8>, LogiscoreError> {
    let scale = &SCALES[header.scale_id as usize];
    build_midi_binary(data, header, scale)
}

/// MIDI バイナリを直接構築する。
fn build_midi_binary(data: &[u8], header: &Header, scale: &[u8; 16]) -> Result<Vec<u8>, LogiscoreError> {
    let mut track_data: Vec<u8> = Vec::new();

    // Track Name
    write_vlq(&mut track_data, 0);
    track_data.push(0xFF);
    track_data.push(0x03);
    let name_bytes = "Logiscore".as_bytes();
    write_vlq(&mut track_data, name_bytes.len() as u32);
    track_data.extend_from_slice(name_bytes);

    // Tempo
    write_vlq(&mut track_data, 0);
    track_data.extend_from_slice(&[0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]);

    // Metadata
    let mut meta_strings = header.to_full_meta_strings();
    meta_strings.extend(header.to_minimal_meta_strings(data.len()));
    
    for text in &meta_strings {
        write_vlq(&mut track_data, 0);
        track_data.push(0xFF);
        track_data.push(0x01);
        let bytes = text.as_bytes();
        write_vlq(&mut track_data, bytes.len() as u32);
        track_data.extend_from_slice(bytes);
    }

    // Instrument
    write_vlq(&mut track_data, 0);
    track_data.push(0xC0); // Channel 0
    track_data.push(0);    // Grand Piano

    // Encoding notes
    let bpt = header.bytes_per_tick as usize;
    let num_ticks = if data.is_empty() { 0 } else { data.len().div_ceil(bpt) };
    
    let target_total_ticks = 43200u32;
    let delta_per_tick = if num_ticks > 0 {
        (target_total_ticks / num_ticks as u32).clamp(48, 480)
    } else {
        480
    };
    let note_duration = (delta_per_tick as f32 * 0.8) as u32;
    let rest_duration = delta_per_tick - note_duration;

    let mut data_idx = 0;
    let mut last_status: u8 = 0;
    for tick in 0..num_ticks {
        // NoteOn
        for note_in_tick in 0..bpt {
            let byte = if data_idx < data.len() { data[data_idx] } else { 0x00 };
            data_idx += 1;
            let hb = HarmonicByte::from_byte(byte);
            let note = hb.to_midi_note(header.root_key, scale, tick as u64);
            let velocity = hb.to_midi_velocity();

            let delta = if note_in_tick == 0 { if tick == 0 { 0 } else { rest_duration } } else { 0 };
            write_vlq(&mut track_data, delta);
            
            // Running Status: 0x90
            let status = 0x90;
            if status != last_status {
                track_data.push(status);
                last_status = status;
            }
            track_data.push(note & 0x7F);
            track_data.push(velocity & 0x7F);
        }
        // NoteOff
        let off_start = tick * bpt;
        for note_in_tick in 0..bpt {
            let idx = off_start + note_in_tick;
            let byte = if idx < data.len() { data[idx] } else { 0x00 };
            let hb = HarmonicByte::from_byte(byte);
            let note = hb.to_midi_note(header.root_key, scale, tick as u64);

            let delta = if note_in_tick == 0 { note_duration } else { 0 };
            write_vlq(&mut track_data, delta);
            
            // Running Status: 0x80
            let status = 0x80;
            if status != last_status {
                track_data.push(status);
                last_status = status;
            }
            track_data.push(note & 0x7F);
            track_data.push(0x00);
        }
    }

    // End of Track
    write_vlq(&mut track_data, 0);
    track_data.extend_from_slice(&[0xFF, 0x2F, 0x00]);

    // SMF Header
    let mut midi_binary: Vec<u8> = Vec::new();
    midi_binary.extend_from_slice(b"MThd");
    midi_binary.extend_from_slice(&6u32.to_be_bytes());
    midi_binary.extend_from_slice(&0u16.to_be_bytes()); // Format 0
    midi_binary.extend_from_slice(&1u16.to_be_bytes()); // 1 track
    midi_binary.extend_from_slice(&480u16.to_be_bytes()); // PPQ

    midi_binary.extend_from_slice(b"MTrk");
    midi_binary.extend_from_slice(&(track_data.len() as u32).to_be_bytes());
    midi_binary.extend_from_slice(&track_data);

    Ok(midi_binary)
}

/// MIDI バイナリからデータバイト列を復元する。
pub fn decode_from_midi(midi_bytes: &[u8]) -> Result<(Header, Vec<u8>), LogiscoreError> {
    let smf = Smf::parse(midi_bytes)
        .map_err(|e| LogiscoreError::MidiParseError(e.to_string()))?;
    let track = smf.tracks.first().ok_or_else(|| LogiscoreError::InvalidMidi("No tracks found".into()))?;
    
    let mut meta_texts = Vec::new();
    for event in track {
        if let TrackEventKind::Meta(MetaMessage::Text(text_bytes)) = event.kind {
            if let Ok(text) = std::str::from_utf8(text_bytes) {
                meta_texts.push(text.to_string());
            }
        }
    }

    let mut notes = Vec::new();
    let mut abs_tick = 0;
    let mut last_tick = 0;
    let mut order_counter = 0;
    for event in track {
        abs_tick += event.delta.as_int() as u64;
        if let TrackEventKind::Midi { message: MidiMessage::NoteOn { key, vel }, channel } = event.kind {
            if vel.as_int() > 0 {
                if abs_tick != last_tick {
                    order_counter = 0;
                    last_tick = abs_tick;
                }
                notes.push(NoteEvent {
                    abs_tick,
                    channel: channel.as_int(),
                    order: order_counter,
                    note: key.as_int(),
                    velocity: vel.as_int(),
                });
                order_counter += 1;
            }
        }
    }

    decode_notes_to_data(&meta_texts, &notes, None)
}

/// プロジェクト全体の MIDI (Sequential Marker 入) から各ファイルを復元する。
pub fn decode_project_from_midi(midi_bytes: &[u8]) -> Result<Vec<(String, Header, Vec<u8>)>, LogiscoreError> {
    let smf = Smf::parse(midi_bytes)
        .map_err(|e| LogiscoreError::MidiParseError(e.to_string()))?;

    let track = smf.tracks.first().ok_or_else(|| LogiscoreError::InvalidMidi("No tracks found".into()))?;

    let mut projects: Vec<(String, Header, Vec<u8>)> = Vec::new();
    let mut current_file_name: Option<String> = None;
    let mut current_meta_texts: Vec<String> = Vec::new();
    
    let mut current_notes: Vec<NoteEvent> = Vec::new();
    let mut abs_tick: u64 = 0;
    let mut order_counter: usize = 0;
    let mut last_tick: u64 = 0;

    let mut global_header: Option<Header> = None;

    for event in track {
        abs_tick += event.delta.as_int() as u64;

        match event.kind {
            TrackEventKind::Meta(MetaMessage::Marker(marker_bytes)) => {
                // ファイルの切り替わり
                if let Some(name) = current_file_name.take() {
                    // 前のファイルを処理
                    let (header, data) = decode_notes_to_data(&current_meta_texts, &current_notes, global_header)?;
                    projects.push((name, header, data));
                }
                // 初期化
                if let Ok(marker_text) = std::str::from_utf8(marker_bytes) {
                    if marker_text.starts_with("FILE:") {
                        current_file_name = Some(marker_text[5..].to_string());
                    }
                }
                current_meta_texts.clear();
                current_notes.clear();
                order_counter = 0;
            }
            TrackEventKind::Meta(MetaMessage::Text(text_bytes)) => {
                if let Ok(text) = std::str::from_utf8(text_bytes) {
                    current_meta_texts.push(text.to_string());
                    // 最初のヘッダーをグローバル設定として記録
                    if global_header.is_none() && text.starts_with("LOGISCORE:") {
                        if let Ok((h, _)) = Header::from_meta_strings(&current_meta_texts, None) {
                            global_header = Some(h);
                        }
                    }
                }
            }
            TrackEventKind::Midi { channel, message: MidiMessage::NoteOn { key, vel } } => {
                if vel.as_int() > 0 {
                    if abs_tick != last_tick {
                        order_counter = 0;
                        last_tick = abs_tick;
                    }
                    current_notes.push(NoteEvent {
                        abs_tick,
                        channel: channel.as_int(),
                        order: order_counter,
                        note: key.as_int(),
                        velocity: vel.as_int(),
                    });
                    order_counter += 1;
                }
            }
            _ => {}
        }
    }

    // 最後のファイルを処理
    if let Some(name) = current_file_name {
        let (header, data) = decode_notes_to_data(&current_meta_texts, &current_notes, global_header)?;
        projects.push((name, header, data));
    }

    Ok(projects)
}

fn decode_notes_to_data(meta_texts: &[String], notes: &[NoteEvent], default: Option<Header>) -> Result<(Header, Vec<u8>), LogiscoreError> {
    let (header, data_length) = Header::from_meta_strings(meta_texts, default)?;
    let scale = &SCALES[header.scale_id as usize];
    
    // ソート (絶対時間 + チャンネル + 出現順)
    let mut sorted_notes = notes.to_vec();
    sorted_notes.sort_by(|a, b| {
        a.abs_tick.cmp(&b.abs_tick)
            .then(a.channel.cmp(&b.channel))
            .then(a.order.cmp(&b.order))
    });

    let bpt = header.bytes_per_tick as usize;
    let mut restored_bytes: Vec<u8> = Vec::new();
    
    for (index, event) in sorted_notes.iter().enumerate() {
        let logical_tick = (index / bpt) as u64;
        let pitch_offset = HarmonicByte::pitch_from_midi_note(event.note, header.root_key, scale, logical_tick)?;
        let velocity_idx = HarmonicByte::velocity_from_midi(event.velocity);
        let hb = HarmonicByte { pitch_offset, velocity: velocity_idx };
        restored_bytes.push(hb.to_byte());
    }

    if restored_bytes.len() < data_length {
        return Err(LogiscoreError::InvalidMidi(format!("Truncated data: expected {}, got {}", data_length, restored_bytes.len())));
    }
    restored_bytes.truncate(data_length);
    Ok((header, restored_bytes))
}

/// Variable-Length Quantity (VLQ) エンコード
fn write_vlq(buf: &mut Vec<u8>, mut value: u32) {
    if value == 0 {
        buf.push(0);
        return;
    }

    let mut bytes = Vec::new();
    while value > 0 {
        bytes.push((value & 0x7F) as u8);
        value >>= 7;
    }
    bytes.reverse();
    for (i, b) in bytes.iter().enumerate() {
        if i < bytes.len() - 1 {
            buf.push(b | 0x80);
        } else {
            buf.push(*b);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vlq_zero() {
        let mut buf = Vec::new();
        write_vlq(&mut buf, 0);
        assert_eq!(buf, vec![0x00]);
    }

    #[test]
    fn vlq_small() {
        let mut buf = Vec::new();
        write_vlq(&mut buf, 127);
        assert_eq!(buf, vec![0x7F]);
    }

    #[test]
    fn vlq_two_bytes() {
        let mut buf = Vec::new();
        write_vlq(&mut buf, 128);
        assert_eq!(buf, vec![0x81, 0x00]);
    }

    #[test]
    fn encode_decode_single_byte() {
        let header = Header::new(0, 0, 8).unwrap();
        let data = vec![0x42];
        let midi = encode_to_midi(&data, &header).unwrap();
        let (dec_header, dec_data) = decode_from_midi(&midi).unwrap();
        assert_eq!(header, dec_header);
        assert_eq!(data, dec_data);
    }

    #[test]
    fn encode_decode_48_bytes() {
        let header = Header::new(2, 0, 8).unwrap();
        let data: Vec<u8> = (0..48).collect();
        let midi = encode_to_midi(&data, &header).unwrap();
        let (_, dec_data) = decode_from_midi(&midi).unwrap();
        assert_eq!(data, dec_data);
    }

    #[test]
    fn encode_decode_49_bytes() {
        // 48 + 1: パディングが正しく除去されるか
        let header = Header::new(2, 0, 8).unwrap();
        let data: Vec<u8> = (0..49).map(|i| i as u8).collect();
        let midi = encode_to_midi(&data, &header).unwrap();
        let (_, dec_data) = decode_from_midi(&midi).unwrap();
        assert_eq!(data, dec_data);
    }

    #[test]
    fn velocity_zero_bytes_survive() {
        // 0x00, 0x10, 0x20, ..., 0xF0 が消失しないことを検証
        let header = Header::new(0, 0, 8).unwrap();
        let data: Vec<u8> = (0..16).map(|i| i << 4).collect(); // Velocity=0 のバイト群
        let midi = encode_to_midi(&data, &header).unwrap();
        let (_, dec_data) = decode_from_midi(&midi).unwrap();
        assert_eq!(data, dec_data, "Velocity=0 bytes must survive roundtrip");
    }

    #[test]
    fn encode_decode_all_byte_values() {
        let header = Header::new(2, 0, 8).unwrap();
        let data: Vec<u8> = (0..=255).collect();
        let midi = encode_to_midi(&data, &header).unwrap();
        let (_, dec_data) = decode_from_midi(&midi).unwrap();
        assert_eq!(data, dec_data, "All 256 byte values must survive roundtrip");
    }

    #[test]
    fn encode_decode_empty() {
        let header = Header::new(0, 0, 8).unwrap();
        let data: Vec<u8> = vec![];
        let midi = encode_to_midi(&data, &header).unwrap();
        let (_, dec_data) = decode_from_midi(&midi).unwrap();
        assert_eq!(data, dec_data);
    }

    #[test]
    fn encode_decode_all_scales() {
        let data: Vec<u8> = (0..100).map(|i| i as u8).collect();
        for scale_id in 0u8..5 {
            for root_key in [0u8, 5, 11] {
                let header = Header::new(scale_id, root_key, 8).unwrap();
                let midi = encode_to_midi(&data, &header).unwrap();
                let (dec_header, dec_data) = decode_from_midi(&midi).unwrap();
                assert_eq!(header, dec_header, "Header mismatch for scale={}, root={}", scale_id, root_key);
                assert_eq!(data, dec_data, "Data mismatch for scale={}, root={}", scale_id, root_key);
            }
        }
    }

    #[test]
    fn midi_starts_with_mthd() {
        let header = Header::new(0, 0, 8).unwrap();
        let midi = encode_to_midi(&[0x42], &header).unwrap();
        assert_eq!(&midi[0..4], b"MThd");
    }

    #[test]
    fn reject_non_logiscore_midi() {
        // 有効な MIDI だが LOGISCORE メタイベントがないものを拒否する
        // 最小の有効 MIDI: MThd + MTrk with End of Track
        let mut midi = Vec::new();
        midi.extend_from_slice(b"MThd");
        midi.extend_from_slice(&6u32.to_be_bytes());
        midi.extend_from_slice(&0u16.to_be_bytes());
        midi.extend_from_slice(&1u16.to_be_bytes());
        midi.extend_from_slice(&480u16.to_be_bytes());
        midi.extend_from_slice(b"MTrk");
        let track_data = [0x00, 0xFF, 0x2F, 0x00]; // End of Track
        midi.extend_from_slice(&(track_data.len() as u32).to_be_bytes());
        midi.extend_from_slice(&track_data);

        let result = decode_from_midi(&midi);
        assert!(result.is_err(), "Should reject MIDI without LOGISCORE magic");
    }
}
