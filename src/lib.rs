//! BtBencode is a library which can help with [Bencode][wikipedia_bencode]
//! encoding/decoding.  Bencode is primarily used in [BitTorrent][bep_0003] related
//! applications.
//!
//! It provides a [Serde][serde] serializer and deserializer.
//!
//! [wikipedia_bencode]: https://en.wikipedia.org/wiki/Bencode
//! [bep_0003]: http://www.bittorrent.org/beps/bep_0003.html
//! [serde]: https://serde.rs

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
extern crate serde;

mod de;
mod error;
mod read;
#[cfg(feature = "std")]
mod ser;
mod value;

pub use de::{from_slice, Deserializer};
pub use error::{Error, Result};
pub use value::{from_value, Value};

#[cfg(feature = "std")]
pub use ser::{to_vec, to_writer, Serializer};

#[cfg(feature = "std")]
pub use de::from_reader;
