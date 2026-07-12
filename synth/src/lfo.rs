use core::f32::consts::TAU;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LfoWaveform {
    Sine,
    Triangle,
}

/// A low-frequency oscillator producing bipolar modulation in `[-1.0, 1.0]`.
pub struct Lfo {
    frequency: f32,
    sample_rate: f32,
    phase: f32,
    waveform: LfoWaveform,
}

impl Lfo {
    pub fn new(frequency: f32, sample_rate: f32, waveform: LfoWaveform) -> Self {
        assert!(frequency.is_finite() && frequency >= 0.0);
        assert!(sample_rate.is_finite() && sample_rate > 0.0);

        Self {
            frequency,
            sample_rate,
            phase: 0.0,
            waveform,
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let sample = match self.waveform {
            LfoWaveform::Sine => libm::sinf(self.phase * TAU),
            LfoWaveform::Triangle => 1.0 - 4.0 * (self.phase - 0.5).abs(),
        };

        self.phase = (self.phase + self.frequency / self.sample_rate) % 1.0;
        sample
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        assert!(frequency.is_finite() && frequency >= 0.0);
        self.frequency = frequency;
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_lfo_produces_one_cycle() {
        let mut lfo = Lfo::new(1.0, 4.0, LfoWaveform::Sine);
        let samples = [
            lfo.next_sample(),
            lfo.next_sample(),
            lfo.next_sample(),
            lfo.next_sample(),
        ];

        assert!(samples[0].abs() < 1e-6);
        assert!((samples[1] - 1.0).abs() < 1e-6);
        assert!(samples[2].abs() < 1e-6);
        assert!((samples[3] + 1.0).abs() < 1e-6);
    }

    #[test]
    fn triangle_lfo_stays_bipolar() {
        let mut lfo = Lfo::new(1.0, 4.0, LfoWaveform::Triangle);

        for _ in 0..8 {
            assert!((-1.0..=1.0).contains(&lfo.next_sample()));
        }
    }
}
