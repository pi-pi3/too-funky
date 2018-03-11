#![feature(lang_items)]
#![feature(const_fn)]
#![feature(compiler_builtins_lib)]
#![feature(core_panic)]
#![feature(asm)]
#![feature(naked_functions)]
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
pub mod drivers;

use interrupt::pic::{self, Pic};

pub mod kernel {
    use spin::{Mutex, MutexGuard};

    use drivers::vga::Vga;
    use interrupt::lidt;
    use interrupt::idt::{self, Idt, Idtr};
    use interrupt::pic::{Pic, Mode as PicMode};

    static mut VGA: Option<Mutex<Vga>> = None;
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

    pub unsafe fn init_idt() {
        IDTR = Some(IDT.idtr());
        // isr should be set here
        /* ... */
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

        irq::disable();

        kernel::init_idt();
        kernel::init_pic(Pic::new(pic::PIC1), Pic::new(pic::PIC2));

        let mut pic = kernel::pic();
        pic.0.set_all();
        pic.1.set_all();

        irq::enable();
    }

    kprintln!("OK");

    loop {}
}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt(msg: fmt::Arguments,
                        file: &'static str,
                        line: u32,
                        col: u32) -> ! {
    use core::fmt::Write;
    kernel::try_vga().map(|mut vga| {
        let _ = vga.write_str("kernel panicked at '");
        let _ = vga.write_fmt(msg);
        let _ = write!(vga, "', {}:{}:{}", file, line, col);
    });

    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn eh_personality() {
}
