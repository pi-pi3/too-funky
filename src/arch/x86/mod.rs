use core::mem;

use multiboot2;
use x86::shared::irq;

use kmain;

#[macro_use]
pub mod interrupt;
pub mod paging;
pub mod segmentation;

use drivers::keyboard::{self, Scanset};
use mem::frame::{FRAME_SIZE, Allocator as FrameAllocator};
use mem::page::Allocator as PageAllocator;
use arch::paging::table::{self, ActiveTable};

#[no_mangle]
pub unsafe extern "C" fn _rust_start(
    mb2_addr: usize,
    kernel_start: usize,
    kernel_end: usize,
) -> ! {
    // align up
    let page_addr = (kernel_end + 0xfff) & 0xfffff000;
    let page_table = kernel::init_paging(page_addr - kernel::KERNEL_BASE);

    kinit(
        page_table,
        mb2_addr,
        kernel_start,
        page_addr + 1024 * mem::size_of::<table::Entry>(),
    );
    kmain();

    loop {}
}

pub fn kinit(
    page_table: ActiveTable<'static>,
    mb2_addr: usize,
    _kernel_start: usize,
    kernel_end: usize,
) {
    use spin::Once;
    static KINIT: Once<()> = Once::new();

    KINIT.call_once(|| {
        let mb2 = unsafe { multiboot2::load(mb2_addr) };

        let mut mem_min = kernel_end - kernel::KERNEL_BASE;
        let mut mem_max = 0;
        let mem_size;

        // first get total available memory
        let memory_map = mb2.memory_map_tag()
            .unwrap_or_else(|| panic!("no memory map in mb2 header"));

        for area in memory_map.memory_areas() {
            let mut addr = area.start_address() as usize;
            if addr > kernel::KERNEL_BASE {
                addr -= kernel::KERNEL_BASE;
            }
            mem_min = mem_min.max(addr);
            mem_max = mem_max.max(area.end_address() as usize);
        }

        // then subtract elf sections
        let elf_sections = mb2.elf_sections_tag()
            .unwrap_or_else(|| panic!("no elf sections in mb2 header"));

        for sect in elf_sections.sections() {
            let mut addr = sect.start_address() as usize;
            if addr > kernel::KERNEL_BASE {
                addr -= kernel::KERNEL_BASE;
            }
            mem_min = mem_min.max(addr);
        }

        // round to page boundaries
        mem_min = (mem_min + FRAME_SIZE) & 0xffc00000;
        mem_max = (mem_max & 0xffc00000) - 1;
        mem_size = mem_max - mem_min + 1;

        let frame_alloc = FrameAllocator::with_range(mem_min..mem_max);
        let page_alloc = PageAllocator::with_used(&page_table);
        kernel::set_allocator_pair(frame_alloc, page_alloc);
        kernel::set_page_table(page_table);

        let heap_start = kernel::KERNEL_BASE + mem_min;
        let heap_end = heap_start + 2 * FRAME_SIZE;

        unsafe { kernel::init_heap(heap_start, heap_end); }

        kernel::init_vga();

        kprint!("paging... ");
        kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");

        kprint!("memory areas... ");
        kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");
        kprintln!(
            "available memory: {:x}..{:x} == {}MB",
            mem_min,
            mem_max + 1,
            mem_size / (1024 * 1024)
        );

        kprint!("kernel heap... ");
        kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");

        kprintln!(
            "heap size: {:x}..{:x} == {}kB",
            heap_start,
            heap_end,
            (heap_end - heap_start) / 1024
        );

        kprint!("video graphics array driver... ");
        kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");

        kprint!("global descriptor table... ");
        unsafe { kernel::init_gdt(); }
        kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");

        kprint!("interrupt descriptor table... ");
        unsafe { kernel::init_idt(); }
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

            let mut pic = kernel::pic();
            pic.0.set_all();
            pic.1.set_all();

            pic.0.clear_mask(1);

            kprintln!("{green}[OK]{reset}", green = "\x1b[32m", reset = "\x1b[0m");
        }


        kprintln!("enabling hardware interrupts...");
        unsafe { irq::enable(); }
        kprintln!("you're on your own now...");
    });
}

pub mod kernel {
    use core::mem;
    use core::slice;
    use core::ptr::Unique;

    use alloc::allocator::{Alloc, Layout};

    use spin::{Once, Mutex, MutexGuard};

    use ::ALLOCATOR;

