# `no_std` and allocation boundary

OpenRSynth separates portable domain/DSP code from platform adapters in three
layers.

## 1. `core` only

The `music` crate is always `#![no_std]` and allocator-free. Music-domain value
types and events must remain fixed-size and must not expose `Vec`, `String`,
`Box`, I/O, threads, or operating-system types. Portable mathematical functions
that are unavailable in `core` use `libm`.

## 2. `core` plus `alloc`

The `synth` and `engine` crates are `#![no_std]` and use owned collections for
instruments, voices, and DSP delay buffers through explicit `alloc` imports.
Allocation policy and the global allocator are supplied by the final
application.

The engine must depend only on `music` and `synth`. MIDI decoding and device
access are input adapters and therefore do not belong in the engine dependency
graph.

## 3. `std` platform adapters

The `desktop` and desktop-input portion of `midi` may use files, XML, audio and
MIDI devices, threads, channels, and GUI APIs. The `stm32` application supplies
embedded hardware adapters and an allocator if the selected synth configuration
requires one.

## Verification

The portable music boundary is checked with:

```sh
cargo check -p music --no-default-features
cargo check -p synth --no-default-features
cargo check -p engine --no-default-features
cargo test -p music
cargo test -p synth
cargo test -p engine
```

Host integration is checked separately from the embedded binary:

```sh
cargo check --workspace --exclude stm32
```

The STM32 target requires its own target-specific build and panic strategy; it
must not be validated as a normal host binary.
