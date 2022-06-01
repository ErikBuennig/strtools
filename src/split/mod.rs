//! This module contains functions with the primary purpose of splitting [str]s.

mod char_boundary;
pub use char_boundary::*;

mod non_escaped;
pub use non_escaped::*;
