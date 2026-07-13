use music::event::NoteEvent;

use crate::parse_message;

/// One USB MIDI 1.0 event packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventPacket([u8; 4]);

impl EventPacket {
    pub const fn new(bytes: [u8; 4]) -> Self {
        Self(bytes)
    }

    pub const fn cable_number(self) -> u8 {
        self.0[0] >> 4
    }

    pub const fn code_index_number(self) -> u8 {
        self.0[0] & 0x0f
    }

    /// Converts channel-voice USB packets into the engine's note events.
    pub fn note_event(self) -> Option<NoteEvent> {
        // CIN 0x8 and 0x9 are three-byte note-off and note-on messages.
        match self.code_index_number() {
            0x08 | 0x09 => parse_message(&self.0[1..4]),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use music::{event::NoteEvent, note::Note};

    use super::*;

    #[test]
    fn decodes_usb_note_on() {
        let packet = EventPacket::new([0x19, 0x90, 69, 100]);
        assert_eq!(packet.cable_number(), 1);
        assert_eq!(
            packet.note_event(),
            Some(NoteEvent::NoteOn {
                note: Note::A4,
                velocity: 100
            })
        );
    }

    #[test]
    fn decodes_zero_velocity_as_note_off() {
        let packet = EventPacket::new([0x09, 0x90, 60, 0]);
        assert_eq!(
            packet.note_event(),
            Some(NoteEvent::NoteOff { note: Note::C4 })
        );
    }

    #[test]
    fn ignores_non_note_packets() {
        assert_eq!(EventPacket::new([0x0b, 0xb0, 1, 127]).note_event(), None);
    }
}
