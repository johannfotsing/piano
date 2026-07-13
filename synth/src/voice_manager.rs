use alloc::vec::Vec;
use core::num::NonZeroUsize;

use crate::{Instrument, Voice};
use music::note::Note;

pub const DEFAULT_MAX_POLYPHONY: usize = 16;

/// Selects which voice is removed when bounded polyphony is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceStealingPolicy {
    /// Remove the voice that has been alive the longest.
    Oldest,
    /// Prefer the oldest voice already in its release stage, then fall back to
    /// the oldest voice overall.
    OldestReleased,
}

/// Controls whether every voice is mixed or the mixer enforces bounded polyphony.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceMixingPolicy {
    Unlimited,
    Limited {
        max_voices: NonZeroUsize,
        stealing: VoiceStealingPolicy,
    },
}

impl VoiceMixingPolicy {
    pub fn limited(max_voices: usize, stealing: VoiceStealingPolicy) -> Self {
        Self::Limited {
            max_voices: NonZeroUsize::new(max_voices).expect("maximum polyphony must be non-zero"),
            stealing,
        }
    }
}

impl Default for VoiceMixingPolicy {
    fn default() -> Self {
        Self::limited(DEFAULT_MAX_POLYPHONY, VoiceStealingPolicy::OldestReleased)
    }
}

pub struct VoiceManager {
    voices: Vec<Voice>,
    master_gain: f32,
    mixing_policy: VoiceMixingPolicy,
}

impl VoiceManager {
    pub fn new() -> Self {
        let mixing_policy = VoiceMixingPolicy::default();
        Self {
            voices: Vec::with_capacity(DEFAULT_MAX_POLYPHONY),
            master_gain: 0.2,
            mixing_policy,
        }
    }

    pub fn note_on(&mut self, note: Note, velocity: u8, sample_rate: f32, instrument: &Instrument) {
        self.cleanup();
        self.make_room_for_voice();
        self.voices
            .push(Voice::new(note, velocity, sample_rate, instrument));
    }

    pub fn note_off(&mut self, note: Note) {
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
        mixed_sample * self.master_gain
    }

    fn cleanup(&mut self) {
        self.voices.retain(|v| !v.is_finished());
    }

    pub fn set_master_gain(&mut self, gain: f32) {
        self.master_gain = gain.clamp(0.0, 1.0);
    }

    pub fn set_mixing_policy(&mut self, policy: VoiceMixingPolicy) {
        if let VoiceMixingPolicy::Limited { max_voices, .. } = policy {
            self.voices
                .reserve(max_voices.get().saturating_sub(self.voices.len()));
        }
        self.mixing_policy = policy;
        self.enforce_voice_limit();
    }

    pub const fn mixing_policy(&self) -> VoiceMixingPolicy {
        self.mixing_policy
    }

    pub fn active_voice_count(&self) -> usize {
        self.voices.len()
    }

    fn make_room_for_voice(&mut self) {
        let VoiceMixingPolicy::Limited {
            max_voices,
            stealing,
        } = self.mixing_policy
        else {
            return;
        };

        while self.voices.len() >= max_voices.get() {
            self.steal_voice(stealing);
        }
    }

    fn enforce_voice_limit(&mut self) {
        let VoiceMixingPolicy::Limited {
            max_voices,
            stealing,
        } = self.mixing_policy
        else {
            return;
        };

        while self.voices.len() > max_voices.get() {
            self.steal_voice(stealing);
        }
    }

    fn steal_voice(&mut self, policy: VoiceStealingPolicy) {
        let index = match policy {
            VoiceStealingPolicy::Oldest => 0,
            VoiceStealingPolicy::OldestReleased => self
                .voices
                .iter()
                .position(Voice::is_releasing)
                .unwrap_or(0),
        };
        self.voices.remove(index);
    }
}

impl Default for VoiceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::{OscillatorAssignment, Waveform};

    fn instrument() -> Instrument {
        Instrument::new("Test", vec![OscillatorAssignment::new(Waveform::Sine, 1.0)])
    }

    #[test]
    fn unlimited_mixing_keeps_every_voice() {
        let mut manager = VoiceManager::new();
        manager.set_mixing_policy(VoiceMixingPolicy::Unlimited);
        let instrument = instrument();
        for note in [Note::C4, Note::D4, Note::E4] {
            manager.note_on(note, 100, 48_000.0, &instrument);
        }

        assert_eq!(manager.active_voice_count(), 3);
    }

    #[test]
    fn bounded_released_first_mixing_is_the_default() {
        let manager = VoiceManager::new();

        assert_eq!(
            manager.mixing_policy(),
            VoiceMixingPolicy::limited(DEFAULT_MAX_POLYPHONY, VoiceStealingPolicy::OldestReleased,)
        );
    }

    #[test]
    fn limited_mixing_never_exceeds_maximum_polyphony() {
        let mut manager = VoiceManager::new();
        manager.set_mixing_policy(VoiceMixingPolicy::limited(2, VoiceStealingPolicy::Oldest));
        let instrument = instrument();
        for note in [Note::C4, Note::D4, Note::E4] {
            manager.note_on(note, 100, 48_000.0, &instrument);
        }

        assert_eq!(manager.active_voice_count(), 2);
        assert_eq!(manager.voices[0].note(), Note::D4);
        assert_eq!(manager.voices[1].note(), Note::E4);
    }

    #[test]
    fn released_voices_are_stolen_before_held_voices() {
        let mut manager = VoiceManager::new();
        manager.set_mixing_policy(VoiceMixingPolicy::limited(
            2,
            VoiceStealingPolicy::OldestReleased,
        ));
        let instrument = instrument();
        manager.note_on(Note::C4, 100, 48_000.0, &instrument);
        manager.note_on(Note::D4, 100, 48_000.0, &instrument);
        manager.note_off(Note::C4);
        manager.note_on(Note::E4, 100, 48_000.0, &instrument);

        assert_eq!(manager.active_voice_count(), 2);
        assert_eq!(manager.voices[0].note(), Note::D4);
        assert_eq!(manager.voices[1].note(), Note::E4);
    }
}
