//! This crate provides the [StrTools] trait which exposes a variety of helper functions for
//! handling strings for use cases like handling user input.
//!
//! # Examples
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use strtools::StrTools;
//!
//! // split a string by some separator but ignore escaped ones
//! let parts: Vec<_> = r"this string\ is split by\ spaces unless they are\ escaped"
//!     .split_non_escaped_sanitize('\\', ' ')?
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
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use strtools::StrTools;
//!
//! let parts: Vec<_> = r"\.\/.*s(\d\d)e(\d\d[a-d])/S$1E$2/gu"
//!     .split_non_escaped_sanitize('\\', '/')?
//!     .collect();
//!
//! // parsing user input regex rules like `<rule>/<replace>/<flags>`
//! // the rule contained an escaped separator but we don't want to
//! // actually escape it for the regex engine
//! assert_eq!(parts, [r"\./.*s(\d\d)e(\d\d[a-d])", "S$1E$2", "gu"]);
//! # Ok(())
//! # }
//! ```
// keep the nightly features set small in hopes that all used features are stabilized by the time
// this crate will stabilize
#![feature(cow_is_borrowed, let_chains)]
// check for missing documentation
#![warn(
    missing_docs,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    clippy::missing_safety_doc
)]
// reduce unsafe scopes to thier minimum
#![deny(unsafe_op_in_unsafe_fn)]

pub mod split;

mod sealed {
    pub trait Sealed {}
    impl Sealed for str {}
}

/// The main trait of this crate, providing various extension methods for [str].
/// See the individual function documentation for more info. **The methods on this trait are subject
/// to change during the development of the crates core functionality.**
pub trait StrTools: sealed::Sealed {
    /// Splits a [str] by the given delimiters unless they are preceded by an escape.
    /// Escapes before significant chars are removed, significant chars are the delimiters and the
    /// escape itself. Trailing escapes are ignored as if followed by a non-significant char.
    ///
    /// # Errors
    /// Returns an error if:
    /// - `esc == delim`
    ///
    /// # Complexity
    /// This algorithm requires `O(n)` time where `n` is the length of the input string.
    ///
    /// # Allocation
    /// If no escapes are encountered in a part, no allocations are done and the part is borrowed,
    /// otherwise a [String] and all but the escape chars before delimiters are copied over.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use strtools::StrTools;
    ///
    /// let value = r"Pa\rt0:Part1:Part2\:StillPart2";
    /// let parts: Vec<_> = value.split_non_escaped_sanitize('\\', ':')?.collect();
    ///
    /// // notice that the escape char was removed in Part2 but not in Part1 as it's just used as
    /// // an indicator for escaping the delimiters or escapes themselves
    /// assert_eq!(parts, [r"Pa\rt0", "Part1", "Part2:StillPart2"]);
    /// # Ok(())
    /// # }
    /// ```
    fn split_non_escaped_sanitize(
        &self,
        esc: char,
        delim: char,
    ) -> Result<split::NonEscapedSanitize<'_>, split::NonEscapedError>;

    /// Splits a [str] by the given delimiters unless they are preceded by an escape.
    /// Escapes before significant chars are removed, significant chars are the delimiters and the
    /// escape itself. Trailing escapes are ignored as if followed by a non-significant char.
    ///
    /// # Errors
    /// Returns an error if:
    /// - `esc == delim`
    ///
    /// # Complexity
    /// This algorithm requires `O(n)` time where `n` is the length of the input string.
    ///
    /// # Allocation
    /// No allocations are done.
    ///
    /// # Examples
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use strtools::StrTools;
    ///
    /// let value = r"Pa\rt0:Part1:Part2\:StillPart2";
    /// let parts: Vec<_> = value.split_non_escaped('\\', ':')?.collect();
    ///
    /// // no sanitization is done here the separators are simply ignored
    /// assert_eq!(parts, [r"Pa\rt0", "Part1", r"Part2\:StillPart2"]);
    /// # Ok(())
    /// # }
    /// ```
    fn split_non_escaped(
        &self,
        esc: char,
        delim: char,
    ) -> Result<split::NonEscaped<'_>, split::NonEscapedError>;
}

impl StrTools for str {
    fn split_non_escaped_sanitize<'d>(
        &self,
        esc: char,
        delim: char,
    ) -> Result<split::NonEscapedSanitize<'_>, split::NonEscapedError> {
        split::non_escaped_sanitize(self, esc, delim)
    }

    fn split_non_escaped<'d>(
        &self,
        esc: char,
        delim: char,
    ) -> Result<split::NonEscaped<'_>, split::NonEscapedError> {
        split::non_escaped(self, esc, delim)
    }
}
