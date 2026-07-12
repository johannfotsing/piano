use crate::{ChorusProcessor, FlangerProcessor, Instrument, ReverbProcessor, VoiceManager};
use music::note::Note;

/// Top-level synthesizer responsible for creating and rendering voices.
pub struct Synthesizer {
    voice_manager: VoiceManager,
    sample_rate: f32,
    chorus: Option<ChorusProcessor>,
    flanger: Option<FlangerProcessor>,
    reverb: Option<ReverbProcessor>,
}

impl Synthesizer {
    pub fn new(sample_rate: f32) -> Self {
        assert!(sample_rate.is_finite() && sample_rate > 0.0);

        Self {
            voice_manager: VoiceManager::new(),
            sample_rate,
            chorus: None,
            flanger: None,
            reverb: None,
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
        let mut sample = self.voice_manager.next_sample();

        if let Some(chorus) = &mut self.chorus {
            sample = chorus.process(sample);
        }
        if let Some(flanger) = &mut self.flanger {
            sample = flanger.process(sample);
        }
        if let Some(reverb) = &mut self.reverb {
            sample = reverb.process(sample);
        }

        // Final soft limiter runs after the effects and their feedback paths.
        sample.tanh()
    }

    pub fn set_master_gain(&mut self, gain: f32) {
        self.voice_manager.set_master_gain(gain);
    }

    pub fn configure_effects(&mut self, instrument: &Instrument) {
        self.chorus = instrument
            .chorus()
            .map(|settings| ChorusProcessor::new(settings, self.sample_rate));
        self.flanger = instrument
            .flanger()
            .map(|settings| FlangerProcessor::new(settings, self.sample_rate));
        self.reverb = instrument
            .reverb()
            .map(|settings| ReverbProcessor::new(settings, self.sample_rate));
    }
}
