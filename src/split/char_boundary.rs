use core::{slice, str};

/// An [Error][0] for `char_boundary_*` functions, see thier documentation for more info.
///
/// [0]: std::error::Error
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum CharBoundaryError {
    /// Indicates that a given input was empty.
    #[error("the input must contain at least one char, but was empty")]
    InputEmpty,

    /// Indicates that the given index was out of range of a givne length.
    #[error("the index is {0}, but the length is {1}")]
    IndexOutOfRange(usize, usize),

    /// Indicates that the given index was not on a utf-8 sequence boundary.
    #[error("the index ({0}) was not on a UTF-8 sequence boundary")]
    NotUTF8Boundary(usize),
}

/// Splits `input` into a triple of before, the char at `index` and after like so:
/// ```text
/// [before @ .., char, after @ ..]
/// ```
///
/// # Errors
/// Returns an error if:
/// - `input == ""`, eg.: it contains no char
/// - `index >= input.len()`, eg.: there is no char starting at index
/// - `index` is not on a UTF-8 sequence boundary
///
/// # Examples
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use strtools::split;
/// let input = "aöböc";
///
/// // we know that ö is 2 bytes, so we can only split at 0, 1, 3, 4 and 6
/// let (before, char_at_idx, after) = split::char_boundary(input, 3)?;
/// assert_eq!("aö", before);
/// assert_eq!('b', char_at_idx);
/// assert_eq!("öc", after);
/// # Ok(())
/// # }
/// ```
/// This will return an error:
/// ```
/// # use strtools::split::{self, CharBoundaryError};
/// # let input = "aöböc";
/// #
/// // that's not a sequence boundary
/// let result = split::char_boundary(input, 2);
/// assert_eq!(result, Err(CharBoundaryError::NotUTF8Boundary(2)));
/// ```
pub fn char_boundary(input: &str, index: usize) -> Result<(&str, char, &str), CharBoundaryError> {
    if input.is_empty() {
        Err(CharBoundaryError::InputEmpty)
    } else if index >= input.len() {
        Err(CharBoundaryError::IndexOutOfRange(index, input.len()))
    } else if !input.is_char_boundary(index) {
        Err(CharBoundaryError::NotUTF8Boundary(index))
    } else {
        // SAFETY:
        // - the input is not empty
        // - the index is not greater than or equal to the length
        // - the index is on a UTF-8 sequence boundary
        unsafe { Ok(char_boundary_unchecked(input, index)) }
    }
}

/// Splits `input` into a triple of before, the char at `index` and after like so:
/// ```text
/// [before @ .., char, after @ ..]
/// ```
///
/// # Safety
/// The caller must ensure that:
/// - `input != ""`, eg.: there is at least one char to yield
/// - `index < input.len()`, eg.: the index is not past the last char
/// - `index` is on a UTF-8 sequence boundary
///
/// # Examples
/// ```
/// use strtools::split;
/// let input = "aöböc";
///
/// // we know that ö is 2 bytes, so we can only split at 0, 1, 3, 4 and 6
/// let (before, char_at_idx, after) = unsafe { split::char_boundary_unchecked(input, 3) };
/// assert_eq!("aö", before);
/// assert_eq!('b', char_at_idx);
/// assert_eq!("öc", after);
/// ```
/// Undefined behavior:
/// ```no_run
/// # use strtools::split;
/// # let input = "aöböc";
/// // we're not upholding str and char invariants, this causes undefined behavior
/// let _ = unsafe { split::char_boundary_unchecked(input, 2) };
/// ```
pub unsafe fn char_boundary_unchecked(input: &str, index: usize) -> (&str, char, &str) {
    // SAFETY: teh caller must ensure boundary conditions
    unsafe {
        let char_at = input
            .get_unchecked(index..)
            .chars()
            .next()
            .unwrap_unchecked();

        let before = input.get_unchecked(..index);
        let after = input.get_unchecked(index + char_at.len_utf8()..);

        (before, char_at, after)
    }
}

/// Splits `input` mutably into a triple of before, the char at `index` and after like so:
/// ```text
/// [before @ .., char, after @ ..]
/// ```
///
/// # Errors
/// Returns an error if:
/// - `input == ""`, eg.: it contains no char
/// - `index >= input.len()`, eg.: there is no char starting at index
/// - `index` is not on a UTF-8 sequence boundary
///
/// # Examples
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use strtools::split;
/// let mut input = String::from("aöböc");
///
/// // we know that ö is 2 bytes, so we can only split at 0, 1, 3, 4 and 6
/// let (before, char_at_idx, after) = split::char_boundary_mut(&mut input, 3)?;
/// assert_eq!("aö", before);
/// assert_eq!('b', char_at_idx);
/// assert_eq!("öc", after);
/// # Ok(())
/// # }
/// ```
/// This will return an error:
/// ```
/// # use strtools::split::{self, CharBoundaryError};
/// # let mut input = String::from("aöböc");
/// // that's not a sequence boundary
/// let result = split::char_boundary_mut(&mut input, 2);
/// assert_eq!(result, Err(CharBoundaryError::NotUTF8Boundary(2)));
/// ```
pub fn char_boundary_mut(
    input: &mut str,
    index: usize,
) -> Result<(&mut str, char, &mut str), CharBoundaryError> {
    if input.is_empty() {
        Err(CharBoundaryError::InputEmpty)
    } else if index >= input.len() {
        Err(CharBoundaryError::IndexOutOfRange(index, input.len()))
    } else if !input.is_char_boundary(index) {
        Err(CharBoundaryError::NotUTF8Boundary(index))
    } else {
        // SAFETY:
        // - the input is not empty
        // - the index is not greater than or equal to the length
        // - the index is on a UTF-8 sequence boundary
        unsafe { Ok(char_boundary_mut_unchecked(input, index)) }
    }
}

