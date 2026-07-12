mod audio;
mod gui;
mod presets;

use std::{path::PathBuf, sync::mpsc};

use eframe::egui;

use gui::PresetEditor;
use presets::PresetBank;

fn main() -> eframe::Result {
    let preset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("presets.xml");
    let bank = PresetBank::load(&preset_path).unwrap_or_else(|error| panic!("{error}"));
    let instruments = bank
        .to_instruments()
        .unwrap_or_else(|error| panic!("{error}"));

    let (midi_sender, midi_receiver) = mpsc::channel();
    let (command_sender, command_receiver) = mpsc::channel();

    let _stream = audio::start_audio(midi_receiver, command_receiver, instruments);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1_050.0, 760.0])
            .with_maximized(true),
        ..Default::default()
    };

    eframe::run_native(
        "OpenRSynth - Desktop",
        options,
        Box::new(move |_creation_context| {
            Ok(Box::new(PresetEditor::new(
                bank,
                preset_path,
                command_sender,
                midi_sender,
            )))
        }),
    )
}
