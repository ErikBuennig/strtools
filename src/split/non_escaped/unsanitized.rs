use super::NonEscapedError;
use crate::split;
use std::iter::FusedIterator;

/// Splits a [str] by the given delimiters unless they are precided by an escape. This is a
/// sanitization free version of [`non_escaped_sanitize`][0].
///
/// # Errors
/// Returns an Error if `delims` contains `esc`
///
/// # Complexity & Allocation
/// This algorithm requires `O(n * m)` time where `n` is the length of the input string and `m`
/// is the length of `delims`. This algorithm does not allocate.
///
/// [0]: super::non_escaped_sanitize
///
/// # Examples
/// ```
/// use strtools::split;
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // split a string by some separator but ignore escaped ones
/// let parts: Vec<_> = split::non_escaped(
///     r"this string\ is split by\ spaces unless they are\ escaped",
///     '\\',
///     &[' ']
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
pub fn non_escaped<'s, 'd>(
    input: &'s str,
    esc: char,
    delims: &'d [char],
) -> Result<NonEscaped<'s, 'd>, NonEscapedError> {
    if !delims.contains(&esc) {
        Ok(NonEscaped {
            rest: Some(input),
            esc,
            delims,
        })
    } else {
        Err(NonEscapedError::EscapeIsDelimiter(esc))
    }
}

/// An [Iterator] that yields parts of a [str] that are separated by a delimiter. This struct is
/// created by the [`non_escaped`] method, See it's documentation for more info.
#[derive(Debug)]
pub struct NonEscaped<'s, 'd> {
    rest: Option<&'s str>,
    esc: char,
    delims: &'d [char],
}

impl<'s, 'd> Iterator for NonEscaped<'s, 'd> {
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
            if !is_escaped && self.delims.contains(&ch) {
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

impl<'s, 'd> FusedIterator for NonEscaped<'s, 'd> {}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_impl {
        ($split:expr; $from:literal => [$($to:literal),+]) => {
            eprintln!("boundary");
            assert_eq!(
                non_escaped($from, '\\', &$split)
                    .expect("delim and escape are not the same")
                    .collect::<Vec<_>>(),
                    vec![$($to),+]
            )
        };
        ($from:literal => [$($to:literal),+]) => {
            test_impl!([':']; $from => [$($to),+]);
        };
    }

    #[test]
    fn empty() {
        assert!(non_escaped("", '\\', &[':']).is_ok());
    }

    #[test]
    fn delim_is_escape() {
        assert_eq!(
            non_escaped("", '\\', &['\\']).unwrap_err(),
            NonEscapedError::EscapeIsDelimiter('\\')
        );
    }

    #[test]
    fn no_escape() {
        test_impl!(r"aaaaa:bbbbb" => ["aaaaa", "bbbbb"]);
    }

    #[test]
    fn multiple() {
        test_impl!(['/', ':']; r"aaaaa/bbb:bb" => ["aaaaa", "bbb", "bb"]);
        test_impl!(['/', ':']; r"aaaaa/bbb\:bb" => ["aaaaa", r"bbb\:bb"]);
        test_impl!(['/', ':']; r"aaaaa\/bbb\:bb" => [r"aaaaa\/bbb\:bb"]);
    }

    #[test]
    fn single_escape() {
        test_impl!(r"aa\:aa:bbbb" => [r"aa\:aa", "bbbb"]);
        test_impl!(r"\:aaaa:bbbb" => [r"\:aaaa", "bbbb"]);
        test_impl!(r"aaaa\::bbbb" => [r"aaaa\:", "bbbb"]);
        test_impl!(r"aaaa:bb\:bb" => ["aaaa", r"bb\:bb"]);
        test_impl!(r"aaaa:\:bbbb" => ["aaaa", r"\:bbbb"]);
        test_impl!(r"aaaa:bbbb\:" => ["aaaa", r"bbbb\:"]);
    }

    #[test]
    fn double_escapes() {
        test_impl!(r"aaaa\\:bbbb" => [r"aaaa\\", "bbbb"]);
        test_impl!(r"aaaa\\\:bbbb" => [r"aaaa\\\:bbbb"]);
        test_impl!(r"aaaa\\\\:bbbb" => [r"aaaa\\\\", "bbbb"]);
        test_impl!(r"aaaa\\\\\:bbbb" => [r"aaaa\\\\\:bbbb"]);
    }

    #[test]
    fn ignore_other_escapes() {
        test_impl!(r"aa\.aa:bbbbb" => [r"aa\.aa", "bbbbb"]);
        test_impl!(r"\.aaaa:bbbbb" => [r"\.aaaa", "bbbbb"]);
        test_impl!(r"aaaa\.:bbbbb" => [r"aaaa\.", "bbbbb"]);
        test_impl!(r"aaaa:\.bbbbb" => ["aaaa", r"\.bbbbb"]);
        test_impl!(r"aaaa:bbbbb\." => ["aaaa", r"bbbbb\."]);
    }
}
