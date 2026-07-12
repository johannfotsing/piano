use crate::{Adsr, Instrument, Oscillator};
use music::note::Note;

struct VoiceOscillator {
    oscillator: Oscillator,
    gain: f32,
}

pub struct Voice {
    oscillators: Vec<VoiceOscillator>,
    envelope: Adsr,

    note: Note,
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

        let oscillators = instrument
            .oscillators()
            .iter()
            .map(|assignment| VoiceOscillator {
                oscillator: Oscillator::with_waveform(
                    note.frequency() as f32,
                    sample_rate,
                    assignment.waveform(),
                ),
                gain: assignment.gain(),
            })
            .collect();

        Self {
            oscillators,
            envelope,
            note,
            velocity: (velocity as f32 / 127.0).powf(2.0),
        }
    }

    pub fn next_sample(&mut self) -> f32 {
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
