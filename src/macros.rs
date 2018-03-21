use core::fmt::{self, Write};
use arch::kernel;

pub macro kprintln {
    () => {
        kprint!("\n")
    },
    ($fmt: expr) => {
        kprint!(concat!($fmt, "\n"))
    },
    ($fmt: expr, $($args: tt)*) => {
        kprint!(concat!($fmt, "\n"), $($args)*)
    }
}

pub macro kprint {
    () => { },
    ($fmt: expr) => {
        {
            let mut vga = kernel::vga();
            let _ = vga.write_str($fmt); // always succeeds
        }
    },
    ($fmt: expr, $($args: tt)*) => {
        {
            let result = {
                let mut vga = kernel::vga();
                write!(vga, $fmt, $($args)*)
            };

            if let Err(err) = result {
                panic!("{}", err);
            }
        }
    }
}
