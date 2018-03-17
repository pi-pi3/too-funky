use core::fmt::{self, Write};
use arch::kernel;

#[macro_export]
macro_rules! kprintln {
    () => {};
    ($fmt:expr) => {
        kprint!(concat!($fmt, "\n"))
    };
    ($fmt:expr, $($args:tt)*) => {
        kprint!(concat!($fmt, "\n"), $( $args )*)
    }
}

#[macro_export]
macro_rules! kprint {
    () => {};
    ($fmt:expr) => {
        $crate::macros::kputs($fmt)
    };
    ($fmt:expr, $($args:tt)*) => {
        $crate::macros::kprintf(
            format_args!($fmt, $( $args )*),
            file!(),
            line!()
        )
    }
}

pub fn kprintf(args: fmt::Arguments, file: &'static str, line: u32) {
    let result = {
        let mut vga = unsafe { kernel::vga() };
        vga.write_fmt(args)
    };

    if let Err(err) = result {
        use core::panicking::panic_fmt;
        panic_fmt(format_args!("{}", err), &(file, line, 0));
    }
}

pub fn kputs(string: &str) {
    let mut vga = unsafe { kernel::vga() };
    let _ = vga.write_str(string); // always succeeds
}
