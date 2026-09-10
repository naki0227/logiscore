use std::collections::BTreeMap;

use midly::{Format, MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};

use crate::error::LogiscoreError;
use crate::musical::{RhythmicEvent, RhythmicMusicalCodec, TonalContext, Voicing};
use crate::transport::Transport;

const TRACK_NAME: &[u8] = b"Logiscore v2 Rhythmic";
const TICKS_PER_BEAT: u16 = 480;
const REST_DURATION: u32 = 120;
const VELOCITY: u8 = 80;
const PROGRAMS: [u8; 4] = [32, 42, 41, 40];
const MAX_PACKET_BYTES: usize = 64 * 1024;
const MAX_EVENTS: usize = MAX_PACKET_BYTES * 2;

#[derive(Debug, Clone, Copy)]
pub struct RhythmicMidiTransport {
    codec: RhythmicMusicalCodec,
}

impl Default for RhythmicMidiTransport {
    fn default() -> Self {
        Self {
            codec: RhythmicMusicalCodec::new(TonalContext::C_MAJOR),
        }
    }
}

impl Transport for RhythmicMidiTransport {
    fn encode(&self, packet: &[u8]) -> Result<Vec<u8>, LogiscoreError> {
        if packet.len() > MAX_PACKET_BYTES {
            return Err(LogiscoreError::InvalidPacket(format!(
                "Rhythmic MIDI packet exceeds {MAX_PACKET_BYTES} bytes"
            )));
        }
        let events = self.codec.encode(packet).map_err(musical_error)?;
        build_midi(&events, packet.len())
    }

    fn decode(&self, transport_data: &[u8]) -> Result<Vec<u8>, LogiscoreError> {
        let (events, packet_length) = parse_midi(transport_data)?;
        self.codec
            .decode(&events, packet_length)
            .map_err(musical_error)
    }
}

fn build_midi(events: &[RhythmicEvent], packet_length: usize) -> Result<Vec<u8>, LogiscoreError> {
    let mut track = Vec::new();
    write_meta(&mut track, 0, 0x03, TRACK_NAME);
    write_meta(&mut track, 0, 0x01, format!("L:{packet_length}").as_bytes());
    write_meta(&mut track, 0, 0x51, &[0x07, 0xA1, 0x20]);
    for (channel, program) in PROGRAMS.into_iter().enumerate() {
        write_vlq(&mut track, 0);
        track.extend_from_slice(&[0xC0 | channel as u8, program]);
    }
    for (event_index, event) in events.iter().enumerate() {
        for (voice, note) in event.voicing().notes().into_iter().enumerate() {
            write_vlq(
                &mut track,
                if voice == 0 && event_index > 0 {
                    REST_DURATION
                } else {
                    0
                },
            );
            track.extend_from_slice(&[0x90 | voice as u8, note, VELOCITY]);
        }
        for (voice, note) in event.voicing().notes().into_iter().enumerate() {
            write_vlq(
                &mut track,
                if voice == 0 {
                    u32::from(event.duration_ticks())
                } else {
                    0
                },
            );
            track.extend_from_slice(&[0x80 | voice as u8, note, 0]);
        }
    }
    write_meta(&mut track, 0, 0x2f, &[]);
    let track_length = u32::try_from(track.len())
        .map_err(|_| LogiscoreError::InvalidMidi("rhythmic track exceeds u32 length".into()))?;
    let mut midi = Vec::with_capacity(22 + track.len());
    midi.extend_from_slice(b"MThd");
    midi.extend_from_slice(&6u32.to_be_bytes());
    midi.extend_from_slice(&0u16.to_be_bytes());
    midi.extend_from_slice(&1u16.to_be_bytes());
    midi.extend_from_slice(&TICKS_PER_BEAT.to_be_bytes());
    midi.extend_from_slice(b"MTrk");
    midi.extend_from_slice(&track_length.to_be_bytes());
    midi.extend_from_slice(&track);
    Ok(midi)
}

#[derive(Default)]
struct ParsedChord {
    notes: [Option<u8>; 4],
    durations: [Option<u16>; 4],
}

fn parse_midi(bytes: &[u8]) -> Result<(Vec<RhythmicEvent>, usize), LogiscoreError> {
    let smf =
        Smf::parse(bytes).map_err(|error| LogiscoreError::MidiParseError(error.to_string()))?;
    validate_header(&smf)?;
    let mut found_marker = false;
    let mut packet_length = None;
    let mut tick = 0u64;
    let mut active = [None::<(u64, u8)>; 4];
    let mut chords = BTreeMap::<u64, ParsedChord>::new();
    for event in &smf.tracks[0] {
        tick = tick
            .checked_add(u64::from(event.delta.as_int()))
            .ok_or_else(|| LogiscoreError::InvalidMidi("absolute tick overflow".into()))?;
        match event.kind {
            TrackEventKind::Meta(MetaMessage::TrackName(name)) if name == TRACK_NAME => {
                found_marker = true;
            }
            TrackEventKind::Meta(MetaMessage::Text(text)) if text.starts_with(b"L:") => {
                packet_length = Some(parse_packet_length(&text[2..])?);
            }
            TrackEventKind::Midi {
                channel,
                message: MidiMessage::NoteOn { key, vel },
            } if vel.as_int() > 0 => {
                let voice = checked_voice(channel.as_int())?;
                if chords.len() >= MAX_EVENTS && !chords.contains_key(&tick) {
                    return Err(LogiscoreError::InvalidMidi(
                        "rhythmic MIDI exceeds the event limit".into(),
                    ));
                }
                if active[voice].replace((tick, key.as_int())).is_some() {
                    return Err(LogiscoreError::InvalidMidi(
                        "overlapping voice event".into(),
                    ));
                }
                let chord = chords.entry(tick).or_default();
                if chord.notes[voice].replace(key.as_int()).is_some() {
                    return Err(LogiscoreError::InvalidMidi("duplicate voice event".into()));
                }
            }
            TrackEventKind::Midi { channel, message }
                if matches!(
                    message,
                    MidiMessage::NoteOff { .. } | MidiMessage::NoteOn { .. }
                ) =>
            {
                let voice = checked_voice(channel.as_int())?;
                let (key, is_off) = match message {
                    MidiMessage::NoteOff { key, .. } => (key.as_int(), true),
                    MidiMessage::NoteOn { key, vel } => (key.as_int(), vel.as_int() == 0),
                    _ => (0, false),
                };
                if is_off {
                    finish_note(&mut chords, &mut active, voice, key, tick)?;
                }
            }
            _ => {}
        }
    }
    if !found_marker {
        return Err(LogiscoreError::InvalidMidi(
            "missing Logiscore v2 Rhythmic marker".into(),
        ));
    }
    if active.iter().any(Option::is_some) {
        return Err(LogiscoreError::InvalidMidi(
            "unterminated voice event".into(),
        ));
    }
    let length = packet_length
        .ok_or_else(|| LogiscoreError::InvalidMidi("missing packet length metadata".into()))?;
    let events = chords
        .into_values()
        .map(parsed_event)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((events, length))
}

