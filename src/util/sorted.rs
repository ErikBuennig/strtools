use super::{SortedError, SortedSlice};
use std::{borrow::Borrow, fmt::Debug, ops::Deref};

/// Represents a `[T; N]` that is guaranteed to be sorted by [`T: PartialOrd`][pord]. Unlike
/// [Sorted][sorted] this is not a [DST][dst] and thus has a slightly different API.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use strtools::util::Sorted;
/// // only checks if the array is sorted
/// let sorted: Sorted<_, 3> = Sorted::new(['a', 'b', 'c'])?;
///
/// // sorts the slice and is therefore not fallible, requires T: Ord, the returned array is
/// // immutable
/// let sorted: Sorted<_, 3> = Sorted::new_sorted(['a', 'c', 'b']);
/// # Ok(())
/// # }
/// ```
///
/// [sorted]: super::Sorted
/// [dst]: https://doc.rust-lang.org/book/ch19-04-advanced-types.html#dynamically-sized-types-and-the-sized-trait
/// [pord]: PartialOrd
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sorted<T: PartialOrd, const N: usize>([T; N]);

impl<T: PartialOrd, const N: usize> Sorted<T, N> {
    /// Creates a new [`Sorted`] from the given `array` if it was sorted.
    ///
    /// # Errors
    /// Returns an error if:
    /// - `array` was not sorted
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::Sorted;
    /// let sorted: Sorted<_, 3> = Sorted::new(['a', 'b', 'c'])?;
    /// # Ok(())
    /// # }
    /// ```
    /// This will return an error:
    /// ```should_panic
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::Sorted;
    /// // this is not sorted
    /// let sorted: Sorted<_, 3> = Sorted::new(['a', 'c', 'b'])?;
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn new(array: [T; N]) -> Result<Self, SortedError> {
        if array.is_sorted() {
            // SAFETY: the array is sorted according to R
            Ok(unsafe { Self::new_unchecked(array) })
        } else {
            Err(SortedError::NotSorted)
        }
    }

    /// Sorts the given array and creates a new mutable [`Sorted`] from it.
    ///
    /// # Examples
    /// ```
    /// # use strtools::util::Sorted;
    /// let sorted: Sorted<_, 3> = Sorted::new_sorted(['a', 'c', 'b']);
    /// assert_eq!(sorted.as_array_ref(), &['a', 'b', 'c']);
    /// ```
    #[inline]
    pub fn new_sorted(mut array: [T; N]) -> Self
    where
        T: Ord,
    {
        array.sort();

        // SAFETY: the array has been sorted
        unsafe { Self::new_unchecked(array) }
    }

    /// Creates a new [`Sorted`] from the given `array`, assuming it was sorted.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - `array` is sorted
    ///
    /// # Examples
    /// ```
    /// # use strtools::util::Sorted;
    /// let sorted: Sorted<_, 3> = unsafe { Sorted::new_unchecked(['a', 'b', 'c']) };
    /// ```
    /// Violation of invariants:
    /// ```
    /// # use strtools::util::Sorted;
    /// // this is not sorted, Sorted invariants are violated
    /// let sorted: Sorted<_, 3> = unsafe { Sorted::new_unchecked(['a', 'c', 'b']) };
    /// ```
    #[inline]
    pub const unsafe fn new_unchecked(array: [T; N]) -> Self {
        Self(array)
    }

    /// Borrows this as a reference to an array `&[T; N]`.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::Sorted;
    /// let sorted: Sorted<_, 3> = Sorted::new(['a', 'b', 'c'])?;
    /// let array_ref: &[char; 3] = sorted.as_array_ref();
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub const fn as_array_ref(&self) -> &[T; N] {
        &self.0
    }

    /// Borrows this as a mutable reference to an array `&mut [T; N]`.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - `array` remains sorted according to `T: PartialOrd` if mutated
    ///
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::Sorted;
    /// let mut sorted: Sorted<_, 3> = Sorted::new(['a', 'b', 'c'])?;
    /// let array_mut: &[char; 3] = unsafe { sorted.as_array_mut() };
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub const unsafe fn as_array_mut(&mut self) -> &mut [T; N] {
        &mut self.0
    }

    /// Borrows this as a [`SortedSlice<T>`].
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::{Sorted, SortedSlice};
    /// let sorted: Sorted<_, 3> = Sorted::new(['a', 'b', 'c'])?;
    /// let sorted_slice: &SortedSlice<char> = sorted.as_sorted_slice();
    /// # Ok(())
    /// # }
    pub const fn as_sorted_slice(&self) -> &SortedSlice<T> {
        // SAFETY: the array is sorted
        unsafe { SortedSlice::new_unchecked(&self.0) }
    }

    /// Borrows this as a [`SortedSlice<T>`].
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - `array` remains sorted according to `T: PartialOrd` if mutated
    ///
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # use strtools::util::{Sorted, SortedSlice};
    /// let mut sorted: Sorted<_, 3> = Sorted::new(['a', 'b', 'c'])?;
    /// let sorted_slice_mut: &mut SortedSlice<char> = unsafe { sorted.as_sorted_slice_mut() };
    /// # Ok(())
    /// # }
    /// ```
    pub const unsafe fn as_sorted_slice_mut(&mut self) -> &mut SortedSlice<T> {
        // SAFETY: the array is sorted
        unsafe { SortedSlice::new_unchecked_mut(&mut self.0) }
    }
}

impl<T: PartialOrd + Debug, const N: usize> Debug for Sorted<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: PartialOrd, const N: usize> Deref for Sorted<T, N> {
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        self.as_array_ref()
    }
}

impl<T: PartialOrd, const N: usize> AsRef<[T; N]> for Sorted<T, N> {
    fn as_ref(&self) -> &[T; N] {
        self.as_array_ref()
    }
}

impl<T: PartialOrd, const N: usize> Borrow<[T; N]> for Sorted<T, N> {
    fn borrow(&self) -> &[T; N] {
        self.as_array_ref()
    }
}

impl<T: PartialOrd, const N: usize> AsRef<SortedSlice<T>> for Sorted<T, N> {
    fn as_ref(&self) -> &SortedSlice<T> {
        self.as_sorted_slice()
    }
}

impl<T: PartialOrd, const N: usize> Borrow<SortedSlice<T>> for Sorted<T, N> {
    fn borrow(&self) -> &SortedSlice<T> {
        self.as_sorted_slice()
    }
}

impl<T: Ord, const N: usize> From<[T; N]> for Sorted<T, N> {
    fn from(value: [T; N]) -> Self {
        Sorted::new_sorted(value)
    }
}

impl<T: PartialOrd> From<T> for Sorted<T, 1> {
    fn from(value: T) -> Self {
        // SAFETY: single item must not be sorted
        unsafe { Sorted::new_unchecked([value]) }
    }
}
