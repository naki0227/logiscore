use super::MusicalError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Major,
    NaturalMinor,
    Dorian,
    Lydian,
}

impl Mode {
    pub(crate) const fn intervals(self) -> [u8; 7] {
        match self {
            Self::Major => [0, 2, 4, 5, 7, 9, 11],
            Self::NaturalMinor => [0, 2, 3, 5, 7, 8, 10],
            Self::Dorian => [0, 2, 3, 5, 7, 9, 10],
            Self::Lydian => [0, 2, 4, 6, 7, 9, 11],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TonalContext {
    root: u8,
    mode: Mode,
}

impl TonalContext {
    pub const C_MAJOR: Self = Self {
        root: 0,
        mode: Mode::Major,
    };

    pub fn new(root: u8, mode: Mode) -> Result<Self, MusicalError> {
        if root > 11 {
            return Err(MusicalError::InvalidRoot(root));
        }
        Ok(Self { root, mode })
    }

    pub const fn root(self) -> u8 {
        self.root
    }

    pub const fn mode(self) -> Mode {
        self.mode
    }

    pub fn scale_pitch_classes(self) -> [u8; 7] {
        self.mode
            .intervals()
            .map(|interval| (self.root + interval) % 12)
    }
}
