use std::{collections::HashSet, path::PathBuf, sync::mpsc::Sender};

use eframe::egui;
use music::{event::NoteEvent, note::Note};

use crate::{
    audio::AudioCommand,
    presets::{
        ChorusPreset, EnvelopePreset, FilterModePreset, FilterPreset, FlangerPreset,
        OscillatorPreset, PresetBank, ReverbPreset, TremoloPreset, VibratoPreset, WaveformPreset,
    },
};

pub struct PresetEditor {
    bank: PresetBank,
    preset_path: PathBuf,
    selected: usize,
    command_sender: Sender<AudioCommand>,
    status: String,
    keyboard_notes: HashSet<Note>,
}

impl PresetEditor {
    pub fn new(
        bank: PresetBank,
        preset_path: PathBuf,
        command_sender: Sender<AudioCommand>,
    ) -> Self {
        Self {
            bank,
            preset_path,
            selected: 0,
            command_sender,
            status: "Presets loaded from XML".into(),
            keyboard_notes: HashSet::new(),
        }
    }

    fn select_preset(&mut self, index: usize) {
        if index >= self.bank.presets.len() {
            return;
        }

        if self
            .command_sender
            .send(AudioCommand::SelectInstrument(index))
            .is_ok()
        {
            self.selected = index;
            self.status = format!("Selected {}", self.bank.presets[index].name);
        } else {
            self.status = "Audio thread is unavailable".into();
        }
    }

    fn apply_presets(&mut self) -> Result<(), String> {
        let instruments = self.bank.to_instruments()?;
        self.command_sender
            .send(AudioCommand::ReplaceInstruments(instruments))
            .map_err(|_| "Audio thread is unavailable".to_string())?;
        self.command_sender
            .send(AudioCommand::SelectInstrument(self.selected))
            .map_err(|_| "Audio thread is unavailable".to_string())?;
        Ok(())
    }

    fn apply(&mut self) {
        self.status = match self.apply_presets() {
            Ok(()) => "Changes applied to the synthesizer".into(),
            Err(error) => error,
        };
    }

    fn save(&mut self) {
        self.status = match self
            .apply_presets()
            .and_then(|()| self.bank.save(&self.preset_path))
        {
            Ok(()) => format!("Saved {}", self.preset_path.display()),
            Err(error) => error,
        };
    }

    fn reload(&mut self) {
        self.status = match PresetBank::load(&self.preset_path) {
            Ok(bank) => {
                self.bank = bank;
                self.selected = self.selected.min(self.bank.presets.len() - 1);
                match self.apply_presets() {
                    Ok(()) => format!("Reloaded {}", self.preset_path.display()),
                    Err(error) => error,
                }
            }
            Err(error) => error,
        };
    }

    fn send_note(&mut self, event: NoteEvent) {
        if self
            .command_sender
            .send(AudioCommand::Event(event))
            .is_err()
        {
            self.status = "Audio thread is unavailable".into();
        }
    }

    fn handle_computer_keyboard(&mut self, context: &egui::Context) {
        if context.egui_wants_keyboard_input() {
            for note in self.keyboard_notes.drain().collect::<Vec<_>>() {
                self.send_note(NoteEvent::NoteOff { note });
            }
            return;
        }

        const KEYS: [(egui::Key, Note); 12] = [
            (egui::Key::A, Note::C4),
            (egui::Key::W, Note::CSHARP4),
            (egui::Key::S, Note::D4),
            (egui::Key::E, Note::DSHARP4),
            (egui::Key::D, Note::E4),
            (egui::Key::F, Note::F4),
            (egui::Key::T, Note::FSHARP4),
            (egui::Key::G, Note::G4),
            (egui::Key::Y, Note::GSHARP4),
            (egui::Key::H, Note::A4),
            (egui::Key::U, Note::ASHARP4),
            (egui::Key::J, Note::B4),
        ];

        for (key, note) in KEYS {
            let (pressed, released) =
                context.input(|input| (input.key_pressed(key), input.key_released(key)));
            if pressed && self.keyboard_notes.insert(note) {
                self.send_note(NoteEvent::NoteOn {
                    note,
                    velocity: 100,
                });
            }
            if released && self.keyboard_notes.remove(&note) {
                self.send_note(NoteEvent::NoteOff { note });
            }
        }
    }

