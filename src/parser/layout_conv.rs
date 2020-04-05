//! Layout conversion utilities
//!
//!
//! The purpose of this module is to convert a layout (indentation) sensitive stream of tokens
//! into an insensitive one, by adding brackets ( `{`, `}`) and semicolons around the content of functions and where blocks
//! A simple example:
//! ```haskell
//! add x y = z
//!         where z = x + y
//! ```
//! should become
//! ```haskell
//! add x y = {
//!     z;
//!     where {
//!         z = {x + y};
//!     }
//! }
//! ```
