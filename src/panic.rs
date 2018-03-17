use core::fmt::{self, Write};

use kernel;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn panic_fmt(
    msg: fmt::Arguments,
    file: &'static str,
    line: u32,
    col: u32,
) -> ! {
    kernel::try_vga().map(|mut vga| {
        let _ = vga.write_str("\x1b[0;31mkernel panicked at '");
        let _ = vga.write_fmt(msg);
        let _ = write!(vga, "', {}:{}:{}\x1b[0m", file, line, col);
    });

    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}
