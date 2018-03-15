use core::fmt::{self, Display};

use drivers::keyboard::Scanset;

bitflags! {
    #[derive(Default)]
    pub struct Mod: u16 {
        const CONTROL_LEFT = 1 << 0;
        const CONTROL_RIGHT = 1 << 1;
        const ALT_LEFT = 1 << 2;
        const ALT_RIGHT = 1 << 3;
        const SHIFT_LEFT = 1 << 4;
        const SHIFT_RIGHT = 1 << 5;
        const META_LEFT = 1 << 6;
        const META_RIGHT = 1 << 7;
        const SUPER_LEFT = 1 << 8;
        const SUPER_RIGHT = 1 << 9;
    }
}

impl Mod {
    pub fn iscontrol(&self) -> bool {
        self.intersects(Mod::CONTROL_LEFT | Mod::CONTROL_RIGHT)
    }

    pub fn isalt(&self) -> bool {
        self.intersects(Mod::ALT_LEFT | Mod::ALT_RIGHT)
    }

    pub fn isshift(&self) -> bool {
        self.intersects(Mod::SHIFT_LEFT | Mod::SHIFT_RIGHT)
    }

    pub fn ismeta(&self) -> bool {
        self.intersects(Mod::META_LEFT | Mod::META_RIGHT)
    }

    pub fn issuper(&self) -> bool {
        self.intersects(Mod::SUPER_LEFT | Mod::SUPER_RIGHT)
    }
}

// incomplete list of keycodes
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Keycode {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    Plus,
    Minus,
    Asterisk,
    Equal,

    Backslash,
    Slash,
    BracketLeft,
    BracketRight,
    Semicolon,
    Colon,
    SingleQuote,
    DoubleQuote,
    Backtick,
    Comma,
    Period,

    Enter,
    Space,
    Tab,
    Backspace,

    NumPlus,
    NumMinus,
    NumAsterisk,
    NumSlash,
    NumComma,
    NumPeriod,

    NumZero,
    NumOne,
    NumTwo,
    NumThree,
    NumFour,
    NumFive,
    NumSix,
    NumSeven,
    NumEight,
    NumNine,

    CapsLock,
    NumLock,
    ScrollLock,

    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    ControlLeft,
    ControlRight,
    AltLeft,
    AltRight,
    ShiftLeft,
    ShiftRight,
    MetaLeft,
    MetaRight,
    SuperLeft,
    SuperRight,

    Unknown,
}

impl Keycode {
    pub fn from_scancode(code: [u8; 8]) -> Keycode {
        Keycode::from_scancode_with_scanset(code, Scanset::default())
    }

    pub fn from_scancode_with_scanset(
        scancode: [u8; 8],
        _set: Scanset,
    ) -> Keycode {
        match scancode[0] {
            0xe0 => unimplemented!(),
            0xe1 => unimplemented!(),
            byte if byte >= 0x80 => SCANSET_1[byte as usize - 0x80],
            byte => SCANSET_1[byte as usize],
        }
    }

    pub fn as_byte(&self) -> u8 {
        // this might be unstable,
        // but it might be fine,
        // because it doesn't really matter what this returns,
        // as long as its unique
        use core::intrinsics;
        unsafe { intrinsics::discriminant_value(self) as u8 }
    }

    pub fn into_char(self) -> Option<u8> {
        match CHARS[self.as_byte() as usize] {
            0xff => None,
            ch => Some(ch),
        }
    }
}

impl Display for Keycode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.into_char()
            .map(|ch| write!(f, "{}", ch as char))
            .unwrap_or_else(|| write!(f, "<{:?}>", self))
    }
}

