//! Embedded control-panel event detection and debouncing.

/// Logical buttons exposed by the embedded control panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Button {
    Up,
    Down,
    In,
    Out,
    Select,
    On,
    Off,
}

impl Button {
    const ALL: [Self; 7] = [
        Self::Up,
        Self::Down,
        Self::In,
        Self::Out,
        Self::Select,
        Self::On,
        Self::Off,
    ];

    pub(crate) const fn index(self) -> usize {
        self as usize
    }

    const fn event(self) -> GpioEvent {
        match self {
            Self::Up => GpioEvent::Up,
            Self::Down => GpioEvent::Down,
            Self::In => GpioEvent::In,
            Self::Out => GpioEvent::Out,
            Self::Select => GpioEvent::Select,
            Self::On => GpioEvent::On,
            Self::Off => GpioEvent::Off,
        }
    }
}

/// Events produced by the board's buttons and rotary encoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioEvent {
    Up,
    Down,
    In,
    Out,
    Select,
    On,
    Off,
    Reset,
    KnobIncrement(i16),
}

/// Electrical GPIO boundary. Implementations translate active-low or
/// active-high pins into logical pressed states.
pub trait GpioHardware {
    type Error;

    fn button_is_pressed(&mut self, button: Button) -> Result<bool, Self::Error>;

