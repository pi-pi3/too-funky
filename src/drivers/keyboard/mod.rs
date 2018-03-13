use spin::Mutex;

mod keyboard;
mod scancode;
pub mod keycode;

use self::keyboard::*;
use self::scancode::*;
pub use self::keycode::*;
pub use self::keyboard::Scanset;

static mut KEYS: [bool; 256] = [false; 256];
static mut INPUT: [Keycode; 256] = [Keycode::Unknown; 256];
static mut KEYBOARD: Option<Mutex<Keyboard<'static>>> = None;

interrupt_handlers! {
    pub unsafe extern fn handler() {
        let key = Scancode::poll(0x60);
        KEYBOARD.as_mut()
            .and_then(|keyboard| keyboard.try_lock())
            .map(|mut keyboard| {
                keyboard.input(key);
            });

        use kernel;
        let mut pic = kernel::try_pic().unwrap();
        pic.0.eoi();
        pic.1.eoi();
    }
}

pub unsafe fn init_keys(delay: u8, repeat: u16, scanset: Scanset) -> Result<(), ()> {
    let keyboard = Keyboard::new(delay, repeat, &mut KEYS, &mut INPUT, scanset)
        .map(|keyboard| Mutex::new(keyboard));
    if keyboard.is_some() {
        KEYBOARD = keyboard;
        Ok(())
    } else {
        Err(())
    }
}
