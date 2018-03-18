use port::Port;

use super::*;

#[allow(dead_code)] // most commands aren't used
mod consts {
    pub const PORT: u16 = 0x60;
    pub const ATTEMPT_COUNT: usize = 3;

    pub const ECHO: u8 = 0xee;
    pub const LED: u8 = 0xed;
    pub const SCANSET: u8 = 0xf0; // 0 = get, 1, 2, 3 = set x
    pub const IDTY: u8 = 0xf2;
    pub const TYPE: u8 = 0xf3;
    pub const ENABLE: u8 = 0xf4;
    pub const DISABLE: u8 = 0xf5;
    pub const DEFAULT: u8 = 0xf6;
    pub const ALL_TYPEMATIC: u8 = 0xf7;
    pub const ALL_MAKE_REL: u8 = 0xf8;
    pub const ALL_MAKE: u8 = 0xf9;
    pub const ALL_TPMT_MAKE_REL: u8 = 0xfa;
    pub const KEY_TYPEMATIC: u8 = 0xfb;
    pub const KEY_MAKE_REL: u8 = 0xfc;
    pub const KEY_MAKE: u8 = 0xfd;
    pub const RESET: u8 = 0xff;

    pub const ERR_1: u8 = 0x00;
    pub const ERR_2: u8 = 0xff;
    pub const TEST_PASS: u8 = 0xaa;
    pub const TEST_FAIL_1: u8 = 0xfc;
    pub const TEST_FAIL_2: u8 = 0xfd;
    pub const ACK: u8 = 0xfa;
    pub const RESEND: u8 = 0xfe;
}

use self::consts::*;

