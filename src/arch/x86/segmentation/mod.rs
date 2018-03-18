pub mod gdt;

use x86::shared::segmentation::{load_ds, load_es, load_fs, load_gs, load_ss,
                                set_cs, SegmentSelector};

use self::gdt::Gdtr;

pub unsafe fn lgdt(gdtr: &Gdtr) {
    asm!("lgdtl   $0" : : "*m"(gdtr) : "memory" : "volatile");
}

pub unsafe fn reload_segments(code: u16, data: u16) {
    set_cs(SegmentSelector::from_raw(code));
    load_ds(SegmentSelector::from_raw(data));
    load_es(SegmentSelector::from_raw(data));
    load_fs(SegmentSelector::from_raw(data));
    load_gs(SegmentSelector::from_raw(data));
    load_ss(SegmentSelector::from_raw(data));
}
