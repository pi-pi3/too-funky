use core::cmp::Ordering;
use core::fmt;

use x86::shared::io;

const VGA_BUFFER: usize = 0xe00b8000;
const WIDTH: usize = 80;
const HEIGHT: usize = 25;

unsafe fn reg_write(port: u16, index: u8, data: u8) {
    io::outb(port, index);
    io::outb(port + 1, data);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    White,
}

impl From<Color> for u8 {
    fn from(c: Color) -> u8 {
        match c {
            Color::Black => 0,
            Color::Blue => 1,
            Color::Green => 2,
            Color::Cyan => 3,
            Color::Red => 4,
            Color::Magenta => 5,
            Color::Brown => 6,
            Color::White => 7,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Shade {
    Dark(Color),
    Bright(Color),
}

impl Shade {
    pub const fn default_bg() -> Shade {
        Shade::Dark(Color::Black)
    }

    pub const fn default_fg() -> Shade {
        Shade::Dark(Color::White)
    }
}

impl From<Shade> for u8 {
    fn from(c: Shade) -> u8 {
        match c {
            Shade::Dark(c) => u8::from(c),
            Shade::Bright(c) => u8::from(c) + 8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Char(Shade, Shade, u8);

impl From<u8> for Char {
    #[inline]
    fn from(byte: u8) -> Char {
        Char(Shade::default_bg(), Shade::default_fg(), byte)
    }
}

impl From<Char> for u8 {
    #[inline]
    fn from(ch: Char) -> u8 {
        ch.2
    }
}

impl From<Char> for u16 {
    fn from(ch: Char) -> u16 {
        let bg = u8::from(ch.0) as u16;
        let fg = u8::from(ch.1) as u16;
        let ch = ch.2 as u16;
        bg << 12 | fg << 8 | ch
    }
}

impl PartialOrd for Char {
    #[inline]
    fn partial_cmp(&self, other: &Char) -> Option<Ordering> {
        self.2.partial_cmp(&other.2)
    }
}

impl Ord for Char {
    #[inline]
    fn cmp(&self, other: &Char) -> Ordering {
        self.2.cmp(&other.2)
    }
}

pub struct Vga {
    vga: *mut u16,
    color: (Shade, Shade),
    y: u16,
    x: u16,
}

impl Vga {
    pub fn new() -> Vga {
        // enable cursor
        // copied from osdev
        unsafe {
            reg_write(0x3d4, 0x0a, io::inb(0x3d5) & 0xc0);
            reg_write(0x3d4, 0x0b, (io::inb(0x3e0) & 0xe0) | 15);

            reg_write(0x3d4, 0x0f, 0);
            reg_write(0x3d4, 0x0e, 0);
        }

        let vga = VGA_BUFFER as *mut _;
        let color = (Shade::default_bg(), Shade::default_fg());
        Vga { vga, color, y: 0, x: 0 }
    }

    fn offset(&self) -> usize {
        self.y as usize * WIDTH + self.x as usize
    }

    fn next(&mut self) {
        self.x += 1;
        if self.x as usize > WIDTH {
            self.endl();
        }
    }

    fn back(&mut self) {
        if self.x == 0 {
            self.x = WIDTH as u16 - 1;
            if self.y == 0 {
                self.y = HEIGHT as u16 - 1;
            } else {
                self.y -= 1;
            }
        } else {
            self.x -= 1;
        }
    }

    fn cr(&mut self) {
        self.x = 0;
    }

    fn lf(&mut self) {
        self.y += 1;
        if self.y as usize >= HEIGHT {
            self.y = 0;
        }
    }

    fn endl(&mut self) {
        self.cr();
        self.lf();
    }

    unsafe fn write_raw(&mut self, ch: u16, offset: usize) {
        *self.vga.add(offset) = ch;
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\0' => {},
            0x08 => self.back(),
            b'\t' => self.write_byte(b' '), // tab
            b'\n' => self.endl(),
            b'\r' => self.cr(),
            byte if byte < 0x20 => {
                self.write_byte(b'^');
                self.write_byte(byte + b'A');
            }
            byte => {
                let ch = Char(self.color.0, self.color.1, byte);
                let offset = self.offset();
                unsafe {
                    self.write_raw(u16::from(ch), offset);
                }
                self.next();
            }
        }
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_byte(byte);
        }

        // update cursor
        let offset = self.offset() as u16;
        let hi = (offset >> 8) as u8;
        let lo = (offset & 0xff) as u8;
        unsafe {
            reg_write(0x3d4, 0x0f, lo);
            reg_write(0x3d4, 0x0e, hi);
        }
    }
}

impl fmt::Write for Vga {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.write_bytes(s.as_bytes());
        Ok(())
    }
}
