#![feature(lang_items)]
#![no_std]

use core::fmt;

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    unsafe {
        let vga = 0xe00b8000 as *mut u16;
        *vga.offset(0) = 0x0200 | 'O' as u32 as u16;
        *vga.offset(1) = 0x0200 | 'K' as u32 as u16;
    }
    loop {}
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(_msg: fmt::Arguments,
                               _file: &'static str,
                               _line: u32) -> ! {
    loop {}
}
