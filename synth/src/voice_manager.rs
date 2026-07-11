use crate::Voice;

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
        let gained_sample = mixed_sample * self.master_gain;

        // Smoothly constrain peaks to [-1.0, 1.0].
        gained_sample.tanh()
    }

    fn cleanup(&mut self) {
        self.voices.retain(|v| !v.is_finished());
    }

    pub fn set_master_gain(&mut self, gain: f32) {
        self.master_gain = gain.clamp(0.0, 1.0);
    }
}

// TODO: Use soft clipping instead of tanh for better performance and less CPU usage.
/// Soft clipping function to prevent harsh distortion.
fn soft_clip(sample: f32) -> f32 {
    sample / (1.0 + sample.abs())
}