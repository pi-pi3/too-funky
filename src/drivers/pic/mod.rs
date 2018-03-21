use spin::{Mutex, MutexGuard, Once};

pub mod driver;

pub use self::driver::{Mode, PIC1, PIC2, Pic};

static PIC: Once<Mutex<(Pic, Pic)>> = Once::new();

pub fn init() -> &'static Mutex<(Pic, Pic)> {
    PIC.call_once(|| {
        let (master, slave) = unsafe { (Pic::new(PIC1), Pic::new(PIC2)) };

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

        master.mode(Mode::M8086); // 8086/88 mode
        slave.mode(Mode::M8086); // 8086/88 mode

        let mut master = master.end_init();
        let mut slave = slave.end_init();

        master.restore_mask(master_mask);
        slave.restore_mask(slave_mask);

        Mutex::new((master, slave))
    })
}

pub fn handle() -> MutexGuard<'static, (Pic, Pic)> {
    init().lock()
}

pub fn try_handle() -> Option<MutexGuard<'static, (Pic, Pic)>> {
    PIC.try().and_then(|pic| pic.try_lock())
}
