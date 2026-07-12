#![no_std]

extern crate alloc;

pub mod effects;
pub mod envelope;
pub mod filter;
pub mod instrument;
pub mod lfo;
pub mod oscillator;
pub mod synthesizer;
pub mod voice;
pub mod voice_manager;

pub use effects::{ChorusProcessor, FlangerProcessor, ReverbProcessor};
pub use envelope::{Adsr, EnvelopeSettings};
pub use filter::{FilterMode, FilterSettings, StateVariableFilter};
pub use instrument::{
    Chorus, Flanger, Hammer, Instrument, OscillatorAssignment, Reverb, Tremolo, Vibrato,
};
pub use lfo::{Lfo, LfoWaveform};
pub use oscillator::{Oscillator, Waveform};
pub use synthesizer::Synthesizer;
pub use voice::Voice;
pub use voice_manager::VoiceManager;
