use music::event::NoteEvent;
use synth::VoiceManager;

pub struct App {
    synth: VoiceManager,
    sample_rate: f32,
}

impl App {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            synth: VoiceManager::new(),
            sample_rate,
        }
    }

    pub fn handle_event(&mut self, event: NoteEvent) {
        match event {
            NoteEvent::NoteOn { note, velocity } => {
                self.synth.note_on(
                    note.midi_number(),
                    note.frequency() as f32,
                    velocity,
                    self.sample_rate,
                );
            }

            NoteEvent::NoteOff { note } => {
                self.synth.note_off(note.midi_number());
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
        self.synth.next_sample()
    }
}
