#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayItem {
    Instrument,
    MasterGain,
}

impl DisplayItem {
    pub const fn previous(self) -> Self {
        match self {
            Self::Instrument => Self::MasterGain,
            Self::MasterGain => Self::Instrument,
        }
    }

    pub const fn next(self) -> Self {
        self.previous()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayCommand {
    Select(DisplayItem),
    SetEditing(bool),
    SetInstrument(usize),
    SetMasterGain(u8),
}

/// Complete state required to draw the board's settings window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowGui {
    pub selected_item: DisplayItem,
    pub editing: bool,
    pub selected_instrument: usize,
    pub master_gain_percent: u8,
}

impl Default for WindowGui {
    fn default() -> Self {
        Self {
            selected_item: DisplayItem::Instrument,
            editing: false,
            selected_instrument: 0,
            master_gain_percent: 20,
        }
    }
}

/// LCD/framebuffer boundary implemented by the board display driver.
pub trait DisplayTarget {
    type Error;

    fn draw(&mut self, window: &WindowGui) -> Result<(), Self::Error>;
}

pub struct Display<T> {
    target: T,
    window: WindowGui,
    dirty: bool,
}

impl<T: DisplayTarget> Display<T> {
    pub const fn new(target: T, initial_window: WindowGui) -> Self {
        Self {
            target,
            window: initial_window,
            dirty: true,
        }
    }

    pub const fn window(&self) -> &WindowGui {
        &self.window
    }

    pub fn send(&mut self, command: DisplayCommand) {
        match command {
            DisplayCommand::Select(item) => self.window.selected_item = item,
            DisplayCommand::SetEditing(editing) => self.window.editing = editing,
            DisplayCommand::SetInstrument(index) => self.window.selected_instrument = index,
            DisplayCommand::SetMasterGain(gain) => self.window.master_gain_percent = gain,
        }
        self.dirty = true;
    }

    pub fn render_if_dirty(&mut self) -> Result<bool, T::Error> {
        if !self.dirty {
            return Ok(false);
        }
        self.target.draw(&self.window)?;
        self.dirty = false;
        Ok(true)
    }
}
