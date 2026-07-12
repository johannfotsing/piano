use crate::{Instrument, VoiceManager};
use music::note::Note;

/// Top-level synthesizer responsible for creating and rendering voices.
pub struct Synthesizer {
    voice_manager: VoiceManager,
    sample_rate: f32,
}

impl Synthesizer {
    pub fn new(sample_rate: f32) -> Self {
        assert!(sample_rate.is_finite() && sample_rate > 0.0);

        Self {
            voice_manager: VoiceManager::new(),
            sample_rate,
        }
    }

    pub fn note_on(&mut self, note: Note, velocity: u8, instrument: &Instrument) {
        self.voice_manager
            .note_on(note, velocity, self.sample_rate, instrument);
    }

    pub fn note_off(&mut self, note: Note) {
        self.voice_manager.note_off(note);
    }

    pub fn next_sample(&mut self) -> f32 {
        self.voice_manager.next_sample()
    }

    pub fn set_master_gain(&mut self, gain: f32) {
        self.voice_manager.set_master_gain(gain);
    }
}
