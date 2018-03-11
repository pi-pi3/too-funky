
pub mod entry;
pub use self::entry::*;

#[repr(C, packed)]
pub struct Idtr {
    limit: u16,
    base: u32,
}

pub struct Idt<'a> {
    pub(crate) inner: &'a mut [Entry],
}

impl<'a> Idt<'a> {
    pub fn with_table(inner: &'a mut [Entry]) -> Idt<'a> {
        Idt { inner }
    }

    pub fn idtr(&self) -> Idtr {
        Idtr {
            limit: (self.inner.len() - 1) as u16,
            base: self.inner.as_ptr() as usize as u32,
        }
    }

    pub fn new_handler(&mut self, num: u8, entry: Entry) {
        self.inner[num as usize] = entry;
    }
}
