use core::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    LowPass,
    BandPass,
    HighPass,
}

/// Configuration copied from an instrument into each new voice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilterSettings {
    mode: FilterMode,
    cutoff_hz: f32,
    resonance_q: f32,
}

impl FilterSettings {
    pub fn new(mode: FilterMode, cutoff_hz: f32, resonance_q: f32) -> Self {
        assert!(cutoff_hz.is_finite() && cutoff_hz > 0.0);
        assert!(resonance_q.is_finite() && (0.5..=20.0).contains(&resonance_q));

        Self {
            mode,
            cutoff_hz,
            resonance_q,
        }
    }

    pub fn low_pass(cutoff_hz: f32, resonance_q: f32) -> Self {
        Self::new(FilterMode::LowPass, cutoff_hz, resonance_q)
    }

    pub const fn mode(&self) -> FilterMode {
        self.mode
    }

    pub const fn cutoff_hz(&self) -> f32 {
        self.cutoff_hz
    }

    pub const fn resonance_q(&self) -> f32 {
        self.resonance_q
    }
}

/// Topology-preserving state-variable filter with per-voice state.
pub struct StateVariableFilter {
    mode: FilterMode,
    k: f32,
    a1: f32,
    a2: f32,
    a3: f32,
    integrator_1: f32,
    integrator_2: f32,
}

impl StateVariableFilter {
    pub fn new(settings: FilterSettings, sample_rate: f32) -> Self {
        assert!(sample_rate.is_finite() && sample_rate > 0.0);

        let cutoff_hz = settings.cutoff_hz().clamp(1.0, sample_rate * 0.49);
        let g = libm::tanf(PI * cutoff_hz / sample_rate);
        let k = 1.0 / settings.resonance_q();
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;

        Self {
            mode: settings.mode(),
            k,
            a1,
            a2,
            a3,
            integrator_1: 0.0,
            integrator_2: 0.0,
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let v3 = input - self.integrator_2;
        let band_pass = self.a1 * self.integrator_1 + self.a2 * v3;
        let low_pass = self.integrator_2 + self.a2 * self.integrator_1 + self.a3 * v3;

        self.integrator_1 = 2.0 * band_pass - self.integrator_1;
        self.integrator_2 = 2.0 * low_pass - self.integrator_2;

        match self.mode {
            FilterMode::LowPass => low_pass,
            FilterMode::BandPass => band_pass,
            FilterMode::HighPass => input - self.k * band_pass - low_pass,
        }
    }

    pub fn reset(&mut self) {
        self.integrator_1 = 0.0;
        self.integrator_2 = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_pass_converges_to_dc_input() {
        let mut filter =
            StateVariableFilter::new(FilterSettings::low_pass(1_000.0, 0.707), 48_000.0);
        let mut output = 0.0;

        for _ in 0..4_800 {
            output = filter.process(1.0);
        }

        assert!((output - 1.0).abs() < 1e-3);
    }

    #[test]
    fn low_pass_attenuates_nyquist_signal() {
        let mut filter =
            StateVariableFilter::new(FilterSettings::low_pass(1_000.0, 0.707), 48_000.0);
        let mut output_sum = 0.0;

        for index in 0..2_000 {
            let input = if index % 2 == 0 { 1.0 } else { -1.0 };
            let output = filter.process(input);
            if index >= 1_000 {
                output_sum += output.abs();
            }
        }

        assert!(output_sum / 1_000.0 < 0.01);
    }

    #[test]
    fn every_mode_remains_finite_with_high_resonance() {
        for mode in [
            FilterMode::LowPass,
            FilterMode::BandPass,
            FilterMode::HighPass,
        ] {
            let mut filter =
                StateVariableFilter::new(FilterSettings::new(mode, 8_000.0, 20.0), 48_000.0);

            for index in 0..10_000 {
                let input = if index == 0 { 1.0 } else { 0.0 };
                assert!(filter.process(input).is_finite());
            }
        }
    }
}
