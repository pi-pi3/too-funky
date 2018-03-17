use core::mem;
use core::fmt;

pub mod entry;
pub use self::entry::*;

const ENTRY_SIZE: usize = 8;

#[repr(C, packed)]
pub struct Gdtr {
    limit: u16,
    base: u32,
}

impl fmt::LowerHex for Gdtr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { write!(f, "{:04x}{:08x}", self.limit, self.base) }
    }
}

pub struct Gdt<'a> {
    inner: &'a mut [Entry],
}

impl<'a> Gdt<'a> {
    pub fn with_table(inner: &'a mut [Entry]) -> Gdt<'a> {
        Gdt { inner }
    }

    pub fn gdtr(&self) -> Gdtr {
        Gdtr {
            limit: (self.inner.len() * mem::size_of::<Entry>() - 1) as u16,
            base: self.inner.as_ptr() as usize as u32,
        }
    }

    pub fn new_entry(&mut self, num: u16, entry: Entry) {
        assert!(
            num & (ENTRY_SIZE as u16 - 1) == 0,
            "gdt entry num must be a multiple of 8"
        );
        self.inner[num as usize / ENTRY_SIZE] = entry;
    }
}
