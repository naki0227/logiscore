use super::TonalContext;

const PROGRESSION: [usize; 4] = [0, 4, 5, 3];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chord {
    degree: u8,
    pitch_classes: [u8; 3],
}

impl Chord {
    pub const fn degree(self) -> u8 {
        self.degree
    }

    pub const fn pitch_classes(self) -> [u8; 3] {
        self.pitch_classes
    }

    pub fn contains(self, note: u8) -> bool {
        self.pitch_classes.contains(&(note % 12))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChordPlanner {
    context: TonalContext,
}

impl ChordPlanner {
    pub const fn new(context: TonalContext) -> Self {
        Self { context }
    }

    pub fn chord_at(self, event_index: usize) -> Chord {
        let degree = PROGRESSION[event_index % PROGRESSION.len()];
        let scale = self.context.scale_pitch_classes();
        Chord {
            degree: (degree + 1) as u8,
            pitch_classes: [
                scale[degree],
                scale[(degree + 2) % 7],
                scale[(degree + 4) % 7],
            ],
        }
    }
}
