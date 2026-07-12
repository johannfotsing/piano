#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvelopeSettings {
    attack_seconds: f32,
    decay_seconds: f32,
    sustain_level: f32,
    release_seconds: f32,
}

impl EnvelopeSettings {
    pub fn new(
        attack_seconds: f32,
        decay_seconds: f32,
        sustain_level: f32,
        release_seconds: f32,
    ) -> Self {
        assert!(attack_seconds.is_finite() && attack_seconds >= 0.0);
        assert!(decay_seconds.is_finite() && decay_seconds >= 0.0);
        assert!(sustain_level.is_finite() && (0.0..=1.0).contains(&sustain_level));
        assert!(release_seconds.is_finite() && release_seconds >= 0.0);

        Self {
            attack_seconds,
            decay_seconds,
            sustain_level,
            release_seconds,
        }
    }

    pub const fn attack_seconds(&self) -> f32 {
        self.attack_seconds
    }

    pub const fn decay_seconds(&self) -> f32 {
        self.decay_seconds
    }

    pub const fn sustain_level(&self) -> f32 {
        self.sustain_level
    }

    pub const fn release_seconds(&self) -> f32 {
        self.release_seconds
    }
}

impl Default for EnvelopeSettings {
    fn default() -> Self {
        Self::new(0.1, 0.2, 0.7, 0.5)
    }
}

pub struct Adsr {
    attack_samples: u32,
    decay_samples: u32,
    sustain_level: f32,
    release_samples: u32,

    state: EnvelopeState,

    current_level: f32,
    release_start_level: f32,
    sample_counter: u32,
}

impl Adsr {
    pub fn from_settings(sample_rate: f32, settings: EnvelopeSettings) -> Self {
        Self::new(
            sample_rate,
            settings.attack_seconds(),
            settings.decay_seconds(),
            settings.sustain_level(),
            settings.release_seconds(),
        )
    }

    pub fn new(
        sample_rate: f32,
        attack_seconds: f32,
        decay_seconds: f32,
        sustain_level: f32,
        release_seconds: f32,
    ) -> Self {
        Self {
            attack_samples: (attack_seconds * sample_rate) as u32,

            decay_samples: (decay_seconds * sample_rate) as u32,

            sustain_level,

            release_samples: (release_seconds * sample_rate) as u32,

            state: EnvelopeState::Idle,

            current_level: 0.0,
            release_start_level: 0.0,

            sample_counter: 0,
        }
    }

    pub fn note_on(&mut self) {
        self.state = EnvelopeState::Attack;
        self.sample_counter = 0;
    }

    pub fn note_off(&mut self) {
        self.release_start_level = self.current_level;
        self.state = EnvelopeState::Release;
        self.sample_counter = 0;
    }

    pub fn next_sample(&mut self) -> f32 {
        match self.state {
            EnvelopeState::Idle => {
                self.current_level = 0.0;
            }

            EnvelopeState::Attack => {
                if self.attack_samples == 0 {
                    self.current_level = 1.0;
                    //
                    self.state = EnvelopeState::Decay;
                    self.sample_counter = 0;
                } else {
                    self.current_level = self.sample_counter as f32 / self.attack_samples as f32;

                    self.sample_counter += 1;

                    if self.sample_counter >= self.attack_samples {
                        self.current_level = 1.0;
                        //
                        self.state = EnvelopeState::Decay;
                        self.sample_counter = 0;
                    }
                }
            }

            EnvelopeState::Decay => {
                if self.decay_samples == 0 {
                    self.current_level = self.sustain_level;
                    //
                    self.state = EnvelopeState::Sustain;
                    self.sample_counter = 0;
                } else {
                    let progress = self.sample_counter as f32 / self.decay_samples as f32;

                    self.current_level = 1.0 - progress * (1.0 - self.sustain_level);

                    self.sample_counter += 1;

                    if self.sample_counter >= self.decay_samples {
                        self.current_level = self.sustain_level;
                        //
                        self.state = EnvelopeState::Sustain;
                        self.sample_counter = 0;
                    }
                }
            }

            EnvelopeState::Sustain => {
                self.current_level = self.sustain_level;
            }

            EnvelopeState::Release => {
                if self.release_samples == 0 {
                    self.current_level = 0.0;
                    self.state = EnvelopeState::Idle;
                } else {
                    let progress = self.sample_counter as f32 / self.release_samples as f32;

                    self.current_level = self.release_start_level * (1.0 - progress);

                    self.sample_counter += 1;

                    if self.sample_counter >= self.release_samples {
                        self.current_level = 0.0;
                        self.state = EnvelopeState::Idle;
                        self.sample_counter = 0;
                    }
                }
            }
        }

        self.current_level
    }

    pub fn is_finished(&self) -> bool {
        self.state == EnvelopeState::Idle
    }
}
