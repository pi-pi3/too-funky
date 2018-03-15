use core::fmt::{self, LowerHex};
use core::ops::{Add, BitAnd, Shl, Shr};
use core::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Physical(usize);

impl Physical {
    #[inline]
    pub fn new(addr: usize) -> Physical {
        Physical(addr)
    }

    #[inline]
    pub fn into_inner(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Virtual(usize);

impl Virtual {
    #[inline]
    pub fn new(addr: usize) -> Virtual {
        Virtual(addr)
    }

    #[inline]
    pub fn into_inner(self) -> usize {
        self.0
    }
}

impl LowerHex for Physical {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        LowerHex::fmt(&self.0, f)
    }
}

impl LowerHex for Virtual {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        LowerHex::fmt(&self.0, f)
    }
}

impl PartialEq<usize> for Physical {
    fn eq(&self, rhs: &usize) -> bool {
        self.0.eq(rhs)
    }
}

impl PartialEq<usize> for Virtual {
    fn eq(&self, rhs: &usize) -> bool {
        self.0.eq(rhs)
    }
}

impl PartialOrd<usize> for Physical {
    fn partial_cmp(&self, rhs: &usize) -> Option<Ordering> {
        self.0.partial_cmp(rhs)
    }
}

impl PartialOrd<usize> for Virtual {
    fn partial_cmp(&self, rhs: &usize) -> Option<Ordering> {
        self.0.partial_cmp(rhs)
    }
}

macro_rules! impl_op {
    ($type:ident, $op:ty, $fun:ident) => {
        impl $op for $type {
            type Output = $type;

            fn $fun(self, rhs: $type) -> $type {
                $type :: new ( (self.0).$fun ( rhs.0 ) )
            }
        }
    };
    ($type:ident, $rhs:ident, $op:ty, $fun:ident) => {
        impl $op for $type {
            type Output = $type;

            fn $fun(self, rhs: $rhs) -> $type {
                $type :: new ( (self.0).$fun ( rhs ) )
            }
        }
    }
}

impl_op!(Physical, Add, add);
impl_op!(Physical, BitAnd, bitand);

impl_op!(Physical, usize, Add<usize>, add);
impl_op!(Physical, usize, BitAnd<usize>, bitand);
impl_op!(Physical, u8, Shl<u8>, shl);
impl_op!(Physical, u8, Shr<u8>, shr);

impl_op!(Virtual, Add, add);
impl_op!(Virtual, BitAnd, bitand);

impl_op!(Virtual, usize, Add<usize>, add);
impl_op!(Virtual, usize, BitAnd<usize>, bitand);
impl_op!(Virtual, u8, Shl<u8>, shl);
impl_op!(Virtual, u8, Shr<u8>, shr);
