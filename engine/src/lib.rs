#![no_std]

extern crate alloc;

pub mod app;
pub mod config;
pub mod controller;
pub mod event;
pub mod piano;
pub mod recorder;
pub mod state;
pub mod transport;
pub mod voice_manager;

pub use app::App;

pub trait AudioSink {
    fn write_samples(&mut self, samples: &[f32]);
}

pub trait InputSource {
    fn poll_event(&mut self) -> Option<music::event::NoteEvent>;
}
