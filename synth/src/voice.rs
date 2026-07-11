use crate::{Adsr, Oscillator};

pub struct Voice {
    oscillator: Oscillator,
    envelope: Adsr,

    note: u8,
    velocity: f32,
}

impl Voice {
    pub fn new(note: u8, frequency: f32, velocity: u8, sample_rate: f32) -> Self {
        let mut envelope = Adsr::new(
            sample_rate,
            0.1, // attack 300ms
            0.2,  // decay 200ms
            0.7,  // sustain 70%
            0.5,  // release 500ms
        );

        envelope.note_on();

        Self {
            oscillator: Oscillator::new(frequency, sample_rate),

            envelope,

            note,

            velocity: (velocity as f32 / 127.0).powf(2.0), // Normalize velocity to [0.0, 1.0]
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let envelope = self.envelope.next_sample();
        let amplitude = envelope * self.velocity; // Scale by velocity
        self.oscillator.next_sample() * amplitude
    }

    pub fn note_off(&mut self) {
        self.envelope.note_off();
    }

    pub fn note(&self) -> u8 {
        self.note
    }

    pub fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }
}
