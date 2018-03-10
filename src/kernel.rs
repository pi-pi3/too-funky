#![feature(lang_items)]
#![feature(const_fn)]
#![feature(compiler_builtins_lib)]
#![feature(core_panic)]
#![no_std]

extern crate rlibc;
extern crate x86;
extern crate spin;
extern crate compiler_builtins;

use core::fmt;

#[macro_use]
pub mod macros;
pub mod drivers;

pub mod kernel {
    use drivers::vga::Vga;

    use spin::{Mutex, MutexGuard};

    pub static mut VGA: Option<Mutex<Vga>> = None;

    pub unsafe fn init_vga() {
        VGA = Some(Mutex::new(Vga::new()));
    }

    pub unsafe fn vga() -> MutexGuard<'static, Vga> {
        VGA.as_ref()
            .unwrap()
            .lock()
    }

    pub fn try_vga() -> Option<MutexGuard<'static, Vga>> {
        unsafe {
            VGA.as_ref()
                .and_then(|vga| vga.try_lock())
        }
    }
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    unsafe {
        kernel::init_vga();
    }

    kprintln!("OK");

    loop {}
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(msg: fmt::Arguments,
                        file: &'static str,
                        line: u32,
                        col: u32) -> ! {
    use core::fmt::Write;
    kernel::try_vga().map(|mut vga| {
        let _ = vga.write_str("kernel panicked at '");
        let _ = vga.write_fmt(msg);
        let _ = write!(vga, "', {}:{}:{}", file, line, col);
    });

    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn eh_personality() {
}
