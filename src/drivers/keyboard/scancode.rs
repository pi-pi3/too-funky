use x86::shared::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scancode {
    // byte code in big endian
    Invalid,
    Pressed([u8; 8]),
    Released([u8; 8]),
}

impl Scancode {
    pub fn try_unwrap(self) -> Option<[u8; 8]> {
        match self {
            Scancode::Invalid => None,
            Scancode::Pressed(inner) => Some(inner),
            Scancode::Released(inner) => Some(inner),
        }
    }

    pub fn unwrap(self) -> [u8; 8] {
        self.try_unwrap()
            .expect("attempt to call `unwrap` on `Invalid` scancode")
    }

    pub fn is_valid(&self) -> bool {
        match *self {
            Scancode::Invalid => false,
            _ => true,
        }
    }

    pub fn is_pressed(&self) -> bool {
        match *self {
            Scancode::Pressed(_) => true,
            _ => false,
        }
    }

    pub unsafe fn poll(port: u16) -> Scancode {
        let mut byte = io::inb(port);
        while byte == 0 {
            byte = io::inb(port)
        }

        match byte {
            0xe0 => Scancode::poll_long(port),
            0xe1 => Scancode::poll_very_long(port),
            byte if byte < 0x80 => {
                Scancode::Pressed([byte, 0, 0, 0, 0, 0, 0, 0])
            }
            byte => Scancode::Released([byte, 0, 0, 0, 0, 0, 0, 0]),
        }
    }

    unsafe fn poll_long(port: u16) -> Scancode {
        match io::inb(port) {
            0x2a => {
                if io::inb(port) == 0xe0 && io::inb(port) == 0x37 {
                    Scancode::Pressed([0xe0, 0x2a, 0xe0, 0x37, 0, 0, 0, 0])
                } else {
                    Scancode::Invalid
                }
            }
            0xb7 => {
                if io::inb(port) == 0xe0 && io::inb(port) == 0xaa {
                    Scancode::Pressed([0xe0, 0xb7, 0xe0, 0xaa, 0, 0, 0, 0])
                } else {
                    Scancode::Invalid
                }
            }
            byte if byte < 0x80 => {
                Scancode::Pressed([0xe0, byte, 0, 0, 0, 0, 0, 0])
            }
            byte => Scancode::Released([0xe0, byte, 0, 0, 0, 0, 0, 0]),
        }
    }

    unsafe fn poll_very_long(port: u16) -> Scancode {
        if io::inb(port) == 0x1d && io::inb(port) == 0x45
            && io::inb(port) == 0xe1 && io::inb(port) == 0x9d
            && io::inb(port) == 0xc5
        {
            Scancode::Pressed([0xe1, 0x1d, 0x45, 0xe1, 0x9d, 0xc5, 0, 0])
        } else {
            Scancode::Invalid
        }
    }
}