/// Splits `input` mutably  into a triple of before, the char at `index` and after like so:
/// ```text
/// [before @ .., char, after @ ..]
/// ```
///
/// # Safety
/// The caller must ensure that:
/// - `input != ""`, eg.: there is at least one char to yield
/// - `index < input.len()`, eg.: the index is not past the last char
/// - `index` is on a UTF-8 sequence boundary
///
/// # Examples
/// ```
/// use strtools::split;
/// let mut input = String::from("aöböc");
///
/// // we know that ö is 2 bytes, so we can only split at 0, 1, 3, 4 and 6
/// let (before, char_at_idx, after) = unsafe { split::char_boundary_mut_unchecked(&mut input, 3) };
/// assert_eq!("aö", before);
/// assert_eq!('b', char_at_idx);
/// assert_eq!("öc", after);
/// ```
/// Undefined behavior:
/// ```no_run
/// # use strtools::split;
/// # let mut input = String::from("aöböc");
/// // we're not upholding str and  char invariants, this causes undefined behavior
/// let _ = unsafe { split::char_boundary_mut_unchecked(&mut input, 2) };
/// ```
pub unsafe fn char_boundary_mut_unchecked(
    input: &mut str,
    index: usize,
) -> (&mut str, char, &mut str) {
    let len = input.len();
    let ptr = input.as_mut_ptr();

    // SAFETY:
    // - the caller must ensure boundary conditions
    // - the slices do not overlap
    unsafe {
        let char_at = input
            .get_unchecked_mut(index..)
            .chars()
            .next()
            .unwrap_unchecked();
        let offset_after = index + char_at.len_utf8();

        let before = str::from_utf8_unchecked_mut(slice::from_raw_parts_mut(ptr, index));
        let after = str::from_utf8_unchecked_mut(slice::from_raw_parts_mut(
            ptr.add(offset_after),
            len - offset_after,
        ));

        (before, char_at, after)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert_eq!(char_boundary("", 0), Err(CharBoundaryError::InputEmpty));
    }

    #[test]
    fn out_of_range() {
        assert_eq!(
            char_boundary("a", 1),
            Err(CharBoundaryError::IndexOutOfRange(1, 1))
        );
    }

    #[test]
    fn non_boundary() {
        assert_eq!(
            char_boundary("ö", 1),
            Err(CharBoundaryError::NotUTF8Boundary(1))
        );
    }

    #[test]
    fn empty_mut() {
        let mut input = String::from("");
        assert_eq!(
            char_boundary_mut(&mut input, 0),
            Err(CharBoundaryError::InputEmpty)
        );
    }

    #[test]
    fn out_of_range_mut() {
        let mut input = String::from("a");
        assert_eq!(
            char_boundary_mut(&mut input, 1),
            Err(CharBoundaryError::IndexOutOfRange(1, 1))
        );
    }

    #[test]
    fn non_boundary_mut() {
        let mut input = String::from("ö");
        assert_eq!(
            char_boundary_mut(&mut input, 1),
            Err(CharBoundaryError::NotUTF8Boundary(1))
        );
    }

    #[test]
    fn on_boundary() {
        assert_eq!(char_boundary("ö", 0), Ok(("", 'ö', "")));
        assert_eq!(char_boundary("öb", 0), Ok(("", 'ö', "b")));
        assert_eq!(char_boundary("aö", 1), Ok(("a", 'ö', "")));
        assert_eq!(char_boundary("aöb", 1), Ok(("a", 'ö', "b")));
    }

    #[test]
    fn on_boundary_mut() {
        macro_rules! test {
            ($input:literal; $idx:literal => ($before:literal, $char:literal, $after:literal)) => {
                let mut input = String::from($input);
                let mut before = String::from($before);
                let mut after = String::from($after);
                assert_eq!(
                    char_boundary_mut(&mut input, $idx),
                    Ok((&mut before[..], $char, &mut after[..]))
                );
            };
        }

        test!("ö";   0 => ( "", 'ö',  ""));
        test!("öb";  0 => ( "", 'ö', "b"));
        test!("aö";  1 => ("a", 'ö',  ""));
        test!("aöb"; 1 => ("a", 'ö', "b"));
    }
}
