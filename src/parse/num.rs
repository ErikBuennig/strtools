use crate::{
    parse::{FromStrBack, FromStrFront},
    util,
};
use std::fmt::Debug;

/// An [`Error`][0] for [`FromStrFront`]/[`FromStrBack`] implementations of integers.
///
/// [0]: std::error::Error
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum ParseIntPartialError {
    /// The parsed integer value did not fit into the integer type.
    #[error("the given integer representation would cause overflow")]
    Overflow,

    /// The parsed integer value did not fit into the integer type. Never occurs for unsigned types.
    #[error("the given integer representation would cause underflow")]
    Underflow,

    /// The input was either empty or had a prefix or sign followed by invalid input.
    #[error(
        "invalid input, expected: `['+' | '-']? ['0' - '9']+` for signed or `['0' - '9']+` for unsigned"
    )]
    Insufficient,
}

/// An extension for all integers that adds `from_str_radix` equivalents of the [`FromStrFront`] &
/// [`FromStrBack`] functions, see it's documentation for more info.
pub trait FromStrPartialRadixExt: util::sealed::Sealed + FromStrFront + FromStrBack {
    /// Behaves like [`FromStrFront::from_str_front`] for the given radix.
    #[allow(clippy::missing_errors_doc)]
    fn from_str_radix_front(
        input: &str,
        radix: u32,
    ) -> Result<(Self, &str), <Self as FromStrFront>::Error>;

    /// Behaves like [`FromStrBack::from_str_back`] for the given radix.
    #[allow(clippy::missing_errors_doc)]
    fn from_str_radix_back(
        input: &str,
        radix: u32,
    ) -> Result<(Self, &str), <Self as FromStrBack>::Error>;
}

// Most of the implementations details match those form `std::str::FromStr` for integers with the
// difference that invalid chars don't cause an error on parsing but a yield what was parsed until
// then.

trait FromStrRadixHelper: Copy {
    type Error;

    const ERROR_INSUFFICIENT: Self::Error;
    const ERROR_OVERFLOW: Self::Error;
    const ERROR_UNDERFLOW: Self::Error;

    const IS_SIGNED: bool;
    const ZERO: Self;

    fn checked_neg(self) -> Option<Self>;
    fn checked_mul(self, other: u32) -> Option<Self>;
    fn checked_sub(self, other: u32) -> Option<Self>;
    fn checked_add(self, other: u32) -> Option<Self>;
}

fn from_str_radix_front<T: FromStrRadixHelper>(
    input: &str,
    radix: u32,
) -> Result<(T, &str), T::Error> {
    assert!(
        matches!(radix, 2..=36),
        "radix must be in `[2, 36]` - found {}",
        radix
    );

    let (is_neg, rest) = match input.as_bytes() {
        [b'-', ..] => {
            if T::IS_SIGNED {
                (true, &input[1..])
            } else {
                return Err(T::ERROR_INSUFFICIENT);
            }
        }
        [b'+', ..] => (false, &input[1..]),
        _ => (false, input),
    };

    if rest.is_empty() {
        return Err(T::ERROR_INSUFFICIENT);
    }

    let iter = rest
        .as_bytes()
        .iter()
        .enumerate()
        .map(|(idx, &byte)| (idx, (byte as char).to_digit(radix)));

    let mut num = false;
    let mut buf = T::ZERO;
    let mut rest_start = 0;
    if is_neg {
        for (idx, maybe_digit) in iter {
            let sub = match maybe_digit {
                Some(val) => {
                    rest_start = idx + 1;
                    val
                }
                None => {
                    rest_start = idx;
                    break;
                }
            };

            num = true;
            buf = buf.checked_mul(radix).ok_or(T::ERROR_UNDERFLOW)?;
            buf = buf.checked_sub(sub).ok_or(T::ERROR_UNDERFLOW)?;
        }
    } else {
        for (idx, maybe_digit) in iter {
            let add = match maybe_digit {
                Some(val) => {
                    rest_start = idx + 1;
                    val
                }
                None => {
                    rest_start = idx;
                    break;
                }
            };

            num = true;
            buf = buf.checked_mul(radix).ok_or(T::ERROR_OVERFLOW)?;
            buf = buf.checked_add(add).ok_or(T::ERROR_OVERFLOW)?;
        }
    }

    if num {
        Ok((buf, &rest[rest_start..]))
    } else {
        Err(T::ERROR_INSUFFICIENT)
    }
}

