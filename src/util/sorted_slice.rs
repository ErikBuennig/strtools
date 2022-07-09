use super::SortedError;
use std::{borrow::Borrow, fmt::Debug, ops::Deref};

/// Represents a `[T]` that is guaranteed to be sorted by [`T: PartialOrd`][pord]. This is a
/// [DST][dst], therefore constructors only return references.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use strtools::util::SortedSlice;
/// // only checks if the slice is sorted
/// let sorted: &SortedSlice<_> = ['a', 'b', 'c'][..].try_into()?;
///
/// // sorts the slice and is therefore not fallible, requires T: Ord, the returned slice is
/// // immutable
/// let sorted: &SortedSlice<_> = SortedSlice::new_sorted(&mut ['a', 'c', 'b']);
/// # Ok(())
/// # }
/// ```
///
/// [dst]: https://doc.rust-lang.org/book/ch19-04-advanced-types.html#dynamically-sized-types-and-the-sized-trait
/// [pord]: PartialOrd
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SortedSlice<T: PartialOrd>([T]);

impl<T: PartialOrd> SortedSlice<T> {
    /// Creates a new [`SortedSlice`] from the given `slice` if it was sorted.
    ///
    /// # Errors
    /// Returns an error if:
    /// - `slice` was not sorted
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::SortedSlice;
    /// let sorted: &SortedSlice<_> = SortedSlice::new(&['a', 'b', 'c'])?;
    /// # Ok(())
    /// # }
    /// ```
    /// This will return an error:
    /// ```should_panic
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::SortedSlice;
    /// // this is not sorted
    /// let sorted: &SortedSlice<_> = SortedSlice::new(&['a', 'c', 'b'])?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn new(slice: &[T]) -> Result<&Self, SortedError> {
        if slice.is_sorted() {
            // SAFETY: the slice is sorted according to R
            Ok(unsafe { Self::new_unchecked(slice) })
        } else {
            Err(SortedError::NotSorted)
        }
    }

    /// Sorts the given slice and creates a new mutable [`SortedSlice`] from it.
    ///
    /// # Examples
    /// ```
    /// # use strtools::util::SortedSlice;
    /// let mut slice = ['a', 'c', 'b'];
    /// let sorted: &SortedSlice<_> = SortedSlice::new_sorted(&mut slice);
    /// assert_eq!(sorted.as_slice(), &['a', 'b', 'c']);
    /// ```
    #[inline]
    pub fn new_sorted(slice: &mut [T]) -> &Self
    where
        T: Ord,
    {
        slice.sort();

        // SAFETY: the slice has been sorted
        unsafe { Self::new_unchecked(slice) }
    }

    /// Creates a new [`SortedSlice`] from the given `slice`, assuming it was sorted.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - `slice` is sorted
    ///
    /// # Examples
    /// ```
    /// # use strtools::util::SortedSlice;
    /// let sorted: &SortedSlice<_> = unsafe { SortedSlice::new_unchecked(&['a', 'b', 'c']) };
    /// ```
    /// Violation of invariants:
    /// ```
    /// # use strtools::util::SortedSlice;
    /// // this is not sorted, Sorted invariants are violated
    /// let sorted: &SortedSlice<_> = unsafe { SortedSlice::new_unchecked(&['a', 'c', 'b']) };
    /// ```
    #[inline]
    pub const unsafe fn new_unchecked(slice: &[T]) -> &Self {
        // SAFETY:
        // - the caller must ensure that the slice is sorted
        // - #[repr(transparent)] ensures layout compatibility of &[T] and &Self
        // - the lifetime of &Self is the same as `slice`
        unsafe { std::mem::transmute(slice) }
    }

    /// Creates a new mutable [`SortedSlice`] from the given `slice` if it was sorted.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - if the slice is mutated, it remains sorted according to `T: PartialOrd`
    ///
    /// # Errors
    /// Returns an error if:
    /// - `slices` was not sorted
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::SortedSlice;
    /// let sorted: &mut SortedSlice<_> = unsafe { SortedSlice::new_mut(&mut ['a', 'b', 'c'])? };
    /// # Ok(())
    /// # }
    /// ```
    /// This will return an error:
    /// ```should_panic
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::SortedSlice;
    /// // this is not sorted
    /// let sorted: &mut SortedSlice<_> = unsafe { SortedSlice::new_mut(&mut ['a', 'c', 'b'])? };
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub unsafe fn new_mut(slice: &mut [T]) -> Result<&mut Self, SortedError> {
        if slice.is_sorted() {
            // SAFETY: the slice is sorted according to R
            Ok(unsafe { Self::new_unchecked_mut(slice) })
        } else {
            Err(SortedError::NotSorted)
        }
    }

    /// Sorts the given slice and creates a new mutable [`SortedSlice`] from it.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - `slice` remains sorted according to `T: PartialOrd` if mutated
    ///
    /// # Examples
    /// ```
    /// # use strtools::util::SortedSlice;
    /// let mut slice = ['a', 'c', 'b'];
    /// let sorted: &SortedSlice<_> = unsafe { SortedSlice::new_sorted_mut(&mut slice) };
    /// assert_eq!(sorted.as_slice(), &['a', 'b', 'c']);
    /// ```
    #[inline]
    pub unsafe fn new_sorted_mut(slice: &mut [T]) -> &mut Self
    where
        T: Ord,
    {
        slice.sort();

        // SAFETY: the slice has been sorted
        unsafe { Self::new_unchecked_mut(slice) }
    }
    /// Creates a new [`SortedSlice`] from the given `slice`, assuming it was sorted.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - `slice` is sorted
    ///
    /// # Examples
    /// ```
    /// # use strtools::util::SortedSlice;
    /// let sorted: &mut SortedSlice<_> = unsafe {
    ///     SortedSlice::new_unchecked_mut(&mut ['a', 'b', 'c'])
    /// };
    /// ```
    /// Violation of invariants:
    /// ```
    /// # use strtools::util::SortedSlice;
    /// // this is not sorted, Sorted invariants are violated
    /// let sorted: &mut SortedSlice<_> = unsafe {
    ///     SortedSlice::new_unchecked_mut(&mut ['a', 'c', 'b'])
    /// };
    /// ```
    #[inline]
    pub const unsafe fn new_unchecked_mut(slice: &mut [T]) -> &mut Self {
        // SAFETY:
        // - the caller must ensure that the slice is sorted
        // - #[repr(transparent)] ensures layout compatibility of &[T] and &Self
        // - the lifetime of &Self is the same as `slice`
        unsafe { std::mem::transmute(slice) }
    }

    /// Borrows this as a slice `&[T]`.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::SortedSlice;
    /// let sorted: &SortedSlice<_> = ['a', 'b', 'c'][..].try_into()?;
    /// let slice: &[char] = sorted.as_slice();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub const fn as_slice(&self) -> &[T] {
        &self.0
    }

    /// Borrows this as a mutable slice `&mut [T]`. This function is not unsafe as getting a
    /// `&mut SortedSlice` is already unsafe.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::SortedSlice;
    /// let mut slice = ['a', 'b', 'c'];
    /// let sorted: &mut SortedSlice<_> = unsafe { SortedSlice::new_mut(&mut slice)? };
    /// let slice: &[char] = sorted.as_slice_mut();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub const fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

impl<T: PartialOrd + Debug> Debug for SortedSlice<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: PartialOrd> Deref for SortedSlice<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T: PartialOrd> AsRef<[T]> for SortedSlice<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: PartialOrd> Borrow<[T]> for SortedSlice<T> {
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'s, T: PartialOrd> TryFrom<&'s [T]> for &'s SortedSlice<T> {
    type Error = SortedError;

    fn try_from(value: &'s [T]) -> Result<Self, Self::Error> {
        SortedSlice::new(value)
    }
}

impl<'s, T: PartialOrd> TryFrom<&'s mut [T]> for &'s SortedSlice<T> {
    type Error = SortedError;

    fn try_from(value: &'s mut [T]) -> Result<Self, Self::Error> {
        SortedSlice::new(value)
    }
}
