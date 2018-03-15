use core::ops::Range;

pub mod addr;
pub mod table;

use self::addr::Physical;

pub const PAGE_SIZE: usize = 0x400000;

pub fn pages(inner: Range<usize>) -> Pages {
    let inner = Range {
        start: inner.start >> 22,
        end: inner.end >> 22,
    };
    Pages { inner }
}

// page iterator
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pages {
    inner: Range<usize>,
}

impl Iterator for Pages {
    type Item = Physical;

    fn next(&mut self) -> Option<Physical> {
        self.inner
            .next()
            .map(|page| Physical::new(page << 22))
    }
}
