use std::{borrow::Cow, iter::Peekable, mem, str::CharIndices};

/// An error type that indicates that a delimiter and an escape char cannot be the same.
#[derive(Debug)]
pub struct EscapeIsDelimiterError(());

impl std::fmt::Display for EscapeIsDelimiterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a delimiter cannot be it's own escape char")
    }
}

impl std::error::Error for EscapeIsDelimiterError {}

// TODO: after a bit of cleanup this should be doable with less state and double ended

/// An [Iterator] that yields parts of a [str] that are separated by a delimiter.
/// This struct is created by the [`StrTools::split_non_escaped`][self_on_StrTools] method, See
/// it's documentation for more info.
///
/// [self_on_StrTools]: crate::StrTools::split_non_escaped
#[derive(Debug)]
pub struct SplitNonEscaped<'s, 'd> {
    input: &'s str,
    done: usize,
    esc: char,
    delims: &'d [char],
    iter: Peekable<CharIndices<'s>>,
    curr: Option<Cow<'s, str>>,
}

impl<'s, 'd> SplitNonEscaped<'s, 'd> {
    pub(crate) fn new(
        input: &'s str,
        esc: char,
        delims: &'d [char],
    ) -> Result<Self, EscapeIsDelimiterError> {
        if !delims.contains(&esc) {
            Ok(Self {
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
}

impl<'s, 'd> Iterator for SplitNonEscaped<'s, 'd> {
    type Item = Cow<'s, str>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((idx, ch)) = self.iter.next() {
            // escape
            if ch == self.esc && let Some(_) = self.iter.peek() {
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
                let mut yielded = Some(Cow::Borrowed(""));
                mem::swap(&mut yielded, &mut self.curr);
                self.done = idx + ch.len_utf8();
                return yielded;
            }

            // regular char
            let mut jump = idx + ch.len_utf8();

            while let Some((i, c)) = self.iter.peek()
                && (*c != self.esc && !self.delims.contains(c))
            {
                jump = *i + c.len_utf8();
                let _ = self.iter.next();
            }

            if self.curr.as_mut().unwrap().is_borrowed() {
                self.curr = Some(Cow::Borrowed(&self.input[self.done..jump]));
            } else {
                self.curr
                    .as_mut()
                    .unwrap()
                    .to_mut()
                    .push_str(&self.input[self.done..jump]);
            }

            self.done = jump;
        }

        if self.done < self.input.len() {
            if self.curr.as_mut().unwrap().is_borrowed() {
                self.curr = Some(Cow::Borrowed(&self.input[self.done..self.input.len()]));
            } else {
                self.curr
                    .as_mut()
                    .unwrap()
                    .to_mut()
                    .push_str(&self.input[self.done..self.input.len()]);
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
        SplitNonEscaped::new("", '\\', &[':']).expect("delim and escape are not the same");
    }

    #[test]
    fn delim_is_escape() {
        SplitNonEscaped::new("", '\\', &['\\']).expect_err("delim and escape are the same");
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
        // only copy when sanitizing a escape
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
