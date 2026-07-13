/// Events produced by the board's directional buttons and rotary encoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpioEvent {
    Up,
    Down,
    In,
    Out,
    KnobIncrement(i16),
}
