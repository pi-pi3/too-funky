use core::ptr::Unique;
use core::cmp::Ordering;
use core::fmt::{self, Write};

use port::Port;

const WIDTH: usize = 80;
const HEIGHT: usize = 25;
const SIZE: usize = WIDTH * HEIGHT;

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

impl Color {
    pub fn from_attr(byte: u8) -> Option<Color> {
        match byte {
            0 => Some(Color::Black),
            1 => Some(Color::Red),
            2 => Some(Color::Green),
            3 => Some(Color::Brown),
            4 => Some(Color::Blue),
            5 => Some(Color::Magenta),
            6 => Some(Color::Cyan),
            7 => Some(Color::White),
            _ => None,
        }
    }
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

    pub fn from_attr(byte: u8) -> Shade {
        if byte & 0x8 == 0 {
            Shade::Dark(Color::from_attr(byte & 0x7).unwrap())
        } else {
            Shade::Bright(Color::from_attr(byte & 0x7).unwrap())
        }
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

impl Default for Char {
    fn default() -> Char {
        Char::from(0)
    }
}

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

// supported VT100 escape codes:
//
//   - ^[[<COUNT>A, ^[<COUNT>B, ^[<COUNT>C, ^[<COUNT>D
//   - ^[[<ROW>;<COLUMN>H
//   - ^[7, ^[8
//   - ^[[<ATTR>m, ^[[<ATTR1>;<ATTR2>m
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum State {
    Default,
    Esc,
    Bracket,
    Count(i16),
    Tuple(i16, i16),
}

pub struct Vga {
    vga: Unique<u16>,
    color: (Shade, Shade),
    y: i16,
    x: i16,
    state: State,
    port_a: Port, // 0x3d4
    port_b: Port, // 0x3d5
    _port_c: Port, // 0x3e0
}

impl Vga {
    pub fn new(ptr: Unique<u16>) -> Vga {
        let port = unsafe { Port::new(0x3d4) };
        let (mut port_a, mut port_b) = unsafe { port.into_siblings() };
        let mut port_c = unsafe { Port::new(0x3e0) };

        // enable cursor
        // copied from osdev
        port_a.write_byte(0x0a);
        let byte = port_b.read_byte();
        port_b.write_byte(byte & 0xc0);
        port_a.write_byte(0x0b);
        port_b.write_byte((port_c.read_byte() & 0xe0) | 15);

        port_a.write_byte(0x0f);
        port_b.write_byte(0);
        port_a.write_byte(0x0e);
        port_b.write_byte(0);

        let color = (Shade::default_bg(), Shade::default_fg());
        let mut vga = Vga {
            vga: ptr,
            color,
            y: 0,
            x: 0,
            state: State::Default,
            port_a,
            port_b,
            _port_c: port_c,
        };
        vga.cls();
        vga
    }

    pub fn cls(&mut self) {
        let ch = Char::default().into();
        for i in 0..WIDTH * HEIGHT {
            self.write_raw(ch, i);
        }
    }

    fn offset(&self) -> usize {
        self.y as usize * WIDTH + self.x as usize
    }

    fn set_cursor(&mut self, y: i16, x: i16) {
        self.y = y % HEIGHT as i16;
        self.x = x % WIDTH as i16;
    }

    fn move_cursor(&mut self, y: i16, x: i16) {
        self.x += x;
        let y = y + self.x / WIDTH as i16;
        self.y = (self.y + y) % HEIGHT as i16;
        self.x %= WIDTH as i16;
    }

    #[inline]
    fn next(&mut self) {
        self.move_cursor(0, 1);
    }

    #[inline]
    fn back(&mut self) {
        self.move_cursor(0, -1);
    }

    #[inline]
    fn cr(&mut self) {
        self.x = 0;
    }

    #[inline]
    fn lf(&mut self) {
        self.move_cursor(1, 0);
    }

    #[inline]
    fn endl(&mut self) {
        self.cr();
        self.lf();
    }

    // implemented attributes: bg, fg colours, bright, reset
    fn set_attr(&mut self, attr0: i16, attr1: i16) -> Result<(), ()> {
        match (attr0, attr1) {
            (0, 0) => self.color = (Shade::default_bg(), Shade::default_fg()),
            (1, c) if c >= 30 && c <= 37 => {
                self.color.1 = Shade::from_attr(8 | (c as u8 - 30))
            }
            (c, 1) if c >= 30 && c <= 37 => {
                self.color.1 = Shade::from_attr(8 | (c as u8 - 30))
            }
            (0, c) if c >= 30 && c <= 37 => {
                self.color.1 = Shade::from_attr(c as u8 - 30)
            }
            (c, 0) if c >= 30 && c <= 37 => {
                self.color.1 = Shade::from_attr(c as u8 - 30)
            }
            (1, c) if c >= 40 && c <= 47 => {
                self.color.0 = Shade::from_attr(8 | (c as u8 - 30))
            }
            (c, 1) if c >= 40 && c <= 47 => {
                self.color.0 = Shade::from_attr(8 | (c as u8 - 30))
            }
            (0, c) if c >= 40 && c <= 47 => {
                self.color.0 = Shade::from_attr(c as u8 - 30)
            }
            (c, 0) if c >= 40 && c <= 47 => {
                self.color.0 = Shade::from_attr(c as u8 - 30)
            }
            _ => return Err(()),
        }
        Ok(())
    }

    #[inline]
    fn write_raw(&mut self, ch: u16, offset: usize) {
        unsafe {
            let ptr = self.vga.as_ptr().add(offset % SIZE);
            *ptr = ch;
        }
    }

    //   - ^[[<COUNT>A, ^[<COUNT>B, ^[<COUNT>C, ^[<COUNT>D
    //   - ^[[<ROW>;<COLUMN>H
    //   - ^[7, ^[8
    //   - ^[[<ATTR>m, ^[[<ATTR1>;<ATTR2>m
    fn write_byte(&mut self, byte: u8) {
        match self.state {
            State::Esc => match byte {
                b'[' => self.state = State::Bracket,
                b'7' => unimplemented!("^[7"),
                b'8' => unimplemented!("^[8"),
                byte => {
                    self.write_byte(b'^');
                    self.write_byte(b'[');
                    self.write_byte(byte);
                }
            },
            State::Bracket => match byte {
                b'A' => {
                    self.state = State::Default;
                    self.move_cursor(1, 0);
                }
                b'B' => {
                    self.state = State::Default;
                    self.move_cursor(-1, 0);
                }
                b'C' => {
                    self.state = State::Default;
                    self.move_cursor(0, 1);
                }
                b'D' => {
                    self.state = State::Default;
                    self.move_cursor(0, -1);
                }
                b'H' => {
                    self.state = State::Default;
                    self.set_cursor(0, 0);
                }
                x if x >= b'0' && x <= b'9' => {
                    self.state = State::Count(x as i16 - b'0' as i16);
                }
                byte => {
                    self.write_byte(b'^');
                    self.write_byte(b'[');
                    self.write_byte(b'[');
                    self.write_byte(byte);
                }
            },
            State::Count(i) => match byte {
                b'A' => {
                    self.state = State::Default;
                    self.move_cursor(i, 0);
                }
                b'B' => {
                    self.state = State::Default;
                    self.move_cursor(-i, 0);
                }
                b'C' => {
                    self.state = State::Default;
                    self.move_cursor(0, i);
                }
                b'D' => {
                    self.state = State::Default;
                    self.move_cursor(0, -i);
                }
                b';' => self.state = State::Tuple(i, 0),
                b'm' => {
                    self.state = State::Default;
                    let _ = self.set_attr(i, 0);
                }
                x if x >= b'0' && x <= b'9' => {
                    self.state = State::Count(i * 10 + x as i16 - b'0' as i16);
                }
                byte => {
                    let _ = write!(self, "^[[{}{}", i, byte as char);
                }
            },
            State::Tuple(i0, i1) => match byte {
                b'H' => {
                    self.state = State::Default;
                    self.set_cursor(i0, i1);
                }
                b'm' => {
                    self.state = State::Default;
                    let _ = self.set_attr(i0, i1);
                }
                x if x >= b'0' && x <= b'9' => {
                    self.state =
                        State::Tuple(i0, i1 * 10 + x as i16 - b'0' as i16)
                }
                byte => {
                    let _ = write!(self, "^[[{};{}{}", i0, i1, byte as char);
                }
            },
            State::Default => match byte {
                b'\0' => {}
                0x08 => self.back(),
                b'\t' => {
                    // tabs are just four spaces, god dammit
                    let n = 4 - (self.x & 3);
                    for _ in 0..n {
                        self.write_byte(b' ');
                    }
                }
                b'\n' => self.endl(),
                b'\r' => self.cr(),
                0x1b => self.state = State::Esc,
                byte if byte < 0x20 => {
                    self.write_byte(b'^');
                    self.write_byte(byte + b'A');
                }
                byte => {
                    let ch = Char(self.color.0, self.color.1, byte);
                    let offset = self.offset();
                    self.write_raw(u16::from(ch), offset);
                    self.next();
                }
            },
        }
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_byte(byte);
        }

        let offset = self.offset();
        self.write_raw(Char::default().into(), offset);

        // update cursor
        let offset = self.offset() as u16;
        let hi = (offset >> 8) as u8;
        let lo = (offset & 0xff) as u8;
        self.port_a.write_byte(0x0f);
        self.port_b.write_byte(lo);
        self.port_a.write_byte(0x0e);
        self.port_b.write_byte(hi);
    }
}

impl Write for Vga {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.write_bytes(s.as_bytes());
        Ok(())
    }
}
