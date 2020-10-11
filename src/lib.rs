//! BtBencode is a library which can help with [Bencode][wikipedia_bencode]
//! encoding/decoding.  Bencode is primarily used in [BitTorrent][bep_0003] related
//! applications.
//!
//! It uses the [Serde][serde] library to serialize and deserialize Bencode data.
//!
//! # Examples
//!
//! An example serializing from a standard Rust collection type into a custom type:
//!
//! ```
//! # use bt_bencode::Error;
//! # use std::collections::BTreeMap;
//! #
//! # fn main() -> Result<(), Error> {
//! use serde_bytes::ByteBuf;
//! use serde_derive::Deserialize;
//!
//! let mut dict: BTreeMap<String, String> = BTreeMap::new();
//! dict.insert(String::from("url"), String::from("https://example.com/"));
//!
//! let serialized_bytes = bt_bencode::to_vec(&dict)?;
//!
//! #[derive(Deserialize)]
//! struct Info {
//!     url: String,
//! }
//!
//! let info: Info = bt_bencode::from_slice(&serialized_bytes)?;
//! assert_eq!(info.url, "https://example.com/");
//! #   Ok(())
//! # }
//! ```
//!
//! An example deserializing from an unknown slice of bytes, and then into a custom type.
//!
//! ```
//! # use bt_bencode::Error;
//! # use std::collections::BTreeMap;
//! #
//! # fn main() -> Result<(), Error> {
//! use bt_bencode::Value;
//! use serde_bytes::ByteBuf;
//! use serde_derive::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct Info {
//!     t: String,
//!     url: String,
//! }
//!
//! let serialized_bytes = bt_bencode::to_vec(&Info {
//!     t: String::from("query"),
//!     url: String::from("https://example.com/"),
//! })?;
//!
//! let value: Value = bt_bencode::from_slice(&serialized_bytes)?;
//! assert_eq!(value["t"].as_str().ok_or(Error::UnsupportedType)?, "query");
//!
//! let info: Info = bt_bencode::from_value(value)?;
//! assert_eq!(info.url, "https://example.com/");
//! #   Ok(())
//! # }
//! ```
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
pub mod read;
#[cfg(feature = "std")]
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
pub use ser::{to_vec, to_writer, Serializer};

#[doc(inline)]
#[cfg(feature = "std")]
pub use de::from_reader;
