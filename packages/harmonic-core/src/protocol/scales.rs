/// 音階定義: 各要素は Root Key からの半音数。
///
/// 不変条件:
/// - 各配列の全16要素はユニーク（一意性制約）
/// - 各配列の要素は昇順
/// - Pitch Offset -8 ~ +7 がインデックス 0 ~ 15 に対応
pub const SCALES: [[u8; 16]; 8] = [
    // 0: Major Pentatonic (明るく開放的)
    [0, 2, 4, 7, 9, 12, 14, 16, 19, 21, 24, 26, 28, 31, 33, 36],
    // 1: Minor Pentatonic (哀愁のある静かな響き)
    [0, 3, 5, 7, 10, 12, 15, 17, 19, 22, 24, 27, 29, 31, 34, 36],
    // 2: Lydian Melody (明るく神秘的)
    [0, 2, 4, 6, 7, 9, 11, 12, 14, 16, 18, 19, 21, 23, 24, 26],
    // 3: Future Dorian (サイバー、浮遊感)
    [0, 2, 3, 5, 7, 9, 10, 12, 14, 15, 17, 19, 21, 22, 24, 26],
    // 4: Deep Ambient (深い響き - JSON/YAML)
    [0, 4, 7, 11, 12, 16, 19, 23, 24, 28, 31, 35, 36, 40, 43, 47],
    // 5: Melodic Minor (テクニカルで重厚 - C/C++)
    [0, 2, 3, 5, 7, 9, 11, 12, 14, 15, 17, 19, 21, 23, 24, 26],
    // 6: Mixolydian (安定感と躍動感 - Ruby/Go)
    [0, 2, 4, 5, 7, 9, 10, 12, 14, 16, 17, 19, 21, 22, 24, 26],
    // 7: Double Harmonic (情熱的でエキゾチック - CSS/Style)
    [0, 1, 4, 5, 7, 8, 11, 12, 13, 16, 17, 19, 20, 23, 24, 25],
];

/// 音階名称
pub const SCALE_NAMES: [&str; 8] = [
    "Major Pentatonic",
    "Minor Pentatonic",
    "Lydian Melody",
    "Future Dorian",
    "Deep Ambient",
    "Melodic Minor",
    "Mixolydian",
    "Double Harmonic",
];

/// コンパイル時に音階テーブルの一意性を検証する。
///
/// 重複が存在するとコンパイルエラーになる。
const fn assert_unique_scale(scale: &[u8; 16]) {
    let mut i = 0;
    while i < 16 {
        let mut j = i + 1;
        while j < 16 {
            assert!(scale[i] != scale[j], "Scale elements must be unique");
            j += 1;
        }
        i += 1;
    }
}

/// コンパイル時に全音階の一意性と昇順を検証する。
const fn assert_scales_valid() {
    let mut s = 0;
    while s < SCALES.len() {
        assert_unique_scale(&SCALES[s]);
        // 昇順チェック
        let mut i = 0;
        while i < 15 {
            assert!(
                SCALES[s][i] < SCALES[s][i + 1],
                "Scale elements must be in ascending order"
            );
            i += 1;
        }
        s += 1;
    }
}

// コンパイル時検証を実行
const _: () = assert_scales_valid();

/// MIDI Note Number の最大値を検証（127 以下であること）。
const fn assert_note_range_valid() {
    // 最大 Root Key = 11 (B)、各 Scale の最大値を確認
    let mut s = 0;
    while s < SCALES.len() {
        let max_offset = SCALES[s][15];
        let max_note = 11u8 + max_offset; // Root Key B + 最大オフセット
        assert!(
            max_note <= 127,
            "MIDI note number exceeds 127"
        );
        s += 1;
    }
}

const _: () = assert_note_range_valid();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_scales_have_16_elements() {
        for scale in &SCALES {
            assert_eq!(scale.len(), 16);
        }
    }

    #[test]
    fn all_elements_unique_per_scale() {
        for (i, scale) in SCALES.iter().enumerate() {
            let mut seen = std::collections::HashSet::new();
            for &val in scale {
                assert!(
                    seen.insert(val),
                    "Duplicate value {} in scale {}",
                    val,
                    SCALE_NAMES[i]
                );
            }
        }
    }

    #[test]
    fn all_elements_ascending() {
        for (i, scale) in SCALES.iter().enumerate() {
            for j in 0..15 {
                assert!(
                    scale[j] < scale[j + 1],
                    "Scale {} not ascending at index {}",
                    SCALE_NAMES[i],
                    j
                );
            }
        }
    }

    #[test]
    fn midi_note_within_range() {
        // コード進行の最大オフセットは 9 (vi)
        let max_prog_offset = 9;
        for scale in &SCALES {
            for root_key in 0u8..=11 {
                for &offset in scale {
                    let note = 48 + root_key + offset + max_prog_offset;
                    assert!(
                        note <= 127,
                        "Note {} exceeds 127 (root={}, offset={}, prog={})",
                        note,
                        root_key,
                        offset,
                        max_prog_offset
                    );
                }
            }
        }
    }
}
