//! This module contains parsing extensions to the standard library implementations. Notably
//! implementations of [FromStrPartial] which will try to parse as much of a string as it can.

use std::str::FromStr;

// TODO: floats and other notable types

mod num;
pub use num::{FromStrPartialRadixExt, ParseIntPartialError};

/// Types that may try parsing from the beginning of a [`str`]. While [`FromStr`] generally requires
/// the whole input to be a valid representation of `Self`, this trait tries to parse until it
/// encounters unknown input and ignores it. Since it is trivial to [`FromStr`] if [`FromStrFront`]
/// is implemented it is a required super trait, see the [forward_fromstr] macro for forwarding the
/// implementation easily on your own types.
pub trait FromStrFront: Sized + FromStr {
    /// The [`Error`][0] that is returned if parsing fails.
    ///
    /// [0]: std::error::Error
    type Error = <Self as FromStr>::Err;

    /// Attempts to parse `Self` from the beginning of the [str], returns the rest of the `input`
    /// and `Self` if parsing succeeded.
    ///
    /// # Error
    /// Returns an error if:
    /// - the start of `input` contain any valid representation of `Self`
    /// - `input` did not contain a complete representation of `Self`
    ///
    /// # Examples
    /// ```
    /// # use std::{str::FromStr, num::IntErrorKind};
    /// use strtools::parse::FromStrPartial;
    ///
    /// assert_eq!(u8::from_str_front("123;"), Ok((123, ";")));
    /// assert_eq!(u8::from_str("123;").unwrap_err().kind(), &IntErrorKind::InvalidDigit);
    /// ```
    fn from_str_front(input: &str) -> Result<(Self, &str), Self::Error>;

    /// Removes the prefix of the given [`&str`][str] in place if parsing with succeeds. This
    /// ensures that subsequent parsers don't consume the same starting str as this one.
    ///
    /// # Examples
    /// ```
    /// # fn doit() -> Option<()> {
    /// use strtools::parse::FromStrPartial;
    ///
    /// let mut input = "1-2+3-4";
    ///
    /// // the consume function automatically strips off the parts that were already parsed
    /// assert_eq!(u8::yield_front(&mut input), Ok(1));
    /// assert_eq!(i8::yield_front(&mut input), Ok(-2));
    /// assert_eq!(u8::yield_front(&mut input), Ok(3));
    /// assert_eq!(i8::yield_front(&mut input), Ok(-4));
    /// # Some(())
    /// # }
    /// # doit().unwrap();
    /// ```
    #[inline]
    fn yield_front(input: &mut &str) -> Result<Self, Self::Error> {
        let (result, rest) = Self::from_str_front(input)?;
        *input = rest;
        Ok(result)
    }
}

/// Types that may try parsing from the end of a [`str`]. While [FromStr] generally requires the
/// whole input to be a valid representation of [Self], this trait tries to parse until it
/// encounters unknown input and ignores it. Since it is trivial to [`FromStr`] if [`FromStrBack`]
/// is implemented it is a required super trait, see the [forward_fromstr] macro for forwarding the
/// implementation easily on your own types.
pub trait FromStrBack: Sized + FromStr {
    /// The [`Error`][0] that is returned if parsing fails.
    ///
    /// [0]: std::error::Error
    type Error = <Self as FromStr>::Err;

    /// Attempts to parse `Self` from the end of the [`str`], returns the rest of the `input` and
    /// `Self` if parsing succeeded.
    ///
    /// # Error
    /// Returns an error if:
    /// - the start of `input` contain any valid representation of `Self`
    /// - `input` did not contain a complete representation of `Self`
    ///
    /// # Examples
    /// ```
    /// # use std::{str::FromStr, num::IntErrorKind};
    /// use strtools::parse::FromStrPartial;
    ///
    /// assert_eq!(u8::from_str_back(";123"), Ok((123, ";")));
    /// assert_eq!(u8::from_str(";123").unwrap_err().kind(), &IntErrorKind::InvalidDigit);
    /// ```
    fn from_str_back(input: &str) -> Result<(Self, &str), Self::Error>;

    /// Removes the suffix of the given [`&str`][str] in place if parsing succeeds. This ensures
    /// that subsequent parsers don't consume the same starting str as this one and allows for an
    /// imperative style for parsing.
    ///
    /// # Examples
    /// ```
    /// # fn doit() -> Option<()> {
    /// use strtools::parse::FromStrPartial;
    ///
    /// let mut input = "-4+3-2+1";
    ///
    /// // the consume function automatically strips off the parts that were already parsed
    /// assert_eq!(u8::yield_back(&mut input), Ok(1));
    /// assert_eq!(i8::yield_back(&mut input), Ok(-2));
    /// assert_eq!(u8::yield_back(&mut input), Ok(3));
    /// assert_eq!(i8::yield_back(&mut input), Ok(-4));
    /// # Some(())
    /// # }
    /// # doit().unwrap();
    /// ```
    #[inline]
    fn yield_back(input: &mut &str) -> Result<Self, Self::Error> {
        let (result, rest) = Self::from_str_back(input)?;
        *input = rest;
        Ok(result)
    }
}

