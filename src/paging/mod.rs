use core::ops::Range;

pub mod addr;
pub mod table;

use self::addr::Physical;

pub const FRAME_SIZE: usize = 0x400000;

pub fn frames(inner: Range<usize>) -> Frames {
    let inner = Range {
        start: inner.start >> 22,
        end: inner.end >> 22,
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
