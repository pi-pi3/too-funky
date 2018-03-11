#![feature(lang_items)]
#![feature(const_fn)]
#![feature(compiler_builtins_lib)]
#![feature(core_panic)]
#![feature(asm)]
#![feature(naked_functions)]
#![no_std]

extern crate rlibc;
extern crate x86;
extern crate spin;
#[macro_use]
extern crate bitflags;
extern crate compiler_builtins;

use core::fmt;

#[macro_use]
pub mod macros;
pub mod interrupt;
pub mod drivers;

pub mod kernel {
    use spin::{Mutex, MutexGuard};

    use drivers::vga::Vga;
    use interrupt::lidt;
    use interrupt::idt::{self, Idt, Idtr};

    static mut VGA: Option<Mutex<Vga>> = None;
    static mut IDTR: Option<Idtr> = None; 
    static mut IDT_INNER: [idt::Entry; 256] = [idt::Entry::empty(); 256];
    static mut IDT: Idt = unsafe { Idt { inner: &mut IDT_INNER } }; // IDT_INNER is known to be valid

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

    pub fn init_idt() {
        unsafe {
            IDTR = Some(IDT.idtr());
            // isr should be set here
            /* ... */
            lidt(IDTR.as_ref().unwrap());
        }
    }
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    unsafe {
        kernel::init_vga();
    }
    kernel::init_idt();

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
