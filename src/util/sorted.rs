use std::{borrow::Borrow, ops::Deref};

/// An [Error][e] indicating that a `[T]` could not be turned into a [`Sorted`] because it was
/// not sorted according to the `REVERSE` const parameter.
///
/// [e]: std::error::Error
#[derive(thiserror::Error, Debug)]
pub enum SortedSliceError {
    /// Indicates that a slice was not sorted.
    #[error("the slice was not sorted")]
    SliceNotSorted,
}

/// Represents a `[T]` that is guaranteed to be sorted by [`T: PartialOrd`][pord]. This is a
/// [DST][dst], therefore constructors only return references.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use strtools::util::Sorted;
/// // only checks if the slice is sorted
/// let sorted: &Sorted<_> = ['a', 'b', 'c'][..].try_into()?;
///
/// // sorts the slice and is therefore not fallible, requires T: Ord, the returned slice is
/// // immutable
/// let sorted: &Sorted<_> = Sorted::new_sorted(&mut ['a', 'c', 'b']);
/// # Ok(())
/// # }
/// ```
///
/// [dst]: https://doc.rust-lang.org/book/ch19-04-advanced-types.html#dynamically-sized-types-and-the-sized-trait
/// [pord]: PartialOrd
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sorted<T: PartialOrd>([T]);

impl<T: PartialOrd> Sorted<T> {
    /// Creates a new [`Sorted`] from the given `slice` if it was sorted.
    ///
    /// # Errors
    /// Returns an error if:
    /// - `slices` was not sorted
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::Sorted;
    /// let sorted: &Sorted<_> = Sorted::new(&['a', 'b', 'c'])?;
    /// # Ok(())
    /// # }
    /// ```
    /// This will return an error:
    /// ```should_panic
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::Sorted;
    /// // this is not sorted
    /// let sorted: &Sorted<_> = Sorted::new(&['a', 'c', 'b'])?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn new(slice: &[T]) -> Result<&Self, SortedSliceError> {
        if slice.is_sorted() {
            // SAFETY: the slice is sorted according to R
            Ok(unsafe { Self::new_unchecked(slice) })
        } else {
            Err(SortedSliceError::SliceNotSorted)
        }
    }

    /// Sorts the given slice and creates a new mutable [`Sorted`] from it.
    ///
    /// # Examples
    /// ```
    /// # use strtools::util::Sorted;
    /// let mut slice = ['a', 'c', 'b'];
    /// let sorted: &Sorted<_> = Sorted::new_sorted(&mut slice);
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

    /// Creates a new [`Sorted`] from the given `slice`, assuming it was sorted.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - `slice` is sorted
    ///
    /// # Examples
    /// ```
    /// # use strtools::util::Sorted;
    /// let sorted: &Sorted<_> = unsafe { Sorted::new_unchecked(&['a', 'b', 'c']) };
    /// ```
    /// Violation of invariants:
    /// ```
    /// # use strtools::util::Sorted;
    /// // this is not sorted, Sorted invariants are violated
    /// let sorted: &Sorted<_> = unsafe { Sorted::new_unchecked(&['a', 'c', 'b']) };
    /// ```
    #[inline]
    pub const unsafe fn new_unchecked(slice: &[T]) -> &Self {
        // SAFETY:
        // - the caller must ensure that the slice is sorted
        // - #[repr(transparent)] ensures layout compatibility of &[T] and &Self
        // - the lifetime of &Self is the same as `slice`
        unsafe { std::mem::transmute(slice) }
    }

    /// Creates a new mutable [`Sorted`] from the given `slice` if it was sorted.
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
    /// # use strtools::util::Sorted;
    /// let sorted: &mut Sorted<_> = unsafe { Sorted::new_mut(&mut ['a', 'b', 'c'])? };
    /// # Ok(())
    /// # }
    /// ```
    /// This will return an error:
    /// ```should_panic
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::Sorted;
    /// // this is not sorted
    /// let sorted: &mut Sorted<_> = unsafe { Sorted::new_mut(&mut ['a', 'c', 'b'])? };
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub unsafe fn new_mut(slice: &mut [T]) -> Result<&mut Self, SortedSliceError> {
        if slice.is_sorted() {
            // SAFETY: the slice is sorted according to R
            Ok(unsafe { Self::new_mut_unchecked(slice) })
        } else {
            Err(SortedSliceError::SliceNotSorted)
        }
    }

    /// Sorts the given slice and creates a new mutable [`Sorted`] from it.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - `slice` remains sorted according to `T: PartialOrd` if mutated
    ///
    /// # Examples
    /// ```
    /// # use strtools::util::Sorted;
    /// let mut slice = ['a', 'c', 'b'];
    /// let sorted: &Sorted<_> = unsafe { Sorted::new_sorted_mut(&mut slice) };
    /// assert_eq!(sorted.as_slice(), &['a', 'b', 'c']);
    /// ```
    #[inline]
    pub unsafe fn new_sorted_mut(slice: &mut [T]) -> &mut Self
    where
        T: Ord,
    {
        slice.sort();

        // SAFETY: the slice has been sorted
        unsafe { Self::new_mut_unchecked(slice) }
    }

    // TODO: make const once `const_mut_refs` stabilizes
    //       see https://github.com/rust-lang/rust/issues/57349

    /// Creates a new [`Sorted`] from the given `slice`, assuming it was sorted.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - `slice` is sorted
    ///
    /// # Examples
    /// ```
    /// # use strtools::util::Sorted;
    /// let sorted: &mut Sorted<_> = unsafe { Sorted::new_mut_unchecked(&mut ['a', 'b', 'c']) };
    /// ```
    /// Violation of invariants:
    /// ```
    /// # use strtools::util::Sorted;
    /// // this is not sorted, Sorted invariants are violated
    /// let sorted: &mut Sorted<_> = unsafe { Sorted::new_mut_unchecked(&mut ['a', 'c', 'b']) };
    /// ```
    #[inline]
    pub unsafe fn new_mut_unchecked(slice: &mut [T]) -> &mut Self {
        // SAFETY:
        // - the caller must ensure that the slice is sorted
        // - #[repr(transparent)] ensures layout compatibility of &[T] and &Self
        // - the lifetime of &Self is the same as `slice`
        unsafe { std::mem::transmute(slice) }
    }

    /// Borrows this as a slice of `[T]`.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::Sorted;
    /// let sorted: &Sorted<_> = ['a', 'b', 'c'][..].try_into()?;
    /// let slice: &[char] = sorted.as_slice();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    /// Borrows this as a mutable slice of `[T]`. This function is not unsafe as getting a
    /// `&mut Sorted` is already unsafe.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::Sorted;
    /// let mut slice = ['a', 'b', 'c'];
    /// let sorted: &mut Sorted<_> = unsafe { Sorted::new_mut(&mut slice)? };
    /// let slice: &[char] = sorted.as_slice_mut();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.0
    }
}

impl<T: PartialOrd> Deref for Sorted<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T: PartialOrd> AsRef<[T]> for Sorted<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: PartialOrd> Borrow<[T]> for Sorted<T> {
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'s, T: PartialOrd> TryFrom<&'s [T]> for &'s Sorted<T> {
    type Error = SortedSliceError;

    fn try_from(value: &'s [T]) -> Result<Self, Self::Error> {
        Sorted::new(value)
    }
}

impl<'s, T: PartialOrd> TryFrom<&'s mut [T]> for &'s Sorted<T> {
    type Error = SortedSliceError;

    fn try_from(value: &'s mut [T]) -> Result<Self, Self::Error> {
        Sorted::new(value)
    }
}
