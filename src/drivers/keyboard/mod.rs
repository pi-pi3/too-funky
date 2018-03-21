use spin::{Mutex, Once};
use x86::shared::irq;

use drivers::pic;
use arch::interrupt::ExceptionStackFrame;

mod keyboard;
mod scancode;
pub mod keycode;

use self::keyboard::*;
use self::scancode::*;
pub use self::keycode::*;
pub use self::keyboard::Scanset;

static mut KEYS: [bool; 256] = [false; 256];
static mut INPUT: [Keycode; 256] = [Keycode::Unknown; 256];
static KEYBOARD: Once<Option<Mutex<Keyboard<'static>>>> = Once::new();

pub unsafe extern "x86-interrupt" fn handler(
    _stack_frame: &ExceptionStackFrame,
) {
    let key = Scancode::poll();

    try_handle()
        .and_then(|keyboard| keyboard.try_lock())
        .and_then(|mut keyboard| keyboard.input(key));

    let mut pic = pic::try_handle().unwrap();
    pic.0.eoi();
    pic.1.eoi();
}

pub fn init(delay: u8, repeat: u16, scanset: Scanset) -> Result<(), ()> {
    let keyboard = KEYBOARD.call_once(move || {
        unsafe { Keyboard::new(delay, repeat, &mut KEYS, &mut INPUT, scanset) }
            .map(|keyboard| Mutex::new(keyboard))
    });

    keyboard.as_ref().map(|_| ()).ok_or(())
}

fn try_handle() -> Option<&'static Mutex<Keyboard<'static>>> {
    KEYBOARD.try().and_then(|keyboard| keyboard.as_ref())
}

pub fn poll() -> Option<Keycode> {
    try_handle().map(|keyboard| loop {
        unsafe {
            irq::disable();
        }

        let result = {
            let mut key = keyboard.lock();
            key.last()
        };

        unsafe {
            irq::enable();
        }

        if result.is_some() {
            break result.unwrap();
        }
    })
}

pub fn modifiers() -> Option<Mod> {
    try_handle().map(|keyboard| {
        unsafe {
            irq::disable();
        }

        let result = {
            let key = keyboard.lock();
            key.modifiers()
        };

        unsafe {
            irq::enable();
        }

        result
    })
}

pub fn is_pressed(keycode: Keycode) -> Option<bool> {
    try_handle().map(|keyboard| {
        unsafe {
            irq::disable();
        }

        let result = {
            let key = keyboard.lock();
            key.is_pressed(keycode)
        };

        unsafe {
            irq::enable();
        }

        result
    })
}
