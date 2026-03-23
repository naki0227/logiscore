use crate::error::LogiscoreError;

/// プロトコルヘッダー（MIDIメタイベントとして格納）
///
/// Header はデータストリームに混入させず、MIDIメタイベント（テキストイベント）
/// として格納する。マジックナンバー `LOGISCORE:v1` で識別する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    /// 使用する音階のインデックス (0–15)
    pub scale_id: u8,
    /// 基準音 (0: C, 1: C#, ..., 11: B)
    pub root_key: u8,
    /// 1拍あたりのバイト数 (動的密度)
    pub bytes_per_tick: u8,
}

impl Header {
    /// 新しい Header を作成する。
    ///
    /// # Errors
    /// `root_key > 11` の場合エラーを返す。
    pub fn new(scale_id: u8, root_key: u8, bytes_per_tick: u8) -> Result<Self, LogiscoreError> {
        if root_key > 11 {
            return Err(LogiscoreError::InvalidHeader(root_key));
        }
        if scale_id as usize >= crate::protocol::scales::SCALES.len() {
            return Err(LogiscoreError::InvalidHeader(scale_id));
        }
        Ok(Self { 
            scale_id, 
            root_key,
            bytes_per_tick,
        })
    }

    /// フルセットのメタイベント用文字列（グローバルヘッダー用）を生成する。
    pub fn to_full_meta_strings(&self) -> Vec<String> {
        vec![
            "LOGISCORE:v1.4".to_string(),
            format!("SCALE:{}", self.scale_id),
            format!("ROOT:{}", self.root_key),
            format!("BPT:{}", self.bytes_per_tick),
        ]
    }

    /// 最小限のメタイベント用文字列（データ長のみ）を生成する。
    pub fn to_minimal_meta_strings(&self, data_length: usize) -> Vec<String> {
        vec![
            format!("L:{}", data_length),
        ]
    }

    /// グローバルヘッダー用（データ長を要求しない）のパース処理
    pub fn from_global_meta_strings(texts: &[String]) -> Result<Self, LogiscoreError> {
        let has_magic = texts.iter().any(|t| t.starts_with("LOGISCORE:"));
        if !has_magic {
            return Err(LogiscoreError::InvalidMidi("Missing LOGISCORE magic for global header".into()));
        }

        let magic = texts.iter().find(|t| t.starts_with("LOGISCORE:")).unwrap();
        if !magic.starts_with("LOGISCORE:v1") {
            return Err(LogiscoreError::UnsupportedVersion(magic.clone()));
        }

        let scale_id = texts
            .iter()
            .find(|t| t.starts_with("SCALE:"))
            .and_then(|t| t[6..].parse::<u8>().ok())
            .ok_or_else(|| LogiscoreError::InvalidMidi("Missing or invalid SCALE".into()))?;

        let root_key = texts
            .iter()
            .find(|t| t.starts_with("ROOT:"))
            .and_then(|t| t[5..].parse::<u8>().ok())
            .ok_or_else(|| LogiscoreError::InvalidMidi("Missing or invalid ROOT".into()))?;

        let bytes_per_tick = texts
            .iter()
            .find(|t| t.starts_with("BPT:"))
            .and_then(|t| t[4..].parse::<u8>().ok())
            .unwrap_or(8);

        Ok(Self { scale_id, root_key, bytes_per_tick })
    }

