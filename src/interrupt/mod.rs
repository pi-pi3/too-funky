
pub mod macros;
pub mod idt;
pub mod pic;

use self::macros::{NoCall, ErrArgs};
use self::idt::Idtr;

pub type InterruptHandler = unsafe extern fn(NoCall);
pub type ExceptionHandler<E> = unsafe extern fn(ErrArgs<E>, NoCall);

pub struct Handler {
    addr: usize,
}

impl Handler {
    pub fn into_inner(self) -> usize {
        self.addr
    }
}

impl From<InterruptHandler> for Handler {
    fn from(isr: InterruptHandler) -> Handler {
        Handler { addr: isr as *const () as usize }
    }
}

impl<E> From<ExceptionHandler<E>> for Handler {
    fn from(isr: ExceptionHandler<E>) -> Handler {
        Handler { addr: isr as *const () as usize }
    }
}

pub unsafe fn lidt(idtr: &Idtr) {
    asm!("lidt [$0]": : "r"(idtr) : "memory" : "intel", "volatile");
}