fn validate_header(smf: &Smf<'_>) -> Result<(), LogiscoreError> {
    if smf.header.format != Format::SingleTrack || smf.tracks.len() != 1 {
        return Err(LogiscoreError::InvalidMidi(
            "rhythmic MIDI must contain exactly one track".into(),
        ));
    }
    if !matches!(smf.header.timing, Timing::Metrical(value) if value.as_int() == TICKS_PER_BEAT) {
        return Err(LogiscoreError::InvalidMidi(
            "rhythmic MIDI must use 480 ticks per beat".into(),
        ));
    }
    Ok(())
}

fn parse_packet_length(bytes: &[u8]) -> Result<usize, LogiscoreError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| LogiscoreError::InvalidMidi("invalid packet length metadata".into()))?;
    let length = text
        .parse::<usize>()
        .map_err(|_| LogiscoreError::InvalidMidi("invalid packet length metadata".into()))?;
    if length > MAX_PACKET_BYTES {
        return Err(LogiscoreError::InvalidMidi(
            "packet length metadata exceeds limit".into(),
        ));
    }
    Ok(length)
}

fn finish_note(
    chords: &mut BTreeMap<u64, ParsedChord>,
    active: &mut [Option<(u64, u8)>; 4],
    voice: usize,
    key: u8,
    end_tick: u64,
) -> Result<(), LogiscoreError> {
    let (start_tick, active_key) = active[voice]
        .take()
        .ok_or_else(|| LogiscoreError::InvalidMidi("note off without note on".into()))?;
    if key != active_key {
        return Err(LogiscoreError::InvalidMidi("note off key mismatch".into()));
    }
    let duration = end_tick
        .checked_sub(start_tick)
        .and_then(|value| u16::try_from(value).ok())
        .ok_or_else(|| LogiscoreError::InvalidMidi("invalid note duration".into()))?;
    chords
        .get_mut(&start_tick)
        .ok_or_else(|| LogiscoreError::InvalidMidi("missing chord start".into()))?
        .durations[voice] = Some(duration);
    Ok(())
}

fn parsed_event(chord: ParsedChord) -> Result<RhythmicEvent, LogiscoreError> {
    let notes = collect_four(chord.notes, "incomplete four-voice event")?;
    let durations = collect_four(chord.durations, "incomplete note duration")?;
    if !durations.iter().all(|duration| *duration == durations[0]) {
        return Err(LogiscoreError::InvalidMidi(
            "voice durations do not match".into(),
        ));
    }
    Ok(RhythmicEvent::from_parts(
        Voicing::from_notes(notes),
        durations[0],
    ))
}

fn collect_four<T: Copy>(values: [Option<T>; 4], message: &str) -> Result<[T; 4], LogiscoreError> {
    values
        .map(|value| value.ok_or_else(|| LogiscoreError::InvalidMidi(message.into())))
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| LogiscoreError::InvalidMidi(message.into()))
}

fn checked_voice(channel: u8) -> Result<usize, LogiscoreError> {
    let voice = usize::from(channel);
    if voice >= 4 {
        return Err(LogiscoreError::InvalidMidi(
            "rhythmic note uses an unsupported channel".into(),
        ));
    }
    Ok(voice)
}

fn write_meta(track: &mut Vec<u8>, delta: u32, kind: u8, data: &[u8]) {
    write_vlq(track, delta);
    track.extend_from_slice(&[0xff, kind]);
    write_vlq(track, data.len() as u32);
    track.extend_from_slice(data);
}

fn write_vlq(output: &mut Vec<u8>, mut value: u32) {
    let mut buffer = [0u8; 4];
    let mut index = 3;
    buffer[index] = (value & 0x7f) as u8;
    while {
        value >>= 7;
        value > 0
    } {
        index -= 1;
        buffer[index] = ((value & 0x7f) as u8) | 0x80;
    }
    output.extend_from_slice(&buffer[index..]);
}

fn musical_error(error: crate::musical::MusicalError) -> LogiscoreError {
    LogiscoreError::InvalidMidi(error.to_string())
}

#[cfg(test)]
mod tests;
