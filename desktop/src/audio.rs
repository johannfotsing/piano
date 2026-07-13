use std::sync::mpsc::Receiver;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use engine::App;
use music::event::NoteEvent;
use synth::{Instrument, Synthesizer};

pub enum AudioCommand {
    Event(NoteEvent),
    SelectInstrument(usize),
    ReplaceInstruments(Vec<Instrument>),
}

pub fn start_audio(
    midi_receiver: Receiver<NoteEvent>,
    command_receiver: Receiver<AudioCommand>,
    instruments: Vec<Instrument>,
) -> cpal::Stream {
    let host = cpal::default_host();

    let device = host.default_output_device().expect("No output device");

    let supported_config = device.default_output_config().expect("No default config");

    let config: cpal::StreamConfig = supported_config.clone().into();

    let channels = config.channels as usize;

    // Use the rate selected by the actual audio device.
    let sample_rate = config.sample_rate as f32;
    let synthesizer = Synthesizer::new(sample_rate);
    let mut app = App::new(synthesizer, instruments);

    let stream = match supported_config.sample_format() {
        cpal::SampleFormat::F32 => {
            device.build_output_stream(
                config,
                move |buffer: &mut [f32], _| {
                    // At every audio callback, we first handle all pending events and commands, then generate the next audio samples.
                    // MIDI events are handled first to ensure that the synthesizer state is updated before generating audio samples.
                    // GUI events are handled after MIDI events to ensure that the synthesizer state is updated before generating audio samples.

                    //
                    // Handle every pending event
                    //
                    while let Ok(event) = midi_receiver.try_recv() {
                        app.handle_event(event);
                    }
                    while let Ok(command) = command_receiver.try_recv() {
                        match command {
                            AudioCommand::Event(event) => app.handle_event(event),
                            AudioCommand::SelectInstrument(index) => {
                                app.select_instrument(index);
                            }
                            AudioCommand::ReplaceInstruments(instruments) => {
                                app.replace_instruments(instruments);
                            }
                        }
                    }

                    //
                    // Generate audio
                    //
                    for frame in buffer.chunks_mut(channels) {
                        let sample = app.next_sample();

                        for channel in frame {
                            *channel = sample;
                        }
                    }
                },
                |err| eprintln!("{err}"),
                None,
            )
        }

        _ => unimplemented!(),
    }
    .unwrap();

    stream.play().unwrap();

    stream
}
