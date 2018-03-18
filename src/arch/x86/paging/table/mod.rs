use core::slice;
use core::ops::{Deref, DerefMut};

use x86::shared::control_regs::{cr3, cr3_write};

use arch::paging::addr::*;
use mem::frame::FRAME_SIZE;

pub mod entry;
pub use self::entry::*;

struct Table<'a> {
    inner: &'a mut [Entry],
}

impl<'a> Table<'a> {
    pub unsafe fn new(inner: &'a mut [Entry]) -> Table<'a> {
        Table { inner }
    }

    pub fn map(&mut self, virt: Virtual, entry: Entry) -> Option<Entry> {
        let idx = virt.into_inner() >> 22;
        let old = if self.inner[idx].is_used() {
            Some(self.inner[idx])
        } else {
            None
        };
        self.inner[idx] = entry;
        old
    }

    pub fn default_map(
        &mut self,
        virt: Virtual,
        phys: Physical,
    ) -> Option<Entry> {
        let entry = EntryBuilder::new()
            .addr(phys)
            .present()
            .read_write()
            .page_size(PageSize::Huge)
            .build();

        self.map(virt, entry)
    }

    pub fn unmap(&mut self, virt: Virtual) -> Option<Entry> {
        self.map(virt, Entry::empty())
    }

    pub fn is_used(&self, virt: Virtual) -> bool {
        let idx = virt.into_inner() >> 22;
        self.inner[idx].is_used()
    }

    pub unsafe fn reset_cache(&mut self) {
        cr3_write(cr3());
    }
}

impl<'a> Deref for Table<'a> {
    type Target = [Entry];

    fn deref(&self) -> &[Entry] {
        self.inner
    }
}

impl<'a> DerefMut for Table<'a> {
    fn deref_mut(&mut self) -> &mut [Entry] {
        self.inner
    }
}

pub struct ActiveTable<'a> {
    inner: Table<'a>,
}

impl<'a> ActiveTable<'a> {
    pub fn map(&mut self, virt: Virtual, entry: Entry) -> Option<Entry> {
        self.inner.map(virt, entry)
    }

    pub fn default_map(
        &mut self,
        virt: Virtual,
        phys: Physical,
    ) -> Option<Entry> {
        self.inner.default_map(virt, phys)
    }

    pub fn unmap(&mut self, virt: Virtual) -> Option<Entry> {
        self.inner.unmap(virt)
    }

    pub fn is_used(&self, virt: Virtual) -> bool {
        self.inner.is_used(virt)
    }

    pub fn reset_cache(&mut self) {
        unsafe {
            self.inner.reset_cache();
        }
    }
}

pub struct InactiveTable<'a> {
    inner: Table<'a>,
}

impl<'a> InactiveTable<'a> {
    pub fn new(inner: &'a mut [Entry]) -> InactiveTable<'a> {
        assert!(
            inner.len() == 1024,
            "page directory must have 1024 entries, is {}",
            inner.len()
        );
        InactiveTable {
            inner: unsafe { Table::new(inner) },
        }
    }

    pub fn into_physical(self) -> Physical {
        Physical::new(self.inner.as_ptr() as usize)
    }

    pub fn map(&mut self, virt: Virtual, entry: Entry) -> Option<Entry> {
        self.inner.map(virt, entry)
    }

    pub fn default_map(
        &mut self,
        virt: Virtual,
        phys: Physical,
    ) -> Option<Entry> {
        self.inner.default_map(virt, phys)
    }

    pub fn unmap(&mut self, virt: Virtual) -> Option<Entry> {
        self.inner.unmap(virt)
    }

    pub fn is_used(&self, virt: Virtual) -> bool {
        self.inner.is_used(virt)
    }

    #[must_use]
    pub unsafe fn load(mut self) -> ActiveTable<'a> {
        let phys = Physical::new(self.inner.as_ptr() as usize);
        let offset = phys.into_inner() & (FRAME_SIZE - 1);
        self.default_map(Virtual::new(0xffc00000), phys & !(FRAME_SIZE - 1));

        cr3_write(phys.into_inner());

        let inner = Table::new(slice::from_raw_parts_mut(
            (0xffc00000 + offset) as *mut _,
            1024,
        ));
        ActiveTable { inner }
    }

    // addr is the virtual address to which the current active table will be
    // mapped
    #[must_use]
    pub fn switch<'b>(
        mut self,
        active: ActiveTable<'b>,
        addr: Virtual,
    ) -> (ActiveTable<'a>, InactiveTable<'b>) {
        let old_phys =
            Physical::new(active.inner[0x3ff].into_physical().into_inner());
        let old_offset = old_phys.into_inner() & (FRAME_SIZE - 1);

        // self virtual idx
        let idx = self.inner.as_ptr() as usize >> 22;
        // self physical address
        let new_phys = active.inner[idx].into_physical();
        let new_offset = new_phys.into_inner() & (FRAME_SIZE - 1);
        self.default_map(
            Virtual::new(0xffc00000),
            new_phys & !(FRAME_SIZE - 1),
        );
        self.default_map(addr, old_phys & !(FRAME_SIZE - 1));

        let new_active = unsafe {
            cr3_write(new_phys.into_inner());

            let inner = Table::new(slice::from_raw_parts_mut(
                (0xffc00000 + new_offset) as *mut _,
                1024,
            ));
            ActiveTable { inner }
        };
        let new_inactive = unsafe {
            let inner = Table::new(slice::from_raw_parts_mut(
                (addr.into_inner() + old_offset) as *mut _,
                1024,
            ));
            InactiveTable { inner }
        };
        (new_active, new_inactive)
    }
}
