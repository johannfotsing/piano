use crate::Voice;

pub struct VoiceManager {
    voices: Vec<Voice>,
}

impl VoiceManager {
    pub fn new() -> Self {
        Self { voices: Vec::new() }
    }

    pub fn note_on(&mut self, note: u8, frequency: f32, velocity: u8, sample_rate: f32) {
        self.voices
            .push(Voice::new(note, frequency, velocity, sample_rate));
    }

    pub fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.note() == note {
                voice.note_off();
            }
        }

        self.cleanup();
    }

    pub fn next_sample(&mut self) -> f32 {
        let mut output = 0.0;

        for voice in &mut self.voices {
            output += voice.next_sample();
        }

        // Prevent clipping
        if !self.voices.is_empty() {
            output /= self.voices.len() as f32;
        }

        output
    }

    fn cleanup(&mut self) {
        self.voices.retain(|v| !v.is_finished());
    }
}