    use arch::paging::addr::*;
    use arch::paging::table::{self, ActiveTable, InactiveTable};

    use arch::segmentation::{lgdt, reload_segments};
    use arch::segmentation::gdt::{self, Gdt, Gdtr};

    use arch::interrupt::{exceptions, lidt};
    use arch::interrupt::idt::{self, Idt, Idtr};

    use mem::frame::Allocator as FrameAllocator;
    use mem::page::{PAGE_SIZE, Allocator as PageAllocator};

    use drivers::vga::Vga;
    use drivers::pic::{Mode as PicMode, Port as PicPort, Pic};
    use drivers::keyboard;

    const VGA_BASE: *mut u16 = 0xb8000 as *mut _;
    pub const KERNEL_BASE: usize = 0xe0000000;

    static HEAP: Once<()> = Once::new();
    static VGA: Once<Mutex<Vga>> = Once::new();

    static GDTR: Once<Gdtr> = Once::new();
    static GDT: Once<Gdt> = Once::new();

    static IDTR: Once<Idtr> = Once::new();
    static IDT: Once<Idt> = Once::new();

    static PIC: Once<Mutex<(Pic, Pic)>> = Once::new();

    static FRAME_ALLOC: Once<Mutex<FrameAllocator>> = Once::new();
    static PAGE_ALLOC: Once<Mutex<PageAllocator>> = Once::new();
    static PAGE_TABLE: Once<Mutex<ActiveTable<'static>>> = Once::new();

    pub unsafe fn init_paging(addr: usize) -> ActiveTable<'static> {
        let page_map = slice::from_raw_parts_mut(addr as *mut _, 1024);

        for entry in page_map.iter_mut() {
            *entry = table::Entry::empty()
        }

        let size = addr + 1024 * mem::size_of::<table::Entry>();
        let mut page_map = InactiveTable::new(page_map);

        // first four megabytes identity
        page_map.default_map(Virtual::new(0), Physical::new(0));
        // first four megabytes higher half
        page_map.default_map(Virtual::new(KERNEL_BASE), Physical::new(0));
        if (addr + size) & 0xffc00000 > 0 {
            let page = (addr + size) & 0xffc00000;
            page_map.default_map(Virtual::new(KERNEL_BASE + page), Physical::new(page));
        }

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

    pub fn set_page_table(page_table: ActiveTable<'static>) {
        PAGE_TABLE.call_once(move || Mutex::new(page_table));
    }

    pub fn set_allocator_pair(frame: FrameAllocator, page: PageAllocator) {
        FRAME_ALLOC.call_once(move || Mutex::new(frame));
        PAGE_ALLOC.call_once(move || Mutex::new(page));
    }

    pub unsafe fn page_table() -> MutexGuard<'static, ActiveTable<'static>> {
        PAGE_TABLE.try().unwrap().lock()
    }

    pub fn try_page_table() -> Option<MutexGuard<'static, ActiveTable<'static>>> {
        PAGE_TABLE.try().and_then(|table| table.try_lock())
    }

