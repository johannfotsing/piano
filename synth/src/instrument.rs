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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tremolo {
    rate_hz: f32,
    depth: f32,
}

impl Tremolo {
    /// Creates tremolo with `depth` in the range `0.0..=1.0`.
    pub fn new(rate_hz: f32, depth: f32) -> Self {
        assert!(rate_hz.is_finite() && rate_hz >= 0.0);
        assert!(depth.is_finite() && (0.0..=1.0).contains(&depth));

        Self { rate_hz, depth }
    }

    pub const fn rate_hz(&self) -> f32 {
        self.rate_hz
    }

    pub const fn depth(&self) -> f32 {
        self.depth
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
    tremolo: Option<Tremolo>,
}

impl Instrument {
    pub fn new(name: &'static str, oscillators: Vec<OscillatorAssignment>) -> Self {
        assert!(!oscillators.is_empty());
        Self {
            name,
            oscillators,
            vibrato: None,
            tremolo: None,
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

    pub fn with_tremolo(mut self, tremolo: Tremolo) -> Self {
        self.tremolo = Some(tremolo);
        self
    }

    pub const fn tremolo(&self) -> Option<Tremolo> {
        self.tremolo
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_tremolo_to_an_instrument() {
        let instrument =
            Instrument::new("Test", vec![OscillatorAssignment::new(Waveform::Sine, 1.0)])
                .with_tremolo(Tremolo::new(4.0, 0.25));

        let tremolo = instrument.tremolo().unwrap();
        assert_eq!(tremolo.rate_hz(), 4.0);
        assert_eq!(tremolo.depth(), 0.25);
    }
}
