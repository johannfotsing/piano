# PROJECT MANAGEMENT

## DEPENDENCY GRAPH

         music
        /     \
       /       \
    midi       synth
       \        /
        \      /
        engine
        /    \
       /      \
desktop       stm32

## Phase 1 – Music Domain

### Goal: Model music independently of any platform

Implement:

- [x] Note
- [x] Pitch
- [x] Octave
- [x] Frequency calculation
- [x] NoteEvent
- [x] Velocity

At the end of this phase, you should be able to write:

'''Rust
let event = NoteEvent::NoteOn {
note: Note::C4,
velocity: 127,
};
'''
without any desktop or embedded code.

### Milestone: Unit tests validate note frequencies and event creation

- [x] Test running OK !

## Phase 2 – Oscillator

### Goal: Produce audio samples

Implement:

- [x] Phase accumulator
- [x] Sine wave
- [x] Sample rate
- [x] Frequency

API:

'''Rust
let mut osc = Oscillator::new(440.0);
let sample = osc.next_sample();
'''

### Milestone: Unit tests verify the oscillator produces a waveform

## Phase 3 – Audio Output

### Goal: Hear a continuous tone

Use:

cpal

Hard-code:

440 Hz

No keyboard.

No MIDI.

No engine.

Just:

Oscillator

↓

CPAL

### Milestone: You hear a steady A4 tone

🎉 This is the first time your synthesizer makes sound.

## Phase 4 – Computer Keyboard

### Goal: Control the oscillator

Use:

winit

Map:

A -> C
S -> D
D -> E
F -> F
...

Initially:

one note at a time

### Milestone: Press A → hear C. Release A → silence

🎉 You can play notes.

## Phase 5 – Polyphony

Support:

multiple active notes

Implement:

Voice
Voice Manager
Mixer

Now:

A + D + G

plays a chord.

### Milestone: Three-note chords work.

## Phase 6 – ADSR

Notes should no longer start and stop abruptly.

Implement:

Attack
Decay
Sustain
Release

The instrument starts to feel expressive.

## Phase 7 – Engine

Now introduce the engine.

Before this point, it doesn't add much value.

Move:

keyboard events
voice manager
state

into the engine.

Desktop becomes:

Keyboard

↓

Engine

↓

Synth

↓

CPAL

Milestone: The desktop is mostly wiring code.

## Phase 8 – MIDI

Now connect a USB MIDI keyboard.

The flow becomes:

Computer keyboard

↓

NoteEvent

USB MIDI

↓

NoteEvent

↓

Engine

↓

Synth

The engine doesn't care where notes originate.

### Milestone: You can play from either your computer keyboard or your MIDI keyboard

## Phase 9 – Better Synth

Improve the sound.

Add:

- [x] square wave
- [x] triangle
- [x] saw wave
- [x] filters
- [x] LFO
- [x] vibrato
- [x] tremolo

At this point, you're building a real synthesizer.

---

## CHAT GPT's project roadmap

Phase 10 – Embedded Preparation

Now make:

synth
music
engine

fully no_std.

Boundary design: see [`NO_STD.md`](NO_STD.md).

- [x] Make `music` allocator-free and `no_std`
- [x] Make `synth` use `core` + `alloc`
- [x] Remove platform adapter dependencies from `engine`
- [x] Make `engine` use `core` + `alloc`
- [ ] Add an STM32 target-specific build check

Desktop still works.

This is a significant milestone because it proves your core logic is portable.

Phase 11 – STM32

Port the desktop application.

Replace:

CPAL

with

DAC

Replace:

Keyboard

with

GPIO

Everything else stays the same.

Phase 12 – Hardware Instrument

Finally:

buttons
knobs
OLED
MIDI DIN
USB MIDI
headphone amplifier
enclosure

Congratulations—you have an instrument.

The dependency evolution
Phase 1
music

Phase 2
music
   │
synth

Phase 3
music
   │
synth
   │
desktop

Phase 4
music
   │
synth
   │
desktop (keyboard)

Phase 5
music
   │
synth
   │
engine
   │
desktop

Phase 8
music
├── midi
├── synth
└── engine
      │
desktop

Phase 11
music
├── midi
├── synth
└── engine
    ├── desktop
    └── stm32
What I'd optimize for

If I were mentoring you through this project, I'd set one concrete objective for each phase that produces a visible or audible result. That keeps momentum high and makes debugging much easier.

Workspace builds.
Unit tests for musical concepts pass.
You hear a continuous tone.
You play a note with your computer keyboard.
You play a chord.
The notes have natural attack and release.
The engine cleanly orchestrates the application.
A USB MIDI keyboard plays your synthesizer.
The synth has multiple waveforms and effects.
The core crates compile for no_std.
The STM32 produces sound.
You play the same instrument on both your PC and your custom embedded hardware.

That sequence gives you frequent, tangible milestones while steadily building toward your ultimate goal: a portable synthesizer engine that powers both a desktop application and an STM32-based musical instrument.
