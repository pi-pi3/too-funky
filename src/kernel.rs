#![feature(lang_items)]
#![feature(const_fn)]
#![feature(compiler_builtins_lib)]
#![feature(core_panic)]
#![feature(asm)]
#![feature(naked_functions)]
#![feature(core_intrinsics)]
#![no_std]

extern crate rlibc;
extern crate x86;
extern crate spin;
#[macro_use]
extern crate bitflags;
extern crate compiler_builtins;

use core::fmt;

#[macro_use]
pub mod macros;
#[macro_use]
pub mod interrupt;
pub mod segmentation;
pub mod drivers;
pub mod syscall;

use drivers::pic::{self, Pic};
use drivers::keyboard::{self, Scanset, Keycode};

pub mod kernel {
    use spin::{Mutex, MutexGuard};

    use drivers::vga::Vga;
    use drivers::pic::{Pic, Mode as PicMode};
    use drivers::keyboard;
    use segmentation::{lgdt, reload_segments};
    use segmentation::gdt::{self, Gdt, Gdtr};
    use interrupt::{lidt, exceptions};
    use interrupt::idt::{self, Idt, Idtr};

    static mut VGA: Option<Mutex<Vga>> = None;

    static mut GDTR: Option<Gdtr> = None; 
    static mut GDT_INNER: [gdt::Entry; 8] = [gdt::Entry::empty(); 8];
    static mut GDT: Gdt = unsafe { Gdt { inner: &mut GDT_INNER } }; // GDT_INNER is known to be valid

    static mut IDTR: Option<Idtr> = None; 
    static mut IDT_INNER: [idt::Entry; 256] = [idt::Entry::empty(); 256];
    static mut IDT: Idt = unsafe { Idt { inner: &mut IDT_INNER } }; // IDT_INNER is known to be valid

    static mut PIC: Option<Mutex<(Pic, Pic)>> = None;

    pub unsafe fn init_vga() {
        VGA = Some(Mutex::new(Vga::new()));
    }

    pub unsafe fn vga() -> MutexGuard<'static, Vga> {
        VGA.as_ref()
            .unwrap()
            .lock()
    }

    pub fn try_vga() -> Option<MutexGuard<'static, Vga>> {
        unsafe {
            VGA.as_ref()
                .and_then(|vga| vga.try_lock())
        }
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
                .build()
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
                .build()
        );

        GDTR = Some(GDT.gdtr());
        lgdt(GDTR.as_ref().unwrap());
        reload_segments(0x8, 0x10); // this causes a hardware exception for now for some reason
    }

    pub unsafe fn init_idt() {
        IDT.new_exception_handler(0x0,  exceptions::de);
        IDT.new_exception_handler(0x1,  exceptions::db);
        IDT.new_exception_handler(0x2,  exceptions::ni);
        IDT.new_exception_handler(0x3,  exceptions::bp);
        IDT.new_exception_handler(0x4,  exceptions::of);
        IDT.new_exception_handler(0x5,  exceptions::br);
        IDT.new_exception_handler(0x6,  exceptions::ud);
        IDT.new_exception_handler(0x7,  exceptions::nm);
        IDT.new_exception_handler(0x8,  exceptions::df);
        IDT.new_exception_handler(0xa,  exceptions::ts);
        IDT.new_exception_handler(0xb,  exceptions::np);
        IDT.new_exception_handler(0xc,  exceptions::ss);
        IDT.new_exception_handler(0xd,  exceptions::gp);
        IDT.new_exception_handler(0xe,  exceptions::pf);
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
        PIC.as_ref()
            .unwrap()
            .lock()
    }

    pub fn try_pic() -> Option<MutexGuard<'static, (Pic, Pic)>> {
        unsafe {
            PIC.as_ref()
                .and_then(|pic| pic.try_lock())
        }
    }
}

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    use x86::shared::irq;

    unsafe {
        kernel::init_vga();
        kprint!("vga... ");
        kprintln!(
            "{green}[OK]{reset}",
            green = "\x1b[32m",
            reset = "\x1b[0m"
        );

        kprint!("gdt... ");
        kernel::init_gdt();
        kprintln!(
            "{green}[OK]{reset}",
            green = "\x1b[32m",
            reset = "\x1b[0m"
        );

        irq::disable();

        kprint!("idt... ");
        kernel::init_idt();
        kprintln!(
            "{green}[OK]{reset}",
            green = "\x1b[32m",
            reset = "\x1b[0m"
        );

        keyboard::init_keys(0, 250, Scanset::Set1);

        {
            kprint!("pic... ");
            kernel::init_pic(Pic::new(pic::PIC1), Pic::new(pic::PIC2));

            let mut pic = kernel::pic();
            pic.0.set_all();
            pic.1.set_all();
            kprintln!(
                "{green}[OK]{reset}",
                green = "\x1b[32m",
                reset = "\x1b[0m"
            );

            pic.0.clear_mask(1);
        }

        irq::enable();
    }

    kprint!("> ");
    loop {
        match keyboard::poll() {
            Keycode::Enter => kprint!("\n> "),
            k => kprint!("{}", k),
        }
    }
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(msg: fmt::Arguments,
                        file: &'static str,
                        line: u32,
                        col: u32) -> ! {
    use core::fmt::Write;
    kernel::try_vga().map(|mut vga| {
        let _ = vga.write_str("\x1b[0;31mkernel panicked at '");
        let _ = vga.write_fmt(msg);
        let _ = write!(vga, "', {}:{}:{}\x1b[0m", file, line, col);
    });

    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn eh_personality() {
}
