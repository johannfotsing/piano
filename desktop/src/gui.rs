use std::{
    collections::{BTreeMap, HashSet},
    ops::RangeInclusive,
    path::PathBuf,
    sync::mpsc::Sender,
};

use eframe::egui;
use music::{event::NoteEvent, note::Note};

use crate::{
    audio::AudioCommand,
    presets::{
        ChorusPreset, EnvelopePreset, FilterModePreset, FilterPreset, FlangerPreset, HammerPreset,
        OscillatorPreset, PluckPreset, Preset, PresetBank, ReverbPreset, TremoloPreset,
        VibratoPreset, WaveformPreset,
    },
};

pub struct PresetEditor {
    bank: PresetBank,
    last_applied_bank: PresetBank,
    preset_path: PathBuf,
    selected: usize,
    command_sender: Sender<AudioCommand>,
    midi_sender: Sender<NoteEvent>,
    midi_ports: Vec<String>,
    selected_midi_port: Option<usize>,
    midi_connection: Option<midi::InputConnection>,
    show_midi_setup: bool,
    midi_status: String,
    status: String,
    master_gain: f32,
    keyboard_notes: HashSet<Note>,
    pending_preset_deletion: Option<usize>,
}

impl PresetEditor {
    pub fn new(
        bank: PresetBank,
        preset_path: PathBuf,
        command_sender: Sender<AudioCommand>,
        midi_sender: Sender<NoteEvent>,
    ) -> Self {
        let (midi_ports, midi_status) = match midi::input_ports() {
            Ok(ports) if ports.is_empty() => (ports, "No MIDI input devices found".into()),
            Ok(ports) => (ports, "Select a MIDI input device".into()),
            Err(error) => (Vec::new(), format!("Could not list MIDI inputs: {error}")),
        };
        let selected_midi_port = (!midi_ports.is_empty()).then_some(0);

        let last_applied_bank = bank.clone();
        Self {
            bank,
            last_applied_bank,
            preset_path,
            selected: 0,
            command_sender,
            midi_sender,
            midi_ports,
            selected_midi_port,
            midi_connection: None,
            show_midi_setup: true,
            midi_status,
            status: "Presets loaded from XML".into(),
            master_gain: 0.2,
            keyboard_notes: HashSet::new(),
            pending_preset_deletion: None,
        }
    }

    fn refresh_midi_ports(&mut self) {
        match midi::input_ports() {
            Ok(ports) => {
                self.midi_ports = ports;
                self.selected_midi_port = (!self.midi_ports.is_empty()).then_some(0);
                self.midi_status = if self.midi_ports.is_empty() {
                    "No MIDI input devices found".into()
                } else {
                    "Select a MIDI input device".into()
                };
            }
            Err(error) => {
                self.midi_ports.clear();
                self.selected_midi_port = None;
                self.midi_status = format!("Could not list MIDI inputs: {error}");
            }
        }
    }

    fn connect_midi(&mut self) {
        let Some(index) = self.selected_midi_port else {
            return;
        };
        let name = self
            .midi_ports
            .get(index)
            .cloned()
            .unwrap_or_else(|| format!("Port {index}"));

        match midi::connect_input_port(index, self.midi_sender.clone()) {
            Ok(connection) => {
                self.midi_connection = Some(connection);
                self.midi_status = format!("Connected to {name}");
                self.show_midi_setup = false;
            }
            Err(error) => {
                let message = format!("Could not connect to {name}: {error}");
                self.refresh_midi_ports();
                self.midi_status = message;
            }
        }
    }

