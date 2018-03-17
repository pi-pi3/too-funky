interrupt_handlers! {
    pub unsafe extern fn de<()>(;) {
        panic!("divide-by-zero error"); // 0x0
    }

    pub unsafe extern fn db<()>(;) {
        panic!("debug"); // 0x1
    }

    pub unsafe extern fn ni<()>(;) {
        panic!("non-maskable interrupt"); // 0x2
    }

    pub unsafe extern fn bp<()>(;) {
        panic!("breakpoint"); // 0x3
    }

    pub unsafe extern fn of<()>(;) {
        panic!("overflow"); // 0x4
    }

    pub unsafe extern fn br<()>(;) {
        panic!("bound range exceeded"); // 0x5
    }

    pub unsafe extern fn ud<()>(;) {
        panic!("invalid opcode"); // 0x6
    }

    pub unsafe extern fn nm<()>(;) {
        panic!("device not available"); // 0x7
    }

    pub unsafe extern fn df<u32>(;) {
        panic!("double fault"); // 0x8
    }

    pub unsafe extern fn ts<u32>(;) {
        panic!("invalid tss"); // 0xA
    }

    pub unsafe extern fn np<u32>(;) {
        panic!("segment not present"); // 0xB
    }

    pub unsafe extern fn ss<u32>(;) {
        panic!("stack-segment fault"); // 0xC
    }

    pub unsafe extern fn gp<u32>(;) {
        panic!("general protection fault"); // 0xD
    }

    pub unsafe extern fn pf<u32>(;) {
        panic!("page fault"); // 0xE
    }

    pub unsafe extern fn mf<()>(;) {
        panic!("x87"); // 0x10
    }

    pub unsafe extern fn ac<u32>(;) {
        panic!("alignment check"); // 0x11
    }

    pub unsafe extern fn mc<()>(;) {
        panic!("machine check"); // 0x12
    }

    pub unsafe extern fn xm<()>(;) {
        panic!("simd floating-point exception"); // 0x13
    }

    pub unsafe extern fn ve<()>(;) {
        panic!("virtualization exception"); // 0x14
    }

    pub unsafe extern fn sx<u32>(;) {
        panic!("security exception"); // 0x1E
    }
}
