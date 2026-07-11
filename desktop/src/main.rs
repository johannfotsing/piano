mod audio;
mod keyboard;

use std::sync::mpsc::{self, Sender};

use engine::App;
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
}

impl ApplicationHandler for DesktopApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.window = Some(
                event_loop
                    .create_window(Window::default_attributes().with_title("Rust Piano"))
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

    let app = App::new(44_100.0);
    let _stream = audio::start_audio(event_receiver, app);

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut desktop_app = DesktopApp {
        event_sender,
        window: None,
    };

    event_loop
        .run_app(&mut desktop_app)
        .expect("Event loop failed");
}
