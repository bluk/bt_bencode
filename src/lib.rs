//! # BtBencode
//!
//! `BtBencode` is a library which can help with [Bencode][wikipedia_bencode]
//! encoding/decoding.  Bencode is primarily used in [BitTorrent][bep_0003] related
//! applications.
//!
//! It uses the [Serde][serde] library to serialize and deserialize Bencode data.
//! It is similar to [Serde JSON][serde_json] in terms of functionality and
//! implementation.
//!
//! ## Documentation
//!
//! * [Latest API Docs][docs_rs_bt_bencode]
//!
//! ## Examples
//!
//! An example serializing a standard Rust collection type and then deserializing
//! into a custom type:
//!
//! ```rust
//! use std::collections::BTreeMap;
//! use serde_derive::Deserialize;
//!
//! let mut dict: BTreeMap<String, String> = BTreeMap::new();
//! dict.insert(String::from("url"), String::from("https://example.com/"));
//!
//! let serialized_bytes = bt_bencode::to_vec(&dict)?;
//!
//! #[derive(Deserialize)]
//! struct Info<'a> {
//!     url: &'a str,
//! }
//!
//! let info: Info = bt_bencode::from_slice(&serialized_bytes)?;
//! assert_eq!(info.url, "https://example.com/");
//! # Ok::<(), bt_bencode::Error>(())
//! ```
//!
//! An example deserializing from a slice of bytes into a general `Value`
//! representation and then from the `Value` instance into a more strongly typed
//! data structure.
//!
//! ```rust
//! use serde_derive::{Serialize, Deserialize};
//!
//! use bt_bencode::Value;
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
//! assert_eq!(value["t"].as_str().unwrap(), "query");
//! assert_eq!(
//!     value.get("url").and_then(|url| url.as_str()).unwrap(),
//!     "https://example.com/"
//! );
//!
//! let info: Info = bt_bencode::from_value(value)?;
//! assert_eq!(info.t, "query");
//! assert_eq!(info.url, "https://example.com/");
//! # Ok::<(), bt_bencode::Error>(())
//! ```
//!
//! ## License
//!
//! Licensed under either of [Apache License, Version 2.0][LICENSE_APACHE] or [MIT
//! License][LICENSE_MIT] at your option.
//!
//! ### Contributions
//!
//! Unless you explicitly state otherwise, any contribution intentionally submitted
//! for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
//! dual licensed as above, without any additional terms or conditions.
//!
//! [LICENSE_APACHE]: LICENSE-APACHE
//! [LICENSE_MIT]: LICENSE-MIT
//! [wikipedia_bencode]: https://en.wikipedia.org/wiki/Bencode
//! [bep_0003]: http://www.bittorrent.org/beps/bep_0003.html
//! [serde]: https://serde.rs
//! [serde_json]: https://github.com/serde-rs/json
//! [docs_rs_bt_bencode]: https://docs.rs/bt_bencode/latest/bt_bencode/

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

#[doc(inline)]
#[cfg(feature = "std")]
pub use de::take_from_reader;