    pub unsafe fn frame_alloc() -> MutexGuard<'static, FrameAllocator> {
        FRAME_ALLOC.try().unwrap().lock()
    }

    pub fn try_frame_alloc() -> Option<MutexGuard<'static, FrameAllocator>> {
        FRAME_ALLOC.try().and_then(|alloc| alloc.try_lock())
    }

    pub unsafe fn page_alloc() -> MutexGuard<'static, PageAllocator> {
        PAGE_ALLOC.try().unwrap().lock()
    }

    pub fn try_page_alloc() -> Option<MutexGuard<'static, PageAllocator>> {
        PAGE_ALLOC.try().and_then(|alloc| alloc.try_lock())
    }

    pub unsafe fn init_heap(heap_start: usize, heap_end: usize) {
        HEAP.call_once(|| {
            let heap_size = heap_end - heap_start;
            assert!(
                heap_size >= PAGE_SIZE,
                "the heap must be at least {}kB big, is {}kB",
                PAGE_SIZE / 1024,
                heap_size / 1024,
            );

            let mut page_table = page_table();
            let mut page_alloc = page_alloc();
            let mut frame_alloc = frame_alloc();

            let pages = heap_size >> 22;
            
            for i in 0..pages {
                // these will never be freed anyway
                let page = page_alloc.allocate_at(Virtual::new(heap_start));
                let frame = frame_alloc.allocate();
                assert!(
                    page.is_some(),
                    "couldn't allocate {}-th heap page at {}",
                    i,
                    heap_start,
                );
                assert!(
                    frame.is_some(),
                    "couldn't allocate {}-th heap frame",
                    i,
                );
                let page = page.unwrap();
                let frame = frame.unwrap();
                page_table.default_map(*page.addr(), *frame.addr());
            }

            ALLOCATOR.lock().init(heap_start, heap_size);
        });
    }

    pub fn init_vga<'a>() -> &'a Mutex<Vga> {
        let ptr = unsafe { Unique::new_unchecked(VGA_BASE.add(KERNEL_BASE / 2)) };
        VGA.call_once(|| Mutex::new(Vga::new(ptr)))
    }

    pub fn vga() -> MutexGuard<'static, Vga> {
        init_vga().lock()
    }

    pub fn try_vga() -> Option<MutexGuard<'static, Vga>> {
        VGA.try().and_then(|vga| vga.try_lock())
    }

    pub unsafe fn init_gdt() {
        let gdt = GDT.call_once(|| {
            let len = 8;
            let ptr = (&ALLOCATOR).alloc(
                Layout::from_size_align_unchecked(
                    len * mem::size_of::<gdt::Entry>(),
                    mem::size_of::<gdt::Entry>(),
                )
            ).unwrap();
            let table = slice::from_raw_parts_mut(ptr as *mut _, len);

            for entry in table.iter_mut() {
                *entry = gdt::Entry::empty()
            }

            let mut gdt = Gdt::with_table(table);
            gdt.new_entry(
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
            gdt.new_entry(
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
            gdt
        });

        let gdtr = GDTR.call_once(|| gdt.gdtr());

        lgdt(gdtr);
        reload_segments(0x8, 0x10);
    }

    pub unsafe fn init_idt() {
        let idt = IDT.call_once(|| {
            let len = 256;
            let ptr = (&ALLOCATOR).alloc(
                Layout::from_size_align_unchecked(
                    len * mem::size_of::<idt::Entry>(),
                    mem::size_of::<idt::Entry>(),
                )
            ).unwrap();
            let table = slice::from_raw_parts_mut(ptr as *mut _, len);

            for entry in table.iter_mut() {
                *entry = idt::Entry::empty()
            }

            let mut idt = Idt::with_table(table);

            idt.new_exception_handler(0x0, exceptions::de);
            idt.new_exception_handler(0x1, exceptions::db);
            idt.new_exception_handler(0x2, exceptions::ni);
            idt.new_exception_handler(0x3, exceptions::bp);
            idt.new_exception_handler(0x4, exceptions::of);
            idt.new_exception_handler(0x5, exceptions::br);
            idt.new_exception_handler(0x6, exceptions::ud);
            idt.new_exception_handler(0x7, exceptions::nm);
            idt.new_exception_handler(0x8, exceptions::df);
            idt.new_exception_handler(0xa, exceptions::ts);
            idt.new_exception_handler(0xb, exceptions::np);
            idt.new_exception_handler(0xc, exceptions::ss);
            idt.new_exception_handler(0xd, exceptions::gp);
            idt.new_exception_handler(0xe, exceptions::pf);
            idt.new_exception_handler(0x10, exceptions::mf);
            idt.new_exception_handler(0x11, exceptions::ac);
            idt.new_exception_handler(0x12, exceptions::mc);
            idt.new_exception_handler(0x13, exceptions::xm);
            idt.new_exception_handler(0x14, exceptions::ve);
            idt.new_exception_handler(0x1e, exceptions::sx);

            idt.new_interrupt_handler(0x21, keyboard::handler);
            idt.new_interrupt_handler(0x80, ::syscall::handler);

            idt
        });

        let idtr = IDTR.call_once(|| idt.idtr());
        lidt(idtr);
    }

    pub fn init_pic(master: Pic, slave: Pic) -> &'static Mutex<(Pic, Pic)> {
        PIC.call_once(move || {
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

            Mutex::new((master, slave))
        })
    }

    pub fn pic() -> MutexGuard<'static, (Pic, Pic)> {
        unsafe { init_pic(Pic::new(PicPort::Pic1), Pic::new(PicPort::Pic2)).lock() }
    }

    pub fn try_pic() -> Option<MutexGuard<'static, (Pic, Pic)>> {
        PIC.try().and_then(|pic| pic.try_lock())
    }
}
