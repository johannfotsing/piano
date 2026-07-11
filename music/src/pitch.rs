#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pitch {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

impl Pitch {
    pub fn semitone(self) -> u8 {
        match self {
            Self::C => 0,
            Self::CSharp => 1,
            Self::D => 2,
            Self::DSharp => 3,
            Self::E => 4,
            Self::F => 5,
            Self::FSharp => 6,
            Self::G => 7,
            Self::GSharp => 8,
            Self::A => 9,
            Self::ASharp => 10,
            Self::B => 11,
        }
    }

    pub fn from_semitone(semitone: u8) -> Option<Self> {
        match semitone {
            0 => Some(Self::C),
            1 => Some(Self::CSharp),
            2 => Some(Self::D),
            3 => Some(Self::DSharp),
            4 => Some(Self::E),
            5 => Some(Self::F),
            6 => Some(Self::FSharp),
            7 => Some(Self::G),
            8 => Some(Self::GSharp),
            9 => Some(Self::A),
            10 => Some(Self::ASharp),
            11 => Some(Self::B),
            _ => None,
        }
    }
}
