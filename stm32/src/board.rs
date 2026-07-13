use embedded_hal::digital::InputPin;

use crate::control::{Button, GpioHardware};

const BUTTON_COUNT: usize = 7;

/// GPIO pin collection for the OpenRSynth control panel.
///
/// STM32 pins connected to different ports can be converted to the HAL's
/// type-erased input-pin type before constructing this adapter. Button polarity
/// is configurable; encoder A/B levels are always returned as raw logic levels.
pub struct BoardGpio<P> {
    buttons: [P; BUTTON_COUNT],
    knob_a: P,
    knob_b: P,
    buttons_active_low: bool,
}

impl<P> BoardGpio<P> {
    pub const fn new(
        buttons: [P; BUTTON_COUNT],
        knob_a: P,
        knob_b: P,
        buttons_active_low: bool,
    ) -> Self {
        Self {
            buttons,
            knob_a,
            knob_b,
            buttons_active_low,
        }
    }

    pub fn button_pin_mut(&mut self, button: Button) -> &mut P {
        &mut self.buttons[button.index()]
    }

    pub fn knob_pins_mut(&mut self) -> (&mut P, &mut P) {
        (&mut self.knob_a, &mut self.knob_b)
    }

    pub fn into_pins(self) -> ([P; BUTTON_COUNT], P, P) {
        (self.buttons, self.knob_a, self.knob_b)
    }
}

impl<P> GpioHardware for BoardGpio<P>
where
    P: InputPin,
{
    type Error = P::Error;

    fn button_is_pressed(&mut self, button: Button) -> Result<bool, Self::Error> {
        let pin = &mut self.buttons[button.index()];
        if self.buttons_active_low {
            pin.is_low()
        } else {
            pin.is_high()
        }
    }

    fn knob_state(&mut self) -> Result<(bool, bool), Self::Error> {
        Ok((self.knob_a.is_high()?, self.knob_b.is_high()?))
    }
}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;
    use embedded_hal::digital::ErrorType;

    use super::*;

    #[derive(Clone, Copy)]
    struct Pin(bool);

    impl ErrorType for Pin {
        type Error = Infallible;
    }

    impl InputPin for Pin {
        fn is_high(&mut self) -> Result<bool, Self::Error> {
            Ok(self.0)
        }

        fn is_low(&mut self) -> Result<bool, Self::Error> {
            Ok(!self.0)
        }
    }

    #[test]
    fn reads_active_low_buttons_and_raw_encoder_levels() {
        let mut board = BoardGpio::new([Pin(true); BUTTON_COUNT], Pin(false), Pin(true), true);
        board.button_pin_mut(Button::Select).0 = false;

        assert_eq!(board.button_is_pressed(Button::Up), Ok(false));
        assert_eq!(board.button_is_pressed(Button::Select), Ok(true));
        assert_eq!(board.knob_state(), Ok((false, true)));
    }
}
