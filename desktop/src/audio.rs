use std::sync::mpsc::Receiver;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use engine::App;
use music::event::NoteEvent;

pub fn start_audio(receiver: Receiver<NoteEvent>, mut app: App) -> cpal::Stream {
    let host = cpal::default_host();

    let device = host.default_output_device().expect("No output device");

    let supported_config = device.default_output_config().expect("No default config");

    let config: cpal::StreamConfig = supported_config.clone().into();

    let channels = config.channels as usize;

    let stream = match supported_config.sample_format() {
        cpal::SampleFormat::F32 => {
            device.build_output_stream(
                config,
                move |buffer: &mut [f32], _| {
                    //
                    // Handle every pending event
                    //
                    while let Ok(event) = receiver.try_recv() {
                        app.handle_event(event);
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
