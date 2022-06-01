//! This crate provides the [StrTools] trait which exposes a variety of helper functions for
//! handling strings for use cases like handling user input.
//!
//! # Examples
//! ```
//! # use strtools::StrTools;
//! # use std::error::Error;
//! # fn main() -> Result<(), Box<dyn Error>> {
//! // split a string by some separator but ignore escaped ones
//! let parts: Vec<_> = r"this string\ is split by\ spaces unless they are\ escaped"
//!     .split_non_escaped('\\', &[' '])?
//!     .collect();
//!
//! assert_eq!(
//!     parts,
//!     [
//!         "this",
//!         "string is",
//!         "split",
//!         "by spaces",
//!         "unless",
//!         "they",
//!         "are escaped"
//!     ]
//! );
//! # Ok(())
//! # }
//! ```
#![feature(cow_is_borrowed, let_chains)]
#![warn(missing_docs, clippy::missing_panics_doc)]
#![deny(unsafe_op_in_unsafe_fn)]

pub mod split;

mod sealed {
    pub trait Sealed {}
    impl Sealed for str {}
}

/// The main trait of this crate, providing various extension methods for [str].
/// See the individual function documentation for more info.
///
/// [crate_doc]: crate
pub trait StrTools: sealed::Sealed {
    /// Splits a [str] by the given delimiters unless they are precided by an escape.
    /// Escapes before significant chars are removed, significant chars are the delimters and the
    /// escape itself. Trailing escapes are ignored as if followed by a non-significant char.
    ///
    /// # Errors
    /// Returns an Error if `delims` contains `esc`
    ///
    /// # Complexity & Allocation
    /// This algorithm requires `O(n * m)` time where `n` is the length of the input string and `m`
    /// is the length of `delims` to split the full string. If no escapes are encountered in a
    /// part, no allocations are done and the part is borrowed, otherwise a [String] and all but
    /// the escape char are copied over.
    ///
    /// # Examples
    /// ```
    /// # use strtools::StrTools;
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let value = r"Pa\rt0:Part1:Part2\:StillPart2";
    /// let parts: Vec<_> = value.split_non_escaped('\\', &[':'])?.collect();
    ///
    /// // notice that the escape char was removed in Part2 but not in Part1 as it's just used as
    /// // an indicator for escaping the delimiters or escapes themselves
    /// assert_eq!(parts, [r"Pa\rt0", "Part1", "Part2:StillPart2"]);
    /// # Ok(())
    /// # }
    /// ```
    fn split_non_escaped<'d>(
        &self,
        esc: char,
        delims: &'d [char],
    ) -> Result<split::SplitNonEscaped<'_, 'd>, split::EscapeIsDelimiterError>;
}

impl StrTools for str {
    fn split_non_escaped<'d>(
        &self,
        esc: char,
        delims: &'d [char],
    ) -> Result<split::SplitNonEscaped<'_, 'd>, split::EscapeIsDelimiterError> {
        split::non_escaped(self, esc, delims)
    }
}
