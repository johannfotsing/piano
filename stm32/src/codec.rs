//! Control-plane driver for the STM32H747I-DISCO's WM8994 codec.
//!
//! I2C only configures the codec. PCM samples are delivered separately over
//! SAI1 with DMA; callers should keep the output muted until that stream is
//! clocked and contains valid samples.

use embedded_hal_02::blocking::{
    delay::DelayMs,
    i2c::{Write, WriteRead},
};

/// The 7-bit form of the WM8994's `0x34` write address.
pub const I2C_ADDRESS: u8 = 0x1a;
pub const DEVICE_ID: u16 = 0x8994;

const SW_RESET: u16 = 0x0000;
const POWER_MANAGEMENT_1: u16 = 0x0001;
const POWER_MANAGEMENT_5: u16 = 0x0005;
const LEFT_OUTPUT_VOLUME: u16 = 0x001c;
const RIGHT_OUTPUT_VOLUME: u16 = 0x001d;
const OUTPUT_MIXER_1: u16 = 0x002d;
const OUTPUT_MIXER_2: u16 = 0x002e;
const ANTI_POP_2: u16 = 0x0039;
const WRITE_SEQUENCER_CONTROL_1: u16 = 0x0110;
const AIF1_CLOCKING_1: u16 = 0x0200;
const CLOCKING_1: u16 = 0x0208;
const AIF1_RATE: u16 = 0x0210;
const AIF1_CONTROL_1: u16 = 0x0300;
const AIF1_MASTER_SLAVE: u16 = 0x0302;
const AIF1_DAC1_FILTER_1: u16 = 0x0420;
const AIF1_DAC2_FILTER_1: u16 = 0x0422;
const AIF1_DAC1_LEFT_MIXER_ROUTING: u16 = 0x0601;
const AIF1_DAC1_RIGHT_MIXER_ROUTING: u16 = 0x0602;
const AIF1_DAC2_LEFT_MIXER_ROUTING: u16 = 0x0604;
const AIF1_DAC2_RIGHT_MIXER_ROUTING: u16 = 0x0605;
const DAC1_LEFT_VOLUME: u16 = 0x0610;
const DAC1_RIGHT_VOLUME: u16 = 0x0611;
const DAC2_LEFT_VOLUME: u16 = 0x0612;
const DAC2_RIGHT_VOLUME: u16 = 0x0613;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<E> {
    Bus(E),
    UnexpectedDevice(u16),
}

/// WM8994 control interface for 48 kHz, 16-bit stereo headphone playback.
pub struct Wm8994<I2C> {
    i2c: I2C,
}

impl<I2C> Wm8994<I2C> {
    pub const fn new(i2c: I2C) -> Self {
        Self { i2c }
    }

    pub fn release(self) -> I2C {
        self.i2c
    }
}

impl<I2C, E> Wm8994<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
{
    pub fn read_register(&mut self, register: u16) -> Result<u16, Error<E>> {
        let address = register.to_be_bytes();
        let mut value = [0; 2];
        self.i2c
            .write_read(I2C_ADDRESS, &address, &mut value)
            .map_err(Error::Bus)?;
        Ok(u16::from_be_bytes(value))
    }

    pub fn write_register(&mut self, register: u16, value: u16) -> Result<(), Error<E>> {
        let register = register.to_be_bytes();
        let value = value.to_be_bytes();
        self.i2c
            .write(I2C_ADDRESS, &[register[0], register[1], value[0], value[1]])
            .map_err(Error::Bus)
    }

    pub fn verify_device(&mut self) -> Result<(), Error<E>> {
        let id = self.read_register(SW_RESET)?;
        if id == DEVICE_ID {
            Ok(())
        } else {
            Err(Error::UnexpectedDevice(id))
        }
    }

    pub fn reset_and_verify<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
    where
        D: DelayMs<u32>,
    {
        self.write_register(SW_RESET, 0)?;
        delay.delay_ms(5);
        self.verify_device()
    }

