use core::fmt;

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entry {
    offset_1: u16,
    selector: u16,
    zero: u8,
    flags: u8,
    offset_2: u16,
}

impl Entry {
    // the compiler thinks this method is unused, but it is actually used
    #[allow(dead_code)]
    pub(crate) const fn empty() -> Entry {
        Entry {
            offset_1: 0,
            selector: 0,
            zero: 0,
            flags: 0,
            offset_2: 0,
        }
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            write!(f, "Entry {}", '{')?;
            write!(f, "selector: {:04x}, ", self.selector)?;
            write!(f, "offset: {:04x}{:04x}, ", self.offset_2, self.offset_1)?;
            write!(f, "zero: {:02x}, ", self.zero)?;
            write!(f, "flags: {:02x} ", self.flags)?;
            write!(f, "{}", '}')
        }
    }
}

impl fmt::LowerHex for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            write!(
                f,
                "{:04x}{:02x}{:02x}{:04x}{:04x}",
                self.offset_2,
                self.flags,
                self.zero,
                self.selector,
                self.offset_1,
            )
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct AttrFlags: u8 {
        const PRESENT = 0b10000000;
        const STORAGE = 0b00010000;
        const RING0 = 0b00000000;
        const RING1 = 0b00100000;
        const RING2 = 0b01000000;
        const RING3 = 0b01100000;
        const INT_GATE = 0x0e;
        const TRP_GATE = 0x0f;
    }
}

#[repr(u8)]
pub enum RingLevel {
    Ring0,
    Ring1,
    Ring2,
    Ring3,
}

impl From<RingLevel> for AttrFlags {
    fn from(r: RingLevel) -> AttrFlags {
        match r {
            RingLevel::Ring0 => AttrFlags::RING0,
            RingLevel::Ring1 => AttrFlags::RING1,
            RingLevel::Ring2 => AttrFlags::RING2,
            RingLevel::Ring3 => AttrFlags::RING3,
        }
    }
}

#[repr(u8)]
pub enum Gate {
    Interrupt,
    Trap,
}

impl From<Gate> for AttrFlags {
    fn from(g: Gate) -> AttrFlags {
        match g {
            Gate::Interrupt => AttrFlags::INT_GATE,
            Gate::Trap => AttrFlags::TRP_GATE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntryBuilder {
    offset: Option<u32>,
    selector: Option<u16>,
    flags: Option<AttrFlags>,
}

impl EntryBuilder {
    pub fn new() -> EntryBuilder {
        EntryBuilder {
            offset: None,
            selector: None,
            flags: None,
        }
    }

    pub fn build(self) -> Entry {
        let offset = self.offset.unwrap();
        let selector = self.selector.unwrap();
        let flags = self.flags.unwrap();
        Entry {
            offset_1: (offset & 0xffff) as u16,
            selector: selector,
            zero: 0,
            flags: flags.bits(),
            offset_2: ((offset >> 16) & 0xffff) as u16,
        }
    }

    pub fn try_build(self) -> Option<Entry> {
        if self.offset.is_none() | self.selector.is_none()
            | self.flags.is_none()
        {
            return None;
        }
        Some(self.build())
    }

    pub fn nul(mut self) -> EntryBuilder {
        self.offset = Some(0);
        self.selector = Some(0);
        self.flags = Some(AttrFlags::from_bits_truncate(0));
        self
    }

    pub fn isr(mut self, isr: *const ()) -> EntryBuilder {
        self.offset = Some(isr as usize as u32);
        self
    }

    pub fn selector(mut self, sel: u16) -> EntryBuilder {
        self.selector = Some(sel);
        self
    }

    pub fn present(mut self) -> EntryBuilder {
        self.flags = Some(self.flags.unwrap_or_default() | AttrFlags::PRESENT);
        self
    }

    pub fn ring(mut self, ring: RingLevel) -> EntryBuilder {
        self.flags = Some({
            let mut flags = self.flags.unwrap_or_default();
            flags.remove(AttrFlags::RING3); // reset ring level to zero
            flags | AttrFlags::from(ring)
        });
        self
    }

    pub fn gate(mut self, gate: Gate) -> EntryBuilder {
        self.flags = Some({
            let mut flags = self.flags.unwrap_or_default();
            flags.remove(AttrFlags::TRP_GATE); // reset gate to zero
            flags | AttrFlags::from(gate)
        });
        self
    }
}
