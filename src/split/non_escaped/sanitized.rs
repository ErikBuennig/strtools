use super::NonEscapedError;
use std::{borrow::Cow, iter::Peekable, str::CharIndices};

/// Splits a [str] by the given delimiter unless it is preceded by a given escape. Escapes before
/// significant chars are removed, significant chars are the delimiters and the escape itself.
/// Trailing escapes are ignored as if followed by a non-significant char. See [`non_escaped`][0]
/// for an allocation free version of this function.
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
/// [0]: super::non_escaped
///
/// # Examples
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use strtools::split;
///
/// // split a string by some separator but ignore escaped ones
/// let parts: Vec<_> = split::non_escaped_sanitize(
///     r"this string\ is split by\ spaces unless they are\ escaped",
///     '\\',
///     ' '
/// )?.collect();
///
/// // the splits are sanitized, the escapes are removed
/// assert_eq!(
///     parts,
///     [
///         "this",
///         "string is",
///         "split",
///         "by spaces",
///         "unless",
///         "they",
///         "are escaped"
///     ]
/// );
/// # Ok(())
/// # }
/// ```
pub fn non_escaped_sanitize(
    input: &str,
    esc: char,
    delim: char,
) -> Result<NonEscapedSanitize<'_>, NonEscapedError> {
    if esc == delim {
        Err(NonEscapedError::EscapeIsDelimiter(esc))
    } else {
        Ok(NonEscapedSanitize {
            input,
            done: 0,
            esc,
            delim,
            iter: input.char_indices().peekable(),
            curr: Some(Cow::Borrowed("")),
        })
    }
}

// TODO: reduce unwraps, technically curr can be local and something else can be used to check if
//       it's finished, reduce the overall complexity to be more akin ot it's non-sanitizing
//       counterpart

/// An [Iterator] that yields parts of a [str] that are separated by a delimiter. This struct is
/// created by the [`non_escaped_sanitize`] method, see it's documentation for more info.
#[derive(Debug)]
pub struct NonEscapedSanitize<'s> {
    input: &'s str,
    done: usize,
    esc: char,
    delim: char,
    iter: Peekable<CharIndices<'s>>,
    curr: Option<Cow<'s, str>>,
}

impl<'s> Iterator for NonEscapedSanitize<'s> {
    type Item = Cow<'s, str>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((idx, ch)) = self.iter.next() {
            // escape
            if ch == self.esc && self.iter.peek().is_some() {
                let (next_idx, escaped) = self.iter.next().unwrap();

                let mutate = self.curr.as_mut().unwrap().to_mut();
                if escaped != self.esc && escaped != self.delim {
                    mutate.push(self.esc);
                }

                mutate.push(escaped);
                self.done = next_idx + escaped.len_utf8();
                continue;
            }

            // normal delimiter
            if ch == self.delim {
                self.done = idx + ch.len_utf8();
                return self.curr.replace(Cow::Borrowed(""));
            }

            // regular char
            let mut jump = idx + ch.len_utf8();

            while let Some(&(i, c)) = self.iter.peek()
                && (c != self.esc && c != self.delim)
            {
                jump = i + c.len_utf8();
                let _ = self.iter.next();
            }

            let remaining = &self.input[self.done..jump];
            let curr = self.curr.as_mut().unwrap();
            if curr.is_borrowed() {
                *curr = Cow::Borrowed(remaining);
            } else {
                curr.to_mut().push_str(remaining);
            }

