
interrupt_handlers! {
    pub unsafe extern fn handler(num: eax) {
        kprintln!("syscall {}",num);
    }
}
