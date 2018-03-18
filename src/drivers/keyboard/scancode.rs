use port::Port;

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

    pub unsafe fn poll() -> Scancode {
        static mut PORT: Port = unsafe { Port::new(0x60) };

        let mut byte = PORT.read_byte();
        while byte == 0 {
            byte = PORT.read_byte();
        }

        match byte {
            0xe0 => Scancode::poll_long(&mut PORT),
            0xe1 => Scancode::poll_very_long(&mut PORT),
            byte if byte < 0x80 => {
                Scancode::Pressed([byte, 0, 0, 0, 0, 0, 0, 0])
            }
            byte => Scancode::Released([byte, 0, 0, 0, 0, 0, 0, 0]),
        }
    }

    unsafe fn poll_long(port: &mut Port) -> Scancode {
        match port.read_byte() {
            0x2a => {
                if port.read_byte() == 0xe0 && port.read_byte() == 0x37 {
                    Scancode::Pressed([0xe0, 0x2a, 0xe0, 0x37, 0, 0, 0, 0])
                } else {
                    Scancode::Invalid
                }
            }
            0xb7 => {
                if port.read_byte() == 0xe0 && port.read_byte() == 0xaa {
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

    unsafe fn poll_very_long(port: &mut Port) -> Scancode {
        if port.read_byte() == 0x1d && port.read_byte() == 0x45
            && port.read_byte() == 0xe1 && port.read_byte() == 0x9d
            && port.read_byte() == 0xc5
        {
            Scancode::Pressed([0xe1, 0x1d, 0x45, 0xe1, 0x9d, 0xc5, 0, 0])
        } else {
            Scancode::Invalid
        }
    }
}