    /// Configure the analog headphone path and AIF1 for SAI-provided clocks.
    ///
    /// The codec is deliberately left muted. Start SAI/DMA first, then call
    /// [`Self::set_muted`] with `false` once valid zero or audio samples flow.
    pub fn configure_headphone_playback<D>(
        &mut self,
        delay: &mut D,
        volume_percent: u8,
    ) -> Result<(), Error<E>>
    where
        D: DelayMs<u32>,
    {
        self.reset_and_verify(delay)?;

        // WM8994 errata workarounds used by ST's board component driver.
        self.write_register(0x0102, 0x0003)?;
        self.write_register(0x0817, 0x0000)?;
        self.write_register(0x0102, 0x0000)?;

        self.write_register(ANTI_POP_2, 0x006c)?;
        self.write_register(POWER_MANAGEMENT_1, 0x0003)?;
        delay.delay_ms(50);

        // Route AIF1 time slot 0 through DAC1 to the headphone mixers.
        self.write_register(POWER_MANAGEMENT_5, 0x0303)?;
        self.write_register(AIF1_DAC1_LEFT_MIXER_ROUTING, 0x0001)?;
        self.write_register(AIF1_DAC1_RIGHT_MIXER_ROUTING, 0x0001)?;
        self.write_register(AIF1_DAC2_LEFT_MIXER_ROUTING, 0x0000)?;
        self.write_register(AIF1_DAC2_RIGHT_MIXER_ROUTING, 0x0000)?;

        // 48 kHz, 256fs, 16-bit I2S. The codec is an AIF1 clock slave and
        // receives MCLK1, bit clock, and frame clock from the STM32 SAI.
        self.write_register(AIF1_RATE, 0x0083)?;
        self.write_register(AIF1_CONTROL_1, 0x4010)?;
        self.write_register(AIF1_MASTER_SLAVE, 0x0000)?;
        self.write_register(CLOCKING_1, 0x000a)?;
        self.write_register(AIF1_CLOCKING_1, 0x0001)?;

        self.write_register(OUTPUT_MIXER_1, 0x0100)?;
        self.write_register(OUTPUT_MIXER_2, 0x0100)?;
        self.write_register(WRITE_SEQUENCER_CONTROL_1, 0x8100)?;
        delay.delay_ms(325);

        for register in [
            DAC1_LEFT_VOLUME,
            DAC1_RIGHT_VOLUME,
            DAC2_LEFT_VOLUME,
            DAC2_RIGHT_VOLUME,
        ] {
            self.write_register(register, 0x00c0)?;
        }

        self.set_output_volume(volume_percent)?;
        self.set_muted(true)
    }

    /// Set the headphone PGA volume on a linear 0–100 control scale.
    pub fn set_output_volume(&mut self, percent: u8) -> Result<(), Error<E>> {
        let percent = percent.min(100) as u16;
        let level = (percent * 63 + 50) / 100;
        let value = 0x0140 | level;
        self.write_register(LEFT_OUTPUT_VOLUME, value)?;
        self.write_register(RIGHT_OUTPUT_VOLUME, value)
    }

    pub fn set_muted(&mut self, muted: bool) -> Result<(), Error<E>> {
        let value = if muted { 0x0200 } else { 0x0010 };
        self.write_register(AIF1_DAC1_FILTER_1, value)?;
        self.write_register(AIF1_DAC2_FILTER_1, value)
    }

    pub fn power_down(&mut self) -> Result<(), Error<E>> {
        self.set_muted(true)?;
        self.write_register(OUTPUT_MIXER_1, 0)?;
        self.write_register(OUTPUT_MIXER_2, 0)?;
        self.write_register(POWER_MANAGEMENT_5, 0)?;
        self.write_register(SW_RESET, 0)
    }
}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;

    use super::*;

    #[derive(Default)]
    struct Bus {
        last_write: [u8; 4],
    }

    impl Write for Bus {
        type Error = Infallible;

        fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
            assert_eq!(address, I2C_ADDRESS);
            self.last_write.copy_from_slice(bytes);
            Ok(())
        }
    }

    impl WriteRead for Bus {
        type Error = Infallible;

        fn write_read(
            &mut self,
            address: u8,
            bytes: &[u8],
            output: &mut [u8],
        ) -> Result<(), Self::Error> {
            assert_eq!(address, I2C_ADDRESS);
            assert_eq!(bytes, &[0, 0]);
            output.copy_from_slice(&DEVICE_ID.to_be_bytes());
            Ok(())
        }
    }

    #[test]
    fn serializes_register_and_value_most_significant_byte_first() {
        let mut codec = Wm8994::new(Bus::default());
        codec.write_register(0x0210, 0x0083).unwrap();
        assert_eq!(codec.release().last_write, [0x02, 0x10, 0x00, 0x83]);
    }

    #[test]
    fn reads_and_verifies_the_wm8994_device_id() {
        let mut codec = Wm8994::new(Bus::default());
        assert_eq!(codec.verify_device(), Ok(()));
    }
}