    fn show_oscillators(ui: &mut egui::Ui, preset: &mut crate::presets::Preset) {
        ui.heading("Oscillators");
        let mut remove = None;
        for (index, oscillator) in preset.oscillators.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.strong(format!("Oscillator {}:", index + 1));
                ui.label("Waveform");
                egui::ComboBox::from_id_salt(("waveform", index))
                    .selected_text(oscillator.waveform.label())
                    .show_ui(ui, |ui| {
                        for waveform in WaveformPreset::ALL {
                            ui.selectable_value(
                                &mut oscillator.waveform,
                                waveform,
                                waveform.label(),
                            );
                        }
                    });
                ui.label("Gain");
                ui.add(
                    egui::DragValue::new(&mut oscillator.gain)
                        .range(0.0..=2.0)
                        .speed(0.01),
                );
                if ui.button("Remove").clicked() {
                    remove = Some(index);
                }
            });
        }
        if let Some(index) = remove {
            preset.oscillators.remove(index);
        }
        if ui.button("+ Add oscillator").clicked() {
            preset.oscillators.push(OscillatorPreset {
                waveform: WaveformPreset::Sine,
                gain: 0.5,
            });
        }
    }

    fn show_filter(ui: &mut egui::Ui, filter: &mut Option<FilterPreset>) {
        ui.heading("Filter");
        let mut enabled = filter.is_some();
        ui.horizontal(|ui| {
            if ui.checkbox(&mut enabled, "Enabled").changed() {
                *filter = enabled.then_some(FilterPreset {
                    mode: FilterModePreset::LowPass,
                    cutoff_hz: 2_500.0,
                    resonance_q: 0.707,
                });
            }
            if let Some(filter) = filter {
                ui.label("Mode");
                egui::ComboBox::from_id_salt("filter_mode")
                    .selected_text(filter.mode.label())
                    .show_ui(ui, |ui| {
                        for mode in FilterModePreset::ALL {
                            ui.selectable_value(&mut filter.mode, mode, mode.label());
                        }
                    });
                ui.label("Cutoff Hz");
                ui.add(egui::DragValue::new(&mut filter.cutoff_hz).range(20.0..=20_000.0));
                ui.label("Resonance Q");
                ui.add(
                    egui::DragValue::new(&mut filter.resonance_q)
                        .range(0.5..=20.0)
                        .speed(0.01),
                );
            }
        });
    }

    fn show_vibrato(ui: &mut egui::Ui, vibrato: &mut Option<VibratoPreset>) {
        let mut enabled = vibrato.is_some();
        ui.horizontal(|ui| {
            ui.strong("Vibrato:");
            if ui.checkbox(&mut enabled, "Enabled").changed() {
                *vibrato = enabled.then_some(VibratoPreset {
                    rate_hz: 5.0,
                    depth_cents: 12.0,
                });
            }
            if let Some(vibrato) = vibrato {
                ui.label("Rate Hz");
                ui.add(egui::DragValue::new(&mut vibrato.rate_hz).range(0.0..=20.0));
                ui.label("Depth cents");
                ui.add(egui::DragValue::new(&mut vibrato.depth_cents).range(0.0..=200.0));
            }
        });
    }

    fn show_tremolo(ui: &mut egui::Ui, tremolo: &mut Option<TremoloPreset>) {
        let mut enabled = tremolo.is_some();
        ui.horizontal(|ui| {
            ui.strong("Tremolo:");
            if ui.checkbox(&mut enabled, "Enabled").changed() {
                *tremolo = enabled.then_some(TremoloPreset {
                    rate_hz: 4.0,
                    depth: 0.2,
                });
            }
            if let Some(tremolo) = tremolo {
                ui.label("Rate Hz");
                ui.add(egui::DragValue::new(&mut tremolo.rate_hz).range(0.0..=20.0));
                ui.label("Depth");
                ui.add(
                    egui::DragValue::new(&mut tremolo.depth)
                        .range(0.0..=1.0)
                        .speed(0.01),
                );
            }
        });
    }

    fn show_chorus(ui: &mut egui::Ui, chorus: &mut Option<ChorusPreset>) {
        let mut enabled = chorus.is_some();
        ui.horizontal(|ui| {
            ui.strong("Chorus:");
            if ui.checkbox(&mut enabled, "Enabled").changed() {
                *chorus = enabled.then_some(ChorusPreset {
                    rate_hz: 0.6,
                    base_delay_ms: 20.0,
                    depth_ms: 5.0,
                    mix: 0.3,
                });
            }
            if let Some(chorus) = chorus {
                ui.label("Rate Hz");
                ui.add(egui::DragValue::new(&mut chorus.rate_hz).range(0.0..=20.0));
                ui.label("Base delay ms");
                ui.add(egui::DragValue::new(&mut chorus.base_delay_ms).range(0.1..=100.0));
                ui.label("Depth ms");
                ui.add(egui::DragValue::new(&mut chorus.depth_ms).range(0.0..=50.0));
                ui.label("Mix");
                ui.add(
                    egui::DragValue::new(&mut chorus.mix)
                        .range(0.0..=1.0)
                        .speed(0.01),
                );
            }
        });
    }

    fn show_flanger(ui: &mut egui::Ui, flanger: &mut Option<FlangerPreset>) {
        let mut enabled = flanger.is_some();
        ui.horizontal(|ui| {
            ui.strong("Flanger:");
            if ui.checkbox(&mut enabled, "Enabled").changed() {
                *flanger = enabled.then_some(FlangerPreset {
                    rate_hz: 0.2,
                    base_delay_ms: 1.0,
                    depth_ms: 2.0,
                    feedback: 0.5,
                    mix: 0.25,
                });
            }
            if let Some(flanger) = flanger {
                ui.label("Rate Hz");
                ui.add(egui::DragValue::new(&mut flanger.rate_hz).range(0.0..=20.0));
                ui.label("Base delay ms");
                ui.add(egui::DragValue::new(&mut flanger.base_delay_ms).range(0.1..=20.0));
                ui.label("Depth ms");
                ui.add(egui::DragValue::new(&mut flanger.depth_ms).range(0.0..=20.0));
                ui.label("Feedback");
                ui.add(
                    egui::DragValue::new(&mut flanger.feedback)
                        .range(-0.95..=0.95)
                        .speed(0.01),
                );
                ui.label("Mix");
                ui.add(
                    egui::DragValue::new(&mut flanger.mix)
                        .range(0.0..=1.0)
                        .speed(0.01),
                );
            }
        });
    }

    fn show_reverb(ui: &mut egui::Ui, reverb: &mut Option<ReverbPreset>) {
        let mut enabled = reverb.is_some();
        ui.horizontal(|ui| {
            ui.strong("Reverb:");
            if ui.checkbox(&mut enabled, "Enabled").changed() {
                *reverb = enabled.then_some(ReverbPreset {
                    room_size: 0.65,
                    damping: 0.4,
                    mix: 0.2,
                });
            }
            if let Some(reverb) = reverb {
                ui.label("Room size");
                ui.add(
                    egui::DragValue::new(&mut reverb.room_size)
                        .range(0.0..=1.0)
                        .speed(0.01),
                );
                ui.label("Damping");
                ui.add(
                    egui::DragValue::new(&mut reverb.damping)
                        .range(0.0..=1.0)
                        .speed(0.01),
                );
                ui.label("Mix");
                ui.add(
                    egui::DragValue::new(&mut reverb.mix)
                        .range(0.0..=1.0)
                        .speed(0.01),
                );
            }
        });
    }

    fn show_effects(
        ui: &mut egui::Ui,
        vibrato: &mut Option<VibratoPreset>,
        tremolo: &mut Option<TremoloPreset>,
        chorus: &mut Option<ChorusPreset>,
        flanger: &mut Option<FlangerPreset>,
        reverb: &mut Option<ReverbPreset>,
    ) {
        ui.heading("Effects");
        Self::show_vibrato(ui, vibrato);
        Self::show_tremolo(ui, tremolo);
        Self::show_chorus(ui, chorus);
        Self::show_flanger(ui, flanger);
        Self::show_reverb(ui, reverb);
    }

    fn show_envelope(ui: &mut egui::Ui, envelope: &mut EnvelopePreset) {
        ui.heading("Envelope (ADSR)");
        ui.horizontal(|ui| {
            ui.label("Attack (s)");
            ui.add(egui::DragValue::new(&mut envelope.attack_seconds).range(0.0..=10.0));
            ui.label("Decay (s)");
            ui.add(egui::DragValue::new(&mut envelope.decay_seconds).range(0.0..=10.0));
            ui.label("Sustain");
            ui.add(
                egui::DragValue::new(&mut envelope.sustain_level)
                    .range(0.0..=1.0)
                    .speed(0.01),
            );
            ui.label("Release (s)");
            ui.add(egui::DragValue::new(&mut envelope.release_seconds).range(0.0..=20.0));
        });
    }
}

