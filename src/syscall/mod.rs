use arch::interrupt::ExceptionStackFrame;

pub unsafe extern "x86-interrupt" fn handler(
    _stack_frame: &ExceptionStackFrame,
) {
    kprintln!("syscall");
}
