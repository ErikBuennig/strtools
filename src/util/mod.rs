//! This module contains helper functions and types, some for easy of use, some for upholding some
//! invariants statically.

mod sorted_slice;
pub use sorted_slice::SortedSlice;

mod sorted;
pub use sorted::Sorted;

/// An [Error][e] indicating that a `[T]`/`[T; N]` could not be turned into a
/// [`SortedSlice`]/[`Sorted`] because it was not sorted according to [`T: PartialOrd`][pord].
///
/// [e]: std::error::Error
/// [pord]: PartialOrd
#[derive(thiserror::Error, Debug)]
pub enum SortedError {
    /// Indicates that a slice/array was not sorted.
    #[error("the slice/array was not sorted")]
    NotSorted,
}

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
