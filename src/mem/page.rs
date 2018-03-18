use core::ops::Range;

use bit_field::{BitArray, BitField};

use arch::paging::addr::Virtual;
use arch::paging::table::ActiveTable;

#[cfg(target_pointer_width = "32")]
const USIZE_BITS: usize = 32;

const PAGES: usize = 1024;
const LEN: usize = PAGES / USIZE_BITS;

pub const PAGE_SIZE: usize = 0x400000;

pub fn pages(inner: Range<usize>) -> Pages {
    let inner = Range {
        start: inner.start >> 22,
        end: (inner.end >> 22) + 1,
    };
    Pages { inner }
}

// page iterator
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pages {
    inner: Range<usize>,
}

impl Iterator for Pages {
    type Item = Virtual;

    fn next(&mut self) -> Option<Virtual> {
        self.inner.next().map(|page| Virtual::new(page << 22))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Page {
    addr: Virtual,
}

impl Page {
    pub fn addr(&self) -> &Virtual {
        &self.addr
    }
}

pub struct Allocator {
    // an array of 1024 bits
    // on x86 it's 32 bytes long, so no worries about size
    bitmap: [usize; LEN],
}

impl Allocator {
    pub fn with_used<'a>(active: &'a ActiveTable<'a>) -> Allocator {
        let mut bitmap = [0_usize; LEN];

        for (idx, page) in pages(0..usize::max_value()).enumerate() {
            if active.is_used(page) {
                bitmap.set_bit(idx, true);
            }
        }

        Allocator { bitmap }
    }

    pub fn allocate(&mut self) -> Option<Page> {
        self.allocate_at(Virtual::new(0))
    }

    pub fn allocate_at(&mut self, virt: Virtual) -> Option<Page> {
        let idx = virt.into_inner() >> 27;

        for idx in idx..LEN {
            if self.bitmap[idx] != !0 {
                let word = self.bitmap[idx];
                for bit in 0..USIZE_BITS {
                    if !word.get_bit(bit) {
                        let page = idx << 5 | bit;
                        self.bitmap.set_bit(page, true);
                        let addr = page << 22;
                        let page = Page {
                            addr: Virtual::new(addr),
                        };
                        return Some(page);
                    }
                }
            }
        }

        None
    }

    pub fn deallocate(&mut self, page: Page) {
        let idx = page.addr.into_inner() >> 22;
        self.bitmap.set_bit(idx, false);
    }

    // returns the count of free pages
    pub fn free(&self) -> usize {
        let mut count = 0;

        for idx in 0..LEN {
            count += self.bitmap[idx].count_zeros() as usize;
        }

        count
    }

    pub fn used(&self) -> usize {
        PAGES - self.free()
    }
}
