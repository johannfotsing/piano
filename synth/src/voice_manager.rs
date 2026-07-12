use crate::{Instrument, Voice};
use music::note::Note;

pub struct VoiceManager {
    voices: Vec<Voice>,
    master_gain: f32,
}

impl VoiceManager {
    pub fn new() -> Self {
        Self {
            voices: Vec::new(),
            master_gain: 0.2,
        }
    }

    pub fn note_on(&mut self, note: Note, velocity: u8, sample_rate: f32, instrument: &Instrument) {
        self.voices
            .push(Voice::new(note, velocity, sample_rate, instrument));
    }

    pub fn note_off(&mut self, note: Note) {
        for voice in &mut self.voices {
            if voice.note() == note {
                voice.note_off();
            }
        }

        // self.cleanup();
    }

    pub fn next_sample(&mut self) -> f32 {
        let mut mixed_sample = 0.0;

        for voice in &mut self.voices {
            mixed_sample += voice.next_sample();
        }

        // Remove voices after their release envelope reaches Idle.
        self.cleanup();

        // Fixed gain keeps existing notes at the same level when polyphony changes.
        mixed_sample * self.master_gain
    }

    fn cleanup(&mut self) {
        self.voices.retain(|v| !v.is_finished());
    }

    pub fn set_master_gain(&mut self, gain: f32) {
        self.master_gain = gain.clamp(0.0, 1.0);
    }
}
