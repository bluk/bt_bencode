#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unused_lifetimes,
    unused_qualifications
)]

#[cfg(all(feature = "alloc", not(feature = "std")))]
extern crate alloc;

#[macro_use]
extern crate serde;

mod de;
mod error;

pub mod read;
pub mod write;

mod ser;
pub mod value;

#[doc(inline)]
pub use de::{from_slice, Deserializer};
#[doc(inline)]
pub use error::{Error, Result};
#[doc(inline)]
pub use value::{from_value, to_value, Value};

#[doc(inline)]
#[cfg(feature = "std")]
pub use ser::to_writer;

#[doc(inline)]
pub use ser::{to_vec, Serializer};

#[doc(inline)]
#[cfg(feature = "std")]
pub use de::from_reader;
