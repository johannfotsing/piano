use crate::Waveform;

/// An oscillator definition and its contribution to an instrument's sound.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OscillatorAssignment {
    waveform: Waveform,
    gain: f32,
}

impl OscillatorAssignment {
    pub fn new(waveform: Waveform, gain: f32) -> Self {
        assert!(gain.is_finite() && gain >= 0.0);
        Self { waveform, gain }
    }

    pub const fn waveform(&self) -> Waveform {
        self.waveform
    }

    pub const fn gain(&self) -> f32 {
        self.gain
    }
}

/// A synthesizer preset used when creating new voices.
#[derive(Debug, Clone, PartialEq)]
pub struct Instrument {
    name: &'static str,
    oscillators: Vec<OscillatorAssignment>,
}

impl Instrument {
    pub fn new(name: &'static str, oscillators: Vec<OscillatorAssignment>) -> Self {
        assert!(!oscillators.is_empty());
        Self { name, oscillators }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub fn oscillators(&self) -> &[OscillatorAssignment] {
        &self.oscillators
    }
}
