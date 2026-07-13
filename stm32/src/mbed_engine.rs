use music::event::NoteEvent;

use crate::{
    audio::{AudioCommand, AudioModule, AudioOutput},
    control::GpioEvent,
    display::{Display, DisplayCommand, DisplayItem, DisplayTarget, WindowGui},
    midi::{UsbMidiEndpoint, UsbMidiInput},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputError<E> {
    AudioQueueFull,
    Audio(E),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MidiError<E> {
    Usb(E),
    AudioQueueFull,
}

/// Top-level coordinator for the STM32 application.
pub struct MbedEngine<O, D, const BUFFER_SAMPLES: usize, const COMMAND_CAPACITY: usize> {
    audio: AudioModule<O, BUFFER_SAMPLES, COMMAND_CAPACITY>,
    display: Display<D>,
}

impl<O, D, const BUFFER_SAMPLES: usize, const COMMAND_CAPACITY: usize>
    MbedEngine<O, D, BUFFER_SAMPLES, COMMAND_CAPACITY>
where
    O: AudioOutput,
    D: DisplayTarget,
{
    pub fn new(audio: AudioModule<O, BUFFER_SAMPLES, COMMAND_CAPACITY>, display_target: D) -> Self {
        let window = WindowGui {
            selected_instrument: audio.selected_instrument_index(),
            ..WindowGui::default()
        };
        Self {
            audio,
            display: Display::new(display_target, window),
        }
    }

    pub fn initialize(&mut self) -> Result<(), O::Error> {
        self.audio.start()
    }

    pub fn terminate(&mut self) -> Result<(), O::Error> {
        self.audio.stop()
    }

    pub fn process_gpio_event(&mut self, event: GpioEvent) -> Result<(), InputError<O::Error>> {
        match event {
            GpioEvent::Up if !self.display.window().editing => {
                let item = self.display.window().selected_item.previous();
                self.display.send(DisplayCommand::Select(item));
            }
            GpioEvent::Down if !self.display.window().editing => {
                let item = self.display.window().selected_item.next();
                self.display.send(DisplayCommand::Select(item));
            }
            GpioEvent::In => self.display.send(DisplayCommand::SetEditing(true)),
            GpioEvent::Out => self.display.send(DisplayCommand::SetEditing(false)),
            GpioEvent::Select => {
                let editing = !self.display.window().editing;
                self.display.send(DisplayCommand::SetEditing(editing));
            }
            GpioEvent::On => self.audio.start().map_err(InputError::Audio)?,
            GpioEvent::Off => self.audio.stop().map_err(InputError::Audio)?,
            GpioEvent::Reset => {
                self.audio
                    .enqueue(AudioCommand::Reset)
                    .map_err(|_| InputError::AudioQueueFull)?;
                self.display
                    .send(DisplayCommand::Select(DisplayItem::Instrument));
                self.display.send(DisplayCommand::SetEditing(false));
                self.display.send(DisplayCommand::SetInstrument(0));
                self.display.send(DisplayCommand::SetMasterGain(20));
            }
            GpioEvent::KnobIncrement(delta) if self.display.window().editing => {
                self.process_knob(delta)?;
            }
            GpioEvent::Up | GpioEvent::Down | GpioEvent::KnobIncrement(_) => {}
        }
        Ok(())
    }

    pub fn process_note_event(
        &mut self,
        event: NoteEvent,
    ) -> Result<(), InputError<core::convert::Infallible>> {
        self.audio
            .enqueue(AudioCommand::Note(event))
            .map_err(|_| InputError::AudioQueueFull)
    }

    pub fn poll_usb_midi<E>(
        &mut self,
        usb_midi: &mut UsbMidiInput<E>,
    ) -> Result<bool, MidiError<E::Error>>
    where
        E: UsbMidiEndpoint,
    {
        let Some(event) = usb_midi.poll().map_err(MidiError::Usb)? else {
            return Ok(false);
        };
        self.process_note_event(event)
            .map_err(|_| MidiError::AudioQueueFull)?;
        Ok(true)
    }

    pub fn service_audio(&mut self) -> Result<(), O::Error> {
        self.audio.service()
    }

    pub fn render_display(&mut self) -> Result<bool, D::Error> {
        self.display.render_if_dirty()
    }

    pub const fn window(&self) -> &WindowGui {
        self.display.window()
    }

    pub const fn audio_is_running(&self) -> bool {
        self.audio.is_running()
    }

    fn process_knob(&mut self, delta: i16) -> Result<(), InputError<O::Error>> {
        match self.display.window().selected_item {
            DisplayItem::Instrument => {
                let current = self.display.window().selected_instrument as i32;
                let last = self.audio.instrument_count().saturating_sub(1) as i32;
                let selected = (current + i32::from(delta)).clamp(0, last) as usize;
                self.audio
                    .enqueue(AudioCommand::SelectInstrument(selected))
                    .map_err(|_| InputError::AudioQueueFull)?;
                self.display.send(DisplayCommand::SetInstrument(selected));
            }
            DisplayItem::MasterGain => {
                let current = i16::from(self.display.window().master_gain_percent);
                let gain = (current + delta).clamp(0, 100) as u8;
                self.audio
                    .enqueue(AudioCommand::SetMasterGain(f32::from(gain) / 100.0))
                    .map_err(|_| InputError::AudioQueueFull)?;
                self.display.send(DisplayCommand::SetMasterGain(gain));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use engine::App;
    use std::vec;
    use synth::{Instrument, OscillatorAssignment, Synthesizer, Waveform};

    use super::*;

    struct Output;

    impl AudioOutput for Output {
        type Error = ();

        fn start(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
        fn stop(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
        fn write_interleaved_stereo(&mut self, _samples: &[i16]) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    struct Target;

    impl DisplayTarget for Target {
        type Error = ();

        fn draw(&mut self, _window: &WindowGui) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    fn mbed_engine() -> MbedEngine<Output, Target, 8, 8> {
        let app = App::new(
            Synthesizer::new(48_000.0),
            vec![
                Instrument::new("One", vec![OscillatorAssignment::new(Waveform::Sine, 1.0)]),
                Instrument::new("Two", vec![OscillatorAssignment::new(Waveform::Sine, 1.0)]),
            ],
        );
        MbedEngine::new(AudioModule::new(app, Output), Target)
    }

    #[test]
    fn gpio_navigation_updates_display_and_audio_settings() {
        let mut hardware = mbed_engine();
        hardware.process_gpio_event(GpioEvent::In).unwrap();
        hardware
            .process_gpio_event(GpioEvent::KnobIncrement(1))
            .unwrap();
        assert_eq!(hardware.window().selected_instrument, 1);

        hardware.process_gpio_event(GpioEvent::Out).unwrap();
        hardware.process_gpio_event(GpioEvent::Down).unwrap();
        hardware.process_gpio_event(GpioEvent::In).unwrap();
        hardware
            .process_gpio_event(GpioEvent::KnobIncrement(15))
            .unwrap();
        assert_eq!(hardware.window().master_gain_percent, 35);

        hardware.service_audio().unwrap();
    }

    #[test]
    fn power_and_reset_events_control_the_embedded_application() {
        let mut hardware = mbed_engine();

        hardware.process_gpio_event(GpioEvent::On).unwrap();
        assert!(hardware.audio_is_running());
        hardware.process_gpio_event(GpioEvent::Off).unwrap();
        assert!(!hardware.audio_is_running());

        hardware.process_gpio_event(GpioEvent::In).unwrap();
        hardware
            .process_gpio_event(GpioEvent::KnobIncrement(1))
            .unwrap();
        hardware.process_gpio_event(GpioEvent::Reset).unwrap();
        assert_eq!(hardware.window(), &WindowGui::default());
        hardware.service_audio().unwrap();
    }
}
