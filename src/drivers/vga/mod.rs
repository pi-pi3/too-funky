use core::ptr::Unique;

use spin::{Mutex, MutexGuard, Once};

pub mod driver;

pub use self::driver::Vga;

const VGA_BASE: usize = 0xe00b8000;
static VGA: Once<Mutex<Vga>> = Once::new();

pub fn init() -> &'static Mutex<Vga> {
    let ptr = VGA_BASE as *mut _;
    let ptr = unsafe { Unique::new_unchecked(ptr) };
    VGA.call_once(|| Mutex::new(Vga::new(ptr)))
}

pub fn handle() -> MutexGuard<'static, Vga> {
    init().lock()
}

pub fn try_handle() -> Option<MutexGuard<'static, Vga>> {
    VGA.try().and_then(|vga| vga.try_lock())
}