            self.done = jump;
        }

        if self.done < self.input.len() {
            let remaining = &self.input[self.done..self.input.len()];
            let curr = self.curr.as_mut().unwrap();
            if curr.is_borrowed() {
                *curr = Cow::Borrowed(remaining);
            } else {
                curr.to_mut().push_str(remaining);
            }

            self.done = self.input.len();
        }

        self.curr.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_impl {
        ($split:expr; $from:literal => [$($to:literal),+]) => {
            assert_eq!(
                non_escaped_sanitize($from, '\\', $split)
                    .expect("delim and escape are not the same")
                    .collect::<Vec<_>>(),
                vec![$($to),+]
            )
        };
    }

    #[test]
    fn empty() {
        assert!(non_escaped_sanitize("", '\\', ':').is_ok());
    }

    #[test]
    fn delim_is_escape() {
        assert_eq!(
            non_escaped_sanitize("", '\\', '\\').unwrap_err(),
            NonEscapedError::EscapeIsDelimiter('\\')
        );
    }

    #[test]
    fn no_escape() {
        test_impl!(':'; r"aaaaa:bbbbb" => ["aaaaa", "bbbbb"]);
    }

    #[test]
    fn single_escape() {
        test_impl!(':'; r"aa\:aa:bbbb" => ["aa:aa", "bbbb"]);
        test_impl!(':'; r"\:aaaa:bbbb" => [":aaaa", "bbbb"]);
        test_impl!(':'; r"aaaa\::bbbb" => ["aaaa:", "bbbb"]);
        test_impl!(':'; r"aaaa:bb\:bb" => ["aaaa", "bb:bb"]);
        test_impl!(':'; r"aaaa:\:bbbb" => ["aaaa", ":bbbb"]);
        test_impl!(':'; r"aaaa:bbbb\:" => ["aaaa", "bbbb:"]);
    }

    #[test]
    fn double_escapes() {
        test_impl!(':'; r"aaaa\\:bbbb" => [r"aaaa\", "bbbb"]);
        test_impl!(':'; r"aaaa\\\:bbbb" => [r"aaaa\:bbbb"]);
        test_impl!(':'; r"aaaa\\\\:bbbb" => [r"aaaa\\", "bbbb"]);
        test_impl!(':'; r"aaaa\\\\\:bbbb" => [r"aaaa\\:bbbb"]);
    }

    #[test]
    fn ignore_other_escapes() {
        test_impl!(':'; r"aa\.aa:bbbbb" => [r"aa\.aa", "bbbbb"]);
        test_impl!(':'; r"\.aaaa:bbbbb" => [r"\.aaaa", "bbbbb"]);
        test_impl!(':'; r"aaaa\.:bbbbb" => [r"aaaa\.", "bbbbb"]);
        test_impl!(':'; r"aaaa:\.bbbbb" => ["aaaa", r"\.bbbbb"]);
        test_impl!(':'; r"aaaa:bbbbb\." => ["aaaa", r"bbbbb\."]);
    }

    #[test]
    fn copy_on_sanitize() {
        // only copy when sanitizing an escape
        let res = non_escaped_sanitize(r"a\:aa:bbb:cc\.c:ddd", '\\', ':')
            .expect("delim and escape are not the same")
            .collect::<Vec<_>>();

        // owned
        assert_eq!(res[0], "a:aa");
        assert!(!res[0].is_borrowed());

        // borrowed
        assert_eq!(res[1], "bbb");
        assert!(res[1].is_borrowed());

        // owned
        assert_eq!(res[2], r"cc\.c");
        assert!(!res[2].is_borrowed());

        // borrowed
        assert_eq!(res[3], "ddd");
        assert!(res[3].is_borrowed());
    }

    // the tests in ths module are examples of where this was first used in a private program I made
    // that parsed user input's as `<regex-rule>/<regex-replace>/<regex-flags>`, the given input
    // rules must still allow the separator path to be included somehow
    mod field_tests {
        use super::*;

        #[test]
        fn trailing_ignored_escape() {
            // the trailing escape caused the split to not include the last char
            test_impl!('/'; r"test\d" => [r"test\d"]);
        }

        #[test]
        fn escaped() {
            // the escape was not correctly removed
            test_impl!('/'; r"^b\/(.*)$/d\/$1" => [r"^b/(.*)$", "d/$1"]);
        }

        #[test]
        fn ignored_escape_offset() {
            // multiple subsequent to-be-ignored escape sequences were not properly being split
            // and resulted in more parts than expected as well as missing chars
            test_impl!('/'; r".*s(\d\d)e(\d\d[a-d])/S$1E$2" => [
                r".*s(\d\d)e(\d\d[a-d])",
                "S$1E$2"
            ]);
        }
    }
}