// us qwerty
static SCANSET_1: [Keycode; 0x80] = [
    Keycode::Unknown,      // 0x00
    Keycode::Escape,       // 0x01
    Keycode::One,          // 0x02
    Keycode::Two,          // 0x03
    Keycode::Three,        // 0x04
    Keycode::Four,         // 0x05
    Keycode::Five,         // 0x06
    Keycode::Six,          // 0x07
    Keycode::Seven,        // 0x08
    Keycode::Eight,        // 0x09
    Keycode::Nine,         // 0x0A
    Keycode::Zero,         // 0x0B
    Keycode::Minus,        // 0x0C
    Keycode::Equal,        // 0x0D
    Keycode::Backspace,    // 0x0E
    Keycode::Tab,          // 0x0F
    Keycode::Q,            // 0x10
    Keycode::W,            // 0x11
    Keycode::E,            // 0x12
    Keycode::R,            // 0x13
    Keycode::T,            // 0x14
    Keycode::Y,            // 0x15
    Keycode::U,            // 0x16
    Keycode::I,            // 0x17
    Keycode::O,            // 0x18
    Keycode::P,            // 0x19
    Keycode::BracketLeft,  // 0x1A
    Keycode::BracketRight, // 0x1B
    Keycode::Enter,        // 0x1C
    Keycode::ControlLeft,  // 0x1D
    Keycode::A,            // 0x1E
    Keycode::S,            // 0x1F
    Keycode::D,            // 0x20
    Keycode::F,            // 0x21
    Keycode::G,            // 0x22
    Keycode::H,            // 0x23
    Keycode::J,            // 0x24
    Keycode::K,            // 0x25
    Keycode::L,            // 0x26
    Keycode::Semicolon,    // 0x27
    Keycode::SingleQuote,  // 0x28
    Keycode::Backtick,     // 0x29
    Keycode::ShiftLeft,    // 0x2A
    Keycode::Backslash,    // 0x2B
    Keycode::Z,            // 0x2C
    Keycode::X,            // 0x2D
    Keycode::C,            // 0x2E
    Keycode::V,            // 0x2F
    Keycode::B,            // 0x30
    Keycode::N,            // 0x31
    Keycode::M,            // 0x32
    Keycode::Comma,        // 0x33
    Keycode::Period,       // 0x34
    Keycode::Slash,        // 0x35
    Keycode::ShiftRight,   // 0x36
    Keycode::NumAsterisk,  // 0x37
    Keycode::AltLeft,      // 0x38
    Keycode::Space,        // 0x39
    Keycode::CapsLock,     // 0x3A
    Keycode::F1,           // 0x3B
    Keycode::F2,           // 0x3C
    Keycode::F3,           // 0x3D
    Keycode::F4,           // 0x3E
    Keycode::F5,           // 0x3F
    Keycode::F6,           // 0x40
    Keycode::F7,           // 0x41
    Keycode::F8,           // 0x42
    Keycode::F9,           // 0x43
    Keycode::F10,          // 0x44
    Keycode::NumLock,      // 0x45
    Keycode::ScrollLock,   // 0x46
    Keycode::NumSeven,     // 0x47
    Keycode::NumEight,     // 0x48
    Keycode::NumNine,      // 0x49
    Keycode::NumMinus,     // 0x4A
    Keycode::NumFour,      // 0x4B
    Keycode::NumFive,      // 0x4C
    Keycode::NumSix,       // 0x4D
    Keycode::NumPlus,      // 0x4E
    Keycode::NumOne,       // 0x4F
    Keycode::NumTwo,       // 0x50
    Keycode::NumThree,     // 0x51
    Keycode::NumZero,      // 0x52
    Keycode::NumPeriod,    // 0x53
    Keycode::Unknown,      // 0x54
    Keycode::Unknown,      // 0x55
    Keycode::Unknown,      // 0x56
    Keycode::F11,          // 0x57
    Keycode::F12,          // 0x58
    // filler
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
    Keycode::Unknown,
];

static CHARS: [u8; 0x80] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b',
    b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n',
    b'o', b'p', b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z',
    b'+', b'-', b'*', b'=', b'\\', b'/', b'[', b']', b';', b':', b'\'', b'"',
    b'`', b',', b'.', b'\n', b' ', b'\t', 0x08, b'+', b'-', b'*', b'/', b',',
    b'.', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, /* filler bytes */ 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
];
