use arch::interrupt::ExceptionStackFrame;

pub unsafe extern "x86-interrupt" fn de(_stack_frame: &ExceptionStackFrame) {
    panic!("divide-by-zero error"); // 0x0
}

pub unsafe extern "x86-interrupt" fn db(_stack_frame: &ExceptionStackFrame) {
    panic!("debug"); // 0x1
}

pub unsafe extern "x86-interrupt" fn ni(_stack_frame: &ExceptionStackFrame) {
    panic!("non-maskable interrupt"); // 0x2
}

pub unsafe extern "x86-interrupt" fn bp(_stack_frame: &ExceptionStackFrame) {
    panic!("breakpoint"); // 0x3
}

pub unsafe extern "x86-interrupt" fn of(_stack_frame: &ExceptionStackFrame) {
    panic!("overflow"); // 0x4
}

pub unsafe extern "x86-interrupt" fn br(_stack_frame: &ExceptionStackFrame) {
    panic!("bound range exceeded"); // 0x5
}

pub unsafe extern "x86-interrupt" fn ud(_stack_frame: &ExceptionStackFrame) {
    panic!("invalid opcode"); // 0x6
}

pub unsafe extern "x86-interrupt" fn nm(_stack_frame: &ExceptionStackFrame) {
    panic!("device not available"); // 0x7
}

pub unsafe extern "x86-interrupt" fn df(
    _stack_frame: &ExceptionStackFrame,
    _code: usize,
) {
    panic!("double fault"); // 0x8
}

pub unsafe extern "x86-interrupt" fn ts(
    _stack_frame: &ExceptionStackFrame,
    _code: usize,
) {
    panic!("invalid tss"); // 0xA
}

pub unsafe extern "x86-interrupt" fn np(
    _stack_frame: &ExceptionStackFrame,
    _code: usize,
) {
    panic!("segment not present"); // 0xB
}

pub unsafe extern "x86-interrupt" fn ss(
    _stack_frame: &ExceptionStackFrame,
    _code: usize,
) {
    panic!("stack-segment fault"); // 0xC
}

pub unsafe extern "x86-interrupt" fn gp(
    _stack_frame: &ExceptionStackFrame,
    _code: usize,
) {
    panic!("general protection fault"); // 0xD
}

pub unsafe extern "x86-interrupt" fn pf(
    _stack_frame: &ExceptionStackFrame,
    _code: usize,
) {
    panic!("page fault"); // 0xE
}

pub unsafe extern "x86-interrupt" fn mf(_stack_frame: &ExceptionStackFrame) {
    panic!("x87"); // 0x10
}

pub unsafe extern "x86-interrupt" fn ac(
    _stack_frame: &ExceptionStackFrame,
    _code: usize,
) {
    panic!("alignment check"); // 0x11
}

pub unsafe extern "x86-interrupt" fn mc(_stack_frame: &ExceptionStackFrame) {
    panic!("machine check"); // 0x12
}

pub unsafe extern "x86-interrupt" fn xm(_stack_frame: &ExceptionStackFrame) {
    panic!("simd floating-point exception"); // 0x13
}

pub unsafe extern "x86-interrupt" fn ve(_stack_frame: &ExceptionStackFrame) {
    panic!("virtualization exception"); // 0x14
}

pub unsafe extern "x86-interrupt" fn sx(
    _stack_frame: &ExceptionStackFrame,
    _code: usize,
) {
    panic!("security exception"); // 0x1E
}
