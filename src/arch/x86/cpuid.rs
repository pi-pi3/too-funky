pub fn available() -> bool {
    let available: i32;
    unsafe {
        asm!("
            pushfl
            pushfl
            xorl    $$0x00200000, (%esp)
            popfl
            pushfl
            popl    %eax
            xorl    (%esp), %eax
            popfl
            andl    $$0x00200000, %eax
            " : "={eax}"(available)
        );
    }
    available != 0
}