/// Forwards [`FromStr`] to [`FromStrFront`]/[`FromStrBack`] by checking if all of the input was
/// consumed. The given closure-like argument binds the parsed value and rest [`str`] if
/// `from_str_*` doesn't completely consume the string and succeeded, allowing the implementor to
/// construct a custom error in this special case, or default if not given.
/// ```
/// use std::str::FromStr;
/// use std::default::Default;
///
/// #[derive(Debug)]
/// struct A
/// struct Error(String);
///
/// impl Default for Error {
///     fn default() -> Self {
///         Self("invalid".to_string())
///     }
/// }
///
/// impl FromStrFront for A {
///     type Error = Error;
///
///     fn from_str_front(input: &str) -> Result<(Self, &str), Self::Error> {
///         if let Some(rest) = strip_prefix('a') {
///             Ok((A, rest))
///         } else {
///             Err(Error(format!("invalid: {:?}, expected \"a...\"", input)))
///         }
///     }
/// }
///
/// forward!(front for A; |value, rest| {
///     Error(format!("parsed {:?}, but had unexpected rest: {:?}", parsed, rest))
/// });
/// ```
/// ```no_run
///
/// // create default error if there are tokens left
/// forward!(front for A);
/// ```
/// ```no_run
///
/// // likewise this forwards to the FromStrBack impl
/// forward!(back for A; |value, rest| {
///     Error(format!("parsed {:?}, but had unexpected rest: {:?}", parsed, rest))
/// });
/// ```
/// ```no_run
///
/// // generates this impl:
/// # use crate::parse::FromStrFront;
/// # use std::str::FromStr;
/// # #[derive(Debug)] struct A; struct Error(String);
/// impl FromStr for A {
///     type Err = Error;
///
///     fn from_str(input: &str) -> Result::<Self, Self::Err> {
///         match A::from_str_front(input) {
///             Ok((value, "")) => Ok(value),
///             // the special error case with left over tokens
///             Ok((value, rest)) => Err({
///                 Error(format!("parsed {:?}, but had unexpected rest: {:?}", parsed, rest))
///             }),
///             Err(err) => Err(err),
///         }
///     }
/// }
/// ```
pub macro forward {
    (front for $type:ty) => {
        forward!(front for $type; |_, _| ::std::default::Default::default())
    },
    (back for $type:ty) => {
        forward!(back for $type; |_, _| ::std::default::Default::default())
    },
    (front for $type:ty; |$value:pat_param, $rest:pat_param| $rest_err:expr) => {
        impl ::std::str::FromStr for $type {
            type Err = <Self as $crate::parse::FromStrFront>::Error;

            fn from_str(input: &str) -> ::std::result::Result::<Self, Self::Err> {
                use ::std::result::Result as Result;
                match <Self as $crate::parse::FromStrFront>::from_str_front(input) {
                    Result::Ok((value, "")) => Result::Ok(value),
                    Result::Ok(($value, $rest)) => Result::Err($rest_err),
                    Result::Err(err) => Result::Err(err),
                }
            }
        }
    },
    (back for $type:ty; |$value:pat_param, $rest:pat_param| $rest_err:expr) => {
        impl ::std::str::FromStr for $type {
            type Err = <Self as $crate::parse::FromStrBack>::Error;

            fn from_str(input: &str) -> ::std::result::Result::<Self, Self::Err> {
                use ::std::result::Result as Result;
                match <Self as $crate::parse::FromStrBack>::from_str_back(input) {
                    Result::Ok((value, "")) => Result::Ok(value),
                    Result::Ok(($value, $rest)) => Result::Err($rest_err),
                    Result::Err(err) => Result::Err(err),
                }
            }
        }
    }
}

/// An [`Error`][0] for [`FromStrBack`] on [`bool`]s.
///
/// [0]: std::error::Error
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("invalid input, expected: `'true' | 'false'`")]
pub struct ParseBoolError;

impl FromStrFront for bool {
    type Error = ParseBoolError;

    fn from_str_front(input: &str) -> Result<(Self, &str), Self::Error> {
        if let Some(rest) = input.strip_prefix("true") {
            Ok((true, rest))
        } else if let Some(rest) = input.strip_prefix("false") {
            Ok((false, rest))
        } else {
            Err(ParseBoolError)
        }
    }
}

impl FromStrBack for bool {
    type Error = ParseBoolError;

    fn from_str_back(input: &str) -> Result<(Self, &str), Self::Error> {
        if let Some(rest) = input.strip_suffix("true") {
            Ok((true, rest))
        } else if let Some(rest) = input.strip_suffix("false") {
            Ok((false, rest))
        } else {
            Err(ParseBoolError)
        }
    }
}

/// Returns true if a given `literal` was yielded form the front, behaves similar to
/// [`FromStrFront::from_str_front`].
pub fn yield_literal_front(input: &mut &str, literal: &str) -> bool {
    if let Some(rest) = input.strip_prefix(literal) {
        *input = rest;
        true
    } else {
        false
    }
}

/// Returns true if a given `literal` was yielded form the back, behaves similar to
/// [`FromStrBack::from_str_back`].
pub fn yield_literal_back(input: &mut &str, literal: &str) -> bool {
    if let Some(rest) = input.strip_suffix(literal) {
        *input = rest;
        true
    } else {
        false
    }
}
