pub mod envelope;
pub mod instrument;
pub mod lfo;
pub mod oscillator;
pub mod synthesizer;
pub mod voice;
pub mod voice_manager;

pub use envelope::Adsr;
pub use instrument::{Instrument, OscillatorAssignment, Tremolo, Vibrato};
pub use lfo::{Lfo, LfoWaveform};
pub use oscillator::{Oscillator, Waveform};
pub use synthesizer::Synthesizer;
pub use voice::Voice;
pub use voice_manager::VoiceManager;
