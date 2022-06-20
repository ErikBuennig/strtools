//! This module contains helper functions and types, some for easy of use, some for upholding some
//! invariants statically.

mod sorted;
pub use sorted::{Sorted, SortedSliceError};

pub(crate) mod sealed {
    pub trait Sealed {}

    macro_rules! impl_trivial {
        ($($t:ty),+) => {
            $(impl Sealed for $t {})+
        };
    }

    impl_trivial!(str);
    impl_trivial!(u8, u16, u32, u64, u128, usize);
    impl_trivial!(i8, i16, i32, i64, i128, isize);
}