    fn show_midi_dialog(&mut self, context: &egui::Context) {
        if !self.show_midi_setup {
            return;
        }

        let mut connect = false;
        let mut refresh = false;
        let mut skip = false;
        egui::Window::new("MIDI input")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(context, |ui| {
                ui.label("Choose the MIDI input device to use with OpenRSynth.");
                ui.add_space(6.0);

                let selected_text = self
                    .selected_midi_port
                    .and_then(|index| self.midi_ports.get(index))
                    .map(String::as_str)
                    .unwrap_or("No device available");
                egui::ComboBox::from_id_salt("midi_input_port")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        for (index, name) in self.midi_ports.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_midi_port, Some(index), name);
                        }
                    });

                ui.label(&self.midi_status);
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("Refresh").clicked() {
                        refresh = true;
                    }
                    if ui
                        .add_enabled(
                            self.selected_midi_port.is_some(),
                            egui::Button::new("Connect"),
                        )
                        .clicked()
                    {
                        connect = true;
                    }
                    if ui.button("Continue without MIDI").clicked() {
                        skip = true;
                    }
                });
            });

        if refresh {
            self.refresh_midi_ports();
        }
        if connect {
            self.connect_midi();
        }
        if skip {
            self.show_midi_setup = false;
            self.midi_status = "MIDI input disabled".into();
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
        self.last_applied_bank = self.bank.clone();
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

    fn add_preset(&mut self) {
        let group = self
            .bank
            .presets
            .get(self.selected)
            .map(|preset| preset.group.clone())
            .unwrap_or_else(|| "General".into());
        let mut suffix = self.bank.presets.len() + 1;
        let name = loop {
            let candidate = format!("New Preset {suffix}");
            if self
                .bank
                .presets
                .iter()
                .all(|preset| preset.name != candidate)
            {
                break candidate;
            }
            suffix += 1;
        };
        let insert_at = self
            .bank
            .presets
            .iter()
            .rposition(|preset| preset.group == group)
            .map_or(self.bank.presets.len(), |index| index + 1);
        self.bank.presets.insert(
            insert_at,
            Preset {
                name,
                group,
                oscillators: vec![OscillatorPreset {
                    waveform: WaveformPreset::Sine,
                    gain: 0.8,
                    frequency_ratio: 1.0,
                    detune_cents: 0.0,
                    decay_seconds: 60.0,
                    velocity_sensitivity: 0.0,
                }],
                hammer: None,
                pluck: None,
                filter: None,
                vibrato: None,
                tremolo: None,
                chorus: None,
                flanger: None,
                reverb: None,
                envelope: EnvelopePreset::default(),
            },
        );
        self.selected = insert_at;
        self.status = match self.apply_presets() {
            Ok(()) => "Preset added".into(),
            Err(error) => error,
        };
    }

    fn delete_preset(&mut self, index: usize) {
        if self.bank.presets.len() <= 1 || index >= self.bank.presets.len() {
            self.status = "A preset bank must contain at least one preset".into();
            return;
        }
        let name = self.bank.presets[index].name.clone();
        self.bank.presets.remove(index);
        self.selected = if self.selected > index {
            self.selected - 1
        } else {
            self.selected.min(self.bank.presets.len() - 1)
        };
        self.status = match self.apply_presets() {
            Ok(()) => format!("Deleted {name}"),
            Err(error) => error,
        };
    }

    fn show_delete_confirmation(&mut self, context: &egui::Context) {
        let Some(index) = self.pending_preset_deletion else {
            return;
        };
        let name = self
            .bank
            .presets
            .get(index)
            .map(|preset| preset.name.as_str())
            .unwrap_or("this preset");
        let mut confirm = false;
        let mut cancel = false;
        egui::Window::new("Delete preset?")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(context, |ui| {
                ui.label(format!("Delete “{name}”? This cannot be undone."));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                    if ui.button("Delete").clicked() {
                        confirm = true;
                    }
                });
            });
        if cancel {
            self.pending_preset_deletion = None;
        } else if confirm {
            self.pending_preset_deletion = None;
            self.delete_preset(index);
        }
    }

    fn knob(
        ui: &mut egui::Ui,
        value: &mut f32,
        range: RangeInclusive<f32>,
        speed: f32,
        decimals: usize,
    ) -> egui::Response {
        let minimum = *range.start();
        let maximum = *range.end();
        let size = egui::vec2(64.0, 68.0);
        let (rect, mut response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
        let input_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 18.0));
        let input_response = ui
            .scope(|ui| {
                ui.style_mut()
                    .text_styles
                    .insert(egui::TextStyle::Body, egui::FontId::proportional(15.0));
                ui.put(
                    input_rect,
                    egui::DragValue::new(value)
                        .range(range.clone())
                        .speed(speed)
                        .fixed_decimals(decimals),
                )
            })
            .inner;
        let knob_rect =
            egui::Rect::from_min_max(egui::pos2(rect.left(), rect.top() + 20.0), rect.max);
        let start_id = response.id.with("drag_start");
        if response.drag_started() {
            ui.memory_mut(|memory| memory.data.insert_temp(start_id, *value));
        }
        if response.dragged() {
            let start = ui
                .memory(|memory| memory.data.get_temp::<f32>(start_id))
                .unwrap_or(*value);
            let next = (start - response.drag_delta().y * speed).clamp(minimum, maximum);
            if next != *value {
                *value = next;
                response.mark_changed();
            }
        }

        let normalized = ((*value - minimum) / (maximum - minimum)).clamp(0.0, 1.0);
        let center = knob_rect.center();
        let radius = 21.0;
        let start_angle = std::f32::consts::PI * 0.75;
        let sweep = std::f32::consts::PI * 1.5;
        let point_at = |angle: f32, radius: f32| {
            center + egui::vec2(angle.cos() * radius, angle.sin() * radius)
        };
        let arc = |fraction: f32| {
            let segments = (40.0 * fraction).ceil().max(1.0) as usize;
            (0..=segments)
                .map(|index| {
                    point_at(
                        start_angle + sweep * fraction * index as f32 / segments as f32,
                        radius,
                    )
                })
                .collect::<Vec<_>>()
        };
        let painter = ui.painter_at(knob_rect);
        painter.add(egui::Shape::line(
            arc(1.0),
            egui::Stroke::new(4.0, ui.visuals().widgets.inactive.bg_stroke.color),
        ));
        if normalized > 0.0 {
            painter.add(egui::Shape::line(
                arc(normalized),
                egui::Stroke::new(4.0, ui.visuals().selection.bg_fill),
            ));
        }
        painter.circle_filled(center, 16.0, ui.visuals().widgets.inactive.bg_fill);
        painter.circle_stroke(
            center,
            16.0,
            egui::Stroke::new(1.0, ui.visuals().widgets.inactive.fg_stroke.color),
        );
        let pointer_angle = start_angle + sweep * normalized;
        painter.line_segment(
            [center, point_at(pointer_angle, 13.0)],
            egui::Stroke::new(2.5, ui.visuals().widgets.active.fg_stroke.color),
        );
        response = response.on_hover_text("Drag vertically to adjust");
        response.union(input_response)
    }

    fn knob_item(
        ui: &mut egui::Ui,
        label: &str,
        value: &mut f32,
        range: RangeInclusive<f32>,
        speed: f32,
        decimals: usize,
    ) {
        Self::setting_item(ui, label, |ui| {
            Self::knob(ui, value, range, speed, decimals);
        });
    }

    fn paint_waveform_preview(ui: &mut egui::Ui, waveform: WaveformPreset) {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(82.0, 34.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);
        painter.line_segment(
            [
                egui::pos2(rect.left() + 3.0, rect.center().y),
                egui::pos2(rect.right() - 3.0, rect.center().y),
            ],
            egui::Stroke::new(0.5, ui.visuals().widgets.noninteractive.bg_stroke.color),
        );
        let points = (0..=80)
            .map(|index| {
                let normalized = index as f32 / 80.0;
                let phase = normalized * 2.0;
                let value = match waveform {
                    WaveformPreset::Sine => (std::f32::consts::TAU * phase).sin(),
                    WaveformPreset::Square => {
                        if (std::f32::consts::TAU * phase).sin() >= 0.0 {
                            1.0
                        } else {
                            -1.0
                        }
                    }
                    WaveformPreset::Triangle => {
                        2.0 / std::f32::consts::PI * (std::f32::consts::TAU * phase).sin().asin()
                    }
                    WaveformPreset::Sawtooth => 2.0 * (phase - (phase + 0.5).floor()),
                };
                egui::pos2(
                    rect.left() + 3.0 + normalized * (rect.width() - 6.0),
                    rect.center().y - value * (rect.height() * 0.38),
                )
            })
            .collect();
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(1.5, ui.visuals().selection.bg_fill),
        ));
    }

    fn paint_filter_mode_preview(ui: &mut egui::Ui, mode: FilterModePreset) {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(82.0, 34.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);
        let points = (0..=80)
            .map(|index| {
                let normalized = index as f32 / 80.0;
                let ratio = 10.0_f32.powf((normalized - 0.5) * 3.0);
                let denominator = ((1.0 - ratio * ratio).powi(2) + ratio.powi(2)).sqrt();
                let numerator = match mode {
                    FilterModePreset::LowPass => 1.0,
                    FilterModePreset::BandPass => ratio,
                    FilterModePreset::HighPass => ratio * ratio,
                };
                let db = (20.0 * (numerator / denominator.max(1e-6)).max(1e-6).log10())
                    .clamp(-36.0, 6.0);
                egui::pos2(
                    rect.left() + 3.0 + normalized * (rect.width() - 6.0),
                    rect.top() + 3.0 + (6.0 - db) / 42.0 * (rect.height() - 6.0),
                )
            })
            .collect();
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(1.5, ui.visuals().selection.bg_fill),
        ));
    }

    fn surface(
        ui: &mut egui::Ui,
        dark_fill: egui::Color32,
        light_fill: egui::Color32,
        add_contents: impl FnOnce(&mut egui::Ui),
    ) {
        let fill = if ui.visuals().dark_mode {
            dark_fill
        } else {
            light_fill
        };
        egui::Frame::NONE
            .inner_margin(egui::Margin::same(8))
            .fill(fill)
            .stroke(egui::Stroke::new(
                0.75,
                ui.visuals().widgets.noninteractive.bg_stroke.color,
            ))
            .corner_radius(6.0)
            .show(ui, add_contents);
    }

    fn setting_item(ui: &mut egui::Ui, label: &str, add_control: impl FnOnce(&mut egui::Ui)) {
        let separator = ui.visuals().widgets.noninteractive.bg_stroke.color;
        let fill = if ui.visuals().dark_mode {
            egui::Color32::from_rgb(35, 43, 54)
        } else {
            egui::Color32::from_rgb(232, 240, 248)
        };
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(6, 6))
            .fill(fill)
            .stroke(egui::Stroke::new(0.5, separator))
            .corner_radius(3.0)
            .show(ui, |ui| {
                let grid_id = ui.next_auto_id();
                egui::Grid::new(grid_id)
                    .num_columns(1)
                    .spacing([0.0, 4.0])
                    .show(ui, |ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(94.0, 22.0),
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                ui.add_sized(
                                    [94.0, 18.0],
                                    egui::Label::new(egui::RichText::new(label).size(11.0))
                                        .truncate()
                                        .halign(egui::Align::Center),
                                )
                                .on_hover_text(label);
                            },
                        );
                        ui.end_row();
                        ui.allocate_ui_with_layout(
                            egui::vec2(94.0, 68.0),
                            egui::Layout::top_down(egui::Align::Center),
                            add_control,
                        );
                        ui.end_row();
                    });
            });
    }

    fn show_optional_card<T>(
        ui: &mut egui::Ui,
        id: &'static str,
        title: &str,
        value: &mut Option<T>,
        default: T,
        card_height: Option<f32>,
        add_settings: impl FnOnce(&mut egui::Ui, &mut T),
    ) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(220.0);
            if let Some(card_height) = card_height {
                ui.set_height(card_height);
            }
            ui.vertical(|ui| {
                let mut enabled = value.is_some();
                let mut toggled = false;
                ui.horizontal(|ui| {
                    ui.strong(title);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        toggled = ui
                            .checkbox(&mut enabled, "")
                            .on_hover_text(if enabled { "Disable" } else { "Enable" })
                            .changed();
                    });
                });
                if toggled {
                    *value = enabled.then_some(default);
                }
                if let Some(settings) = value {
                    egui::Grid::new(id)
                        .num_columns(2)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| add_settings(ui, settings));
                }
            });
        });
    }

    fn show_oscillators(ui: &mut egui::Ui, preset: &mut crate::presets::Preset) {
        ui.heading("Oscillators");
        let mut remove = None;
        let card_width = 220.0;
        let column_spacing = 12.0;
        let columns = ((ui.available_width() + column_spacing) / (card_width + column_spacing))
            .floor()
            .max(1.0) as usize;
        egui::Grid::new("oscillator_grid")
            .num_columns(columns)
            .spacing([column_spacing, 12.0])
            .show(ui, |ui| {
                for (index, oscillator) in preset.oscillators.iter_mut().enumerate() {
                    let card_fill = if ui.visuals().dark_mode {
                        egui::Color32::from_rgb(25, 31, 40)
                    } else {
                        egui::Color32::from_rgb(246, 249, 252)
                    };
                    let card_stroke =
                        egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color);
                    egui::Frame::NONE
                        .inner_margin(egui::Margin::same(6))
                        .fill(card_fill)
                        .stroke(card_stroke)
                        .corner_radius(6.0)
                        .shadow(egui::epaint::Shadow {
                            offset: [0, 3],
                            blur: 8,
                            spread: 0,
                            color: egui::Color32::from_black_alpha(48),
                        })
                        .show(ui, |ui| {
                            ui.set_width(card_width);
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.strong(format!("Oscillator {}", index + 1));
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui
                                                .small_button("🗑")
                                                .on_hover_text("Delete oscillator")
                                                .clicked()
                                            {
                                                remove = Some(index);
                                            }
                                        },
                                    );
                                });
                                ui.separator();
                                egui::Frame::NONE
                                    .inner_margin(egui::Margin::same(2))
                                    .fill(egui::Color32::TRANSPARENT)
                                    .stroke(egui::Stroke::NONE)
                                    .corner_radius(4.0)
                                    .show(ui, |ui| {
                                        egui::Grid::new(("oscillator_settings", index))
                                            .num_columns(2)
                                            //.min_col_width(106.0)
                                            //.min_row_height(106.0)
                                            //.max_col_width(106.0)
                                            //.max_row_height(106.0)
                                            .spacing([6.0, 6.0])
                                            .show(ui, |ui| {
                                                Self::setting_item(ui, "Waveform", |ui| {
                                                    Self::paint_waveform_preview(
                                                        ui,
                                                        oscillator.waveform,
                                                    );
                                                    egui::ComboBox::from_id_salt((
                                                        "waveform", index,
                                                    ))
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
                                                });
                                                Self::knob_item(
                                                    ui,
                                                    "Gain",
                                                    &mut oscillator.gain,
                                                    0.0..=2.0,
                                                    0.01,
                                                    2,
                                                );
                                                ui.end_row();

                                                Self::knob_item(
                                                    ui,
                                                    "Frequency ratio",
                                                    &mut oscillator.frequency_ratio,
                                                    0.01..=16.0,
                                                    0.01,
                                                    2,
                                                );
                                                Self::knob_item(
                                                    ui,
                                                    "Detune cents",
                                                    &mut oscillator.detune_cents,
                                                    -100.0..=100.0,
                                                    0.1,
                                                    1,
                                                );
                                                ui.end_row();

                                                Self::knob_item(
                                                    ui,
                                                    "Decay seconds",
                                                    &mut oscillator.decay_seconds,
                                                    0.01..=60.0,
                                                    0.05,
                                                    2,
                                                );
                                                Self::knob_item(
                                                    ui,
                                                    "Velocity sensitivity",
                                                    &mut oscillator.velocity_sensitivity,
                                                    0.0..=4.0,
                                                    0.02,
                                                    2,
                                                );
                                                ui.end_row();
                                            });
                                    });
                            });
                        });

                    if (index + 1) % columns == 0 {
                        ui.end_row();
                    }
                }

                Self::show_optional_card(
                    ui,
                    "pluck_settings",
                    "Pluck",
                    &mut preset.pluck,
                    PluckPreset {
                        gain: 0.55,
                        decay_seconds: 1.8,
                        cutoff_hz: 4_200.0,
                        velocity_sensitivity: 0.7,
                    },
                    None,
                    |ui, pluck| {
                        Self::knob_item(ui, "Gain", &mut pluck.gain, 0.0..=1.0, 0.005, 2);
                        Self::knob_item(
                            ui,
                            "Decay s",
                            &mut pluck.decay_seconds,
                            0.05..=10.0,
                            0.01,
                            2,
                        );
                        ui.end_row();
                        Self::knob_item(
                            ui,
                            "Cutoff Hz",
                            &mut pluck.cutoff_hz,
                            100.0..=20_000.0,
                            50.0,
                            0,
                        );
                        Self::knob_item(
                            ui,
                            "Velocity",
                            &mut pluck.velocity_sensitivity,
                            0.0..=4.0,
                            0.02,
                            2,
                        );
                        ui.end_row();
                    },
                );

                Self::show_optional_card(
                    ui,
                    "hammer_settings",
                    "Hammer",
                    &mut preset.hammer,
                    HammerPreset {
                        gain: 0.08,
                        decay_seconds: 0.015,
                        cutoff_hz: 5_000.0,
                        velocity_sensitivity: 1.5,
                    },
                    None,
                    |ui, hammer| {
                        Self::knob_item(ui, "Gain", &mut hammer.gain, 0.0..=1.0, 0.005, 2);
                        Self::knob_item(
                            ui,
                            "Decay s",
                            &mut hammer.decay_seconds,
                            0.001..=0.2,
                            0.001,
                            3,
                        );
                        ui.end_row();
                        Self::knob_item(
                            ui,
                            "Cutoff Hz",
                            &mut hammer.cutoff_hz,
                            100.0..=20_000.0,
                            50.0,
                            0,
                        );
                        Self::knob_item(
                            ui,
                            "Velocity",
                            &mut hammer.velocity_sensitivity,
                            0.0..=4.0,
                            0.02,
                            2,
                        );
                        ui.end_row();
                    },
                );
            });
        if let Some(index) = remove {
            preset.oscillators.remove(index);
        }
        if ui.button("+ Add oscillator").clicked() {
            preset.oscillators.push(OscillatorPreset {
                waveform: WaveformPreset::Sine,
                gain: 0.5,
                frequency_ratio: 1.0,
                detune_cents: 0.0,
                decay_seconds: 60.0,
                velocity_sensitivity: 0.0,
            });
        }
    }

    fn paint_filter_graph(ui: &mut egui::Ui, filter: Option<&FilterPreset>) {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(360.0, 190.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);
        painter.rect_stroke(
            rect,
            4.0,
            ui.visuals().widgets.noninteractive.bg_stroke,
            egui::StrokeKind::Inside,
        );

        let graph = egui::Rect::from_min_max(
            rect.min + egui::vec2(38.0, 10.0),
            rect.max - egui::vec2(10.0, 24.0),
        );
        let grid_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        let font = egui::FontId::monospace(10.0);

        for db in [0.0_f32, -24.0, -48.0] {
            let y = graph.top() + (12.0 - db) / 60.0 * graph.height();
            painter.line_segment(
                [egui::pos2(graph.left(), y), egui::pos2(graph.right(), y)],
                egui::Stroke::new(1.0, grid_color),
            );
            painter.text(
                egui::pos2(graph.left() - 4.0, y),
                egui::Align2::RIGHT_CENTER,
                format!("{db:.0}"),
                font.clone(),
                ui.visuals().weak_text_color(),
            );
        }

        for (frequency, label) in [
            (20.0_f32, "20"),
            (200.0, "200"),
            (2_000.0, "2k"),
            (20_000.0, "20k"),
        ] {
            let normalized = (frequency / 20.0).log10() / 1_000.0_f32.log10();
            let x = graph.left() + normalized * graph.width();
            painter.line_segment(
                [egui::pos2(x, graph.top()), egui::pos2(x, graph.bottom())],
                egui::Stroke::new(1.0, grid_color),
            );
            painter.text(
                egui::pos2(x, graph.bottom() + 5.0),
                egui::Align2::CENTER_TOP,
                label,
                font.clone(),
                ui.visuals().weak_text_color(),
            );
        }

        let Some(filter) = filter else {
            painter.text(
                graph.center(),
                egui::Align2::CENTER_CENTER,
                "Filter disabled",
                egui::FontId::proportional(14.0),
                ui.visuals().weak_text_color(),
            );
            return;
        };

        let cutoff = filter.cutoff_hz.clamp(20.0, 20_000.0);
        let cutoff_x =
            graph.left() + ((cutoff / 20.0).log10() / 1_000.0_f32.log10()) * graph.width();
        painter.line_segment(
            [
                egui::pos2(cutoff_x, graph.top()),
                egui::pos2(cutoff_x, graph.bottom()),
            ],
            egui::Stroke::new(1.0, ui.visuals().warn_fg_color),
        );

        let points = (0..=160)
            .map(|index| {
                let normalized = index as f32 / 160.0;
                let frequency = 20.0 * 1_000.0_f32.powf(normalized);
                let ratio = frequency / cutoff;
                let denominator = ((1.0 - ratio * ratio).powi(2)
                    + (ratio / filter.resonance_q).powi(2))
                .sqrt()
                .max(1e-6);
                let numerator = match filter.mode {
                    FilterModePreset::LowPass => 1.0,
                    FilterModePreset::BandPass => ratio / filter.resonance_q,
                    FilterModePreset::HighPass => ratio * ratio,
                };
                let db = (20.0 * (numerator / denominator).max(1e-6).log10()).clamp(-48.0, 12.0);
                egui::pos2(
                    graph.left() + normalized * graph.width(),
                    graph.top() + (12.0 - db) / 60.0 * graph.height(),
                )
            })
            .collect();
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(2.0, ui.visuals().selection.stroke.color),
        ));
        response.on_hover_text(format!(
            "{} · {:.0} Hz · Q {:.2}",
            filter.mode.label(),
            filter.cutoff_hz,
            filter.resonance_q
        ));
    }

    fn show_filter(ui: &mut egui::Ui, filter: &mut Option<FilterPreset>) {
        ui.heading("Filter");
        egui::Frame::group(ui.style()).show(ui, |ui| {
            let mut enabled = filter.is_some();
            let mut toggled = false;
            ui.horizontal(|ui| {
                ui.strong("Filter settings");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    toggled = ui
                        .checkbox(&mut enabled, "")
                        .on_hover_text(if enabled {
                            "Disable filter"
                        } else {
                            "Enable filter"
                        })
                        .changed();
                });
            });
            if toggled {
                *filter = enabled.then_some(FilterPreset {
                    mode: FilterModePreset::LowPass,
                    cutoff_hz: 2_500.0,
                    resonance_q: 0.707,
                });
            }
            ui.separator();
            ui.with_layout(
                egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true),
                |ui| {
                    ui.set_min_width(250.0);
                    ui.vertical(|ui| {
                        if let Some(filter) = filter.as_mut() {
                            egui::Grid::new("filter_settings")
                                .num_columns(2)
                                .spacing([8.0, 4.0])
                                .show(ui, |ui| {
                                    Self::setting_item(ui, "Mode", |ui| {
                                        Self::paint_filter_mode_preview(ui, filter.mode);
                                        egui::ComboBox::from_id_salt("filter_mode")
                                            .selected_text(filter.mode.label())
                                            .show_ui(ui, |ui| {
                                                for mode in FilterModePreset::ALL {
                                                    ui.selectable_value(
                                                        &mut filter.mode,
                                                        mode,
                                                        mode.label(),
                                                    );
                                                }
                                            });
                                    });
                                    Self::knob_item(
                                        ui,
                                        "Cutoff Hz",
                                        &mut filter.cutoff_hz,
                                        20.0..=20_000.0,
                                        50.0,
                                        0,
                                    );
                                    ui.end_row();
                                    Self::knob_item(
                                        ui,
                                        "Resonance Q",
                                        &mut filter.resonance_q,
                                        0.5..=20.0,
                                        0.02,
                                        2,
                                    );
                                    ui.end_row();
                                });
                        }
                    });
                    Self::paint_filter_graph(ui, filter.as_ref());
                },
            );
        });
    }

    fn show_vibrato(ui: &mut egui::Ui, vibrato: &mut Option<VibratoPreset>, card_height: f32) {
        Self::show_optional_card(
            ui,
            "vibrato_settings",
            "Vibrato",
            vibrato,
            VibratoPreset {
                rate_hz: 5.0,
                depth_cents: 12.0,
            },
            Some(card_height),
            |ui, vibrato| {
                Self::knob_item(ui, "Rate Hz", &mut vibrato.rate_hz, 0.0..=20.0, 0.02, 2);
                Self::knob_item(
                    ui,
                    "Depth cents",
                    &mut vibrato.depth_cents,
                    0.0..=200.0,
                    0.2,
                    1,
                );
                ui.end_row();
            },
        );
    }

    fn show_tremolo(ui: &mut egui::Ui, tremolo: &mut Option<TremoloPreset>, card_height: f32) {
        Self::show_optional_card(
            ui,
            "tremolo_settings",
            "Tremolo",
            tremolo,
            TremoloPreset {
                rate_hz: 4.0,
                depth: 0.2,
            },
            Some(card_height),
            |ui, tremolo| {
                Self::knob_item(ui, "Rate Hz", &mut tremolo.rate_hz, 0.0..=20.0, 0.02, 2);
                Self::knob_item(ui, "Depth", &mut tremolo.depth, 0.0..=1.0, 0.005, 2);
                ui.end_row();
            },
        );
    }

    fn show_chorus(ui: &mut egui::Ui, chorus: &mut Option<ChorusPreset>, card_height: f32) {
        Self::show_optional_card(
            ui,
            "chorus_settings",
            "Chorus",
            chorus,
            ChorusPreset {
                rate_hz: 0.6,
                base_delay_ms: 20.0,
                depth_ms: 5.0,
                mix: 0.3,
            },
            Some(card_height),
            |ui, chorus| {
                Self::knob_item(ui, "Rate Hz", &mut chorus.rate_hz, 0.0..=20.0, 0.02, 2);
                Self::knob_item(
                    ui,
                    "Base delay ms",
                    &mut chorus.base_delay_ms,
                    0.1..=100.0,
                    0.1,
                    1,
                );
                ui.end_row();
                Self::knob_item(ui, "Depth ms", &mut chorus.depth_ms, 0.0..=50.0, 0.05, 2);
                Self::knob_item(ui, "Mix", &mut chorus.mix, 0.0..=1.0, 0.005, 2);
                ui.end_row();
            },
        );
    }

    fn show_flanger(ui: &mut egui::Ui, flanger: &mut Option<FlangerPreset>, card_height: f32) {
        Self::show_optional_card(
            ui,
            "flanger_settings",
            "Flanger",
            flanger,
            FlangerPreset {
                rate_hz: 0.2,
                base_delay_ms: 1.0,
                depth_ms: 2.0,
                feedback: 0.5,
                mix: 0.25,
            },
            Some(card_height),
            |ui, flanger| {
                Self::knob_item(ui, "Rate Hz", &mut flanger.rate_hz, 0.0..=20.0, 0.02, 2);
                Self::knob_item(
                    ui,
                    "Base delay ms",
                    &mut flanger.base_delay_ms,
                    0.1..=20.0,
                    0.02,
                    2,
                );
                ui.end_row();
                Self::knob_item(ui, "Depth ms", &mut flanger.depth_ms, 0.0..=20.0, 0.02, 2);
                Self::knob_item(
                    ui,
                    "Feedback",
                    &mut flanger.feedback,
                    -0.95..=0.95,
                    0.005,
                    2,
                );
                ui.end_row();
                Self::knob_item(ui, "Mix", &mut flanger.mix, 0.0..=1.0, 0.005, 2);
                ui.end_row();
            },
        );
    }

    fn show_reverb(ui: &mut egui::Ui, reverb: &mut Option<ReverbPreset>, card_height: f32) {
        Self::show_optional_card(
            ui,
            "reverb_settings",
            "Reverb",
            reverb,
            ReverbPreset {
                room_size: 0.65,
                damping: 0.4,
                mix: 0.2,
            },
            Some(card_height),
            |ui, reverb| {
                Self::knob_item(ui, "Room size", &mut reverb.room_size, 0.0..=1.0, 0.005, 2);
                Self::knob_item(ui, "Damping", &mut reverb.damping, 0.0..=1.0, 0.005, 2);
                ui.end_row();
                Self::knob_item(ui, "Mix", &mut reverb.mix, 0.0..=1.0, 0.005, 2);
                ui.end_row();
            },
        );
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
        let card_width = 232.0;
        let spacing = 12.0;
        let columns = ((ui.available_width() + spacing) / (card_width + spacing))
            .floor()
            .max(1.0) as usize;
        let compact_height = |enabled: bool, setting_rows: usize| {
            if enabled {
                26.0 + setting_rows as f32 * 106.0 + setting_rows.saturating_sub(1) as f32 * 4.0
            } else {
                22.0
            }
        };
        let intrinsic_heights = [
            compact_height(vibrato.is_some(), 1),
            compact_height(tremolo.is_some(), 1),
            compact_height(chorus.is_some(), 2),
            compact_height(flanger.is_some(), 3),
            compact_height(reverb.is_some(), 2),
        ];
        let row_height = |index: usize| {
            let row_start = index / columns * columns;
            intrinsic_heights[row_start..(row_start + columns).min(intrinsic_heights.len())]
                .iter()
                .copied()
                .fold(0.0_f32, f32::max)
        };
        egui::Grid::new("effects_grid")
            .num_columns(columns)
            .spacing([spacing, spacing])
            .show(ui, |ui| {
                let mut index = 0;
                Self::show_vibrato(ui, vibrato, row_height(index));
                index += 1;
                if index % columns == 0 {
                    ui.end_row();
                }
                Self::show_tremolo(ui, tremolo, row_height(index));
                index += 1;
                if index % columns == 0 {
                    ui.end_row();
                }
                Self::show_chorus(ui, chorus, row_height(index));
                index += 1;
                if index % columns == 0 {
                    ui.end_row();
                }
                Self::show_flanger(ui, flanger, row_height(index));
                index += 1;
                if index % columns == 0 {
                    ui.end_row();
                }
                Self::show_reverb(ui, reverb, row_height(index));
            });
    }

    fn paint_envelope_graph(ui: &mut egui::Ui, envelope: &EnvelopePreset) {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(360.0, 190.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 4.0, ui.visuals().extreme_bg_color);
        painter.rect_stroke(
            rect,
            4.0,
            ui.visuals().widgets.noninteractive.bg_stroke,
            egui::StrokeKind::Inside,
        );

        let graph = egui::Rect::from_min_max(
            rect.min + egui::vec2(38.0, 10.0),
            rect.max - egui::vec2(10.0, 24.0),
        );
        let grid_color = ui.visuals().widgets.noninteractive.bg_stroke.color;
        let font = egui::FontId::monospace(10.0);
        for amplitude in [0.0_f32, 0.5, 1.0] {
            let y = graph.bottom() - amplitude * graph.height();
            painter.line_segment(
                [egui::pos2(graph.left(), y), egui::pos2(graph.right(), y)],
                egui::Stroke::new(1.0, grid_color),
            );
            painter.text(
                egui::pos2(graph.left() - 4.0, y),
                egui::Align2::RIGHT_CENTER,
                format!("{amplitude:.1}"),
                font.clone(),
                ui.visuals().weak_text_color(),
            );
        }

        // Sustain lasts until note-off, so use a representative plateau in the preview.
        let sustain_seconds = (envelope.attack_seconds + envelope.decay_seconds)
            .max(envelope.release_seconds)
            .max(0.5);
        let total = (envelope.attack_seconds
            + envelope.decay_seconds
            + sustain_seconds
            + envelope.release_seconds)
            .max(0.001);
        let x_at = |seconds: f32| graph.left() + seconds / total * graph.width();
        let y_at = |amplitude: f32| graph.bottom() - amplitude.clamp(0.0, 1.0) * graph.height();
        let attack_end = envelope.attack_seconds;
        let decay_end = attack_end + envelope.decay_seconds;
        let sustain_end = decay_end + sustain_seconds;
        let curved_progress = |progress: f32, curvature: f32| {
            if curvature.abs() <= f32::EPSILON {
                progress
            } else {
                ((curvature * progress).exp() - 1.0) / (curvature.exp() - 1.0)
            }
        };
        let mut points = vec![egui::pos2(x_at(0.0), y_at(0.0))];
        for index in 1..=32 {
            let progress = index as f32 / 32.0;
            points.push(egui::pos2(
                x_at(envelope.attack_seconds * progress),
                y_at(curved_progress(progress, envelope.attack_curvature)),
            ));
        }
        for index in 1..=32 {
            let progress = index as f32 / 32.0;
            let amplitude = 1.0
                - curved_progress(progress, envelope.decay_curvature)
                    * (1.0 - envelope.sustain_level);
            points.push(egui::pos2(
                x_at(attack_end + envelope.decay_seconds * progress),
                y_at(amplitude),
            ));
        }
        points.push(egui::pos2(x_at(sustain_end), y_at(envelope.sustain_level)));
        for index in 1..=32 {
            let progress = index as f32 / 32.0;
            let amplitude = 1.0 - curved_progress(progress, envelope.release_curvature);
            points.push(egui::pos2(
                x_at(sustain_end + envelope.release_seconds * progress),
                y_at(envelope.sustain_level * amplitude.max(0.0)),
            ));
        }
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(2.0, ui.visuals().selection.stroke.color),
        ));

        for (seconds, label) in [
            (attack_end, "A"),
            (decay_end, "D"),
            (sustain_end, "S / note-off"),
            (total, "R"),
        ] {
            let x = x_at(seconds);
            painter.line_segment(
                [egui::pos2(x, graph.top()), egui::pos2(x, graph.bottom())],
                egui::Stroke::new(1.0, grid_color),
            );
            painter.text(
                egui::pos2(x, graph.bottom() + 5.0),
                egui::Align2::CENTER_TOP,
                label,
                font.clone(),
                ui.visuals().weak_text_color(),
            );
        }
        response.on_hover_text(format!(
            "Attack {:.2}s / {:.2} · Decay {:.2}s / {:.2} · Sustain {:.2} · Release {:.2}s / {:.2}",
            envelope.attack_seconds,
            envelope.attack_curvature,
            envelope.decay_seconds,
            envelope.decay_curvature,
            envelope.sustain_level,
            envelope.release_seconds,
            envelope.release_curvature
        ));
    }

    fn show_envelope(ui: &mut egui::Ui, envelope: &mut EnvelopePreset) {
        ui.heading("Envelope (ADSR)");
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.with_layout(
                egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true),
                |ui| {
                    ui.set_min_width(250.0);
                    egui::Grid::new("envelope_settings")
                        .num_columns(2)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            Self::knob_item(
                                ui,
                                "Attack (s)",
                                &mut envelope.attack_seconds,
                                0.0..=10.0,
                                0.01,
                                2,
                            );
                            Self::knob_item(
                                ui,
                                "Attack curvature",
                                &mut envelope.attack_curvature,
                                -10.0..=10.0,
                                0.02,
                                2,
                            );
                            ui.end_row();
                            Self::knob_item(
                                ui,
                                "Decay (s)",
                                &mut envelope.decay_seconds,
                                0.0..=10.0,
                                0.01,
                                2,
                            );
                            Self::knob_item(
                                ui,
                                "Decay curvature",
                                &mut envelope.decay_curvature,
                                -10.0..=10.0,
                                0.02,
                                2,
                            );
                            ui.end_row();
                            Self::knob_item(
                                ui,
                                "Sustain",
                                &mut envelope.sustain_level,
                                0.0..=1.0,
                                0.005,
                                2,
                            );
                            Self::knob_item(
                                ui,
                                "Release (s)",
                                &mut envelope.release_seconds,
                                0.0..=20.0,
                                0.02,
                                2,
                            );
                            ui.end_row();
                            Self::knob_item(
                                ui,
                                "Release curvature",
                                &mut envelope.release_curvature,
                                -10.0..=10.0,
                                0.02,
                                2,
                            );
                            ui.end_row();
                        });
                    Self::paint_envelope_graph(ui, envelope);
                },
            );
        });
    }
}

