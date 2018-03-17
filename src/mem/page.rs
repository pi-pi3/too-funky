use core::ops::Range;

use paging::addr::Virtual;
use paging::table::ActiveTable;

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
    pub unsafe fn new() -> Allocator {
        let bitmap = [0_usize; LEN];
        Allocator { bitmap }
    }

    pub fn with_used<'a>(active: &'a ActiveTable<'a>) -> Allocator {
        let mut bitmap = [0_usize; LEN];
        let mut idx = 0;
        let mut bit = 1_usize;

        for page in pages(0 .. usize::max_value()) {
            if active.is_used(page) {
                bitmap[idx] |= bit;
            }
            bit = bit.rotate_left(1);
            if bit == 1 {
                idx += 1;
            }
        }

        Allocator { bitmap }
    }

    pub fn allocate(&mut self) -> Option<Page> {
        self.allocate_at(Virtual::new(0))
    }

    pub fn allocate_at(&mut self, virt: Virtual) -> Option<Page> {
        let idx = virt.into_inner() >> 27;
        for idx in idx .. LEN {
            if self.bitmap[idx] != usize::max_value() {
                let word = self.bitmap[idx];
                for bit in 0 .. USIZE_BITS {
                    let mask = 1 << bit;
                    if word & mask == 0 {
                        self.bitmap[idx] |= mask;
                        let addr = idx << 27 | bit << 22;
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
        let bit = (idx >> 22) & 31;
        let idx = (idx >> 27) & 31;
        let mask = 1 << bit;
        self.bitmap[idx] &= !mask;
    }

    // returns the count of free pages
    pub fn free(&self) -> usize {
        let mut count = 0;

        for idx in 0 .. LEN {
            count += self.bitmap[idx].count_zeros() as usize;
        }

        count
    }

    pub fn used(&self) -> usize {
        PAGES - self.free()
    }
}
