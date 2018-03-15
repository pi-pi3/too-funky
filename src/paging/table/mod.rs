use core::slice;
use core::ops::{Deref, DerefMut};

use paging::PAGE_SIZE;
use paging::addr::*;

pub mod entry;
pub use self::entry::*;

struct Table<'a> {
    inner: &'a mut [Entry],
}

impl<'a> Table<'a> {
    pub fn new(inner: &'a mut [Entry]) -> Table<'a> {
        assert!(
            inner.len() == 1024,
            "page directory must have 1024 entries, is {}",
            inner.len()
        );
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

    pub unsafe fn reset_cache(&mut self) {
        asm!("mov eax, cr3; mov cr3, eax" : : : "eax" : "intel", "volatile");
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
        InactiveTable {
            inner: Table::new(inner),
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

    #[must_use]
    pub unsafe fn load(mut self) -> ActiveTable<'a> {
        let phys = Physical::new(self.inner.as_ptr() as usize);
        let offset = phys.into_inner() & (PAGE_SIZE - 1);
        self.default_map(Virtual::new(0xffc00000), phys & !(PAGE_SIZE - 1));

        asm!("mov cr3, $0" : : "r"(phys) : : "intel", "volatile");

        ActiveTable {
            inner: Table::new(slice::from_raw_parts_mut(
                (0xffc00000 + offset) as *mut _,
                1024,
            )),
        }
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
        let old_offset = old_phys.into_inner() & (PAGE_SIZE - 1);

        // self virtual idx
        let idx = self.inner.as_ptr() as usize >> 22;
        // self physical address
        let new_phys = active.inner[idx].into_physical();
        let new_offset = new_phys.into_inner() & (PAGE_SIZE - 1);
        self.default_map(Virtual::new(0xffc00000), new_phys & !(PAGE_SIZE - 1));
        self.default_map(addr, old_phys & !(PAGE_SIZE - 1));

        let new_active = unsafe {
            asm!("mov cr3, $0" : : "r"(new_phys) : : "intel", "volatile");

            ActiveTable {
                inner: Table::new(slice::from_raw_parts_mut(
                    (0xffc00000 + new_offset) as *mut _,
                    1024,
                )),
            }
        };
        let new_inactive = unsafe {
            InactiveTable {
                inner: Table::new(slice::from_raw_parts_mut(
                    (addr.into_inner() + old_offset) as *mut _,
                    1024,
                )),
            }
        };
        (new_active, new_inactive)
    }
}
