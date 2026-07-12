use music::event::NoteEvent;
use synth::{Instrument, OscillatorAssignment, Synthesizer, Waveform};

pub struct App {
    synthesizer: Synthesizer,
    instruments: Vec<Instrument>,
    selected_instrument: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_change_selects_an_instrument() {
        let mut app = App::new(Synthesizer::new(44_100.0));

        app.handle_event(NoteEvent::ProgramChange { program: 4 });

        assert_eq!(app.selected_instrument().name(), "Warm");
    }

    #[test]
    fn invalid_program_keeps_the_current_instrument() {
        let mut app = App::new(Synthesizer::new(44_100.0));

        app.handle_event(NoteEvent::ProgramChange { program: 99 });

        assert_eq!(app.selected_instrument().name(), "Sine");
    }
}

impl App {
    pub fn new(synthesizer: Synthesizer) -> Self {
        Self {
            synthesizer,
            instruments: vec![
                Instrument::new("Sine", vec![OscillatorAssignment::new(Waveform::Sine, 1.0)]),
                Instrument::new(
                    "Square",
                    vec![OscillatorAssignment::new(Waveform::Square, 1.0)],
                ),
                Instrument::new(
                    "Triangle",
                    vec![OscillatorAssignment::new(Waveform::Triangle, 1.0)],
                ),
                Instrument::new(
                    "Sawtooth",
                    vec![OscillatorAssignment::new(Waveform::Sawtooth, 1.0)],
                ),
                Instrument::new(
                    "Warm",
                    vec![
                        OscillatorAssignment::new(Waveform::Triangle, 0.7),
                        OscillatorAssignment::new(Waveform::Sine, 0.3),
                    ],
                ),
            ],
            selected_instrument: 0,
        }
    }

    pub fn handle_event(&mut self, event: NoteEvent) {
        match event {
            NoteEvent::NoteOn { note, velocity } => {
                let instrument = &self.instruments[self.selected_instrument];
                self.synthesizer.note_on(note, velocity, instrument);
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
            NoteEvent::ProgramChange { program } => {
                self.select_instrument(program as usize);
            }
            NoteEvent::Expression { value } => todo!(),
            NoteEvent::OctaveShift { offset } => todo!(),
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        self.synthesizer.next_sample()
    }

    pub fn instruments(&self) -> &[Instrument] {
        &self.instruments
    }

    pub fn selected_instrument(&self) -> &Instrument {
        &self.instruments[self.selected_instrument]
    }

    /// Selects the instrument used by subsequently created voices.
    /// Returns `false` when `index` is outside the instrument list.
    pub fn select_instrument(&mut self, index: usize) -> bool {
        if index >= self.instruments.len() {
            return false;
        }

        self.selected_instrument = index;
        true
    }
}
