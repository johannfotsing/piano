use engine::App;
use music::event::NoteEvent;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioCommand {
    Note(NoteEvent),
    SelectInstrument(usize),
    SetMasterGain(f32),
}

/// Codec/SAI boundary implemented by the STM32H747 board support layer.
pub trait AudioOutput {
    type Error;

    fn start(&mut self) -> Result<(), Self::Error>;
    fn stop(&mut self) -> Result<(), Self::Error>;
    fn write_interleaved_stereo(&mut self, samples: &[i16]) -> Result<(), Self::Error>;
}

struct CommandQueue<const CAPACITY: usize> {
    commands: [Option<AudioCommand>; CAPACITY],
    read: usize,
    write: usize,
    len: usize,
}

impl<const CAPACITY: usize> CommandQueue<CAPACITY> {
    const fn new() -> Self {
        Self {
            commands: [None; CAPACITY],
            read: 0,
            write: 0,
            len: 0,
        }
    }

    fn push(&mut self, command: AudioCommand) -> Result<(), AudioCommand> {
        if self.len == CAPACITY {
            return Err(command);
        }
        self.commands[self.write] = Some(command);
        self.write = (self.write + 1) % CAPACITY;
        self.len += 1;
        Ok(())
    }

    fn pop(&mut self) -> Option<AudioCommand> {
        if self.len == 0 {
            return None;
        }
        let command = self.commands[self.read].take();
        self.read = (self.read + 1) % CAPACITY;
        self.len -= 1;
        command
    }
}

/// Owns the cross-platform engine and feeds its samples to the board codec.
pub struct AudioModule<O, const BUFFER_SAMPLES: usize, const COMMAND_CAPACITY: usize> {
    app: App,
    output: O,
    commands: CommandQueue<COMMAND_CAPACITY>,
    buffer: [i16; BUFFER_SAMPLES],
    running: bool,
}

impl<O, const BUFFER_SAMPLES: usize, const COMMAND_CAPACITY: usize>
    AudioModule<O, BUFFER_SAMPLES, COMMAND_CAPACITY>
where
    O: AudioOutput,
{
    pub fn new(app: App, output: O) -> Self {
        assert!(BUFFER_SAMPLES > 0 && BUFFER_SAMPLES % 2 == 0);
        assert!(COMMAND_CAPACITY > 0);
        Self {
            app,
            output,
            commands: CommandQueue::new(),
            buffer: [0; BUFFER_SAMPLES],
            running: false,
        }
    }

    pub fn start(&mut self) -> Result<(), O::Error> {
        self.output.start()?;
        self.running = true;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), O::Error> {
        self.running = false;
        self.buffer.fill(0);
        self.output.stop()
    }

    pub const fn is_running(&self) -> bool {
        self.running
    }

    pub fn enqueue(&mut self, command: AudioCommand) -> Result<(), AudioCommand> {
        self.commands.push(command)
    }

    pub fn instrument_count(&self) -> usize {
        self.app.instruments().len()
    }

    pub const fn selected_instrument_index(&self) -> usize {
        self.app.selected_instrument_index()
    }

    /// Applies pending commands, renders one stereo buffer, and submits it.
    /// This is the operation that a DMA half/full callback will invoke.
    pub fn service(&mut self) -> Result<(), O::Error> {
        while let Some(command) = self.commands.pop() {
            match command {
                AudioCommand::Note(event) => self.app.handle_event(event),
                AudioCommand::SelectInstrument(index) => {
                    self.app.select_instrument(index);
                }
                AudioCommand::SetMasterGain(gain) => self.app.set_master_gain(gain),
            }
        }

        if !self.running {
            return Ok(());
        }

        for frame in self.buffer.chunks_exact_mut(2) {
            let sample = float_to_i16(self.app.next_sample());
            frame[0] = sample;
            frame[1] = sample;
        }
        self.output.write_interleaved_stereo(&self.buffer)
    }
}

fn float_to_i16(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::vec;
    use synth::{Instrument, OscillatorAssignment, Synthesizer, Waveform};

    use super::*;

    #[derive(Default)]
    struct Output {
        running: bool,
        writes: usize,
    }

    impl AudioOutput for Output {
        type Error = ();

        fn start(&mut self) -> Result<(), Self::Error> {
            self.running = true;
            Ok(())
        }

        fn stop(&mut self) -> Result<(), Self::Error> {
            self.running = false;
            Ok(())
        }

        fn write_interleaved_stereo(&mut self, samples: &[i16]) -> Result<(), Self::Error> {
            assert_eq!(samples.len(), 8);
            self.writes += 1;
            Ok(())
        }
    }

    fn app() -> App {
        App::new(
            Synthesizer::new(48_000.0),
            vec![Instrument::new(
                "Test",
                vec![OscillatorAssignment::new(Waveform::Sine, 1.0)],
            )],
        )
    }

    #[test]
    fn only_submits_audio_while_running() {
        let mut audio = AudioModule::<_, 8, 4>::new(app(), Output::default());
        audio.service().unwrap();
        audio.start().unwrap();
        audio.service().unwrap();
        assert!(audio.is_running());
        audio.stop().unwrap();
        assert!(!audio.is_running());
    }

    #[test]
    fn rejects_commands_when_the_fixed_queue_is_full() {
        let mut audio = AudioModule::<_, 8, 1>::new(app(), Output::default());
        assert!(audio.enqueue(AudioCommand::SetMasterGain(0.5)).is_ok());
        assert!(audio.enqueue(AudioCommand::SetMasterGain(0.6)).is_err());
    }
}
