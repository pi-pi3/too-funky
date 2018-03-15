// this os only uses huge pages
// it makes my life simpler
use core::fmt;

use paging::PAGE_SIZE;
use paging::addr::*;

#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entry {
    addr: u16,
    flags: u16,
}

impl Entry {
    // the compiler thinks this method is unused, but it is actually used
    #[allow(dead_code)]
    pub(crate) const fn empty() -> Entry {
        Entry {
            addr: 0,
            flags: 0,
        }
    }

    pub fn into_physical(&self) -> Physical {
        Physical::new((self.addr as usize) << 16)
    }

    pub fn is_used(&self) -> bool {
        self.flags & Flags::PRESENT.bits() != 0
    }
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            write!(
                f,
                "Entry {} addr: {:08x}, flags: {:04x}, {}",
                '{',
                (self.addr as u32) << 16,
                self.flags,
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
                "{:04x}{:04x}",
                self.flags,
                self.addr,
            )
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct Flags: u16 {
        const GLOBAL = 0b100000000;
        const SIZE = 0b010000000;
        const DIRTY = 0b001000000;
        const ACCESS = 0b000100000;
        const NOCACHE = 0b000010000;
        const WTHR = 0b000001000;
        const USER = 0b000000100;
        const RW = 0b000000010;
        const PRESENT = 0b000000001;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PageSize {
    Normal,
    Huge,
}

impl From<PageSize> for Flags {
    fn from(size: PageSize) -> Flags {
        match size {
            PageSize::Normal => Flags::from_bits_truncate(0),
            PageSize::Huge => Flags::SIZE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntryBuilder {
    addr: Option<Physical>,
    flags: Option<Flags>,
}

impl EntryBuilder {
    pub fn new() -> EntryBuilder {
        EntryBuilder {
            addr: None,
            flags: None,
        }
    }

    pub fn build(self) -> Entry {
        let addr = self.addr.unwrap().into_inner();
        let flags = self.flags.unwrap();
        if addr & (PAGE_SIZE - 1) != 0 {
            panic!("page directory entry address must be page-aligned");
        }
        Entry {
            addr: (addr >> 16) as u16,
            flags: flags.bits(),
        }
    }

    pub fn try_build(self) -> Option<Entry> {
        if self.addr.is_none()
        | self.flags.is_none() {
            return None;
        }
        Some(self.build())
    }

    pub fn nul(mut self) -> EntryBuilder {
        self.addr = Some(Physical::new(0));
        self.flags = Some(Flags::from_bits_truncate(0));
        self
    }

    pub fn addr(mut self, addr: Physical) -> EntryBuilder {
        self.addr = Some(addr);
        self
    }

    pub fn present(mut self) -> EntryBuilder {
        self.flags = Some(
            self.flags.unwrap_or_default() | Flags::PRESENT
        );
        self
    }

    pub fn global(mut self) -> EntryBuilder {
        self.flags = Some(
            self.flags.unwrap_or_default() | Flags::GLOBAL
        );
        self
    }

    pub fn no_cache(mut self) -> EntryBuilder {
        self.flags = Some(
            self.flags.unwrap_or_default() | Flags::NOCACHE
        );
        self
    }

    pub fn write_through(mut self) -> EntryBuilder {
        self.flags = Some(
            self.flags.unwrap_or_default() | Flags::WTHR
        );
        self
    }

    pub fn user(mut self) -> EntryBuilder {
        self.flags = Some(
            self.flags.unwrap_or_default() | Flags::USER
        );
        self
    }

    pub fn read_write(mut self) -> EntryBuilder {
        self.flags = Some(
            self.flags.unwrap_or_default() | Flags::RW
        );
        self
    }

    pub fn page_size(mut self, size: PageSize) -> EntryBuilder {
        let bit = Flags::from(size);
        self.flags = Some({
            let mut flags = self.flags.unwrap_or_default();
            flags.remove(Flags::SIZE);
            flags | bit
        });
        self
    }
}
