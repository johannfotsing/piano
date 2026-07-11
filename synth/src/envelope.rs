#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Adsr {
    attack_samples: u32,
    decay_samples: u32,
    sustain_level: f32,
    release_samples: u32,

    state: EnvelopeState,

    current_level: f32,
    sample_counter: u32,
}

impl Adsr {
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

            sample_counter: 0,
        }
    }

    pub fn note_on(&mut self) {
        self.state = EnvelopeState::Attack;
        self.sample_counter = 0;
    }

    pub fn note_off(&mut self) {
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

                    self.current_level *= 1.0 - progress;

                    self.sample_counter += 1;

                    if self.sample_counter >= self.release_samples {
                        self.current_level = 0.0;
                        self.state = EnvelopeState::Idle;
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
