use crate::pitch::Pitch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Note {
    pub pitch: Pitch,
    pub octave: i8,
}

impl Note {
    pub const C4: Self = Self {
        pitch: Pitch::C,
        octave: 4,
    };
    pub const CSHARP4: Self = Self {
        pitch: Pitch::CSharp,
        octave: 4,
    };
    pub const D4: Self = Self {
        pitch: Pitch::D,
        octave: 4,
    };
    pub const DSHARP4: Self = Self {
        pitch: Pitch::DSharp,
        octave: 4,
    };
    pub const E4: Self = Self {
        pitch: Pitch::E,
        octave: 4,
    };
    pub const F4: Self = Self {
        pitch: Pitch::F,
        octave: 4,
    };
    pub const FSHARP4: Self = Self {
        pitch: Pitch::FSharp,
        octave: 4,
    };
    pub const G4: Self = Self {
        pitch: Pitch::G,
        octave: 4,
    };
    pub const GSHARP4: Self = Self {
        pitch: Pitch::GSharp,
        octave: 4,
    };
    pub const A4: Self = Self {
        pitch: Pitch::A,
        octave: 4,
    };
    pub const ASHARP4: Self = Self {
        pitch: Pitch::ASharp,
        octave: 4,
    };
    pub const B4: Self = Self {
        pitch: Pitch::B,
        octave: 4,
    };

    pub fn frequency(&self) -> f64 {
        const A4_FREQUENCY: f64 = 440.0;
        const REFERENCE_OCTAVE: i32 = 4;

        let semitone_offset = (self.octave as i32 - REFERENCE_OCTAVE) * 12
            + self.pitch.semitone() as i32
            - Pitch::A.semitone() as i32;

        A4_FREQUENCY * libm::pow(2.0, semitone_offset as f64 / 12.0)
    }

    pub fn midi_number(&self) -> u8 {
        ((self.octave as i32 + 1) * 12 + self.pitch.semitone() as i32) as u8
    }

    pub fn from_midi_number(midi_number: u8) -> Self {
        let octave = (midi_number as i32 / 12) - 1;
        let semitone = midi_number % 12;

        let pitch = match semitone {
            0 => Pitch::C,
            1 => Pitch::CSharp,
            2 => Pitch::D,
            3 => Pitch::DSharp,
            4 => Pitch::E,
            5 => Pitch::F,
            6 => Pitch::FSharp,
            7 => Pitch::G,
            8 => Pitch::GSharp,
            9 => Pitch::A,
            10 => Pitch::ASharp,
            11 => Pitch::B,
            _ => unreachable!(),
        };

        Self {
            pitch,
            octave: octave as i8,
        }
    }
}
