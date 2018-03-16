#[macro_use]
pub mod macros;
pub mod idt;
pub mod exceptions;

use self::macros::{ErrArgs, NoCall};
use self::idt::Idtr;

pub type InterruptHandler = unsafe extern "C" fn(NoCall);
pub type ExceptionHandler<E> = unsafe extern "C" fn(ErrArgs<E>, NoCall);

pub unsafe fn lidt(idtr: &Idtr) {
    asm!("lidtl   $0" : : "*m"(idtr) : "memory" : "volatile");
}
