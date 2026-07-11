use music::{event::NoteEvent, note::Note};
use winit::keyboard::KeyCode;

pub fn key_to_note(key: KeyCode) -> Option<NoteEvent> {
    match key {
        KeyCode::KeyA => Some(NoteEvent::NoteOn {
            note: Note::C4, // C4
            velocity: 100,
        }),

        KeyCode::KeyW => Some(NoteEvent::NoteOn {
            note: Note::CSHARP4, // C#4
            velocity: 100,
        }),

        KeyCode::KeyS => Some(NoteEvent::NoteOn {
            note: Note::D4, // D4
            velocity: 100,
        }),

        KeyCode::KeyE => Some(NoteEvent::NoteOn {
            note: Note::DSHARP4, // D#4
            velocity: 100,
        }),

        KeyCode::KeyD => Some(NoteEvent::NoteOn {
            note: Note::E4, // E4
            velocity: 100,
        }),

        KeyCode::KeyF => Some(NoteEvent::NoteOn {
            note: Note::F4, // F4
            velocity: 100,
        }),

        KeyCode::KeyT => Some(NoteEvent::NoteOn {
            note: Note::FSHARP4, // F#4
            velocity: 100,
        }),

        KeyCode::KeyG => Some(NoteEvent::NoteOn {
            note: Note::G4, // G4
            velocity: 100,
        }),

        KeyCode::KeyY => Some(NoteEvent::NoteOn {
            note: Note::GSHARP4, // G#4
            velocity: 100,
        }),

        KeyCode::KeyH => Some(NoteEvent::NoteOn {
            note: Note::A4, // A4
            velocity: 100,
        }),

        KeyCode::KeyU => Some(NoteEvent::NoteOn {
            note: Note::ASHARP4, // A#4
            velocity: 100,
        }),

        KeyCode::KeyJ => Some(NoteEvent::NoteOn {
            note: Note::B4, // B4
            velocity: 100,
        }),

        _ => None,
    }
}

pub fn key_to_note_off(key: KeyCode) -> Option<NoteEvent> {
    match key {
        KeyCode::KeyA => Some(NoteEvent::NoteOff { note: Note::C4 }),

        KeyCode::KeyW => Some(NoteEvent::NoteOff {
            note: Note::CSHARP4,
        }),

        KeyCode::KeyS => Some(NoteEvent::NoteOff { note: Note::D4 }),

        KeyCode::KeyE => Some(NoteEvent::NoteOff {
            note: Note::DSHARP4,
        }),

        KeyCode::KeyD => Some(NoteEvent::NoteOff { note: Note::E4 }),

        KeyCode::KeyF => Some(NoteEvent::NoteOff { note: Note::F4 }),

        KeyCode::KeyT => Some(NoteEvent::NoteOff {
            note: Note::FSHARP4,
        }),

        KeyCode::KeyG => Some(NoteEvent::NoteOff { note: Note::G4 }),

        KeyCode::KeyY => Some(NoteEvent::NoteOff {
            note: Note::GSHARP4,
        }),

        KeyCode::KeyH => Some(NoteEvent::NoteOff { note: Note::A4 }),

        KeyCode::KeyU => Some(NoteEvent::NoteOff {
            note: Note::ASHARP4,
        }),

        KeyCode::KeyJ => Some(NoteEvent::NoteOff { note: Note::B4 }),

        _ => None,
    }
}
