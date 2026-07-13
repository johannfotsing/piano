use alloc::{string::String, vec::Vec};

use crate::{EnvelopeSettings, FilterSettings, Waveform};

/// An oscillator definition and its contribution to an instrument's sound.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OscillatorAssignment {
    waveform: Waveform,
    gain: f32,
    frequency_ratio: f32,
    detune_cents: f32,
    decay_seconds: f32,
    velocity_sensitivity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hammer {
    gain: f32,
    decay_seconds: f32,
    cutoff_hz: f32,
    velocity_sensitivity: f32,
}

/// Velocity-sensitive, physically modelled plucked string mixed into a voice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pluck {
    gain: f32,
    decay_seconds: f32,
    cutoff_hz: f32,
    velocity_sensitivity: f32,
}

impl Pluck {
    pub fn new(gain: f32, decay_seconds: f32, cutoff_hz: f32, velocity_sensitivity: f32) -> Self {
        assert!(gain.is_finite() && gain >= 0.0);
        assert!(decay_seconds.is_finite() && decay_seconds > 0.0);
        assert!(cutoff_hz.is_finite() && cutoff_hz > 0.0);
        assert!(velocity_sensitivity.is_finite() && velocity_sensitivity >= 0.0);
        Self {
            gain,
            decay_seconds,
            cutoff_hz,
            velocity_sensitivity,
        }
    }

    pub const fn gain(&self) -> f32 {
        self.gain
    }
    pub const fn decay_seconds(&self) -> f32 {
        self.decay_seconds
    }
    pub const fn cutoff_hz(&self) -> f32 {
        self.cutoff_hz
    }
    pub const fn velocity_sensitivity(&self) -> f32 {
        self.velocity_sensitivity
    }
}

impl Hammer {
    pub fn new(gain: f32, decay_seconds: f32, cutoff_hz: f32, velocity_sensitivity: f32) -> Self {
        assert!(gain.is_finite() && gain >= 0.0);
        assert!(decay_seconds.is_finite() && decay_seconds > 0.0);
        assert!(cutoff_hz.is_finite() && cutoff_hz > 0.0);
        assert!(velocity_sensitivity.is_finite() && velocity_sensitivity >= 0.0);
        Self {
            gain,
            decay_seconds,
            cutoff_hz,
            velocity_sensitivity,
        }
    }

    pub const fn gain(&self) -> f32 {
        self.gain
    }
    pub const fn decay_seconds(&self) -> f32 {
        self.decay_seconds
    }
    pub const fn cutoff_hz(&self) -> f32 {
        self.cutoff_hz
    }
    pub const fn velocity_sensitivity(&self) -> f32 {
        self.velocity_sensitivity
    }
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chorus {
    rate_hz: f32,
    base_delay_ms: f32,
    depth_ms: f32,
    mix: f32,
}

impl Chorus {
    pub fn new(rate_hz: f32, base_delay_ms: f32, depth_ms: f32, mix: f32) -> Self {
        assert!(rate_hz.is_finite() && rate_hz >= 0.0);
        assert!(base_delay_ms.is_finite() && base_delay_ms > 0.0);
        assert!(depth_ms.is_finite() && depth_ms >= 0.0);
        assert!(mix.is_finite() && (0.0..=1.0).contains(&mix));
        Self {
            rate_hz,
            base_delay_ms,
            depth_ms,
            mix,
        }
    }

