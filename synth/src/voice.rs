use core::f32::consts::TAU;

use crate::{Adsr, Hammer, Instrument, Lfo, LfoWaveform, Oscillator, StateVariableFilter};
use music::note::Note;

struct VoiceOscillator {
    oscillator: Oscillator,
    gain: f32,
    pitch_ratio: f32,
    decay_level: f32,
    decay_multiplier: f32,
}

struct VoiceHammer {
    noise_state: u32,
    level: f32,
    decay_multiplier: f32,
    filter_coefficient: f32,
    filtered_noise: f32,
}

impl VoiceHammer {
    fn new(settings: Hammer, velocity: f32, sample_rate: f32, seed: u32) -> Self {
        let cutoff_hz = settings.cutoff_hz().clamp(20.0, sample_rate * 0.45);
        Self {
            noise_state: seed.max(1),
            level: settings.gain() * velocity.powf(settings.velocity_sensitivity()),
            decay_multiplier: (-1.0 / (settings.decay_seconds() * sample_rate)).exp(),
            filter_coefficient: 1.0 - (-TAU * cutoff_hz / sample_rate).exp(),
            filtered_noise: 0.0,
        }
    }

    fn next_sample(&mut self) -> f32 {
        self.noise_state ^= self.noise_state << 13;
        self.noise_state ^= self.noise_state >> 17;
        self.noise_state ^= self.noise_state << 5;
        let noise = self.noise_state as f32 / u32::MAX as f32 * 2.0 - 1.0;
        self.filtered_noise += self.filter_coefficient * (noise - self.filtered_noise);
        let output = self.filtered_noise * self.level;
        self.level *= self.decay_multiplier;
        output
    }
}

struct VoiceVibrato {
    lfo: Lfo,
    depth_cents: f32,
}

struct VoiceTremolo {
    lfo: Lfo,
    depth: f32,
}

pub struct Voice {
    oscillators: Vec<VoiceOscillator>,
    filter: Option<StateVariableFilter>,
    envelope: Adsr,
    vibrato: Option<VoiceVibrato>,
    tremolo: Option<VoiceTremolo>,
    hammer: Option<VoiceHammer>,

    note: Note,
    base_frequency: f32,
    velocity: f32,
}

impl Voice {
    pub fn new(note: Note, velocity: u8, sample_rate: f32, instrument: &Instrument) -> Self {
        let mut envelope = Adsr::from_settings(sample_rate, instrument.envelope());

        envelope.note_on();

        let base_frequency = note.frequency() as f32;
        let normalized_velocity = velocity as f32 / 127.0;
        let oscillators = instrument
            .oscillators()
            .iter()
            .map(|assignment| {
                let pitch_ratio =
                    assignment.frequency_ratio() * 2.0_f32.powf(assignment.detune_cents() / 1200.0);
                VoiceOscillator {
                    oscillator: Oscillator::with_waveform(
                        base_frequency * pitch_ratio,
                        sample_rate,
                        assignment.waveform(),
                    ),
                    gain: assignment.gain()
                        * normalized_velocity.powf(assignment.velocity_sensitivity()),
                    pitch_ratio,
                    decay_level: 1.0,
                    decay_multiplier: (-1.0 / (assignment.decay_seconds() * sample_rate)).exp(),
                }
            })
            .collect();
        let vibrato = instrument.vibrato().map(|vibrato| VoiceVibrato {
            lfo: Lfo::new(vibrato.rate_hz(), sample_rate, LfoWaveform::Sine),
            depth_cents: vibrato.depth_cents(),
        });
        let tremolo = instrument.tremolo().map(|tremolo| VoiceTremolo {
            lfo: Lfo::new(tremolo.rate_hz(), sample_rate, LfoWaveform::Sine),
            depth: tremolo.depth(),
        });
        let filter = instrument
            .filter()
            .map(|settings| StateVariableFilter::new(settings, sample_rate));
        let hammer = instrument.hammer().map(|settings| {
            VoiceHammer::new(
                settings,
                normalized_velocity,
                sample_rate,
                (note.midi_number() as u32)
                    .wrapping_mul(2_654_435_761)
                    .wrapping_add(1),
            )
        });

        Self {
            oscillators,
            filter,
            envelope,
            vibrato,
            tremolo,
            hammer,
            note,
            base_frequency,
            velocity: normalized_velocity.powf(2.0),
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        if let Some(vibrato) = &mut self.vibrato {
            let cents = vibrato.lfo.next_sample() * vibrato.depth_cents;
            let frequency = self.base_frequency * 2.0_f32.powf(cents / 1200.0);

            for oscillator in &mut self.oscillators {
                oscillator
                    .oscillator
                    .set_frequency(frequency * oscillator.pitch_ratio);
            }
        }

        let envelope = self.envelope.next_sample();
        let tremolo_gain = self.tremolo.as_mut().map_or(1.0, |tremolo| {
            let unipolar_lfo = (tremolo.lfo.next_sample() + 1.0) * 0.5;
            1.0 - tremolo.depth * unipolar_lfo
        });
        let amplitude = envelope * self.velocity * tremolo_gain;
        let mut oscillator_mix = self
            .oscillators
            .iter_mut()
            .map(|oscillator| {
                let sample =
                    oscillator.oscillator.next_sample() * oscillator.gain * oscillator.decay_level;
                oscillator.decay_level *= oscillator.decay_multiplier;
                sample
            })
            .sum::<f32>();
        if let Some(hammer) = &mut self.hammer {
            oscillator_mix += hammer.next_sample();
        }
        let filtered_sample = self
            .filter
            .as_mut()
            .map_or(oscillator_mix, |filter| filter.process(oscillator_mix));

        filtered_sample * amplitude
    }

    pub fn note_off(&mut self) {
        self.envelope.note_off();
    }

    pub fn note(&self) -> Note {
        self.note
    }

    pub fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OscillatorAssignment, Waveform};

    #[test]
    fn creates_and_decays_a_harmonic_partial() {
        let instrument = Instrument::new(
            "Partial",
            vec![OscillatorAssignment::new(Waveform::Sine, 1.0).with_partial(2.0, 0.0, 1.0, 0.0)],
        );
        let mut voice = Voice::new(Note::A4, 127, 44_100.0, &instrument);

        assert!((voice.oscillators[0].oscillator.frequency() - 880.0).abs() < 0.01);
        voice.next_sample();
        assert!(voice.oscillators[0].decay_level < 1.0);
    }
}
