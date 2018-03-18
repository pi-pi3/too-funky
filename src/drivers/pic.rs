use port::Port;

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
    com: Port,
    dat: Port,
}

impl Pic {
    pub unsafe fn new(port: u16) -> Pic {
        let port = Port::new(port);
        let (com, dat) = port.into_siblings();
        Pic { com, dat }
    }

    pub fn begin_init(mut self) -> PicInit {
        self.com.write_byte(0x11); // 0x11 = begin init sequence in cascade mode
        PicInit(self)
    }

    pub fn eoi(&mut self) {
        self.com.write_byte(0x20); // 0x20 = end of interrupt
    }

    pub fn mask(&self) -> u8 {
        unsafe { self.dat.read_byte_unsafe() }
    }

    pub fn set_mask(&mut self, mask: u8) -> u8 {
        let mask = self.mask() | 1 << mask;
        self.dat.write_byte(mask);
        mask
    }

    pub fn clear_mask(&mut self, mask: u8) -> u8 {
        let mask = self.mask() & !(1 << mask);
        self.dat.write_byte(mask);
        mask
    }

    pub fn set_all(&mut self) {
        self.restore_mask(0xff);
    }

    pub fn clear_all(&mut self) {
        self.restore_mask(0);
    }

    pub fn restore_mask(&mut self, mask: u8) {
        self.dat.write_byte(mask);
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
        self.0.dat.write_byte(offset);
    }

    pub fn slave(&mut self, irq: u8) {
        self.0.dat.write_byte(irq);
    }

    pub fn identity(&mut self, irq: u8) {
        self.0.dat.write_byte(irq);
    }

    pub fn mode(&mut self, mode: Mode) {
        self.0.dat.write_byte(mode.into());
    }
}
