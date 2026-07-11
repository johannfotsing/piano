use music::event::NoteEvent;
use synth::Synthesizer;

pub struct App {
    synthesizer: Synthesizer,
}

impl App {
    pub fn new(synthesizer: Synthesizer) -> Self {
        Self { synthesizer }
    }

    pub fn handle_event(&mut self, event: NoteEvent) {
        match event {
            NoteEvent::NoteOn { note, velocity } => {
                self.synthesizer.note_on(note, velocity);
            }

            NoteEvent::NoteOff { note } => {
                self.synthesizer.note_off(note);
            }

            NoteEvent::AllNotesOff => {
                // Add later
            }
            NoteEvent::Sustain { pressed } => todo!(),
            NoteEvent::PitchBend { value } => todo!(),
            NoteEvent::ControlChange { controller, value } => todo!(),
            NoteEvent::ProgramChange { program } => todo!(),
            NoteEvent::Expression { value } => todo!(),
            NoteEvent::OctaveShift { offset } => todo!(),
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        self.synthesizer.next_sample()
    }
}
