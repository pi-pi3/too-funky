
pub mod gdt;

use self::gdt::Gdtr;

pub unsafe fn lgdt(gdtr: &Gdtr) {
    // compiler bug?
    // intel syntax equivalent doesn't work
    //asm!("
    //        lgdt    dword ptr $0
    //     " : : "*m"(gdtr) : "memory" : "intel", "volatile"
    //);
    asm!("
            lgdtl   $0
         " : : "*m"(gdtr) : "memory" : "volatile"
    );
}

pub unsafe fn reload_segments(_code: u16, data: u16) {
    // again, the intel syntax equivalent isn't working
    // perhaps gas is superior?
    // gas > intel
    // nasm > gas
    asm!("
            ljmpl   $$0x8, $$reload_segments.long_jump
        reload_segments.long_jump:
            movw    $0, %ds
            movw    $0, %es
            movw    $0, %fs
            movw    $0, %gs
            movw    $0, %ss
         " : : "r"(data) : : "volatile"
    );
}
