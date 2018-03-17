#![feature(lang_items)]
#![feature(const_fn)]
#![feature(compiler_builtins_lib)]
#![feature(core_panic)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]
#![feature(fn_must_use)]
#![feature(global_allocator)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(ptr_internals)]
#![no_std]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;
#[macro_use]
extern crate bitflags;
extern crate compiler_builtins;
extern crate rlibc;
extern crate spin;
extern crate x86;
extern crate multiboot2;
extern crate linked_list_allocator;

#[macro_use]
pub mod macros;

#[macro_use]
#[cfg_attr(target_arch = "x86", path = "arch/x86/mod.rs")]
pub mod arch;
pub mod panic;
pub mod mem;
pub mod drivers;
pub mod syscall;

#[path = "arch/x86/mod.rs"]
#[cfg(rustfmt)]
pub mod arch_x86;

use drivers::keyboard::{self, Keycode};

// global_allocator doesn't work in modules
// tracking issue: #27389
// issue: #44113
use linked_list_allocator::LockedHeap;
#[global_allocator]
pub static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn kmain() {
    kprint!("> ");
    loop {
        match keyboard::poll().unwrap() {
            Keycode::Enter => kprint!("\n> "),
            k => kprint!("{}", k),
        }
    }
}
