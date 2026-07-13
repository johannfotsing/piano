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
    attack_curvature: f32,
    decay_seconds: f32,
    decay_curvature: f32,
    sustain_level: f32,
    sustain_end_level: f32,
    sustain_curvature: f32,
    maximum_sustain_seconds: f32,
    release_seconds: f32,
    release_curvature: f32,
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
            attack_curvature: 0.0,
            decay_seconds,
            decay_curvature: 0.0,
            sustain_level,
            sustain_end_level: sustain_level,
            sustain_curvature: 0.0,
            maximum_sustain_seconds: 0.0,
            release_seconds,
            release_curvature: 3.0,
        }
    }

    pub fn with_attack_curvature(mut self, attack_curvature: f32) -> Self {
        validate_curvature(attack_curvature);
        self.attack_curvature = attack_curvature;
        self
    }

    pub fn with_decay_curvature(mut self, decay_curvature: f32) -> Self {
        validate_curvature(decay_curvature);
        self.decay_curvature = decay_curvature;
        self
    }

    pub fn with_release_curvature(mut self, release_curvature: f32) -> Self {
        validate_curvature(release_curvature);
        self.release_curvature = release_curvature;
        self
    }

    /// Limits the sustain stage while a note remains held. Zero disables the limit.
    pub fn with_maximum_sustain(mut self, maximum_sustain_seconds: f32) -> Self {
        assert!(maximum_sustain_seconds.is_finite() && maximum_sustain_seconds >= 0.0);
        self.maximum_sustain_seconds = maximum_sustain_seconds;
        self
    }

    /// Shapes a limited sustain from `sustain_level` toward `end_level`.
    pub fn with_sustain_shape(mut self, end_level: f32, curvature: f32) -> Self {
        assert!(end_level.is_finite() && (0.0..=self.sustain_level).contains(&end_level));
        validate_curvature(curvature);
        self.sustain_end_level = end_level;
        self.sustain_curvature = curvature;
        self
    }

    pub const fn attack_seconds(&self) -> f32 {
        self.attack_seconds
    }

    pub const fn attack_curvature(&self) -> f32 {
        self.attack_curvature
    }

    pub const fn decay_seconds(&self) -> f32 {
        self.decay_seconds
    }

    pub const fn decay_curvature(&self) -> f32 {
        self.decay_curvature
    }

    pub const fn sustain_level(&self) -> f32 {
        self.sustain_level
    }

    pub const fn maximum_sustain_seconds(&self) -> f32 {
        self.maximum_sustain_seconds
    }

    pub const fn sustain_end_level(&self) -> f32 {
        self.sustain_end_level
    }

    pub const fn sustain_curvature(&self) -> f32 {
        self.sustain_curvature
    }

    pub const fn release_seconds(&self) -> f32 {
        self.release_seconds
    }

    pub const fn release_curvature(&self) -> f32 {
        self.release_curvature
    }
}

impl Default for EnvelopeSettings {
    fn default() -> Self {
        Self::new(0.1, 0.2, 0.7, 0.5)
    }
}

pub struct Adsr {
    attack_samples: u32,
    attack_curvature: f32,
    attack_curve_scale: f32,
    decay_samples: u32,
    decay_curvature: f32,
    decay_curve_scale: f32,
    sustain_level: f32,
    sustain_end_level: f32,
    sustain_curvature: f32,
    sustain_curve_scale: f32,
    maximum_sustain_samples: u32,
    release_samples: u32,
    release_curvature: f32,
    release_curve_scale: f32,

    state: EnvelopeState,

    current_level: f32,
    release_start_level: f32,
    sample_counter: u32,
}

impl Adsr {
    pub fn from_settings(sample_rate: f32, settings: EnvelopeSettings) -> Self {
        let mut envelope = Self::new_with_curvatures(
            sample_rate,
            settings.attack_seconds(),
            settings.attack_curvature(),
            settings.decay_seconds(),
            settings.decay_curvature(),
            settings.sustain_level(),
            settings.release_seconds(),
            settings.release_curvature(),
        );
        envelope.maximum_sustain_samples =
            (settings.maximum_sustain_seconds() * sample_rate) as u32;
        envelope.sustain_end_level = settings.sustain_end_level();
        envelope.sustain_curvature = settings.sustain_curvature();
        envelope.sustain_curve_scale = curve_scale(settings.sustain_curvature());
        envelope
    }