    /// Returns the current quadrature encoder A/B levels.
    fn knob_state(&mut self) -> Result<(bool, bool), Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpioConfig {
    /// Number of identical samples required before accepting a transition.
    pub debounce_ticks: u16,
    /// Number of stable pressed samples required to emit Reset.
    pub reset_hold_ticks: u32,
    /// Button whose long press triggers Reset.
    pub reset_button: Button,
}

impl Default for GpioConfig {
    fn default() -> Self {
        Self {
            debounce_ticks: 3,
            reset_hold_ticks: 1_000,
            reset_button: Button::Select,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanError<E> {
    Hardware(E),
    EventQueueFull,
}

#[derive(Debug, Clone, Copy)]
struct ButtonState {
    raw_pressed: bool,
    stable_pressed: bool,
    identical_samples: u16,
    held_ticks: u32,
    reset_sent: bool,
}

impl ButtonState {
    const fn new() -> Self {
        Self {
            raw_pressed: false,
            stable_pressed: false,
            identical_samples: 0,
            held_ticks: 0,
            reset_sent: false,
        }
    }
}

const EVENT_QUEUE_CAPACITY: usize = 16;

/// Debounces buttons, detects a long press, and decodes a quadrature knob.
/// Call `poll` at a fixed rate so the tick-based timing remains deterministic.
pub struct GpioScanner {
    config: GpioConfig,
    buttons: [ButtonState; 7],
    events: [Option<GpioEvent>; EVENT_QUEUE_CAPACITY],
    event_read: usize,
    event_write: usize,
    event_len: usize,
    previous_knob_state: u8,
    knob_quarters: i8,
}

impl GpioScanner {
    pub fn new(config: GpioConfig, initial_knob_state: (bool, bool)) -> Self {
        assert!(config.debounce_ticks > 0);
        assert!(config.reset_hold_ticks > 0);
        Self {
            config,
            buttons: [ButtonState::new(); 7],
            events: [None; EVENT_QUEUE_CAPACITY],
            event_read: 0,
            event_write: 0,
            event_len: 0,
            previous_knob_state: encode_knob_state(initial_knob_state),
            knob_quarters: 0,
        }
    }

    /// Samples every input once and returns the oldest pending event.
    pub fn poll<H: GpioHardware>(
        &mut self,
        hardware: &mut H,
    ) -> Result<Option<GpioEvent>, ScanError<H::Error>> {
        self.sample_buttons(hardware)?;
        self.sample_knob(hardware)?;
        Ok(self.next_event())
    }

    pub fn next_event(&mut self) -> Option<GpioEvent> {
        if self.event_len == 0 {
            return None;
        }
        let event = self.events[self.event_read].take();
        self.event_read = (self.event_read + 1) % EVENT_QUEUE_CAPACITY;
        self.event_len -= 1;
        event
    }

    fn sample_buttons<H: GpioHardware>(
        &mut self,
        hardware: &mut H,
    ) -> Result<(), ScanError<H::Error>> {
        for button in Button::ALL {
            let pressed = hardware
                .button_is_pressed(button)
                .map_err(ScanError::Hardware)?;
            let (pressed_event, reset_event) = {
                let state = &mut self.buttons[button.index()];
                let mut pressed_event = false;
                let mut reset_event = false;

                if pressed == state.raw_pressed {
                    state.identical_samples = state.identical_samples.saturating_add(1);
                } else {
                    state.raw_pressed = pressed;
                    state.identical_samples = 1;
                }

                if state.raw_pressed != state.stable_pressed
                    && state.identical_samples >= self.config.debounce_ticks
                {
                    state.stable_pressed = state.raw_pressed;
                    state.held_ticks = 0;
                    state.reset_sent = false;
                    pressed_event = state.stable_pressed;
                }

                if state.stable_pressed {
                    state.held_ticks = state.held_ticks.saturating_add(1);
                    if button == self.config.reset_button
                        && !state.reset_sent
                        && state.held_ticks >= self.config.reset_hold_ticks
                    {
                        state.reset_sent = true;
                        reset_event = true;
                    }
                }

                (pressed_event, reset_event)
            };

            if pressed_event {
                self.push_event(button.event())?;
            }
            if reset_event {
                self.push_event(GpioEvent::Reset)?;
            }
        }
        Ok(())
    }

    fn sample_knob<H: GpioHardware>(
        &mut self,
        hardware: &mut H,
    ) -> Result<(), ScanError<H::Error>> {
        let current = encode_knob_state(hardware.knob_state().map_err(ScanError::Hardware)?);
        let transition = usize::from((self.previous_knob_state << 2) | current);
        const QUADRATURE: [i8; 16] = [0, -1, 1, 0, 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, -1, 0];
        self.knob_quarters += QUADRATURE[transition];
        self.previous_knob_state = current;

        if self.knob_quarters >= 4 {
            self.knob_quarters = 0;
            self.push_event(GpioEvent::KnobIncrement(1))?;
        } else if self.knob_quarters <= -4 {
            self.knob_quarters = 0;
            self.push_event(GpioEvent::KnobIncrement(-1))?;
        }
        Ok(())
    }

    fn push_event<E>(&mut self, event: GpioEvent) -> Result<(), ScanError<E>> {
        if self.event_len == EVENT_QUEUE_CAPACITY {
            return Err(ScanError::EventQueueFull);
        }
        self.events[self.event_write] = Some(event);
        self.event_write = (self.event_write + 1) % EVENT_QUEUE_CAPACITY;
        self.event_len += 1;
        Ok(())
    }
}

fn encode_knob_state((a, b): (bool, bool)) -> u8 {
    (u8::from(a) << 1) | u8::from(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Hardware {
        buttons: [bool; 7],
        knob: (bool, bool),
    }

    impl Hardware {
        fn new() -> Self {
            Self {
                buttons: [false; 7],
                knob: (false, false),
            }
        }

        fn set_button(&mut self, button: Button, pressed: bool) {
            self.buttons[button.index()] = pressed;
        }
    }

    impl GpioHardware for Hardware {
        type Error = ();

        fn button_is_pressed(&mut self, button: Button) -> Result<bool, Self::Error> {
            Ok(self.buttons[button.index()])
        }

        fn knob_state(&mut self) -> Result<(bool, bool), Self::Error> {
            Ok(self.knob)
        }
    }

    fn scanner() -> GpioScanner {
        GpioScanner::new(
            GpioConfig {
                debounce_ticks: 2,
                reset_hold_ticks: 3,
                reset_button: Button::Select,
            },
            (false, false),
        )
    }

    #[test]
    fn debounces_each_button_press() {
        for (button, expected) in [
            (Button::Up, GpioEvent::Up),
            (Button::Down, GpioEvent::Down),
            (Button::In, GpioEvent::In),
            (Button::Out, GpioEvent::Out),
            (Button::Select, GpioEvent::Select),
            (Button::On, GpioEvent::On),
            (Button::Off, GpioEvent::Off),
        ] {
            let mut scanner = scanner();
            let mut hardware = Hardware::new();
            hardware.set_button(button, true);
            assert_eq!(scanner.poll(&mut hardware), Ok(None));
            assert_eq!(scanner.poll(&mut hardware), Ok(Some(expected)));
        }
    }

    #[test]
    fn held_select_emits_one_reset_until_released() {
        let mut scanner = scanner();
        let mut hardware = Hardware::new();
        hardware.set_button(Button::Select, true);

        assert_eq!(scanner.poll(&mut hardware), Ok(None));
        assert_eq!(scanner.poll(&mut hardware), Ok(Some(GpioEvent::Select)));
        assert_eq!(scanner.poll(&mut hardware), Ok(None));
        assert_eq!(scanner.poll(&mut hardware), Ok(Some(GpioEvent::Reset)));
        assert_eq!(scanner.poll(&mut hardware), Ok(None));
    }

    #[test]
    fn decodes_both_knob_directions() {
        let mut scanner = scanner();
        let mut hardware = Hardware::new();

        for state in [(true, false), (true, true), (false, true)] {
            hardware.knob = state;
            assert_eq!(scanner.poll(&mut hardware), Ok(None));
        }
        hardware.knob = (false, false);
        assert_eq!(
            scanner.poll(&mut hardware),
            Ok(Some(GpioEvent::KnobIncrement(1)))
        );

        for state in [(false, true), (true, true), (true, false)] {
            hardware.knob = state;
            assert_eq!(scanner.poll(&mut hardware), Ok(None));
        }
        hardware.knob = (false, false);
        assert_eq!(
            scanner.poll(&mut hardware),
            Ok(Some(GpioEvent::KnobIncrement(-1)))
        );
    }
}