fn from_str_radix_back<T: FromStrRadixHelper>(
    input: &str,
    radix: u32,
) -> Result<(T, &str), T::Error> {
    assert!(
        matches!(radix, 2..=36),
        "radix must be in `[2, 36]` - found {}",
        radix
    );

    if input.is_empty() {
        return Err(T::ERROR_INSUFFICIENT);
    }

    let mut num = false;
    let mut buf = T::ZERO;
    let mut len = 0;
    let mut factor = Some(1);
    let iter = input.as_bytes().iter().rev();

    if T::IS_SIGNED {
        let mut is_neg = false;
        for &byte in iter {
            let sub = match (byte as char).to_digit(radix) {
                Some(val) => val,
                None => {
                    match byte {
                        b'-' => {
                            len += 1;
                            is_neg = true;
                        }
                        b'+' => len += 1,
                        _ => {}
                    }

                    break;
                }
            };

            len += 1;
            num = true;

            let fac = factor.ok_or(T::ERROR_UNDERFLOW)?;
            buf = fac
                .checked_mul(sub)
                .and_then(|s| buf.checked_sub(s))
                .ok_or(T::ERROR_UNDERFLOW)?;
            factor = fac.checked_mul(radix);
        }

        // we're using a neg buffer to fit the lower most value if it occurs, then if it's not neg
        // we invert it returning none if it's too large to fit the positive equivalent
        // allows parsing `-128..127` for u8
        if !is_neg {
            buf = buf.checked_neg().ok_or(T::ERROR_OVERFLOW)?;
        }
    } else {
        for &byte in iter {
            let add = match (byte as char).to_digit(radix) {
                Some(val) => val,
                None => {
                    if byte == b'+' {
                        len += 1;
                    }

                    break;
                }
            };

            len += 1;
            num = true;

            let fac = factor.ok_or(T::ERROR_OVERFLOW)?;
            buf = fac
                .checked_mul(add)
                .and_then(|a| buf.checked_add(a))
                .ok_or(T::ERROR_OVERFLOW)?;

            // return error next time
            factor = fac.checked_mul(radix);
        }
    }

    if num {
        Ok((buf, &input[..input.len() - len]))
    } else {
        Err(T::ERROR_INSUFFICIENT)
    }
}

// currently we wouldn't be able to parse `-2^size` because it would overflow before being flipped
// parse as negative and then flip checking for overflow?
macro_rules! int_impl {
    (int $int:ty) => {
        impl FromStrRadixHelper for $int {
            type Error = ParseIntPartialError;

            const ERROR_INSUFFICIENT: Self::Error = ParseIntPartialError::Insufficient;
            const ERROR_OVERFLOW: Self::Error = ParseIntPartialError::Overflow;
            const ERROR_UNDERFLOW: Self::Error = ParseIntPartialError::Underflow;

            const IS_SIGNED: bool = true;
            const ZERO: Self = 0;

            #[inline]
            fn checked_neg(self) -> Option<Self> {
                self.checked_neg()
            }

            #[inline]
            fn checked_mul(self, other: u32) -> Option<Self> {
                Self::checked_mul(self, other as Self)
            }

            #[inline]
            fn checked_sub(self, other: u32) -> Option<Self> {
                Self::checked_sub(self, other as Self)
            }

            #[inline]
            fn checked_add(self, other: u32) -> Option<Self> {
                Self::checked_add(self, other as Self)
            }
        }

        int_impl!($int);
    };
    (uint $int:ty) => {
        impl FromStrRadixHelper for $int {
            type Error = ParseIntPartialError;

            const ERROR_INSUFFICIENT: Self::Error = ParseIntPartialError::Insufficient;
            const ERROR_OVERFLOW: Self::Error = ParseIntPartialError::Overflow;
            const ERROR_UNDERFLOW: Self::Error = ParseIntPartialError::Overflow;

            const IS_SIGNED: bool = false;
            const ZERO: Self = 0;

            #[inline]
            fn checked_neg(self) -> Option<Self> {
                Some(self)
            }

            #[inline]
            fn checked_mul(self, other: u32) -> Option<Self> {
                Self::checked_mul(self, other as Self)
            }

            #[inline]
            fn checked_sub(self, other: u32) -> Option<Self> {
                Self::checked_sub(self, other as Self)
            }

            #[inline]
            fn checked_add(self, other: u32) -> Option<Self> {
                Self::checked_add(self, other as Self)
            }
        }

        int_impl!($int);
    };
    ($int:ty) => {
        impl FromStrFront for $int {
            type Error = ParseIntPartialError;

            fn from_str_front(input: &str) -> Result<(Self, &str), Self::Error> {
                Self::from_str_radix_front(input, 10)
            }
        }

        impl FromStrBack for $int {
            type Error = ParseIntPartialError;

            fn from_str_back(input: &str) -> Result<(Self, &str), Self::Error> {
                Self::from_str_radix_back(input, 10)
            }
        }

        impl FromStrPartialRadixExt for $int {
            fn from_str_radix_front(
                input: &str,
                radix: u32,
            ) -> Result<(Self, &str), <Self as FromStrFront>::Error> {
                from_str_radix_front(input, radix)
            }

            fn from_str_radix_back(
                input: &str,
                radix: u32,
            ) -> Result<(Self, &str), <Self as FromStrBack>::Error> {
                from_str_radix_back(input, radix)
            }
        }
    };
}

