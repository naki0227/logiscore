use crate::audio::{decode_wav, ChannelModel};
use crate::error::LogiscoreError;

pub(super) fn successes(
    wav: &[u8],
    mut decode: impl FnMut(&[f32]) -> Result<bool, LogiscoreError>,
) -> Result<usize, LogiscoreError> {
    let audio = decode_wav(wav)?;
    models().into_iter().try_fold(0, |successes, model| {
        let samples = model.apply(audio.samples())?;
        Ok(successes + usize::from(decode(&samples).unwrap_or(false)))
    })
}

pub(super) fn case_count() -> usize {
    models().len()
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
