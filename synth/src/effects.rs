use crate::{Chorus, Flanger, Lfo, LfoWaveform, Reverb};

struct FractionalDelay {
    buffer: Vec<f32>,
    write_index: usize,
}

impl FractionalDelay {
    fn new(max_delay_samples: f32) -> Self {
        let length = max_delay_samples.ceil() as usize + 2;
        Self {
            buffer: vec![0.0; length.max(3)],
            write_index: 0,
        }
    }

    fn read(&self, delay_samples: f32) -> f32 {
        let delay_samples = delay_samples.clamp(0.0, self.buffer.len() as f32 - 2.0);
        let length = self.buffer.len() as f32;
        let position = (self.write_index as f32 - delay_samples).rem_euclid(length);
        let first = position.floor() as usize;
        let second = (first + 1) % self.buffer.len();
        let fraction = position - first as f32;
        self.buffer[first] * (1.0 - fraction) + self.buffer[second] * fraction
    }

    fn write(&mut self, sample: f32) {
        self.buffer[self.write_index] = sample;
        self.write_index = (self.write_index + 1) % self.buffer.len();
    }
}

pub struct ChorusProcessor {
    delay: FractionalDelay,
    lfo: Lfo,
    sample_rate: f32,
    base_delay_seconds: f32,
    depth_seconds: f32,
    mix: f32,
}

impl ChorusProcessor {
    pub fn new(settings: Chorus, sample_rate: f32) -> Self {
        let base_delay_seconds = settings.base_delay_ms() / 1_000.0;
        let depth_seconds = settings.depth_ms() / 1_000.0;
        Self {
            delay: FractionalDelay::new((base_delay_seconds + depth_seconds) * sample_rate),
            lfo: Lfo::new(settings.rate_hz(), sample_rate, LfoWaveform::Sine),
            sample_rate,
            base_delay_seconds,
            depth_seconds,
            mix: settings.mix(),
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let modulation = (self.lfo.next_sample() + 1.0) * 0.5;
        let delay_samples =
            (self.base_delay_seconds + modulation * self.depth_seconds) * self.sample_rate;
        let delayed = self.delay.read(delay_samples);
        self.delay.write(input);
        input * (1.0 - self.mix) + delayed * self.mix
    }
}

pub struct FlangerProcessor {
    delay: FractionalDelay,
    lfo: Lfo,
    sample_rate: f32,
    base_delay_seconds: f32,
    depth_seconds: f32,
    feedback: f32,
    mix: f32,
}

impl FlangerProcessor {
    pub fn new(settings: Flanger, sample_rate: f32) -> Self {
        let base_delay_seconds = settings.base_delay_ms() / 1_000.0;
        let depth_seconds = settings.depth_ms() / 1_000.0;
        Self {
            delay: FractionalDelay::new((base_delay_seconds + depth_seconds) * sample_rate),
            lfo: Lfo::new(settings.rate_hz(), sample_rate, LfoWaveform::Sine),
            sample_rate,
            base_delay_seconds,
            depth_seconds,
            feedback: settings.feedback(),
            mix: settings.mix(),
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let modulation = (self.lfo.next_sample() + 1.0) * 0.5;
        let delay_samples =
            (self.base_delay_seconds + modulation * self.depth_seconds) * self.sample_rate;
        let delayed = self.delay.read(delay_samples);
        self.delay.write(input + delayed * self.feedback);
        input * (1.0 - self.mix) + delayed * self.mix
    }
}

struct CombFilter {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
    damping: f32,
    damped: f32,
}

impl CombFilter {
    fn new(delay_seconds: f32, sample_rate: f32, feedback: f32, damping: f32) -> Self {
        Self {
            buffer: vec![0.0; (delay_seconds * sample_rate).round().max(1.0) as usize],
            index: 0,
            feedback,
            damping,
            damped: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.index];
        self.damped = delayed * (1.0 - self.damping) + self.damped * self.damping;
        self.buffer[self.index] = input + self.damped * self.feedback;
        self.index = (self.index + 1) % self.buffer.len();
        delayed
    }
}

struct AllPassFilter {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
}

impl AllPassFilter {
    fn new(delay_seconds: f32, sample_rate: f32, feedback: f32) -> Self {
        Self {
            buffer: vec![0.0; (delay_seconds * sample_rate).round().max(1.0) as usize],
            index: 0,
            feedback,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer[self.index];
        let output = delayed - input;
        self.buffer[self.index] = input + delayed * self.feedback;
        self.index = (self.index + 1) % self.buffer.len();
        output
    }
}

pub struct ReverbProcessor {
    combs: Vec<CombFilter>,
    all_passes: Vec<AllPassFilter>,
    mix: f32,
}

impl ReverbProcessor {
    pub fn new(settings: Reverb, sample_rate: f32) -> Self {
        let feedback = 0.70 + settings.room_size() * 0.28;
        let combs = [0.0297, 0.0371, 0.0411, 0.0437]
            .into_iter()
            .map(|delay| CombFilter::new(delay, sample_rate, feedback, settings.damping()))
            .collect();
        let all_passes = [0.0051, 0.0126]
            .into_iter()
            .map(|delay| AllPassFilter::new(delay, sample_rate, 0.5))
            .collect();

        Self {
            combs,
            all_passes,
            mix: settings.mix(),
        }
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let mut wet = self
            .combs
            .iter_mut()
            .map(|comb| comb.process(input * 0.2))
            .sum::<f32>()
            / self.combs.len() as f32;
        for all_pass in &mut self.all_passes {
            wet = all_pass.process(wet);
        }
        input * (1.0 - self.mix) + wet * self.mix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chorus_dry_mix_is_transparent() {
        let mut chorus = ChorusProcessor::new(Chorus::new(0.6, 20.0, 5.0, 0.0), 48_000.0);
        assert_eq!(chorus.process(0.75), 0.75);
    }

    #[test]
    fn flanger_dry_mix_is_transparent() {
        let mut flanger = FlangerProcessor::new(Flanger::new(0.2, 1.0, 2.0, 0.5, 0.0), 48_000.0);
        assert_eq!(flanger.process(-0.5), -0.5);
    }

    #[test]
    fn reverb_produces_an_impulse_tail() {
        let mut reverb = ReverbProcessor::new(Reverb::new(0.65, 0.4, 1.0), 48_000.0);
        reverb.process(1.0);
        let has_tail = (0..5_000).any(|_| reverb.process(0.0).abs() > 1e-6);
        assert!(has_tail);
    }
}