    pub const fn rate_hz(&self) -> f32 {
        self.rate_hz
    }
    pub const fn base_delay_ms(&self) -> f32 {
        self.base_delay_ms
    }
    pub const fn depth_ms(&self) -> f32 {
        self.depth_ms
    }
    pub const fn mix(&self) -> f32 {
        self.mix
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flanger {
    rate_hz: f32,
    base_delay_ms: f32,
    depth_ms: f32,
    feedback: f32,
    mix: f32,
}

impl Flanger {
    pub fn new(rate_hz: f32, base_delay_ms: f32, depth_ms: f32, feedback: f32, mix: f32) -> Self {
        assert!(rate_hz.is_finite() && rate_hz >= 0.0);
        assert!(base_delay_ms.is_finite() && base_delay_ms > 0.0);
        assert!(depth_ms.is_finite() && depth_ms >= 0.0);
        assert!(feedback.is_finite() && (-0.95..=0.95).contains(&feedback));
        assert!(mix.is_finite() && (0.0..=1.0).contains(&mix));
        Self {
            rate_hz,
            base_delay_ms,
            depth_ms,
            feedback,
            mix,
        }
    }

    pub const fn rate_hz(&self) -> f32 {
        self.rate_hz
    }
    pub const fn base_delay_ms(&self) -> f32 {
        self.base_delay_ms
    }
    pub const fn depth_ms(&self) -> f32 {
        self.depth_ms
    }
    pub const fn feedback(&self) -> f32 {
        self.feedback
    }
    pub const fn mix(&self) -> f32 {
        self.mix
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reverb {
    room_size: f32,
    damping: f32,
    mix: f32,
}

impl Reverb {
    pub fn new(room_size: f32, damping: f32, mix: f32) -> Self {
        assert!(room_size.is_finite() && (0.0..=1.0).contains(&room_size));
        assert!(damping.is_finite() && (0.0..=1.0).contains(&damping));
        assert!(mix.is_finite() && (0.0..=1.0).contains(&mix));
        Self {
            room_size,
            damping,
            mix,
        }
    }

    pub const fn room_size(&self) -> f32 {
        self.room_size
    }
    pub const fn damping(&self) -> f32 {
        self.damping
    }
    pub const fn mix(&self) -> f32 {
        self.mix
    }
}

impl OscillatorAssignment {
    pub fn new(waveform: Waveform, gain: f32) -> Self {
        assert!(gain.is_finite() && gain >= 0.0);
        Self {
            waveform,
            gain,
            frequency_ratio: 1.0,
            detune_cents: 0.0,
            decay_seconds: 60.0,
            velocity_sensitivity: 0.0,
        }
    }

    pub fn with_partial(
        mut self,
        frequency_ratio: f32,
        detune_cents: f32,
        decay_seconds: f32,
        velocity_sensitivity: f32,
    ) -> Self {
        assert!(frequency_ratio.is_finite() && frequency_ratio > 0.0);
        assert!(detune_cents.is_finite());
        assert!(decay_seconds.is_finite() && decay_seconds > 0.0);
        assert!(velocity_sensitivity.is_finite() && velocity_sensitivity >= 0.0);
        self.frequency_ratio = frequency_ratio;
        self.detune_cents = detune_cents;
        self.decay_seconds = decay_seconds;
        self.velocity_sensitivity = velocity_sensitivity;
        self
    }

    pub const fn waveform(&self) -> Waveform {
        self.waveform
    }

    pub const fn gain(&self) -> f32 {
        self.gain
    }

    pub const fn frequency_ratio(&self) -> f32 {
        self.frequency_ratio
    }
    pub const fn detune_cents(&self) -> f32 {
        self.detune_cents
    }
    pub const fn decay_seconds(&self) -> f32 {
        self.decay_seconds
    }
    pub const fn velocity_sensitivity(&self) -> f32 {
        self.velocity_sensitivity
    }
}

/// A synthesizer preset used when creating new voices.
#[derive(Debug, Clone, PartialEq)]
pub struct Instrument {
    name: String,
    oscillators: Vec<OscillatorAssignment>,
    vibrato: Option<Vibrato>,
    tremolo: Option<Tremolo>,
    chorus: Option<Chorus>,
    flanger: Option<Flanger>,
    reverb: Option<Reverb>,
    hammer: Option<Hammer>,
    pluck: Option<Pluck>,
    filter: Option<FilterSettings>,
    envelope: EnvelopeSettings,
}

impl Instrument {
    pub fn new(name: impl Into<String>, oscillators: Vec<OscillatorAssignment>) -> Self {
        assert!(!oscillators.is_empty());
        Self {
            name: name.into(),
            oscillators,
            vibrato: None,
            tremolo: None,
            chorus: None,
            flanger: None,
            reverb: None,
            hammer: None,
            pluck: None,
            filter: None,
            envelope: EnvelopeSettings::default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
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

    pub fn with_chorus(mut self, chorus: Chorus) -> Self {
        self.chorus = Some(chorus);
        self
    }

    pub const fn chorus(&self) -> Option<Chorus> {
        self.chorus
    }

    pub fn with_flanger(mut self, flanger: Flanger) -> Self {
        self.flanger = Some(flanger);
        self
    }

    pub const fn flanger(&self) -> Option<Flanger> {
        self.flanger
    }

    pub fn with_reverb(mut self, reverb: Reverb) -> Self {
        self.reverb = Some(reverb);
        self
    }

    pub const fn reverb(&self) -> Option<Reverb> {
        self.reverb
    }

    pub fn with_hammer(mut self, hammer: Hammer) -> Self {
        self.hammer = Some(hammer);
        self
    }

    pub const fn hammer(&self) -> Option<Hammer> {
        self.hammer
    }

    pub fn with_pluck(mut self, pluck: Pluck) -> Self {
        self.pluck = Some(pluck);
        self
    }

    pub const fn pluck(&self) -> Option<Pluck> {
        self.pluck
    }

    pub fn with_filter(mut self, filter: FilterSettings) -> Self {
        self.filter = Some(filter);
        self
    }

    pub const fn filter(&self) -> Option<FilterSettings> {
        self.filter
    }

    pub fn with_envelope(mut self, envelope: EnvelopeSettings) -> Self {
        self.envelope = envelope;
        self
    }

    pub const fn envelope(&self) -> EnvelopeSettings {
        self.envelope
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn adds_tremolo_to_an_instrument() {
        let instrument =
            Instrument::new("Test", vec![OscillatorAssignment::new(Waveform::Sine, 1.0)])
                .with_tremolo(Tremolo::new(4.0, 0.25));

        let tremolo = instrument.tremolo().unwrap();
        assert_eq!(tremolo.rate_hz(), 4.0);
        assert_eq!(tremolo.depth(), 0.25);
    }

    #[test]
    fn adds_filter_to_an_instrument() {
        let instrument = Instrument::new(
            "Test",
            vec![OscillatorAssignment::new(Waveform::Sawtooth, 1.0)],
        )
        .with_filter(FilterSettings::low_pass(2_500.0, 0.707));

        let filter = instrument.filter().unwrap();
        assert_eq!(filter.mode(), crate::FilterMode::LowPass);
        assert_eq!(filter.cutoff_hz(), 2_500.0);
    }

    #[test]
    fn adds_time_based_effects_to_an_instrument() {
        let instrument =
            Instrument::new("Test", vec![OscillatorAssignment::new(Waveform::Sine, 1.0)])
                .with_chorus(Chorus::new(0.6, 20.0, 5.0, 0.3))
                .with_flanger(Flanger::new(0.2, 1.0, 2.0, 0.5, 0.25))
                .with_reverb(Reverb::new(0.65, 0.4, 0.2));

        assert!(instrument.chorus().is_some());
        assert!(instrument.flanger().is_some());
        assert!(instrument.reverb().is_some());
    }

    #[test]
    fn configures_a_velocity_sensitive_partial_and_hammer() {
        let partial =
            OscillatorAssignment::new(Waveform::Sine, 0.2).with_partial(2.01, 1.5, 1.2, 0.8);
        let instrument = Instrument::new("Piano", vec![partial])
            .with_hammer(Hammer::new(0.06, 0.018, 6_000.0, 1.5));

        assert_eq!(instrument.oscillators()[0].frequency_ratio(), 2.01);
        assert_eq!(instrument.oscillators()[0].decay_seconds(), 1.2);
        assert_eq!(instrument.hammer().unwrap().cutoff_hz(), 6_000.0);
    }

    #[test]
    fn adds_a_velocity_sensitive_pluck_to_an_instrument() {
        let instrument = Instrument::new(
            "Guitar",
            vec![OscillatorAssignment::new(Waveform::Sine, 1.0)],
        )
        .with_pluck(Pluck::new(0.12, 0.025, 6_500.0, 1.1));

        let pluck = instrument.pluck().unwrap();
        assert_eq!(pluck.gain(), 0.12);
        assert_eq!(pluck.decay_seconds(), 0.025);
        assert_eq!(pluck.cutoff_hz(), 6_500.0);
        assert_eq!(pluck.velocity_sensitivity(), 1.1);
    }
}