    /// メタイベント文字列群からパースする。
    ///
    /// # Returns
    /// (Header, data_length) のタプル。
    ///
    /// # Errors
    /// マジックナンバー不在、バージョン不一致、フィールド欠損時にエラー。
    pub fn from_meta_strings(texts: &[String], default: Option<Header>) -> Result<(Self, usize), LogiscoreError> {
        let has_magic = texts.iter().any(|t| t.starts_with("LOGISCORE:"));
        let magic = texts.iter().find(|t| t.starts_with("LOGISCORE:"));

        if let Some(m) = magic {
            if !m.starts_with("LOGISCORE:v1") {
                return Err(LogiscoreError::UnsupportedVersion(m.clone()));
            }
        } else if default.is_none() {
            return Err(LogiscoreError::InvalidMidi("Missing LOGISCORE magic and no global header found".into()));
        }

        // 基本はデフォルト（あれば）、なければ新規パース（magicがある前提）
        let mut header = match default {
            Some(d) => d,
            None => Header { scale_id: 0, root_key: 0, bytes_per_tick: 8 },
        };

        if let Some(s) = texts.iter().find(|t| t.starts_with("SCALE:")) {
            header.scale_id = s[6..].parse::<u8>().unwrap_or(header.scale_id);
        }
        if let Some(s) = texts.iter().find(|t| t.starts_with("ROOT:")) {
            header.root_key = s[5..].parse::<u8>().unwrap_or(header.root_key);
        }
        if let Some(s) = texts.iter().find(|t| t.starts_with("BPT:")) {
            header.bytes_per_tick = s[4..].parse::<u8>().unwrap_or(header.bytes_per_tick);
        }

        // LEN: または L: からデータ長を取得
        let data_length = texts
            .iter()
            .find(|t| t.starts_with("LEN:") || t.starts_with("L:"))
            .and_then(|t| {
                if t.starts_with("LEN:") {
                    t[4..].parse::<usize>().ok()
                } else {
                    t[2..].parse::<usize>().ok()
                }
            })
            .ok_or_else(|| LogiscoreError::InvalidMidi("Missing or invalid LEN/L".into()))?;

        Ok((header, data_length))
    }
}

/// 1音 = 1バイトの表現。
///
/// 8bit のデータバイトを上位4bit (Pitch Offset) と下位4bit (Velocity) に分割し、
/// MIDIノートイベントにマッピングする。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HarmonicByte {
    /// Root からの相対音階移動 (-8 ~ +7, 2の補数)
    pub pitch_offset: i8,
    /// 音の強弱インデックス (0–15)
    pub velocity: u8,
}

impl HarmonicByte {
    /// 生のバイト値から HarmonicByte を構築する。
    pub fn from_byte(byte: u8) -> Self {
        let raw_pitch = ((byte >> 4) & 0x0F) as i8;
        let pitch_offset = if raw_pitch > 7 {
            raw_pitch - 16
        } else {
            raw_pitch
        };
        let velocity = byte & 0x0F;
        Self {
            pitch_offset,
            velocity,
        }
    }

    /// HarmonicByte を生のバイト値に戻す。
    pub fn to_byte(&self) -> u8 {
        let pitch_bits = (self.pitch_offset as u8) & 0x0F;
        (pitch_bits << 4) | (self.velocity & 0x0F)
    }

    /// コード進行のオフセットを取得する (I-V-vi-IV 進行)。
    /// 8 tick ごとに進行する。
    fn get_progression_offset(abs_tick: u64) -> u8 {
        let step = (abs_tick / 8) % 4;
        match step {
            0 => 0, // I
            1 => 7, // V
            2 => 9, // vi
            3 => 5, // IV
            _ => 0,
        }
    }

    /// MIDI Note Number を算出する。
    pub fn to_midi_note(&self, root_key: u8, scale: &[u8; 16], abs_tick: u64) -> u8 {
        let index = (self.pitch_offset + 8) as usize;
        let prog_offset = Self::get_progression_offset(abs_tick);
        // ベースオクターブ + Root + スケールオフセット + コード進行オフセット
        48 + root_key + scale[index] + prog_offset
    }

