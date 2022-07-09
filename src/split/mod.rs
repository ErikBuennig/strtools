//! This module contains functions with the primary purpose of splitting [str]s.

use crate::util::Sorted;

mod char_boundary;
pub use char_boundary::*;

mod non_escaped;
pub use non_escaped::*;

/// Splits a string into `N + 1` pieces.
///
/// # Panics
/// Panics if an index is out of bounds, `index <= input.len()`.
///
/// # Examples
/// ```
/// # use strtools::split;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let ([a, b], c) = split::n_times("abcdefghijkl", &[4, 8].try_into()?);
///
/// assert_eq!((a, b, c), ("abcd", "efgh", "ijkl"));
/// # Ok(())
/// # }
/// ```
pub fn n_times<'s, const N: usize>(
    input: &'s str,
    indices: &Sorted<usize, N>,
) -> ([&'s str; N], &'s str) {
    match indices.last() {
        // N is not 0, since it must be sorted, if the last index is in bounds, then so are all
        // others
        Some(&last) => assert!(last <= input.len(), "index out of bounds"),
        None => return ([""; N], input),
    }

    let mut res = [""; N];
    let mut prev = 0;

    for (idx, &index) in indices.iter().enumerate() {
        // SAFETY: indices checked above
        res[idx] = unsafe { input.get_unchecked(prev..index) };
        prev = index;
    }

    // SAFETY: see above
    (res, unsafe { input.get_unchecked(prev..) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn n_times_non_overlapping() {
        assert_eq!(
            n_times("abcdefghijkl", &[4, 8].try_into().unwrap()),
            (["abcd", "efgh"], "ijkl")
        );
    }

    #[test]
    pub fn n_times_non_boundary() {
        assert_eq!(
            n_times("abcdefgh", &[].try_into().unwrap()),
            ([], "abcdefgh")
        );
        assert_eq!(
            n_times("abcdefgh", &[0].try_into().unwrap()),
            ([""], "abcdefgh")
        );
        assert_eq!(
            n_times("abcdefgh", &[8].try_into().unwrap()),
            (["abcdefgh"], "")
        );
    }

    #[test]
    pub fn n_times_non_repeating() {
        assert_eq!(
            n_times("abcdefghijkl", &[4, 4, 4, 8].try_into().unwrap()),
            (["abcd", "", "", "efgh"], "ijkl")
        );
    }
}
