use crate::{Adsr, Instrument, Lfo, LfoWaveform, Oscillator};
use music::note::Note;

struct VoiceOscillator {
    oscillator: Oscillator,
    gain: f32,
}

struct VoiceVibrato {
    lfo: Lfo,
    depth_cents: f32,
}

pub struct Voice {
    oscillators: Vec<VoiceOscillator>,
    envelope: Adsr,
    vibrato: Option<VoiceVibrato>,

    note: Note,
    base_frequency: f32,
    velocity: f32,
}

impl Voice {
    pub fn new(note: Note, velocity: u8, sample_rate: f32, instrument: &Instrument) -> Self {
        let mut envelope = Adsr::new(
            sample_rate,
            0.1, // attack 100ms
            0.2, // decay 200ms
            0.7, // sustain 70%
            0.5, // release 500ms
        );

        envelope.note_on();

        let base_frequency = note.frequency() as f32;
        let oscillators = instrument
            .oscillators()
            .iter()
            .map(|assignment| VoiceOscillator {
                oscillator: Oscillator::with_waveform(
                    base_frequency,
                    sample_rate,
                    assignment.waveform(),
                ),
                gain: assignment.gain(),
            })
            .collect();
        let vibrato = instrument.vibrato().map(|vibrato| VoiceVibrato {
            lfo: Lfo::new(vibrato.rate_hz(), sample_rate, LfoWaveform::Sine),
            depth_cents: vibrato.depth_cents(),
        });

        Self {
            oscillators,
            envelope,
            vibrato,
            note,
            base_frequency,
            velocity: (velocity as f32 / 127.0).powf(2.0),
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        if let Some(vibrato) = &mut self.vibrato {
            let cents = vibrato.lfo.next_sample() * vibrato.depth_cents;
            let frequency = self.base_frequency * 2.0_f32.powf(cents / 1200.0);

            for oscillator in &mut self.oscillators {
                oscillator.oscillator.set_frequency(frequency);
            }
        }

        let envelope = self.envelope.next_sample();
        let amplitude = envelope * self.velocity;
        let oscillator_mix = self
            .oscillators
            .iter_mut()
            .map(|oscillator| oscillator.oscillator.next_sample() * oscillator.gain)
            .sum::<f32>();

        oscillator_mix * amplitude
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