bitflags! {
    #[derive(Default)]
    struct Led: u8 {
        const SCROLL = 0b0001;
        const NUM = 0b0010;
        const CAPS = 0b0100;
        const KANA = 0b1000; // used by some jap keyboards
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scanset {
    // only set 1 is supported
    Set1,
    /* Set2,
     * Set3, */
}

impl Default for Scanset {
    fn default() -> Scanset {
        Scanset::Set1
    }
}

impl Scanset {
    fn into_byte(self) -> u8 {
        match self {
            Scanset::Set1 => 1,
            /* Scanset::Set2 => 2,
             * Scanset::Set3 => 3, */
        }
    }
}

#[allow(dead_code)] // most commands aren't used
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Command {
    Echo,
    Led(Led),
    GetScanset,
    SetScanset(Scanset),
    Idty,
    Typematic(u8, u16), // repeat, delay
    Enable,
    Disable,
    Default,
    AllTypematic,
    AllMakeRel,
    AllMake,
    AllTpmtMakeRel,
    // not supported
    // KeyTypematic(Scancode),
    // KeyMakeRel(Scancode),
    // KeyMake(Scancode),
    Resend,
    Reset,
}

impl Command {
    pub fn into_bytes(self) -> (u8, Option<u8>) {
        match self {
            Command::Echo => (ECHO, None),
            Command::Led(led) => (LED, Some(led.bits())),
            Command::GetScanset => (SCANSET, Some(0)),
            Command::SetScanset(set) => (SCANSET, Some(set.into_byte())),
            Command::Idty => (IDTY, None),
            Command::Typematic(rep, delay) => (
                TYPE,
                Some((rep & 0x1f) | (((delay / 250 - 1) & 0x3) << 5) as u8),
            ),
            Command::Enable => (ENABLE, None),
            Command::Disable => (DISABLE, None),
            Command::Default => (DEFAULT, None),
            Command::AllTypematic => (ALL_TYPEMATIC, None),
            Command::AllMakeRel => (ALL_MAKE_REL, None),
            Command::AllMake => (ALL_MAKE, None),
            Command::AllTpmtMakeRel => (ALL_TPMT_MAKE_REL, None),
            // Command::KeyTypematic(code)     => (KEY_TYPEMATIC,
            // Some(code.into_byte())), Command::KeyMakeRel(code)
            // => (KEY_MAKE_REL, Some(code.into_byte())),
            // Command::KeyMake(code)          => (KEY_MAKE,
            // Some(code.into_byte())),
            Command::Resend => (RESET, None),
            Command::Reset => (RESET, None),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Response {
    Ack,
    Resend,
    Echo,
    Err1,
    Err2,
    Pass,
    Fail1,
    Fail2,
}

impl Response {
    pub fn from_byte(byte: u8) -> Option<Response> {
        match byte {
            ACK => Some(Response::Ack),
            RESEND => Some(Response::Resend),
            ECHO => Some(Response::Echo),
            ERR_1 => Some(Response::Err1),
            ERR_2 => Some(Response::Err2),
            TEST_PASS => Some(Response::Pass),
            TEST_FAIL_1 => Some(Response::Fail1),
            TEST_FAIL_2 => Some(Response::Fail2),
            _ => None,
        }
    }
}

pub struct Keyboard<'a> {
    port: Port,
    set: Scanset,
    led: Led,
    repeat: u8,
    delay: u16,
    keys: &'a mut [bool],
    input: &'a mut [Keycode],
    input_size: usize,
    modifier: Mod,
}

impl<'a> Keyboard<'a> {
    pub fn new(
        repeat: u8,
        delay: u16,
        keys: &'a mut [bool],
        input: &'a mut [Keycode],
        set: Scanset,
    ) -> Option<Keyboard<'a>> {
        let mut key = Keyboard {
            port: unsafe { Port::new(PORT) },
            set,
            led: Led::default(),
            repeat,
            delay,
            keys,
            input,
            input_size: 0,
            modifier: Mod::default(),
        };

        key.reset()
            .map(|key| {
                key.reinit();
            })
            .map(|_| key)
    }

    pub fn reset(&mut self) -> Option<&mut Self> {
        match self.send(Command::Reset) {
            Ok(Response::Ack) => Some(self),
            _ => None,
        }
    }

    pub fn reinit(&mut self) -> Option<&mut Self> {
        match self.send(Command::Echo) {
            Ok(Response::Echo) => {}
            _ => return None,
        }

        let set = self.set;
        match self.send(Command::SetScanset(set)) {
            Ok(Response::Ack) => {}
            _ => return None,
        }

        let led = self.led;
        match self.send(Command::Led(led)) {
            Ok(Response::Ack) => {}
            _ => return None,
        }

        match self.send(Command::Enable) {
            Ok(Response::Ack) => {}
            _ => return None,
        }

        let repeat = self.repeat;
        let delay = self.delay;
        match self.send(Command::Typematic(repeat, delay)) {
            Ok(Response::Ack) => {}
            _ => return None,
        }

        Some(self)
    }

    fn send(&mut self, com: Command) -> Result<Response, Option<Response>> {
        fn inner(
            port: &mut Port,
            com: Command,
            attempt: usize,
        ) -> Result<Response, Option<Response>> {
            if attempt > ATTEMPT_COUNT {
                return Err(None);
            }

            let (com, arg) = com.into_bytes();
            port.write_byte(com);
            arg.map(|arg| port.write_byte(arg));
            let resp = Response::from_byte(port.read_byte());
            match resp {
                Some(resp @ Response::Ack) => Ok(resp),
                Some(resp @ Response::Resend) => Ok(resp),
                Some(resp @ Response::Echo) => Ok(resp),
                Some(resp @ Response::Err1) => Err(Some(resp)),
                Some(resp @ Response::Err2) => Err(Some(resp)),
                Some(resp @ Response::Pass) => Ok(resp),
                Some(resp @ Response::Fail1) => Err(Some(resp)),
                Some(resp @ Response::Fail2) => Err(Some(resp)),
                None => Err(None),
            }
        }

        let mut attempt = 1;
        loop {
            match inner(&mut self.port, com, attempt) {
                Ok(Response::Resend) => attempt += 1,
                resp => break resp,
            }
        }
    }

    pub fn last(&mut self) -> Option<Keycode> {
        let input_size = self.input_size;
        if input_size > 0 {
            self.input_size -= 1;
            Some(self.input[input_size - 1])
        } else {
            None
        }
    }

    pub fn input(&mut self, scancode: Scancode) -> Option<Keycode> {
        if scancode.is_valid() {
            let keycode = Keycode::from_scancode_with_scanset(
                scancode.unwrap(),
                self.set,
            );

            let mod_bit = match keycode {
                Keycode::ControlLeft => Some(Mod::CONTROL_LEFT),
                Keycode::ControlRight => Some(Mod::CONTROL_RIGHT),
                Keycode::AltLeft => Some(Mod::ALT_LEFT),
                Keycode::AltRight => Some(Mod::ALT_RIGHT),
                Keycode::ShiftLeft => Some(Mod::SHIFT_LEFT),
                Keycode::ShiftRight => Some(Mod::SHIFT_RIGHT),
                Keycode::MetaLeft => Some(Mod::META_LEFT),
                Keycode::MetaRight => Some(Mod::META_RIGHT),
                Keycode::SuperLeft => Some(Mod::SUPER_LEFT),
                Keycode::SuperRight => Some(Mod::SUPER_RIGHT),
                _ => None,
            };
            mod_bit.map(|bit| self.modifier.set(bit, scancode.is_pressed()));

            self.keys[keycode.as_byte() as usize] = scancode.is_pressed();
            if scancode.is_pressed() && self.input_size < self.input.len() {
                self.input[self.input_size] = keycode;
                self.input_size += 1;
            }

            Some(keycode)
        } else {
            None
        }
    }

    pub fn modifiers(&self) -> Mod {
        self.modifier
    }

    pub fn is_pressed(&self, key: Keycode) -> bool {
        self.keys[key.as_byte() as usize]
    }
}
