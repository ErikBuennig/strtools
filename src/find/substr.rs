use indexmap::{map::Entry, IndexMap};
use std::{num::NonZeroUsize, ops::Range};

/// Finds the longest range in `input` such that each char in this range is unique, if there are
/// multiple unique ranges of the same length, then first one is returned.
///
/// # Invariants
/// If max is given, then the length of a range cannot be greater than max.
/// ```text
/// max == Some(max) => range.len() <= max
/// ```
///
/// If there are chars after this range, then the first char after this range is equal to the first
/// inside the range.
/// ```text
/// range.end != input.len() => input[range.start] == input[range.end]
/// ```
/// (assuming `str[usize]` would return char starting at byte index)
///
/// # Complexity
/// This algorithm requires `O(n)` time, ignoring memmoves when draining the indexmap.
///
/// # Allocation
/// An [`IndexMap`] is allocated to keep track of unique chars, the map should take up at most `n`
/// amount of space where `n` is the next allocation size that can fit all chars of the input.
/// An IndexMap is used as opposed to a [`HashMap`][hm] because it preserves ordering by value which
/// allows for draining without iterating all elements using [`HashMap::retain`][hmr].
///
/// [hm]: std::collections::HashMap
/// [hmr]: std::collections::HashMap::retain
///
/// # Examples
/// ```
/// use strtools::find;
///
/// //               v----------------------v longest substr due to '_' occurring twice
/// let input = "abc_defgh_ijklmnopqrstuvwxyz";
/// let range = find::longest_unique_substr(input, None);
///
/// assert_eq!(&input[range], "defgh_ijklmnopqrstuvwxyz");
/// ```
pub fn longest_unique_substr(input: &str, max: Option<NonZeroUsize>) -> Range<usize> {
    let mut seen = IndexMap::new();
    let mut current = 0..0;
    let mut longest = 0..0;

    // Consider a string that is unique apart from 2 occurrence of 'c' like so:
    // "abcdefghcijklmnopqrstuvwxyz" // the input string
    //  ^------^                     // longest range until duplicate 'c'
    //     ^----^                    // the overlap that is retained after encountering 'c'
    //     ^----------------------^  // desired longest range
    for (idx, char) in input.char_indices() {
        // yield current if the next would exceed the max
        if let Some(max) = max && (current.start..idx + char.len_utf8()).len() > max.get() {
            return current;
        }

        match seen.entry(char) {
            Entry::Occupied(mut occupied) => {
                if current.len() > longest.len() {
                    longest = current.clone();
                }

                let prev = occupied.get_mut();

                // set current to start past prev idx
                current.start = *prev + char.len_utf8();

                // last occurrence of dupe is now here after draining the map
                *prev = idx;

                // the range to remove from the index map (the chars are added in the order they
                // occur)
                let range = ..occupied.index();
                seen.drain(range);
            }
            Entry::Vacant(vacant) => {
                vacant.insert(idx);
            }
        }

        // exclusive range, dupe or not this will go to at least until here
        current.end = idx + char.len_utf8();
    }

    // the longest can never exceed max as it is set after checking for exceeding
    if let Some(max) = max && longest.len() == max.get() {
        return longest;
    }

    // current cannot be longer than max here, but it may be longer than longest
    if current.len() > longest.len() {
        return current;
    }

    longest
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_impl {
        ($test:ident : $input:literal, $max:expr => $expected:literal $range:expr) => {
            #[test]
            fn $test() {
                let range =
                    longest_unique_substr($input, $max.map(|zero: usize| zero.try_into().unwrap()));
                assert_eq!(range, $range);
                assert_eq!(&$input[range], $expected);
            }
        };
    }

    test_impl!(empty: "", None => "" 0..0);
    test_impl!(simple_starting: "abcdeeeeeeeee", None => "abcde" 0..5);
    test_impl!(single_repeating: "aaaaaaaaa", None => "a" 0..1);
    test_impl!(full: "abcdefghijklmnopqrstuvwxyz", None => "abcdefghijklmnopqrstuvwxyz" 0..26);
    test_impl!(repeating_first: "abcdeabcde", None => "abcde" 0..5);
    test_impl!(overlapping_current_longest: "abcdeafghijkl", None => "bcdeafghijkl" 1..13);
    test_impl!(max_reached: "abcdefghijklmnopqrstuvwxyz", Some(6) => "abcdef" 0..6);
    test_impl!(max_reached_end: "aaaaabcdef", Some(6) => "abcdef" 4..10);
    test_impl!(max_not_exceeded: "abcdeöfghijkl", Some(6) => "abcde" 0..5);
    test_impl!(max_not_exceeded_end: "aaaaabcdeö", Some(6) => "abcde" 4..9);
}