int_impl!(int i8);
int_impl!(int i16);
int_impl!(int i32);
int_impl!(int i64);
int_impl!(int i128);
int_impl!(int isize);

int_impl!(uint u8);
int_impl!(uint u16);
int_impl!(uint u32);
int_impl!(uint u64);
int_impl!(uint u128);
int_impl!(uint usize);

#[cfg(test)]
mod tests {
    use super::*;

    mod front {
        use super::*;

        #[test]
        fn invalid_prefix() {
            assert_eq!(
                u8::from_str_radix_front("!!!", 10),
                Err(ParseIntPartialError::Insufficient)
            );
            assert_eq!(
                i8::from_str_radix_front("-!!!", 10),
                Err(ParseIntPartialError::Insufficient)
            );
        }

        #[test]
        fn valid() {
            assert_eq!(u8::from_str_radix_front("255", 10), Ok((255, "")));
            assert_eq!(u8::from_str_radix_front("255!!!", 10), Ok((255, "!!!")));
            assert_eq!(i8::from_str_radix_front("-128", 10), Ok((-128, "")));
            assert_eq!(i8::from_str_radix_front("127!!!", 10), Ok((127, "!!!")));
        }

        #[test]
        fn over_under_flow() {
            assert_eq!(
                u8::from_str_radix_front("2550", 10),
                Err(ParseIntPartialError::Overflow)
            );
            assert_eq!(
                u8::from_str_radix_front("256", 10),
                Err(ParseIntPartialError::Overflow)
            );
            assert_eq!(
                i8::from_str_radix_front("-129", 10),
                Err(ParseIntPartialError::Underflow)
            );
            assert_eq!(
                i8::from_str_radix_front("128", 10),
                Err(ParseIntPartialError::Overflow)
            );
        }
    }

    mod back {
        use super::*;

        #[test]
        fn invalid_suffix() {
            assert_eq!(
                u8::from_str_radix_back("!!!", 10),
                Err(ParseIntPartialError::Insufficient)
            );
            assert_eq!(
                i8::from_str_radix_back("!!!-", 10),
                Err(ParseIntPartialError::Insufficient)
            );
        }

        #[test]
        fn valid() {
            assert_eq!(u8::from_str_radix_back("255", 10), Ok((255, "")));
            assert_eq!(u8::from_str_radix_back("!!!255", 10), Ok((255, "!!!")));
            assert_eq!(i8::from_str_radix_back("-128", 10), Ok((-128, "")));
            assert_eq!(i8::from_str_radix_back("!!!127", 10), Ok((127, "!!!")));
        }

        #[test]
        fn over_under_flow() {
            assert_eq!(
                u8::from_str_radix_back("2550", 10),
                Err(ParseIntPartialError::Overflow)
            );
            assert_eq!(
                u8::from_str_radix_back("256", 10),
                Err(ParseIntPartialError::Overflow)
            );
            assert_eq!(
                i8::from_str_radix_back("-129", 10),
                Err(ParseIntPartialError::Underflow)
            );
            assert_eq!(
                i8::from_str_radix_back("128", 10),
                Err(ParseIntPartialError::Overflow)
            );
        }
    }
}
