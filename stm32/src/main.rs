#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use embedded_alloc::LlffHeap as Heap;
use panic_halt as _;

use cortex_m_rt::entry;
use stm32h7xx_hal as hal;

mod control;
mod gpio;
mod midi;

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

    // The M7 is the only active application core during initial bring-up. Taking
    // the device peripherals also proves that this binary was compiled for the
    // STM32H747 M7 PAC rather than the previous STM32F4 target.
    let _device = hal::pac::Peripherals::take().unwrap();

    loop {}
}
