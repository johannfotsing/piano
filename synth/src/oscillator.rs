use core::f32::consts::TAU;

/// Available waveform shapes for the oscillator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}

/// A basic sine wave oscillator.
///
/// Generates samples using a phase accumulator:
///
/// phase += frequency / sample_rate
///
/// sample = sin(phase * 2π)
pub struct Oscillator {
    frequency: f32,
    sample_rate: f32,
    phase: f32,
    waveform: Waveform,
}

impl Oscillator {
    /// Creates a new oscillator using the sine waveform by default.
    pub fn new(frequency: f32, sample_rate: f32) -> Self {
        Self::with_waveform(frequency, sample_rate, Waveform::Sine)
    }

    /// Creates a new oscillator with an explicit waveform.
    pub fn with_waveform(frequency: f32, sample_rate: f32, waveform: Waveform) -> Self {
        Self {
            frequency,
            sample_rate,
            phase: 0.0,
            waveform,
        }
    }

    /// Generates the next audio sample.
    ///
    /// Returns a value between -1.0 and 1.0.
    pub fn next_sample(&mut self) -> f32 {
        let sample = match self.waveform {
          Waveform::Sine => (self.phase * TAU).sin(),
          Waveform::Square => {
              let value = (self.phase * TAU).sin();
              if value >= 0.0 {
                  1.0
              } else {
                  -1.0
              }
          }
          Waveform::Triangle => {
              let value = (self.phase * TAU).sin();
              2.0 * value.abs() - 1.0
          }
          Waveform::Sawtooth => {
              let value = (self.phase * TAU).sin();
              2.0 * value - 1.0
          }
        };

        self.advance_phase();

        sample
    }

    /// Change the oscillator frequency.
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }

    /// Returns the current oscillator frequency.
    pub fn frequency(&self) -> f32 {
        self.frequency
    }

    /// Resets the oscillator phase.
    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    fn advance_phase(&mut self) {
        self.phase += self.frequency / self.sample_rate;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
    }
}