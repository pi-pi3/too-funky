use core::ops::Range;

use bit_field::{BitField, BitArray};

use arch::paging::addr::Physical;

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

impl Frame {
    pub fn addr(&self) -> &Physical {
        &self.addr
    }
}

pub struct Allocator {
    // an array of 1024 bits
    // on x86 it's 32 bytes long, so no worries about size
    bitmap: [usize; LEN],
    range: Range<usize>, // frame blocks, not frames or physical memory
}

impl Allocator {
    pub fn with_range(range: Range<usize>) -> Allocator {
        let mut bitmap = [0xffffffff_usize; LEN];

        for (idx, _) in frames(range.clone()).enumerate() {
            bitmap.set_bit(idx, false);
        }

        let range = (range.start >> 27)..(range.end >> 27) + 1;
        Allocator { bitmap, range }
    }

    pub fn allocate(&mut self) -> Option<Frame> {
        for idx in self.range.clone() {
            if self.bitmap[idx] != !0 {
                let word = self.bitmap[idx];
                for bit in 0..USIZE_BITS {
                    if !word.get_bit(bit) {
                        let frame = idx << 5 | bit;
                        self.bitmap.set_bit(frame, true);
                        let addr = frame << 22;
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
        self.bitmap.set_bit(idx, false);
    }

    // returns the count of free frames
    pub fn free(&self) -> usize {
        let mut count = 0;

        for idx in 0..LEN {
            count += self.bitmap[idx].count_zeros() as usize;
        }

        count
    }

    pub fn used(&self) -> usize {
        self.range.end - self.range.start - self.free()
    }
}