impl eframe::App for PresetEditor {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.handle_computer_keyboard(root.ctx());

        egui::CentralPanel::default().show(root, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Actions ▾", |ui| {
                    if ui.button("Reload XML").clicked() {
                        self.reload();
                        ui.close();
                    }
                    if ui.button("Apply").clicked() {
                        self.apply();
                        ui.close();
                    }
                    if ui.button("Save XML").clicked() {
                        self.save();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("MIDI Input").clicked() {
                        self.show_midi_setup = true;
                        ui.close();
                    }
                });
            });
            ui.separator();

            let workspace_rect = ui.available_rect_before_wrap();
            ui.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(workspace_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
                |ui| {
                    ui.set_clip_rect(workspace_rect);
                    ui.horizontal_top(|ui| {
                        ui.vertical(|ui| {
                            ui.heading("OpenRSynth");
                            ui.label(&self.status);
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                ui.set_width(92.0);
                                ui.vertical_centered(|ui| {
                                    ui.strong("Master volume");
                                    if Self::knob(ui, &mut self.master_gain, 0.0..=1.0, 0.005, 2)
                                        .changed()
                                        && self
                                            .command_sender
                                            .send(AudioCommand::SetMasterGain(self.master_gain))
                                            .is_err()
                                    {
                                        self.status = "Audio thread is unavailable".into();
                                    }
                                });
                            });

                            if let Some(preset) = self.bank.presets.get_mut(self.selected) {
                                egui::Frame::group(ui.style()).show(ui, |ui| {
                                    ui.set_width(350.0);
                                    ui.set_min_height(92.0);
                                    ui.vertical(|ui| {
                                        ui.strong("Instrument");
                                        ui.add_space(5.0);
                                        ui.horizontal(|ui| {
                                            ui.vertical(|ui| {
                                                ui.label(
                                                    egui::RichText::new("Preset name").size(11.0),
                                                );
                                                ui.add_sized(
                                                    [180.0, 24.0],
                                                    egui::TextEdit::singleline(&mut preset.name),
                                                );
                                            });
                                            ui.vertical(|ui| {
                                                ui.label(egui::RichText::new("Group").size(11.0));
                                                ui.add_sized(
                                                    [140.0, 24.0],
                                                    egui::TextEdit::singleline(&mut preset.group),
                                                );
                                            });
                                        });
                                    });
                                });
                            }
                        });
                    });
                    ui.separator();

                    ui.horizontal_top(|ui| {
                        egui::Resize::default()
                            .id_salt("preset_column_resize")
                            .default_width(210.0)
                            .min_width(160.0)
                            .max_width(420.0)
                            .resizable([true, false])
                            .with_stroke(false)
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.heading("Presets");
                                    ui.horizontal(|ui| {
                                        if ui.button("＋ Add").clicked() {
                                            self.add_preset();
                                        }
                                        if ui
                                            .add_enabled(
                                                self.bank.presets.len() > 1,
                                                egui::Button::new("🗑 Delete"),
                                            )
                                            .on_hover_text("Delete selected preset")
                                            .clicked()
                                        {
                                            self.pending_preset_deletion = Some(self.selected);
                                        }
                                    });
                                    ui.add_space(6.0);

                                    let mut groups: BTreeMap<String, Vec<(usize, String)>> =
                                        BTreeMap::new();
                                    for (index, preset) in self.bank.presets.iter().enumerate() {
                                        let group = if preset.group.trim().is_empty() {
                                            "Ungrouped".to_string()
                                        } else {
                                            preset.group.clone()
                                        };
                                        groups
                                            .entry(group)
                                            .or_default()
                                            .push((index, preset.name.clone()));
                                    }
                                    for (group, presets) in groups {
                                        egui::CollapsingHeader::new(group).default_open(true).show(
                                            ui,
                                            |ui| {
                                                for (index, name) in presets {
                                                    if ui
                                                        .selectable_label(
                                                            index == self.selected,
                                                            name,
                                                        )
                                                        .clicked()
                                                    {
                                                        self.select_preset(index);
                                                    }
                                                }
                                            },
                                        );
                                    }
                                });
                            });

                        ui.separator();

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            if let Some(preset) = self.bank.presets.get_mut(self.selected) {
                                ui.vertical(|ui| {
                                    let available_width = ui.available_width();
                                    let splitter_width = 10.0;
                                    let output_width_id =
                                        ui.make_persistent_id("output_column_width");
                                    let automatic_output_width = 640.0_f32
                                        .min((available_width - 260.0 - splitter_width).max(280.0));
                                    let output_width = ui
                                        .ctx()
                                        .data_mut(|data| data.get_persisted::<f32>(output_width_id))
                                        .unwrap_or(automatic_output_width)
                                        .clamp(
                                            280.0,
                                            (available_width - 260.0 - splitter_width).max(280.0),
                                        );
                                    let synthesis_width =
                                        (available_width - output_width - splitter_width)
                                            .max(260.0);
                                    ui.horizontal_top(|ui| {
                                        Self::surface(
                                            ui,
                                            egui::Color32::from_rgb(20, 27, 36),
                                            egui::Color32::from_rgb(238, 244, 249),
                                            |ui| {
                                                ui.set_width((synthesis_width - 12.0).max(248.0));
                                                ui.vertical(|ui| {
                                                    ui.heading("Synthesis");
                                                    ui.separator();
                                                    Self::surface(
                                                        ui,
                                                        egui::Color32::from_rgb(27, 36, 47),
                                                        egui::Color32::from_rgb(247, 250, 253),
                                                        |ui| {
                                                            ui.set_min_width(ui.available_width());
                                                            Self::show_oscillators(ui, preset);
                                                        },
                                                    );
                                                    ui.separator();
                                                    Self::surface(
                                                        ui,
                                                        egui::Color32::from_rgb(31, 30, 43),
                                                        egui::Color32::from_rgb(250, 247, 252),
                                                        |ui| {
                                                            ui.set_min_width(ui.available_width());
                                                            Self::show_effects(
                                                                ui,
                                                                &mut preset.vibrato,
                                                                &mut preset.tremolo,
                                                                &mut preset.chorus,
                                                                &mut preset.flanger,
                                                                &mut preset.reverb,
                                                            );
                                                        },
                                                    );
                                                });
                                            },
                                        );

                                        let splitter = ui
                                            .add(
                                                egui::Separator::default()
                                                    .vertical()
                                                    .spacing(splitter_width),
                                            )
                                            .interact(egui::Sense::drag());
                                        if splitter.hovered() || splitter.dragged() {
                                            ui.ctx().set_cursor_icon(
                                                egui::CursorIcon::ResizeHorizontal,
                                            );
                                        }
                                        let drag_start_id =
                                            splitter.id.with("output_width_drag_start");
                                        if splitter.drag_started() {
                                            ui.ctx().data_mut(|data| {
                                                data.insert_temp(drag_start_id, output_width)
                                            });
                                        }
                                        if splitter.dragged() {
                                            let drag_start = ui
                                                .ctx()
                                                .data_mut(|data| {
                                                    data.get_temp::<f32>(drag_start_id)
                                                })
                                                .unwrap_or(output_width);
                                            let resized_output =
                                                (drag_start - splitter.drag_delta().x).clamp(
                                                    280.0,
                                                    (available_width - 260.0 - splitter_width)
                                                        .max(280.0),
                                                );
                                            ui.ctx().data_mut(|data| {
                                                data.insert_persisted(
                                                    output_width_id,
                                                    resized_output,
                                                )
                                            });
                                            ui.ctx().request_repaint();
                                        }

                                        Self::surface(
                                            ui,
                                            egui::Color32::from_rgb(29, 25, 35),
                                            egui::Color32::from_rgb(247, 242, 249),
                                            |ui| {
                                                ui.set_width((output_width - 12.0).max(268.0));
                                                ui.vertical(|ui| {
                                                    ui.heading("Output");
                                                    ui.separator();
                                                    Self::surface(
                                                        ui,
                                                        egui::Color32::from_rgb(38, 30, 31),
                                                        egui::Color32::from_rgb(252, 246, 244),
                                                        |ui| {
                                                            Self::show_filter(
                                                                ui,
                                                                &mut preset.filter,
                                                            )
                                                        },
                                                    );
                                                    ui.separator();
                                                    Self::surface(
                                                        ui,
                                                        egui::Color32::from_rgb(29, 37, 33),
                                                        egui::Color32::from_rgb(244, 250, 246),
                                                        |ui| {
                                                            Self::show_envelope(
                                                                ui,
                                                                &mut preset.envelope,
                                                            )
                                                        },
                                                    );
                                                });
                                            },
                                        );
                                    });
                                });
                            }
                        });
                    });
                },
            );
        });

        if self.bank != self.last_applied_bank {
            self.status = match self.apply_presets() {
                Ok(()) => "Live preview updated".into(),
                Err(error) => error,
            };
        }

        self.show_midi_dialog(root.ctx());
        self.show_delete_confirmation(root.ctx());
    }
}