    pub fn new(
        sample_rate: f32,
        attack_seconds: f32,
        decay_seconds: f32,
        sustain_level: f32,
        release_seconds: f32,
    ) -> Self {
        Self::new_with_release_curvature(
            sample_rate,
            attack_seconds,
            decay_seconds,
            sustain_level,
            release_seconds,
            3.0,
        )
    }

    pub fn new_with_release_curvature(
        sample_rate: f32,
        attack_seconds: f32,
        decay_seconds: f32,
        sustain_level: f32,
        release_seconds: f32,
        release_curvature: f32,
    ) -> Self {
        Self::new_with_curvatures(
            sample_rate,
            attack_seconds,
            0.0,
            decay_seconds,
            0.0,
            sustain_level,
            release_seconds,
            release_curvature,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_curvatures(
        sample_rate: f32,
        attack_seconds: f32,
        attack_curvature: f32,
        decay_seconds: f32,
        decay_curvature: f32,
        sustain_level: f32,
        release_seconds: f32,
        release_curvature: f32,
    ) -> Self {
        validate_curvature(attack_curvature);
        validate_curvature(decay_curvature);
        validate_curvature(release_curvature);
        Self {
            attack_samples: (attack_seconds * sample_rate) as u32,
            attack_curvature,
            attack_curve_scale: curve_scale(attack_curvature),

            decay_samples: (decay_seconds * sample_rate) as u32,
            decay_curvature,
            decay_curve_scale: curve_scale(decay_curvature),

            sustain_level,
            sustain_end_level: sustain_level,
            sustain_curvature: 0.0,
            sustain_curve_scale: 0.0,
            maximum_sustain_samples: 0,

            release_samples: (release_seconds * sample_rate) as u32,
            release_curvature,
            release_curve_scale: curve_scale(release_curvature),

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
                    let progress = self.sample_counter as f32 / self.attack_samples as f32;
                    self.current_level =
                        curve_progress(progress, self.attack_curvature, self.attack_curve_scale);

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

                    let curved_progress =
                        curve_progress(progress, self.decay_curvature, self.decay_curve_scale);
                    self.current_level = 1.0 - curved_progress * (1.0 - self.sustain_level);

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
                if self.maximum_sustain_samples > 0
                    && self.sample_counter >= self.maximum_sustain_samples
                {
                    self.current_level = self.sustain_end_level;
                    self.note_off();
                } else {
                    if self.maximum_sustain_samples > 0 {
                        let progress =
                            self.sample_counter as f32 / self.maximum_sustain_samples as f32;
                        let curved_progress = curve_progress(
                            progress,
                            self.sustain_curvature,
                            self.sustain_curve_scale,
                        );
                        self.current_level = self.sustain_level
                            + (self.sustain_end_level - self.sustain_level) * curved_progress;
                    } else {
                        self.current_level = self.sustain_level;
                    }
                    self.sample_counter = self.sample_counter.saturating_add(1);
                }
            }

            EnvelopeState::Release => {
                if self.release_samples == 0 {
                    self.current_level = 0.0;
                    self.state = EnvelopeState::Idle;
                } else {
                    let progress = self.sample_counter as f32 / self.release_samples as f32;
                    let amplitude = 1.0
                        - curve_progress(
                            progress,
                            self.release_curvature,
                            self.release_curve_scale,
                        );
                    self.current_level = self.release_start_level * amplitude.max(0.0);

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

    pub fn is_releasing(&self) -> bool {
        self.state == EnvelopeState::Release
    }
}

fn validate_curvature(curvature: f32) {
    assert!(curvature.is_finite() && (-10.0..=10.0).contains(&curvature));
}

fn curve_scale(curvature: f32) -> f32 {
    if curvature.abs() <= f32::EPSILON {
        0.0
    } else {
        libm::expf(curvature) - 1.0
    }
}

fn curve_progress(progress: f32, curvature: f32, scale: f32) -> f32 {
    if curvature.abs() <= f32::EPSILON {
        progress
    } else {
        (libm::expf(curvature * progress) - 1.0) / scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_uses_a_convex_curve_and_finishes_on_time() {
        let mut envelope = Adsr::new(10.0, 0.0, 0.0, 1.0, 1.0);
        envelope.note_on();
        envelope.next_sample();
        envelope.next_sample();
        envelope.note_off();

        assert_eq!(envelope.next_sample(), 1.0);
        for _ in 0..4 {
            envelope.next_sample();
        }
        let midpoint = envelope.next_sample();
        assert!(midpoint > 0.5 && midpoint < 1.0);

        for _ in 0..4 {
            envelope.next_sample();
        }
        assert!(envelope.is_finished());
        assert_eq!(envelope.current_level, 0.0);
    }

    #[test]
    fn zero_release_curvature_is_linear() {
        let mut envelope = Adsr::new_with_release_curvature(10.0, 0.0, 0.0, 1.0, 1.0, 0.0);
        envelope.note_on();
        envelope.next_sample();
        envelope.next_sample();
        envelope.note_off();

        for _ in 0..5 {
            envelope.next_sample();
        }
        assert!((envelope.next_sample() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn negative_release_curvature_is_concave() {
        let mut envelope = Adsr::new_with_release_curvature(10.0, 0.0, 0.0, 1.0, 1.0, -3.0);
        envelope.note_on();
        envelope.next_sample();
        envelope.next_sample();
        envelope.note_off();

        for _ in 0..5 {
            envelope.next_sample();
        }
        assert!(envelope.next_sample() < 0.5);
    }

    #[test]
    fn attack_curvature_moves_between_concave_and_convex() {
        let mut convex = Adsr::new_with_curvatures(10.0, 1.0, 3.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let mut concave = Adsr::new_with_curvatures(10.0, 1.0, -3.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        convex.note_on();
        concave.note_on();
        for _ in 0..5 {
            convex.next_sample();
            concave.next_sample();
        }

        assert!(convex.next_sample() < 0.5);
        assert!(concave.next_sample() > 0.5);
    }

    #[test]
    fn decay_curvature_shapes_the_fall_to_sustain() {
        let mut delayed = Adsr::new_with_curvatures(10.0, 0.0, 0.0, 1.0, 3.0, 0.0, 0.0, 0.0);
        let mut accelerated = Adsr::new_with_curvatures(10.0, 0.0, 0.0, 1.0, -3.0, 0.0, 0.0, 0.0);
        delayed.note_on();
        accelerated.note_on();
        delayed.next_sample();
        accelerated.next_sample();
        for _ in 0..5 {
            delayed.next_sample();
            accelerated.next_sample();
        }

        assert!(delayed.next_sample() > 0.5);
        assert!(accelerated.next_sample() < 0.5);
    }

    #[test]
    fn maximum_sustain_automatically_starts_release() {
        let settings = EnvelopeSettings::new(0.0, 0.0, 1.0, 0.2).with_maximum_sustain(0.3);
        let mut envelope = Adsr::from_settings(10.0, settings);
        envelope.note_on();
        envelope.next_sample(); // Attack -> Decay
        envelope.next_sample(); // Decay -> Sustain

        for _ in 0..3 {
            assert_eq!(envelope.next_sample(), 1.0);
        }
        assert!(!envelope.is_releasing());

        envelope.next_sample();
        assert!(envelope.is_releasing());
    }

    #[test]
    fn zero_maximum_sustain_keeps_a_held_note_sustaining() {
        let mut envelope = Adsr::from_settings(10.0, EnvelopeSettings::new(0.0, 0.0, 0.7, 0.2));
        envelope.note_on();

        for _ in 0..100 {
            envelope.next_sample();
        }

        assert!(!envelope.is_releasing());
        assert_eq!(envelope.current_level, 0.7);
    }

    #[test]
    fn sustain_curvature_shapes_decay_toward_the_end_level() {
        let settings = |curvature| {
            EnvelopeSettings::new(0.0, 0.0, 1.0, 0.2)
                .with_maximum_sustain(1.0)
                .with_sustain_shape(0.0, curvature)
        };
        let mut delayed = Adsr::from_settings(10.0, settings(3.0));
        let mut linear = Adsr::from_settings(10.0, settings(0.0));
        let mut accelerated = Adsr::from_settings(10.0, settings(-3.0));
        for envelope in [&mut delayed, &mut linear, &mut accelerated] {
            envelope.note_on();
            envelope.next_sample(); // Attack -> Decay
            envelope.next_sample(); // Decay -> Sustain
            for _ in 0..6 {
                envelope.next_sample();
            }
        }

        assert!(delayed.current_level > linear.current_level);
        assert!((linear.current_level - 0.5).abs() < 1e-6);
        assert!(accelerated.current_level < linear.current_level);
    }
}
