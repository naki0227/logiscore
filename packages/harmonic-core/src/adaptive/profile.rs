#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AcousticProfileId {
    Quiet = 1,
    Balanced = 2,
    Conversation = 3,
    Noisy = 4,
    Online = 5,
    LongDistance = 6,
    FixedFallback = 7,
}

impl TryFrom<u8> for AcousticProfileId {
    type Error = crate::error::LogiscoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Quiet),
            2 => Ok(Self::Balanced),
            3 => Ok(Self::Conversation),
            4 => Ok(Self::Noisy),
            5 => Ok(Self::Online),
            6 => Ok(Self::LongDistance),
            7 => Ok(Self::FixedFallback),
            _ => Err(crate::error::LogiscoreError::InvalidProfile(format!(
                "unsupported acoustic profile id: {value}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcousticProfile {
    pub id: AcousticProfileId,
    pub timing_percent: u16,
    pub max_polyphony: u8,
    pub fec_profile: u8,
    pub interleave_depth: u8,
    pub repetition: u8,
    pub music_weight: u8,
}

impl AcousticProfile {
    pub const fn for_id(id: AcousticProfileId) -> Self {
        match id {
            AcousticProfileId::Quiet => Self::new(id, 75, 4, 0, 0, 1, 85),
            AcousticProfileId::Balanced => Self::new(id, 100, 4, 1, 8, 3, 60),
            AcousticProfileId::Conversation => Self::new(id, 125, 4, 2, 16, 3, 45),
            AcousticProfileId::Noisy => Self::new(id, 150, 4, 3, 16, 5, 30),
            AcousticProfileId::Online => Self::new(id, 150, 4, 3, 16, 5, 25),
            AcousticProfileId::LongDistance => Self::new(id, 175, 4, 3, 16, 5, 20),
            AcousticProfileId::FixedFallback => Self::new(id, 200, 1, 3, 16, 5, 0),
        }
    }

    const fn new(
        id: AcousticProfileId,
        timing_percent: u16,
        max_polyphony: u8,
        fec_profile: u8,
        interleave_depth: u8,
        repetition: u8,
        music_weight: u8,
    ) -> Self {
        Self {
            id,
            timing_percent,
            max_polyphony,
            fec_profile,
            interleave_depth,
            repetition,
            music_weight,
        }
    }
}
