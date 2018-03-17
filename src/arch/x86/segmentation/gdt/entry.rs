use core::fmt;

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entry {
    limit_1: u16,
    base_1: u16,
    base_2: u8,
    access: u8,
    limit_2_flags: u8,
    base_3: u8,
}

impl Entry {
    // the compiler thinks this method is unused, but it is actually used
    #[allow(dead_code)]
    pub(crate) const fn empty() -> Entry {
        Entry {
            limit_1: 0,
            base_1: 0,
            base_2: 0,
            access: 0,
            limit_2_flags: 0,
            base_3: 0,
        }
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            write!(
                f,
                "Entry {} limit: {:01x}{:04x}, base: {:02x}{:02x}{:04x}, access: {:02x}, flags: {:01x}  {}",
                '{',
                self.limit_2_flags & 0x0f,
                self.limit_1,
                self.base_3,
                self.base_2,
                self.base_1,
                self.access,
                (self.limit_2_flags & 0xf0) >> 4,
                '}',
            )
        }
    }
}

impl fmt::LowerHex for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            write!(
                f,
                "{:02x}{:02x}{:02x}{:02x}{:04x}{:04x}",
                self.base_3,
                self.limit_2_flags,
                self.access,
                self.base_2,
                self.base_1,
                self.limit_1,
            )
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct Flags: u8 {
        const PAGE_GR = 0b10000000;
        const SIZE_32 = 0b01000000;
    }
}

bitflags! {
    pub struct Access: u8 {
        const PRESENT = 0b10000000;
        const RING0 = 0b00000000;
        const RING1 = 0b00100000;
        const RING2 = 0b01000000;
        const RING3 = 0b01100000;
        const ONE = 0b00010000; // has to be one
        const EXEC = 0b00001000;
        const DIR = 0b00000100; // no support for DIR=1
        const RW = 0b00000010;
        const ACCESS = 0b00000001;
    }
}

impl Default for Access {
    fn default() -> Access {
        Access::from_bits_truncate(0b00010000) // bit 5 has to be 1
    }
}

#[repr(u8)]
pub enum RingLevel {
    Ring0,
    Ring1,
    Ring2,
    Ring3,
}

pub enum Granularity {
    Bit,
    Page,
}

impl From<RingLevel> for Access {
    fn from(r: RingLevel) -> Access {
        match r {
            RingLevel::Ring0 => Access::RING0,
            RingLevel::Ring1 => Access::RING1,
            RingLevel::Ring2 => Access::RING2,
            RingLevel::Ring3 => Access::RING3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntryBuilder {
    base: Option<u32>,
    limit: Option<u32>,
    flags: Option<Flags>,
    access: Option<Access>,
}

impl EntryBuilder {
    pub fn new() -> EntryBuilder {
        EntryBuilder {
            base: None,
            limit: None,
            flags: None,
            access: None,
        }
    }

    pub fn build(self) -> Entry {
        let base = self.base.unwrap();
        let limit = self.limit.unwrap();
        let flags = self.flags.unwrap();
        let access = self.access.unwrap();
        Entry {
            limit_1: (limit & 0xffff) as u16,
            base_1: (base & 0xffff) as u16,
            base_2: ((base & 0xff0000) >> 16) as u8,
            access: access.bits(),
            limit_2_flags: flags.bits() | ((limit & 0xf0000) >> 16) as u8,
            base_3: ((base & 0xff000000) >> 24) as u8,
        }
    }

    pub fn try_build(self) -> Option<Entry> {
        if self.base.is_none() | self.limit.is_none() | self.flags.is_none()
            | self.access.is_none()
        {
            return None;
        }
        Some(self.build())
    }

    pub fn nul(mut self) -> EntryBuilder {
        self.base = Some(0);
        self.limit = Some(0);
        self.flags = Some(Flags::default());
        self.access = Some(Access::default());
        self
    }

    pub fn base(mut self, base: usize) -> EntryBuilder {
        self.base = Some(base as u32);
        self
    }

    pub fn limit(mut self, limit: usize) -> EntryBuilder {
        self.limit = Some(limit as u32);
        self
    }

    pub fn granularity(mut self, granularity: Granularity) -> EntryBuilder {
        self.flags = Some(match granularity {
            Granularity::Page => {
                self.flags.unwrap_or_default() | Flags::PAGE_GR
            }
            Granularity::Bit => {
                self.flags.unwrap_or_default() & !Flags::PAGE_GR
            }
        });
        self
    }

    pub fn size(mut self, size: u8) -> EntryBuilder {
        self.flags = Some(match size {
            16 => self.flags.unwrap_or_default() & !Flags::SIZE_32,
            32 => self.flags.unwrap_or_default() | Flags::SIZE_32,
            _ => panic!("gdt::Entry size must be 16 or 32"),
        });
        self
    }

    pub fn present(mut self) -> EntryBuilder {
        self.access = Some(self.access.unwrap_or_default() | Access::PRESENT);
        self
    }

    pub fn ring(mut self, ring: RingLevel) -> EntryBuilder {
        self.access = Some({
            let mut access = self.access.unwrap_or_default();
            access.remove(Access::RING3); // reset ring level to zero
            access | Access::from(ring)
        });
        self
    }

    pub fn executable(mut self) -> EntryBuilder {
        self.access = Some(self.access.unwrap_or_default() | Access::EXEC);
        self
    }

    pub fn read_write(mut self) -> EntryBuilder {
        self.access = Some(self.access.unwrap_or_default() | Access::RW);
        self
    }
}
