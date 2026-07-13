#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use embedded_alloc::LlffHeap as Heap;
use panic_halt as _;

use cortex_m_rt::entry;
use hal::prelude::*;
use stm32h7xx_hal as hal;

use stm32::{board::configure_audio_control_i2c, codec::Wm8994};

#[global_allocator]
static HEAP: Heap = Heap::empty();

const HEAP_SIZE: usize = 128 * 1024;
static mut HEAP_MEMORY: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

#[entry]
fn main() -> ! {
    // The application owns allocation policy. DSP and engine crates only depend
    // on `alloc`; their storage is served from this fixed AXI-SRAM heap.
    unsafe {
        HEAP.init(core::ptr::addr_of_mut!(HEAP_MEMORY) as usize, HEAP_SIZE);
    }

    let core = cortex_m::Peripherals::take().unwrap();
    let device = hal::pac::Peripherals::take().unwrap();

    let power = device.PWR.constrain();
    let power = power.smps().freeze();
    let rcc = device.RCC.constrain();
    let clocks = rcc
        .use_hse(25.MHz())
        .bypass_hse()
        .sys_ck(400.MHz())
        .freeze(power, &device.SYSCFG);

    let i2c = configure_audio_control_i2c(
        device.I2C4,
        device.GPIOD,
        clocks.peripheral.GPIOD,
        clocks.peripheral.I2C4,
        &clocks.clocks,
    );
    let mut delay = core.SYST.delay(clocks.clocks);
    let mut codec = Wm8994::new(i2c);

    // Configure the control plane now, but remain muted until SAI/DMA starts
    // supplying stable clocks and valid PCM data.
    if codec.configure_headphone_playback(&mut delay, 70).is_err() {
        loop {
            cortex_m::asm::wfi();
        }
    }

    loop {
        cortex_m::asm::wfi();
    }
}