impl eframe::App for PresetEditor {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.handle_computer_keyboard(root.ctx());

        egui::CentralPanel::default().show(root, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Rust Piano Preset Editor");
                if ui.button("Reload XML").clicked() {
                    self.reload();
                }
                if ui.button("Apply").clicked() {
                    self.apply();
                }
                if ui.button("Save XML").clicked() {
                    self.save();
                }
                ui.separator();
                ui.label(&self.status);
            });
            ui.separator();

            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_min_width(160.0);
                    ui.set_max_width(160.0);
                    ui.heading("Presets");
                    let presets: Vec<(usize, String)> = self
                        .bank
                        .presets
                        .iter()
                        .enumerate()
                        .map(|(index, preset)| (index, preset.name.clone()))
                        .collect();
                    for (index, name) in presets {
                        if ui.selectable_label(index == self.selected, name).clicked() {
                            self.select_preset(index);
                        }
                    }
                });

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(preset) = self.bank.presets.get_mut(self.selected) {
                        ui.vertical(|ui| {
                            ui.label("Preset name");
                            ui.text_edit_singleline(&mut preset.name);
                            ui.separator();

                            Self::show_oscillators(ui, preset);
                            ui.separator();
                            Self::show_effects(
                                ui,
                                &mut preset.vibrato,
                                &mut preset.tremolo,
                                &mut preset.chorus,
                                &mut preset.flanger,
                                &mut preset.reverb,
                            );
                            ui.separator();
                            Self::show_filter(ui, &mut preset.filter);
                            ui.separator();
                            Self::show_envelope(ui, &mut preset.envelope);
                        });
                    }
                });
            });
        });
    }
}
