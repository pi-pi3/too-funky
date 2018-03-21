use core::fmt::Write;
use drivers::vga;

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
            let mut vga = vga::handle();
            let _ = vga.write_str($fmt); // always succeeds
        }
    },
    ($fmt: expr, $($args: tt)*) => {
        {
            let result = {
                let mut vga = vga::handle();
                write!(vga, $fmt, $($args)*)
            };

            if let Err(err) = result {
                panic!("{}", err);
            }
        }
    }
}
