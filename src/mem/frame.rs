use core::num::Wrapping;
use core::ops::Range;

use paging::addr::Virtual;
use paging::table::ActiveTable;

#[cfg(target_pointer_width = "32")]
const USIZE_BITS: usize = 32;

const LEN: usize = 1024 / USIZE_BITS;

pub const FRAME_SIZE: usize = 0x400000;

pub fn frames(inner: Range<usize>) -> Frames {
    let inner = Range {
        start: inner.start >> 22,
        end: inner.end >> 22,
    };
    Frames { inner }
}

// page iterator
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Frames {
    inner: Range<usize>,
}

impl Iterator for Frames {
    type Item = Virtual;

    fn next(&mut self) -> Option<Virtual> {
        self.inner.next().map(|page| Virtual::new(page << 22))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Frame {
    addr: Virtual,
}

pub struct Allocator {
    // an array of 1024 bits
    // on x86 it's 32 bytes long, so no worries about size
    bitmap: [usize; LEN],
}

impl Allocator {
    pub fn new() -> Allocator {
        let bitmap = [0_usize; LEN];
        Allocator { bitmap }
    }

    pub fn with_used<'a>(active: &'a ActiveTable<'a>) -> Allocator {
        let mut bitmap = [0_usize; LEN];
        let mut idx = 0;
        let mut bit = Wrapping(1_usize);

        for page in frames(0 .. usize::max_value()) {
            if active.is_used(page) {
                bitmap[idx] |= bit.0;
            }
            bit <<= 1;
            if bit.0 == 1 {
                idx += 1;
            }
        }

        Allocator { bitmap }
    }

    pub fn allocate(&mut self) -> Option<Frame> {
        self.allocate_at(Virtual::new(0))
    }

    pub fn allocate_at(&mut self, virt: Virtual) -> Option<Frame> {
        let idx = virt.into_inner() >> 22;
        for idx in idx .. LEN {
            if self.bitmap[idx] != 0 {
                let word = self.bitmap[idx];
                for bit in 0 .. USIZE_BITS {
                    let bit = 1 << bit;
                    if word & bit == 0 {
                        self.bitmap[idx] |= bit;
                        let addr = idx << 27 | bit << 22;
                        let frame = Frame {
                            addr: Virtual::new(addr),
                        };
                        return Some(frame);
                    }
                }
            }
        }

        None
    }

    // returns the count of free frames
    pub fn free(&self) -> usize {
        let mut count = 0;

        for idx in 0 .. LEN {
            count += self.bitmap[idx].count_zeros() as usize;
        }

        count
    }

    pub fn used(&self) -> usize {
        1024 - self.free()
    }
}
