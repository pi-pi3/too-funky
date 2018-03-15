use x86::shared::io;

pub const PIC1: u16 = 0x20;
pub const PIC2: u16 = 0xa0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    M8086,
}

impl From<Mode> for u8 {
    fn from(mode: Mode) -> u8 {
        match mode {
            Mode::M8086 => 1,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Pic {
    com: u16,
    dat: u16,
}

impl Pic {
    pub unsafe fn new(port: u16) -> Pic {
        Pic {
            com: port,
            dat: port + 1,
        }
    }

    pub fn begin_init(self) -> PicInit {
        unsafe {
            io::outb(self.com, 0x11); // 0x11 = begin init sequence in cascade mode
        }
        PicInit(self)
    }

    pub fn eoi(&mut self) {
        unsafe {
            io::outb(self.com, 0x20); // 0x20 = end of interrupt
        }
    }

    pub fn mask(&self) -> u8 {
        unsafe { io::inb(self.dat) }
    }

    pub fn set_mask(&mut self, mask: u8) -> u8 {
        let mask = self.mask() | 1 << mask;
        unsafe {
            io::outb(self.dat, mask);
        }
        mask
    }

    pub fn clear_mask(&mut self, mask: u8) -> u8 {
        let mask = self.mask() & !(1 << mask);
        unsafe {
            io::outb(self.dat, mask);
        }
        mask
    }

    pub fn set_all(&mut self) {
        self.restore_mask(0xff);
    }

    pub fn clear_all(&mut self) {
        self.restore_mask(0);
    }

    pub fn restore_mask(&mut self, mask: u8) {
        unsafe {
            io::outb(self.dat, mask);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PicInit(Pic);

impl PicInit {
    pub fn end_init(mut self) -> Pic {
        self.0.eoi();
        self.0
    }

    pub fn offset(&mut self, offset: u8) {
        unsafe {
            io::outb(self.0.dat, offset);
        }
    }

    pub fn slave(&mut self, irq: u8) {
        unsafe {
            io::outb(self.0.dat, irq);
        }
    }

    pub fn identity(&mut self, irq: u8) {
        unsafe {
            io::outb(self.0.dat, irq);
        }
    }

    pub fn mode(&mut self, mode: Mode) {
        unsafe {
            io::outb(self.0.dat, mode.into());
        }
    }
}
