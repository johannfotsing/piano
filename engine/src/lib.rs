#![no_std]

extern crate alloc;

pub mod app;
pub mod event;

pub use app::App;

pub trait AudioSink {
    fn write_samples(&mut self, samples: &[f32]);
}

pub trait InputSource {
    fn poll_event(&mut self) -> Option<music::event::NoteEvent>;
}
