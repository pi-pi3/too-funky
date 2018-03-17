use kmain;

#[macro_use]
pub mod interrupt;
pub mod paging;
pub mod segmentation;

use drivers::pic::{self, Pic};
use drivers::keyboard::{self, Scanset};
use mem::frame::Allocator as FrameAllocator;
use mem::page::Allocator as PageAllocator;
use arch::paging::table::ActiveTable;

#[no_mangle]
pub unsafe extern "C" fn _rust_start() -> ! {
    let page_table = kernel::init_paging();

    kinit(page_table);
    kmain();

    loop {}
}

pub unsafe fn kinit(page_table: ActiveTable<'static>) {
    use x86::shared::irq;
    // TODO: get available from multiboot tags
    let frame_alloc = FrameAllocator::with_range(0..usize::max_value());
    let page_alloc = PageAllocator::with_used(&page_table);
    kernel::set_allocator_pair(frame_alloc, page_alloc);
    kernel::set_page_table(page_table);

    kernel::init_vga();

    kprint!("paging... ");
    kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");

    kprint!("video graphics array driver... ");
    kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");

    kprint!("global descriptor table... ");
    kernel::init_gdt();
    kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");

    kprint!("interrupt descriptor table... ");
    kernel::init_idt();
    kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");

    kprint!("keyboard driver... ");
    let _ = keyboard::init_keys(0, 250, Scanset::Set1)
        .map(|_| {
            kprintln!(
                "{green}[OK]{reset}",
                green = "\x1b[32m",
                reset = "\x1b[0m"
            )
        })
        .map_err(|_| {
            kprintln!("{red}[ERR]{reset}", red = "\x1b[31m", reset = "\x1b[0m")
        });

    {
        kprint!("programmable interrupt controller... ");
        kernel::init_pic(Pic::new(pic::PIC1), Pic::new(pic::PIC2));

        let mut pic = kernel::pic();
        pic.0.set_all();
        pic.1.set_all();
        kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");

        pic.0.clear_mask(1);
    }

    kprintln!("enabling hardware interrupts...");
    irq::enable();
    kprintln!("you're on your own now...");
}

pub mod kernel {
    use spin::{Mutex, MutexGuard};

    use arch::paging::addr::*;
    use arch::paging::table::{ActiveTable, InactiveTable};

    use arch::segmentation::{lgdt, reload_segments};
    use arch::segmentation::gdt::{self, Gdt, Gdtr};

    use arch::interrupt::{exceptions, lidt};
    use arch::interrupt::idt::{self, Idt, Idtr};

    use mem::frame::Allocator as FrameAllocator;
    use mem::page::Allocator as PageAllocator;

    use drivers::vga::Vga;
    use drivers::pic::{Mode as PicMode, Pic};
    use drivers::keyboard;

    pub const KERNEL_BASE: usize = 0xe0000000;

    static mut VGA: Option<Mutex<Vga>> = None;

    static mut GDTR: Option<Gdtr> = None;
    static mut GDT_INNER: [gdt::Entry; 8] = [gdt::Entry::empty(); 8];
    static mut GDT: Gdt = unsafe {
        Gdt {
            inner: &mut GDT_INNER,
        }
    }; // GDT_INNER is known to be valid

    static mut IDTR: Option<Idtr> = None;
    static mut IDT_INNER: [idt::Entry; 256] = [idt::Entry::empty(); 256];
    static mut IDT: Idt = unsafe {
        Idt {
            inner: &mut IDT_INNER,
        }
    }; // IDT_INNER is known to be valid

    static mut PIC: Option<Mutex<(Pic, Pic)>> = None;

    static mut FRAME_ALLOC: Option<Mutex<FrameAllocator>> = None;
    static mut PAGE_ALLOC: Option<Mutex<PageAllocator>> = None;
    static mut PAGE_TABLE: Option<Mutex<ActiveTable<'static>>> = None;

