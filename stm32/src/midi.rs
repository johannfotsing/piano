use midi::usb::EventPacket;
use music::event::NoteEvent;

/// Minimal boundary implemented by the concrete USB device endpoint.
///
/// Keeping the USB peripheral driver behind this trait lets the MIDI/event
/// layer run independently of interrupt and endpoint ownership choices.
pub trait UsbMidiEndpoint {
    type Error;

    /// Reads one four-byte USB MIDI 1.0 event packet without blocking.
    /// `Ok(false)` means that no packet is currently available.
    fn read_event_packet(&mut self, packet: &mut [u8; 4]) -> Result<bool, Self::Error>;
}

pub struct UsbMidiInput<E> {
    endpoint: E,
}

impl<E> UsbMidiInput<E>
where
    E: UsbMidiEndpoint,
{
    pub const fn new(endpoint: E) -> Self {
        Self { endpoint }
    }

    /// Polls USB once and returns a note event when the packet contains one.
    pub fn poll(&mut self) -> Result<Option<NoteEvent>, E::Error> {
        let mut bytes = [0; 4];
        if !self.endpoint.read_event_packet(&mut bytes)? {
            return Ok(None);
        }

        Ok(EventPacket::new(bytes).note_event())
    }

    pub fn into_inner(self) -> E {
        self.endpoint
    }
}
