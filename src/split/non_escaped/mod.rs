mod sanitized;
pub use sanitized::*;

mod unsanitized;
pub use unsanitized::*;

/// An [Error][0] for `non_escaped*` functions, see their documentation for more info.
///
/// [0]: std::error::Error
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("a delimiter cannot be it's own escape char {0}")]
pub enum NonEscapedError {
    /// Indicates that a given escape char was also given as a delimiter.
    EscapeIsDelimiter(char),
}
