use crate::Waveform;

/// An oscillator definition and its contribution to an instrument's sound.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OscillatorAssignment {
    waveform: Waveform,
    gain: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vibrato {
    rate_hz: f32,
    depth_cents: f32,
}

impl Vibrato {
    pub fn new(rate_hz: f32, depth_cents: f32) -> Self {
        assert!(rate_hz.is_finite() && rate_hz >= 0.0);
        assert!(depth_cents.is_finite() && depth_cents >= 0.0);

        Self {
            rate_hz,
            depth_cents,
        }
    }

    pub const fn rate_hz(&self) -> f32 {
        self.rate_hz
    }

    pub const fn depth_cents(&self) -> f32 {
        self.depth_cents
    }
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
    vibrato: Option<Vibrato>,
}

impl Instrument {
    pub fn new(name: &'static str, oscillators: Vec<OscillatorAssignment>) -> Self {
        assert!(!oscillators.is_empty());
        Self {
            name,
            oscillators,
            vibrato: None,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub fn oscillators(&self) -> &[OscillatorAssignment] {
        &self.oscillators
    }

    pub fn with_vibrato(mut self, vibrato: Vibrato) -> Self {
        self.vibrato = Some(vibrato);
        self
    }

    pub const fn vibrato(&self) -> Option<Vibrato> {
        self.vibrato
    }
}