    /// MIDI Note Number から HarmonicByte の pitch_offset を逆算する。
    pub fn pitch_from_midi_note(
        note: u8,
        root_key: u8,
        scale: &[u8; 16],
        abs_tick: u64,
    ) -> Result<i8, LogiscoreError> {
        let prog_offset = Self::get_progression_offset(abs_tick);
        let base = 48u8.saturating_add(root_key).saturating_add(prog_offset);
        let relative = note.saturating_sub(base);
        
        // 完全一致を試みる
        if let Some(index) = scale.iter().position(|&n| n == relative) {
            return Ok((index as i8) - 8);
        }
        
        // フォールバック: 最も近い音を探す（堅牢性向上）
        let mut best_index = 0usize;
        let mut best_diff = u8::MAX;
        for (i, &n) in scale.iter().enumerate() {
            let diff = if relative >= n { relative - n } else { n - relative };
            if diff < best_diff {
                best_diff = diff;
                best_index = i;
            }
        }
        Ok((best_index as i8) - 8)
    }

    /// MIDI Velocity を算出する (0–127)。
    pub fn to_midi_velocity(&self) -> u8 {
        (self.velocity * 8) + 7
    }

    /// MIDI Velocity から Velocity Index を逆算する。
    pub fn velocity_from_midi(midi_velocity: u8) -> u8 {
        if midi_velocity < 7 {
            0
        } else {
            (midi_velocity - 7) / 8
        }
    }
}

pub mod midi_gen;
pub mod scales;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harmonic_byte_roundtrip_all_256() {
        for byte in 0u8..=255 {
            let hb = HarmonicByte::from_byte(byte);
            assert_eq!(
                hb.to_byte(),
                byte,
                "Roundtrip failed for byte 0x{:02X}",
                byte
            );
        }
    }

    #[test]
    fn pitch_offset_range() {
        for byte in 0u8..=255 {
            let hb = HarmonicByte::from_byte(byte);
            assert!(
                (-8..=7).contains(&hb.pitch_offset),
                "Pitch offset {} out of range for byte 0x{:02X}",
                hb.pitch_offset,
                byte
            );
        }
    }

    #[test]
    fn velocity_range() {
        for byte in 0u8..=255 {
            let hb = HarmonicByte::from_byte(byte);
            assert!(hb.velocity <= 15);
        }
    }

    #[test]
    fn midi_velocity_roundtrip() {
        for v in 0u8..=15 {
            let midi_vel = v * 8 + 7;
            let recovered = HarmonicByte::velocity_from_midi(midi_vel);
            assert_eq!(recovered, v, "Velocity roundtrip failed for {}", v);
        }
    }

    #[test]
    fn header_meta_roundtrip() {
        for scale_id in 0u8..5 {
            for root_key in 0u8..12 {
                let header = Header::new(scale_id, root_key, 8).unwrap();
                let meta = header.to_full_meta_strings();
                let mut full_meta = meta;
                full_meta.push("LEN:42".to_string());
                let (recovered, len) = Header::from_meta_strings(&full_meta, None).unwrap();
                assert_eq!(header, recovered);
                assert_eq!(len, 42);
            }
        }
    }

    #[test]
    fn header_rejects_invalid_root() {
        assert!(Header::new(0, 12, 8).is_err());
    }

    #[test]
    fn header_rejects_invalid_scale() {
        assert!(Header::new(8, 0, 8).is_err());
    }

    #[test]
    fn header_rejects_missing_magic() {
        let texts = vec!["SCALE:0".to_string(), "ROOT:0".to_string(), "LEN:10".to_string()];
        assert!(Header::from_meta_strings(&texts, None).is_err());
    }

    #[test]
    fn velocity_zero_is_valid_data() {
        // 0xN0 バイト群 (Velocity=0) が正しくエンコード/デコードされることを検証
        for high in 0u8..16 {
            let byte = high << 4; // 0x00, 0x10, 0x20, ..., 0xF0
            let hb = HarmonicByte::from_byte(byte);
            assert_eq!(hb.velocity, 0, "Byte 0x{:02X} should have velocity 0", byte);
            assert_eq!(hb.to_byte(), byte, "Roundtrip failed for 0x{:02X}", byte);
        }
    }
}
