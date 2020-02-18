//! BtBencode is a library which can help with [Bencode][wikipedia_bencode]
//! encoding/decoding.  Bencode is primarily used in [BitTorrent][bep_0003] related
//! applications.
//!
//! It provides a [Serde][serde] serializer and deserializer.
//!
//! [wikipedia_bencode]: https://en.wikipedia.org/wiki/Bencode
//! [bep_0003]: http://www.bittorrent.org/beps/bep_0003.html
//! [serde]: https://serde.rs

#[macro_use]
extern crate serde;

mod de;
mod error;
mod read;
mod ser;

pub use de::{from_reader, from_slice, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_vec, to_writer, Serializer};
