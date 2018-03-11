
#[macro_use]
pub mod macros;
pub mod idt;
pub mod pic;

use self::macros::{NoCall, ErrArgs};
use self::idt::Idtr;

pub type InterruptHandler = unsafe extern fn(NoCall);
pub type ExceptionHandler<E> = unsafe extern fn(ErrArgs<E>, NoCall);

pub unsafe fn lidt(idtr: &Idtr) {
    asm!("lidt [$0]": : "r"(idtr) : "memory" : "intel", "volatile");
}
