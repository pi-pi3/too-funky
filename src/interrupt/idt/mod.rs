use core::mem;
use core::fmt;

pub mod entry;
pub use self::entry::*;

use interrupt::{ExceptionHandler, InterruptHandler};

#[repr(C, packed)]
pub struct Idtr {
    limit: u16,
    base: u32,
}

impl fmt::LowerHex for Idtr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            write!(f, "{:04x}{:08x}", self.limit, self.base)
        }
    }
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
            limit: (self.inner.len() * mem::size_of::<Entry>() - 1) as u16,
            base: self.inner.as_ptr() as usize as u32,
        }
    }

    pub fn new_handler(&mut self, num: u8, entry: Entry) {
        self.inner[num as usize] = entry;
    }

    fn new_default_handler(&mut self, num: u8, isr: *const ()) {
        let entry = EntryBuilder::new()
            .present()
            .isr(isr)
            .selector(8)
            .ring(RingLevel::Ring3)
            .gate(Gate::Interrupt)
            .build();
        self.new_handler(num, entry);
    }

    pub fn new_exception_handler<E>(&mut self, num: u8, isr: ExceptionHandler<E>) {
        self.new_default_handler(num, isr as *const ());
    }

    pub fn new_interrupt_handler(&mut self, num: u8, isr: InterruptHandler) {
        self.new_default_handler(num, isr as *const ());
    }
}
