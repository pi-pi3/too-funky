pub mod idt;
pub mod exceptions;

use self::idt::Idtr;

#[repr(C, packed)]
pub struct ExceptionStackFrame {
    pub eip: usize,
    pub cs: usize,
    pub eflags: usize,
    pub sp: usize,
    pub ss: usize,
}

pub type InterruptHandler =
    unsafe extern "x86-interrupt" fn(&ExceptionStackFrame);
pub type ExceptionHandler =
    unsafe extern "x86-interrupt" fn(&ExceptionStackFrame, usize);

pub unsafe fn lidt(idtr: &Idtr) {
    asm!("lidtl   $0" : : "*m"(idtr) : "memory" : "volatile");
}
