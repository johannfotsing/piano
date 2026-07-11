use music::{event::NoteEvent, note::Note};

#[cfg(feature = "desktop-input")]
use std::{
    error::Error,
    io::{self, Write},
    sync::mpsc::Sender,
};

#[cfg(feature = "desktop-input")]
use midir::{Ignore, MidiInput, MidiInputConnection};

/// Converts a raw MIDI message into an event understood by the piano engine.
///
/// MIDI channel information is intentionally ignored for now because the engine
/// currently treats every input as a single instrument.
pub fn parse_message(message: &[u8]) -> Option<NoteEvent> {
    let (&status, data) = message.split_first()?;

    if status < 0x80 || data.len() < 2 {
        return None;
    }

    let note = Note::from_midi_number(data[0]);
    let velocity = data[1];

    match status & 0xF0 {
        0x80 => Some(NoteEvent::NoteOff { note }),
        0x90 if velocity == 0 => Some(NoteEvent::NoteOff { note }),
        0x90 => Some(NoteEvent::NoteOn { note, velocity }),
        _ => None,
    }
}

#[cfg(feature = "desktop-input")]
fn read_port_selection(port_count: usize) -> Result<usize, Box<dyn Error>> {
    loop {
        print!("Select a MIDI input port [0-{}]: ", port_count - 1);
        io::stdout().flush()?;

        let mut selection = String::new();
        if io::stdin().read_line(&mut selection)? == 0 {
            return Err("standard input closed before a MIDI port was selected".into());
        }

        match selection.trim().parse::<usize>() {
            Ok(index) if index < port_count => return Ok(index),
            _ => eprintln!("Please enter a number from 0 to {}.", port_count - 1),
        }
    }
}

/// Opens an available MIDI input port.
///
/// The returned connection must remain alive while MIDI input is needed. If no
/// device is connected, this returns `Ok(None)` so computer-keyboard input can
/// continue to work normally. A single port is connected automatically; when
/// multiple ports are available, the user is prompted to select one.
#[cfg(feature = "desktop-input")]
pub fn connect_input(
    event_sender: Sender<NoteEvent>,
) -> Result<Option<MidiInputConnection<()>>, Box<dyn Error>> {
    let mut midi_input = MidiInput::new("rust-piano-input")?;
    midi_input.ignore(Ignore::None);

    let ports = midi_input.ports();
    if ports.is_empty() {
        println!("No MIDI input device found; computer keyboard input is still available.");
        return Ok(None);
    }

    println!("Available MIDI input ports:");
    for (index, port) in ports.iter().enumerate() {
        println!("  {index}: {}", midi_input.port_name(port)?);
    }

    let port_index = if ports.len() == 1 {
        0
    } else {
        read_port_selection(ports.len())?
    };
    let port = &ports[port_index];
    let port_name = midi_input.port_name(port)?;
    let connection = midi_input.connect(
        port,
        "rust-piano-midi-reader",
        move |_timestamp, message, _| {
            if let Some(event) = parse_message(message) {
                let _ = event_sender.send(event);
            }
        },
        (),
    )?;

    println!("Connected to MIDI input: {port_name}");
    Ok(Some(connection))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_note_on_on_any_channel() {
        assert_eq!(
            parse_message(&[0x92, 69, 100]),
            Some(NoteEvent::NoteOn {
                note: Note::A4,
                velocity: 100,
            })
        );
    }

    #[test]
    fn parses_note_off() {
        assert_eq!(
            parse_message(&[0x80, 60, 64]),
            Some(NoteEvent::NoteOff { note: Note::C4 })
        );
    }

    #[test]
    fn treats_zero_velocity_note_on_as_note_off() {
        assert_eq!(
            parse_message(&[0x90, 60, 0]),
            Some(NoteEvent::NoteOff { note: Note::C4 })
        );
    }

    #[test]
    fn ignores_unsupported_and_incomplete_messages() {
        assert_eq!(parse_message(&[0xB0, 64, 127]), None);
        assert_eq!(parse_message(&[0x90, 60]), None);
        assert_eq!(parse_message(&[]), None);
    }
}
