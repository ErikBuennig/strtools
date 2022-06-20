use crate::{split, util::Sorted};
use std::borrow::Cow;

/// Escapes all chars in `charset` and the `escape` itself inside `input`. The `charset` parameter
/// must be a reference to a [`Sorted`] slice of chars.
///
/// # Complexity
/// This algorithm requires `O(n * log m)` time where `n` is the length of the input string and `m`
/// is the length of the charset.
///
/// # Allocation
/// No allocations are done.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use strtools::{escape, util::Sorted};
///
/// let sorted: &Sorted<char> = ['a', 'e'][..].try_into()?;
/// let escaped = escape::charset("abcdefg", '\\', sorted);
/// assert_eq!(escaped, r"\abcd\efg");
/// # Ok(())
/// # }
/// ```
pub fn charset<'s>(input: &'s str, escape: char, charset: &Sorted<char>) -> Cow<'s, str> {
    let mut rest = input;
    let mut result = Cow::Borrowed("");

    while let Some(idx) = rest.find(|ch| ch == escape || charset.binary_search(&ch).is_ok()) {
        // SAFETY: str::find on rest must give a valid byte offset to a char in rest
        let (head, ch, tail) = unsafe { split::char_boundary_unchecked(rest, idx) };
        let mutate = result.to_mut();
        mutate.push_str(head);
        mutate.push(escape);
        mutate.push(ch);
        rest = tail;
    }

    match result {
        Cow::Borrowed(_) => Cow::Borrowed(rest),
        Cow::Owned(mut owned) => {
            owned.push_str(rest);
            Cow::Owned(owned)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes() {
        assert_eq!(
            charset("injection!'", '\\', ['\''][..].try_into().unwrap()),
            r"injection!\'"
        );
    }

    #[test]
    fn escape() {
        assert_eq!(
            // if only the charset would be escaped then this would create `... \\' ...` which
            // would not be safe for if whatever is using the output interprets `\\` as `\`, the
            // following `'` would be unescaped again
            charset(r"bypass escaping\'", '\\', ['\''][..].try_into().unwrap()),
            r"bypass escaping\\\'"
        );
    }
}
