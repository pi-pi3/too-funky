use core::num::Wrapping;
use core::ops::Range;

use paging::addr::Physical;

#[cfg(target_pointer_width = "32")]
const USIZE_BITS: usize = 32;

const FRAMES: usize = 1024;
const LEN: usize = FRAMES / USIZE_BITS;

pub const FRAME_SIZE: usize = 0x400000;

pub fn frames(inner: Range<usize>) -> Frames {
    let inner = Range {
        start: inner.start >> 22,
        end: (inner.end >> 22) + 1,
    };
    Frames { inner }
}

// frame iterator
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Frames {
    inner: Range<usize>,
}

impl Iterator for Frames {
    type Item = Physical;

    fn next(&mut self) -> Option<Physical> {
        self.inner.next().map(|frame| Physical::new(frame << 22))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Frame {
    addr: Physical,
}

pub struct Allocator {
    // an array of 1024 bits
    // on x86 it's 32 bytes long, so no worries about size
    bitmap: [usize; LEN],
    range: Range<usize>, // frame blocks, not frames or physical memory
}

impl Allocator {
    pub unsafe fn new() -> Allocator {
        let bitmap = [0_usize; LEN];
        let range = 0 .. LEN;
        Allocator { bitmap, range }
    }

    pub fn with_range(range: Range<usize>) -> Allocator {
        let mut bitmap = [0xffffffff_usize; LEN];
        let mut idx = range.start >> 27;
        let mut bit = Wrapping(1_usize);

        for _ in frames(range.clone()) {
            bitmap[idx] &= !bit.0;
            bit <<= 1;
            if bit.0 == 1 {
                idx += 1;
            }
        }

        let range = (range.start >> 27) .. (range.end >> 27) + 1;
        Allocator { bitmap, range }
    }

    pub fn allocate(&mut self) -> Option<Frame> {
        for idx in self.range.clone() {
            if self.bitmap[idx] != 0 {
                let word = self.bitmap[idx];
                for bit in 0 .. USIZE_BITS {
                    let mask = 1 << bit;
                    if word & mask == 0 {
                        self.bitmap[idx] |= mask;
                        let addr = idx << 27 | bit << 22;
                        let frame = Frame {
                            addr: Physical::new(addr),
                        };
                        return Some(frame);
                    }
                }
            }
        }

        None
    }

    pub fn deallocate(&mut self, frame: Frame) {
        let idx = frame.addr.into_inner() >> 22;
        let bit = (idx >> 22) & 31;
        let idx = (idx >> 27) & 31;
        let mask = 1 << bit;
        self.bitmap[idx] &= !mask;
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
        self.range.end - self.range.start - self.free()
    }
}
