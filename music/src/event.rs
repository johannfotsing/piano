use crate::note::Note;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoteEvent {
    /// A key/note was pressed
    NoteOn { note: Note, velocity: u8 },

    /// A key/note was released
    NoteOff { note: Note },

    /// Sustain pedal state changed
    Sustain { pressed: bool },

    /// Change pitch continuously (MIDI pitch bend)
    PitchBend { value: i16 },

    /// Change a controller value
    ControlChange { controller: u8, value: u8 },

    /// Change the currently selected instrument/preset
    ProgramChange { program: u8 },

    /// Change the volume/expression level
    Expression { value: u8 },

    /// Change the octave offset of a keyboard layout
    OctaveShift { offset: i8 },

    /// All currently sounding notes should stop
    AllNotesOff,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_and_displays_note_event() {
        let event = NoteEvent::NoteOn {
            note: Note::A4,
            velocity: 100,
        };

        assert!(matches!(event, NoteEvent::NoteOn { .. }));
    }
}
