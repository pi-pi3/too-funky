use x86::shared::io;

const PORT: u16 = 0x60;

const ECHO: u8 = 0xee;
const LED: u8 = 0xed;
const SCANSET: u8 = 0xf0; // 0 = get, 1, 2, 3 = set x
const IDTY: u8 = 0xf2;
const TYPE: u8 = 0xf3;
const ENABLE: u8 = 0xf4;
const DISABLE: u8 = 0xf5;
const DEFAULT: u8 = 0xf6;
const ALL_TYPEMATIC: u8 = 0xf7;
const ALL_MAKE_REL: u8 = 0xf8;
const ALL_MAKE: u8 = 0xf9;
const ALL_TPMT_MAKE_REL: u8 = 0xfa;
const KEY_TYPEMATIC: u8 = 0xfb;
const KEY_MAKE_REL: u8 = 0xfc;
const KEY_MAKE: u8 = 0xfd;
const RESET: u8 = 0xff;

const ERR_1: u8 = 0x00;
const ERR_2: u8 = 0xff;
const TEST_PASS: u8 = 0xaa;
const TEST_FAIL_1: u8 = 0xfc;
const TEST_FAIL_2: u8 = 0xfc;
const ACK: u8 = 0xfa;
const RESEND: u8 = 0xfe;

interrupt_handlers! {
    pub unsafe extern fn key_handler() {
        let key = io::inb(0x60);
    }
}

bitflags! {
    struct Led: u8 {
        const SCROLL = 0b0001;
        const NUM = 0b0010;
        const CAPS = 0b0100;
        const KANA = 0b1000; // used by some jap keyboards
    }
}

enum Scanset {
    Set1,
    Set2,
    Set3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Command {
    Echo,
    Led(Led), // TODO: Led bitflags
    GetScanset,
    SetScanset(Scanset),
    Idty,
    Type,
    Enable,
    Disable,
    Default,
    AllTypematic,
    AllMakeRel,
    AllMake,
    AllTpmtMakeRel,
    KeyTypematic,
    KeyMakeRel,
    KeyMake,
    Reset,
}

impl Command {
    pub fn into_bytes(self) -> (u8, Option<u8>) {
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
    pub fn from_byte(byte:u8) -> Option<Response> {
        match byte {
            ACK          => Some(Response::Ack),
            RESEND       => Some(Response::Resend),
            ECHO         => Some(Response::Echo),
            ERR_1        => Some(Response::Err1),
            ERR_2        => Some(Response::Err2),
            TEST_PASS    => Some(Response::Pass),
            TEST_FAIL_1  => Some(Response::Fail1),
            TEST_FAIL_2  => Some(Response::Fail2),
            _  => None,
        }
    }
}

pub struct Keyboard {
}
