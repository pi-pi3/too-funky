
interrupt_handlers! {
    pub unsafe extern fn syscall(num: eax) {
        kprintln!("syscall {}", num);
    }
}
