#![cfg_attr(not(feature = "desktop-input"), no_std)]

use music::{event::NoteEvent, note::Note};

pub mod usb;

#[cfg(feature = "desktop-input")]
use std::{error::Error, sync::mpsc::Sender};

#[cfg(feature = "desktop-input")]
use midir::{Ignore, MidiInput, MidiInputConnection};

#[cfg(feature = "desktop-input")]
pub type InputConnection = MidiInputConnection<()>;

/// Converts a raw MIDI message into an event understood by the OpenRSynth engine.
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

/// Returns the display names of all currently available MIDI input ports.
#[cfg(feature = "desktop-input")]
pub fn input_ports() -> Result<Vec<String>, Box<dyn Error>> {
    let midi_input = MidiInput::new("openrsynth-input-list")?;
    midi_input
        .ports()
        .iter()
        .map(|port| midi_input.port_name(port).map_err(Into::into))
        .collect()
}

/// Opens the MIDI input port at `port_index`.
///
/// The returned connection must remain alive while MIDI input is needed.
#[cfg(feature = "desktop-input")]
pub fn connect_input_port(
    port_index: usize,
    event_sender: Sender<NoteEvent>,
) -> Result<InputConnection, Box<dyn Error>> {
    let mut midi_input = MidiInput::new("openrsynth-input")?;
    midi_input.ignore(Ignore::None);

    let ports = midi_input.ports();
    let port = ports.get(port_index).ok_or_else(|| {
        format!(
            "MIDI input port {port_index} is unavailable; {} ports were found",
            ports.len()
        )
    })?;
    let connection = midi_input.connect(
        port,
        "openrsynth-midi-reader",
        move |_timestamp, message, _| {
            if let Some(event) = parse_message(message) {
                let _ = event_sender.send(event);
            }
        },
        (),
    )?;

    Ok(connection)
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
