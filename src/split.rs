use std::{borrow::Cow, iter::Peekable, str::CharIndices};

/// An error type that indicates that a delimiter and an escape char were the same
#[derive(Debug)]
pub struct EscapeIsDelimiterError(());

impl std::fmt::Display for EscapeIsDelimiterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a delimiter cannot be it's own escape char")
    }
}

impl std::error::Error for EscapeIsDelimiterError {}

/// Returns an [Iterator] over slices of a [str] separated by unescaped chars from `delims`.
///
/// # Examples
/// ```
/// use strtools::split;
/// # use std::error::Error;
/// # fn main() -> Result<(), Box<dyn Error>> {
/// // split a string by some separator but ignore escaped ones
/// let parts: Vec<_> = split::non_escaped(
///     r"this string\ is split by\ spaces unless they are\ escaped",
///     '\\',
///     &[' ']
/// )?.collect();
///
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
pub fn non_escaped<'s, 'd>(
    input: &'s str,
    esc: char,
    delims: &'d [char],
) -> Result<SplitNonEscaped<'s, 'd>, EscapeIsDelimiterError> {
    if !delims.contains(&esc) {
        Ok(SplitNonEscaped {
            input,
            done: 0,
            esc,
            delims,
            iter: input.char_indices().peekable(),
            curr: Some(Cow::Borrowed("")),
        })
    } else {
        Err(EscapeIsDelimiterError(()))
    }
}

/// An [Iterator] that yields parts of a [str] that are separated by a delimiter.
/// This struct is created by the [`non_escaped`] method, See it's documentation for more info.
#[derive(Debug)]
pub struct SplitNonEscaped<'s, 'd> {
    input: &'s str,
    done: usize,
    esc: char,
    delims: &'d [char],
    iter: Peekable<CharIndices<'s>>,
    curr: Option<Cow<'s, str>>,
}

impl<'s, 'd> Iterator for SplitNonEscaped<'s, 'd> {
    type Item = Cow<'s, str>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((idx, ch)) = self.iter.next() {
            // escape
            if ch == self.esc && self.iter.peek().is_some() {
                let (next_idx, escaped) = self.iter.next().unwrap();

                let mutate = self.curr.as_mut().unwrap().to_mut();
                if escaped != self.esc && !self.delims.contains(&escaped) {
                    mutate.push(self.esc);
                }

                mutate.push(escaped);
                self.done = next_idx + escaped.len_utf8();
                continue;
            }

            // normal delimiter
            if self.delims.contains(&ch) {
                self.done = idx + ch.len_utf8();
                return self.curr.replace(Cow::Borrowed(""));
            }

            // regular char
            let mut jump = idx + ch.len_utf8();

            while let Some((i, c)) = self.iter.peek()
                && (*c != self.esc && !self.delims.contains(c))
            {
                jump = *i + c.len_utf8();
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
    use crate::StrTools;

    macro_rules! test_impl {
        ($split:expr; $from:literal => [$($to:literal),+]) => {
            assert_eq!(
                $from
                    .split_non_escaped('\\', &$split)
                    .expect("delim and escape are not the same")
                    .collect::<Vec<_>>(),
                vec![$($to),+]
            )
        };
        ($from:literal => [$($to:literal),+]) => {
            assert_eq!(
                $from
                    .split_non_escaped('\\', &[':'])
                    .expect("delim and escape are not the same")
                    .collect::<Vec<_>>(),
                vec![$($to),+]
            )
        };
    }

    #[test]
    fn empty() {
        non_escaped("", '\\', &[':']).expect("delim and escape are not the same");
    }

    #[test]
    fn delim_is_escape() {
        non_escaped("", '\\', &['\\']).expect_err("delim and escape are the same");
    }

    #[test]
    fn no_escape() {
        test_impl!(r"aaaaa:bbbbb" => ["aaaaa", "bbbbb"]);
    }

    #[test]
    fn multiple() {
        test_impl!(['/', ':']; r"aaaaa/bbb:bb" => ["aaaaa", "bbb", "bb"]);
        test_impl!(['/', ':']; r"aaaaa/bbb\:bb" => ["aaaaa", "bbb:bb"]);
        test_impl!(['/', ':']; r"aaaaa\/bbb\:bb" => ["aaaaa/bbb:bb"]);
    }

    #[test]
    fn single_escape() {
        test_impl!(r"aa\:aa:bbbb" => ["aa:aa", "bbbb"]);
        test_impl!(r"\:aaaa:bbbb" => [":aaaa", "bbbb"]);
        test_impl!(r"aaaa\::bbbb" => ["aaaa:", "bbbb"]);
        test_impl!(r"aaaa:bb\:bb" => ["aaaa", "bb:bb"]);
        test_impl!(r"aaaa:\:bbbb" => ["aaaa", ":bbbb"]);
        test_impl!(r"aaaa:bbbb\:" => ["aaaa", "bbbb:"]);
    }

    #[test]
    fn double_escapes() {
        test_impl!(r"aaaa\\:bbbb" => [r"aaaa\", "bbbb"]);
        test_impl!(r"aaaa\\\:bbbb" => [r"aaaa\:bbbb"]);
        test_impl!(r"aaaa\\\\:bbbb" => [r"aaaa\\", "bbbb"]);
        test_impl!(r"aaaa\\\\\:bbbb" => [r"aaaa\\:bbbb"]);
    }

    #[test]
    fn ignore_other_escapes() {
        test_impl!(r"aa\.aa:bbbbb" => [r"aa\.aa", "bbbbb"]);
        test_impl!(r"\.aaaa:bbbbb" => [r"\.aaaa", "bbbbb"]);
        test_impl!(r"aaaa\.:bbbbb" => [r"aaaa\.", "bbbbb"]);
        test_impl!(r"aaaa:\.bbbbb" => ["aaaa", r"\.bbbbb"]);
        test_impl!(r"aaaa:bbbbb\." => ["aaaa", r"bbbbb\."]);
    }

    #[test]
    fn copy_on_sanitize() {
        // only copy when sanitizing an escape
        let res = r"a\:aa:bbb:cc\.c:ddd"
            .split_non_escaped('\\', &[':'])
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

    mod field_tests {
        use super::*;

        #[test]
        fn trailing_ignored_escape() {
            // the trailing escape caused the split to not include the last char
            test_impl!(['/']; r"test\d" => [r"test\d"]);
        }

        #[test]
        fn escaped() {
            // the escape was not correctly removed
            test_impl!(['/']; r"^b\/(.*)$/d\/$1" => [r"^b/(.*)$", "d/$1"]);
        }

        #[test]
        fn ignored_escape_offset() {
            // multiple subsequent to-be-ignored escape sequences were not properly being split
            // and resulted in more parts than expected as well as missing chars
            test_impl!(['/']; r".*s(\d\d)e(\d\d[a-d])/S$1E$2" => [r".*s(\d\d)e(\d\d[a-d])", "S$1E$2"]);
        }
    }
}
