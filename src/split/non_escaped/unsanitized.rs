use super::NonEscapedError;
use crate::{split, util::Sorted};
use std::iter::FusedIterator;

/// Splits a [str] by the given delimiter unless it is preceded by a given escape. This is a
/// sanitization free version of [`non_escaped_sanitize`][0].
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
/// [0]: super::non_escaped_sanitize
///
/// # Examples
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use strtools::split;
///
/// // split a string by some separator but ignore escaped ones
/// let parts: Vec<_> = split::non_escaped(
///     r"this string\ is split by\ spaces unless they are\ escaped",
///     '\\',
///     [' '].try_into()?
/// )?.collect();
///
/// // nothing is sanitized, the escapes are kept
/// assert_eq!(
///     parts,
///     [
///         r"this",
///         r"string\ is",
///         r"split",
///         r"by\ spaces",
///         r"unless",
///         r"they",
///         r"are\ escaped"
///     ]
/// );
/// # Ok(())
/// # }
/// ```
pub fn non_escaped<const N: usize>(
    input: &str,
    esc: char,
    delims: Sorted<char, N>,
) -> Result<NonEscaped<'_, N>, NonEscapedError> {
    if delims.binary_search(&esc).is_ok() {
        Err(NonEscapedError::EscapeContainsDelimiter(esc))
    } else {
        Ok(NonEscaped {
            rest: Some(input),
            esc,
            delims,
        })
    }
}

/// An [Iterator] that yields parts of a [str] that are separated by a delimiter. This struct is
/// created by the [`non_escaped`] method, see it's documentation for more info.
#[derive(Debug)]
pub struct NonEscaped<'input, const DELIMITERS: usize> {
    rest: Option<&'input str>,
    esc: char,
    delims: Sorted<char, DELIMITERS>,
}

impl<'s, const N: usize> Iterator for NonEscaped<'s, N> {
    type Item = &'s str;

    fn next(&mut self) -> Option<Self::Item> {
        let rest = self.rest?;
        let mut iter = rest.char_indices().peekable();
        let mut is_escaped = false;

        while let Some((idx, ch)) = iter.next() {
            // escape
            if ch == self.esc {
                is_escaped = !is_escaped;

                // are we escaping? if yes continue to next
                if is_escaped {
                    continue;
                }

                // are we at the end? yield rest
                if iter.peek().is_none() {
                    break;
                }
            }

            // normal delimiter
            if !is_escaped && self.delims.binary_search(&ch).is_ok() {
                // SAFETY: correctness of index relies on str::char_indices
                let (result, _, rest) = unsafe { split::char_boundary_unchecked(rest, idx) };
                self.rest = Some(rest);
                return Some(result);
            }

            is_escaped = false;
        }

        // no delimiter was found, just yield the rest
        self.rest.take()
    }
}

impl<'s, const N: usize> FusedIterator for NonEscaped<'s, N> {}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_impl {
        ($split:expr; $from:literal => [$($to:literal),+]) => {
            eprintln!("boundary");
            assert_eq!(
                non_escaped($from, '\\', $split.try_into().unwrap())
                    .expect("delim and escape are not the same")
                    .collect::<Vec<_>>(),
                vec![$($to),+]
            )
        };
    }

    #[test]
    fn empty() {
        assert!(non_escaped("", '\\', [':'].try_into().unwrap()).is_ok());
    }

    #[test]
    fn delim_is_escape() {
        assert_eq!(
            non_escaped("", '\\', ['\\'].try_into().unwrap()).unwrap_err(),
            NonEscapedError::EscapeContainsDelimiter('\\')
        );
    }

    #[test]
    fn no_escape() {
        test_impl!([':']; r"aaaaa:bbbbb" => ["aaaaa", "bbbbb"]);
    }

    #[test]
    fn single_escape() {
        test_impl!([':']; r"aa\:aa:bbbb" => [r"aa\:aa", "bbbb"]);
        test_impl!([':']; r"\:aaaa:bbbb" => [r"\:aaaa", "bbbb"]);
        test_impl!([':']; r"aaaa\::bbbb" => [r"aaaa\:", "bbbb"]);
        test_impl!([':']; r"aaaa:bb\:bb" => ["aaaa", r"bb\:bb"]);
        test_impl!([':']; r"aaaa:\:bbbb" => ["aaaa", r"\:bbbb"]);
        test_impl!([':']; r"aaaa:bbbb\:" => ["aaaa", r"bbbb\:"]);
    }

    #[test]
    fn double_escapes() {
        test_impl!([':']; r"aaaa\\:bbbb" => [r"aaaa\\", "bbbb"]);
        test_impl!([':']; r"aaaa\\\:bbbb" => [r"aaaa\\\:bbbb"]);
        test_impl!([':']; r"aaaa\\\\:bbbb" => [r"aaaa\\\\", "bbbb"]);
        test_impl!([':']; r"aaaa\\\\\:bbbb" => [r"aaaa\\\\\:bbbb"]);
    }

    #[test]
    fn ignore_other_escapes() {
        test_impl!([':']; r"aa\.aa:bbbbb" => [r"aa\.aa", "bbbbb"]);
        test_impl!([':']; r"\.aaaa:bbbbb" => [r"\.aaaa", "bbbbb"]);
        test_impl!([':']; r"aaaa\.:bbbbb" => [r"aaaa\.", "bbbbb"]);
        test_impl!([':']; r"aaaa:\.bbbbb" => ["aaaa", r"\.bbbbb"]);
        test_impl!([':']; r"aaaa:bbbbb\." => ["aaaa", r"bbbbb\."]);
    }
}
