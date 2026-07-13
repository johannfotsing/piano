use engine::App;

use crate::gpio::GpioEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayItem {
    Instrument,
    MasterGain,
}

impl DisplayItem {
    const fn previous(self) -> Self {
        match self {
            Self::Instrument => Self::MasterGain,
            Self::MasterGain => Self::Instrument,
        }
    }

    const fn next(self) -> Self {
        self.previous()
    }
}

/// State consumed by the STM32 display renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayState {
    pub selected_item: DisplayItem,
    pub editing: bool,
}

impl Default for DisplayState {
    fn default() -> Self {
        Self {
            selected_item: DisplayItem::Instrument,
            editing: false,
        }
    }
}

/// Generic synth settings exposed by the STM32 hardware controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineSettings {
    pub master_gain_percent: u8,
    pub selected_instrument: usize,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            master_gain_percent: 20,
            selected_instrument: 0,
        }
    }
}

/// STM32 application engine connecting GPIO input, display state, and synth settings.
pub struct ControlEngine {
    display: DisplayState,
    settings: EngineSettings,
}

impl ControlEngine {
    pub const fn new(settings: EngineSettings) -> Self {
        Self {
            display: DisplayState {
                selected_item: DisplayItem::Instrument,
                editing: false,
            },
            settings,
        }
    }

    pub const fn display_state(&self) -> DisplayState {
        self.display
    }

    pub const fn settings(&self) -> EngineSettings {
        self.settings
    }

    pub fn handle_event(&mut self, event: GpioEvent, app: &mut App) {
        match event {
            GpioEvent::Up if !self.display.editing => {
                self.display.selected_item = self.display.selected_item.previous();
            }
            GpioEvent::Down if !self.display.editing => {
                self.display.selected_item = self.display.selected_item.next();
            }
            GpioEvent::In => self.display.editing = true,
            GpioEvent::Out => self.display.editing = false,
            GpioEvent::KnobIncrement(delta) if self.display.editing => {
                self.apply_increment(delta, app);
            }
            GpioEvent::Up | GpioEvent::Down | GpioEvent::KnobIncrement(_) => {}
        }
    }

    fn apply_increment(&mut self, delta: i16, app: &mut App) {
        match self.display.selected_item {
            DisplayItem::Instrument => {
                let current = app.selected_instrument_index() as i32;
                let last = app.instruments().len().saturating_sub(1) as i32;
                let selected = (current + i32::from(delta)).clamp(0, last) as usize;
                if app.select_instrument(selected) {
                    self.settings.selected_instrument = selected;
                }
            }
            DisplayItem::MasterGain => {
                let gain = (i16::from(self.settings.master_gain_percent) + delta).clamp(0, 100);
                self.settings.master_gain_percent = gain as u8;
                app.set_master_gain(f32::from(self.settings.master_gain_percent) / 100.0);
            }
        }
    }
}

impl Default for ControlEngine {
    fn default() -> Self {
        Self::new(EngineSettings::default())
    }
}
