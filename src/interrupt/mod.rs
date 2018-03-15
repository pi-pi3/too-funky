#[macro_use]
pub mod macros;
pub mod idt;
pub mod exceptions;

use self::macros::{ErrArgs, NoCall};
use self::idt::Idtr;

pub type InterruptHandler = unsafe extern "C" fn(NoCall);
pub type ExceptionHandler<E> = unsafe extern "C" fn(ErrArgs<E>, NoCall);

pub unsafe fn lidt(idtr: &Idtr) {
    // compiler bug?
    // intel syntax equivalent doesn't work
    //asm!("
    //        lidt    dword ptr $0
    //     " : : "*m"(idtr) : "memory" : "intel", "volatile"
    //);
    asm!("
            lidtl   $0
         " : : "*m"(idtr) : "memory" : "volatile"
    );
}
