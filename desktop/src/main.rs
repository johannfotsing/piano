mod audio;
mod keyboard;

use std::sync::mpsc::{self, Sender};

use music::event::NoteEvent;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

struct DesktopApp {
    event_sender: Sender<NoteEvent>,
    window: Option<Window>,
    instruments: Vec<&'static str>,
    selected_instrument: usize,
}

impl DesktopApp {
    fn window_title(&self) -> String {
        let instrument = self
            .instruments
            .get(self.selected_instrument)
            .copied()
            .unwrap_or("None");

        format!(
            "Rust Piano — Instrument {}/{}: {} — Select with number keys",
            self.selected_instrument + 1,
            self.instruments.len(),
            instrument,
        )
    }

    fn select_instrument(&mut self, index: usize) -> bool {
        if index >= self.instruments.len() {
            return false;
        }

        if self
            .event_sender
            .send(NoteEvent::ProgramChange {
                program: index as u8,
            })
            .is_err()
        {
            return false;
        }

        self.selected_instrument = index;
        let title = self.window_title();
        if let Some(window) = &self.window {
            window.set_title(&title);
        }

        true
    }
}

impl ApplicationHandler for DesktopApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let title = self.window_title();
            self.window = Some(
                event_loop
                    .create_window(Window::default_attributes().with_title(title))
                    .expect("Failed to create window"),
            );
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self
            .window
            .as_ref()
            .is_none_or(|window| window.id() != window_id)
        {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                let PhysicalKey::Code(key_code) = event.physical_key else {
                    return;
                };

                if key_code == KeyCode::Escape && event.state == ElementState::Pressed {
                    event_loop.exit();
                    return;
                }

                // Ignore OS key-repeat events so a held key owns a single voice.
                if event.repeat {
                    return;
                }

                if event.state == ElementState::Pressed {
                    let instrument = keyboard::key_to_instrument(key_code)
                        .or_else(|| keyboard::logical_key_to_instrument(&event.logical_key));

                    if let Some(index) = instrument {
                        if !self.select_instrument(index) && index < self.instruments.len() {
                            event_loop.exit();
                        }
                        return;
                    }
                }

                let note_event = match event.state {
                    ElementState::Pressed => keyboard::key_to_note(key_code),
                    ElementState::Released => keyboard::key_to_note_off(key_code),
                };

                if let Some(note_event) = note_event {
                    if self.event_sender.send(note_event).is_err() {
                        event_loop.exit();
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let (event_sender, event_receiver) = mpsc::channel();

    let _midi_connection = match midi::connect_input(event_sender.clone()) {
        Ok(connection) => connection,
        Err(error) => {
            eprintln!("Could not initialize MIDI input: {error}");
            None
        }
    };

    let (_stream, instruments) = audio::start_audio(event_receiver);

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut desktop_app = DesktopApp {
        event_sender,
        window: None,
        instruments,
        selected_instrument: 0,
    };

    event_loop
        .run_app(&mut desktop_app)
        .expect("Event loop failed");
}
