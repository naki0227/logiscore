use crate::audio::{decode_wav, ChannelModel};
use crate::error::LogiscoreError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ChannelMetrics {
    pub final_recoveries: usize,
    pub cases: usize,
    pub raw_symbol_accuracy_percent: f64,
    pub corrected_errors: usize,
}

pub(super) struct ChannelObservation {
    pub raw_packet: Option<Vec<u8>>,
    pub final_recovery: bool,
}

pub(super) fn measure(
    wav: &[u8],
    expected_packet: &[u8],
    mut observe: impl FnMut(&[f32]) -> ChannelObservation,
) -> Result<ChannelMetrics, LogiscoreError> {
    let audio = decode_wav(wav)?;
    let observations = models()
        .into_iter()
        .map(|model| {
            model
                .apply(audio.samples())
                .map(|samples| observe(&samples))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(summarize(expected_packet, observations))
}

fn summarize(
    expected_packet: &[u8],
    observations: impl IntoIterator<Item = ChannelObservation>,
) -> ChannelMetrics {
    let observations = observations.into_iter().collect::<Vec<_>>();
    let cases = observations.len();
    let total_symbols = expected_packet
        .len()
        .saturating_mul(8)
        .saturating_mul(cases);
    let mut final_recoveries = 0;
    let mut correct_symbols = 0;
    let mut corrected_errors = 0;

    for observation in observations {
        final_recoveries += usize::from(observation.final_recovery);
        if let Some(packet) = observation
            .raw_packet
            .filter(|packet| packet.len() == expected_packet.len())
        {
            let bit_errors = packet_bit_errors(expected_packet, &packet);
            correct_symbols += expected_packet.len() * 8 - bit_errors;
            if observation.final_recovery {
                corrected_errors += bit_errors;
            }
        }
    }

    let raw_symbol_accuracy_percent = if total_symbols == 0 {
        100.0
    } else {
        correct_symbols as f64 * 100.0 / total_symbols as f64
    };
    ChannelMetrics {
        final_recoveries,
        cases,
        raw_symbol_accuracy_percent,
        corrected_errors,
    }
}

pub(super) fn packet_bit_errors(expected: &[u8], actual: &[u8]) -> usize {
    expected
        .iter()
        .zip(actual)
        .map(|(expected, actual)| (expected ^ actual).count_ones() as usize)
        .sum()
}

fn models() -> [ChannelModel; 5] {
    [
        ChannelModel::default(),
        ChannelModel {
            noise_amplitude: 0.01,
            ..ChannelModel::default()
        },
        ChannelModel {
            noise_amplitude: 0.03,
            echo_delay_samples: 32,
            echo_decay: 0.12,
            ..ChannelModel::default()
        },
        ChannelModel {
            gain: 0.55,
            noise_amplitude: 0.06,
            clip_level: 0.35,
            ..ChannelModel::default()
        },
        ChannelModel {
            gain: 0.30,
            noise_amplitude: 0.08,
            clip_level: 0.25,
            leading_silence_samples: 173,
            echo_delay_samples: 48,
            echo_decay: 0.18,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_packet_bit_errors() {
        assert_eq!(packet_bit_errors(&[0b1010_1010], &[0b1010_0011]), 2);
        assert_eq!(packet_bit_errors(&[0xff, 0], &[0xff, 0]), 0);
    }

    #[test]
    fn summarizes_recovery_accuracy_and_corrected_errors() {
        let metrics = summarize(
            &[0],
            [
                ChannelObservation {
                    raw_packet: Some(vec![0]),
                    final_recovery: true,
                },
                ChannelObservation {
                    raw_packet: Some(vec![1]),
                    final_recovery: true,
                },
                ChannelObservation {
                    raw_packet: None,
                    final_recovery: false,
                },
            ],
        );
        assert_eq!(metrics.final_recoveries, 2);
        assert_eq!(metrics.cases, 3);
        assert_eq!(metrics.raw_symbol_accuracy_percent, 62.5);
        assert_eq!(metrics.corrected_errors, 1);
    }
}
