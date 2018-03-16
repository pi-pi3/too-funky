pub mod gdt;

use self::gdt::Gdtr;

pub unsafe fn lgdt(gdtr: &Gdtr) {
    asm!("lgdtl   $0" : : "*m"(gdtr) : "memory" : "volatile");
}

pub unsafe fn reload_segments(_code: u16, data: u16) {
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
