#![feature(lang_items)]
#![feature(const_fn)]
#![feature(compiler_builtins_lib)]
#![feature(core_panic)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]
#![feature(fn_must_use)]
#![feature(global_allocator)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(ptr_internals)]
#![feature(nonzero)]
#![feature(abi_x86_interrupt)]
#![feature(decl_macro)]
#![no_std]
#![no_main]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;
extern crate bit_field;
#[macro_use]
extern crate bitflags;
extern crate compiler_builtins;
extern crate linked_list_allocator;
extern crate multiboot2;
#[macro_use]
extern crate once;
extern crate rlibc;
extern crate spin;
extern crate x86;

use x86::shared::irq;

#[macro_use]
pub mod macros;
#[macro_use]
#[cfg_attr(target_arch = "x86", path = "arch/x86/mod.rs")]
pub mod arch;
pub mod panic;
pub mod mem;
pub mod port;
pub mod drivers;
pub mod syscall;

#[path = "arch/x86/mod.rs"]
#[cfg(rustfmt)]
pub mod arch_x86;

use arch::Kinfo;
use drivers::vga;
use drivers::pic;
use drivers::keyboard::{self, Keycode, Scanset};
use macros::*;

// global_allocator doesn't work in modules
// tracking issue: #27389
// issue: #44113
use linked_list_allocator::LockedHeap;
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn kmain(kinfo: &Kinfo) {
    kprint!("paging... ");
    kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");

    kprint!("global descriptor table... ");
    kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");

    kprint!("interrupt descriptor table... ");
    kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");

    kprint!("memory areas... ");
    kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");
    kprintln!(
        "available memory: {}MB",
        kinfo.free_memory / (1024 * 1024),
    );

    kprint!("kernel heap... ");
    kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");

    kprintln!(
        "heap size: {}kB",
        kinfo.heap_size() / 1024,
    );

    kprint!("video graphics array driver... ");

    vga::init();

    kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");

    kprint!("keyboard driver... ");
    kprintln!(
        "{yellow}[SKIP]{reset}",
        yellow = "\x1b[33m",
        reset = "\x1b[0m"
    );

    kprint!("programmable interrupt controller... ");

    {
        pic::init();
        let mut pic = pic::handle();
        pic.0.set_all();
        pic.1.set_all();
    }

    kprintln!(
        "{green}[OK]{reset}",
        green = "\x1b[32m",
        reset = "\x1b[0m"
    );

    unsafe {
        irq::enable();
    }

    kprint!("> ");
}