    mod page_tables {
        use core::slice;
        use arch::kernel::KERNEL_BASE;
        use arch::paging::table::Entry;

        extern "C" {
            #[no_mangle]
            static mut KERNEL_MAP_INNER: [Entry; 1024];
        }

        static mut KERNEL_MAP: Option<*mut [Entry; 1024]> =
            unsafe { Some(&mut KERNEL_MAP_INNER as *mut _) };

        pub unsafe fn take_kernel() -> &'static mut [Entry] {
            let ptr = KERNEL_MAP.take().unwrap() as usize - KERNEL_BASE;
            slice::from_raw_parts_mut(ptr as *mut _, 1024)
        }
    }

    pub unsafe fn init_paging() -> ActiveTable<'static> {
        let mut page_map = InactiveTable::new(page_tables::take_kernel());

        // first four megabytes identity
        page_map.default_map(Virtual::new(0), Physical::new(0));
        // first four megabytes higher half
        page_map.default_map(Virtual::new(KERNEL_BASE), Physical::new(0));

        let mut page_map = page_map.load();

        // enable pse
        // enable paging and write protect
        // add KERNEL_BASE to stack pointer
        // add KERNEL_BASE to base pointer
        // walk the call stack and add KERNEL_BASE to every saved ebp and eip
        asm!("
                movl    %cr4, %eax
                orl     $$0x10, %eax
                movl    %eax, %cr4

                movl    %cr0, %eax
                orl     $$0x80010000, %eax
                movl    %eax, %cr0

                leal    init_paging.higher_half, %eax
                jmpl    *%eax
        init_paging.higher_half:
                add     $0, %esp
                add     $0, %ebp
                movl    %ebp, %ebx
        init_paging.stack_loop:
                addl    $0, 4(%ebx)

                movl    (%ebx), %eax
                test    %eax, %eax
                jz      init_paging.stack_loop_done

                addl    $0, (%ebx)
                movl    %eax, %ebx
                jmp     init_paging.stack_loop
        init_paging.stack_loop_done:
             " : : "i"(KERNEL_BASE) : "eax" "ebx" "memory" : "volatile"
        );

        page_map.unmap(Virtual::new(0));
        page_map
    }

    pub unsafe fn set_page_table(page_table: ActiveTable<'static>) {
        PAGE_TABLE = Some(Mutex::new(page_table));
    }

    pub unsafe fn set_allocator_pair(frame: FrameAllocator, page: PageAllocator) {
        FRAME_ALLOC = Some(Mutex::new(frame));
        PAGE_ALLOC = Some(Mutex::new(page));
    }

    pub unsafe fn page_table() -> MutexGuard<'static, ActiveTable<'static>> {
        PAGE_TABLE.as_ref().unwrap().lock()
    }

    pub fn try_page_table() -> Option<MutexGuard<'static, ActiveTable<'static>>> {
        unsafe { PAGE_TABLE.as_ref().and_then(|table| table.try_lock()) }
    }

    pub unsafe fn frame_alloc() -> MutexGuard<'static, FrameAllocator> {
        FRAME_ALLOC.as_ref().unwrap().lock()
    }

    pub fn try_frame_alloc() -> Option<MutexGuard<'static, FrameAllocator>> {
        unsafe { FRAME_ALLOC.as_ref().and_then(|alloc| alloc.try_lock()) }
    }

    pub unsafe fn page_alloc() -> MutexGuard<'static, PageAllocator> {
        PAGE_ALLOC.as_ref().unwrap().lock()
    }

    pub fn try_page_alloc() -> Option<MutexGuard<'static, PageAllocator>> {
        unsafe { PAGE_ALLOC.as_ref().and_then(|alloc| alloc.try_lock()) }
    }

    pub unsafe fn init_vga() {
        VGA = Some(Mutex::new(Vga::new()));
    }

    pub unsafe fn vga() -> MutexGuard<'static, Vga> {
        VGA.as_ref().unwrap().lock()
    }

    pub fn try_vga() -> Option<MutexGuard<'static, Vga>> {
        unsafe { VGA.as_ref().and_then(|vga| vga.try_lock()) }
    }

    pub unsafe fn init_gdt() {
        GDT.new_entry(0, gdt::Entry::empty());
        GDT.new_entry(
            0x8,
            gdt::EntryBuilder::new()
                .base(0)
                .limit(0xfffff)
                .granularity(gdt::Granularity::Page)
                .size(32)
                .present()
                .ring(gdt::RingLevel::Ring0)
                .executable()
                .read_write()
                .build(),
        );
        GDT.new_entry(
            0x10,
            gdt::EntryBuilder::new()
                .base(0)
                .limit(0xfffff)
                .granularity(gdt::Granularity::Page)
                .size(32)
                .present()
                .ring(gdt::RingLevel::Ring0)
                .read_write()
                .build(),
        );

        GDTR = Some(GDT.gdtr());
        lgdt(GDTR.as_ref().unwrap());
        reload_segments(0x8, 0x10);
    }

    pub unsafe fn init_idt() {
        IDT.new_exception_handler(0x0, exceptions::de);
        IDT.new_exception_handler(0x1, exceptions::db);
        IDT.new_exception_handler(0x2, exceptions::ni);
        IDT.new_exception_handler(0x3, exceptions::bp);
        IDT.new_exception_handler(0x4, exceptions::of);
        IDT.new_exception_handler(0x5, exceptions::br);
        IDT.new_exception_handler(0x6, exceptions::ud);
        IDT.new_exception_handler(0x7, exceptions::nm);
        IDT.new_exception_handler(0x8, exceptions::df);
        IDT.new_exception_handler(0xa, exceptions::ts);
        IDT.new_exception_handler(0xb, exceptions::np);
        IDT.new_exception_handler(0xc, exceptions::ss);
        IDT.new_exception_handler(0xd, exceptions::gp);
        IDT.new_exception_handler(0xe, exceptions::pf);
        IDT.new_exception_handler(0x10, exceptions::mf);
        IDT.new_exception_handler(0x11, exceptions::ac);
        IDT.new_exception_handler(0x12, exceptions::mc);
        IDT.new_exception_handler(0x13, exceptions::xm);
        IDT.new_exception_handler(0x14, exceptions::ve);
        IDT.new_exception_handler(0x1e, exceptions::sx);

        IDT.new_interrupt_handler(0x21, keyboard::handler);
        IDT.new_interrupt_handler(0x80, ::syscall::handler);

        IDTR = Some(IDT.idtr());
        lidt(IDTR.as_ref().unwrap());
    }

    pub unsafe fn init_pic(master: Pic, slave: Pic) {
        let master_mask = master.mask();
        let slave_mask = slave.mask();

        let mut master = master.begin_init();
        let mut slave = slave.begin_init();

        master.offset(0x20); // offset master irq's to 0x20:0x27
        slave.offset(0x28); // offset slave irq's to 0x28:0x2f

        master.slave(0b0100); // master has to know where its slave is,
                              // i.e. where it receives irq from the slave
        slave.identity(0b0010); // slave has to know its cascade identity,
                                // i.e where it sends irqs to the master

        master.mode(PicMode::M8086); // 8086/88 mode
        slave.mode(PicMode::M8086); // 8086/88 mode

        let mut master = master.end_init();
        let mut slave = slave.end_init();

        master.restore_mask(master_mask);
        slave.restore_mask(slave_mask);

        PIC = Some(Mutex::new((master, slave)))
    }

    pub unsafe fn pic() -> MutexGuard<'static, (Pic, Pic)> {
        PIC.as_ref().unwrap().lock()
    }

    pub fn try_pic() -> Option<MutexGuard<'static, (Pic, Pic)>> {
        unsafe { PIC.as_ref().and_then(|pic| pic.try_lock()) }
    }
}
