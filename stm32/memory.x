MEMORY
{
  /* M7 firmware starts in Flash Bank 1. Bank 2 remains available for a future
     Cortex-M4 image and independent updates. */
  FLASH : ORIGIN = 0x08000000, LENGTH = 1024K

  /* D1 AXI SRAM. Unlike DTCM, this region can later be shared with DMA-capable
     peripherals. Audio DMA buffers will eventually receive their own D2 SRAM
     linker section to make cache ownership explicit. */
  RAM : ORIGIN = 0x24000000, LENGTH = 512K
}
