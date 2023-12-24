//! Byte string which helps with the deserialization.

use core::{
    borrow::{Borrow, BorrowMut},
    cmp, fmt,
    ops::{Deref, DerefMut},
};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

use serde::{
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer,
};

/// A sequence of bytes like a `Vec<u8>`.
///
/// Bencoded "strings" are not necessarily UTF-8 encoded values so if a field is
/// not guranteed to be a UTF-8 string, then you should use a `ByteString` or
/// another equivalent type.
///
/// Ideally, if you knew a field was a bencoded "string", then you could use
/// `Vec<u8>` or `&[u8]` to represent the field without having to use a wrapper
/// like `ByteString` (which is just a newtype around `Vec<u8>`). However, due
/// to a limitation within `serde` and Rust, a `Vec<u8>` and `&[u8]` will
/// serialize and deserialize as a list of individual byte elements.
///
/// The `serde_bytes` crate can overcome this limitation. `serde_bytes` is still
/// pre-1.0 at the time of this writing, so a specific type within this crate
/// exists.
///
/// # Examples
///
/// ```rust
/// use bt_bencode::ByteString;
///
/// let bstr = ByteString::from("hello");
/// assert_eq!(bstr.as_slice(), b"hello");
/// assert_eq!(&*bstr, b"hello");
/// assert_eq!(bstr, ByteString::from(String::from("hello")));
///
/// let expected: Vec<u8> = b"hello".to_vec();
/// assert_eq!(*&*bstr, expected);
/// assert_eq!(bstr, expected.into());
///
/// let encoded = bt_bencode::to_vec(&bstr)?;
/// assert_eq!(encoded, b"5:hello");
///
/// let decoded: ByteString = bt_bencode::from_slice(&encoded)?;
/// assert_eq!(decoded.as_slice(), b"hello");
///
/// # Ok::<(), bt_bencode::Error>(())
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteString(Vec<u8>);

impl AsRef<[u8]> for ByteString {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsMut<[u8]> for ByteString {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl Borrow<[u8]> for ByteString {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

impl BorrowMut<[u8]> for ByteString {
    fn borrow_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl fmt::Debug for ByteString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl Deref for ByteString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ByteString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> From<&'a [u8]> for ByteString {
    fn from(value: &'a [u8]) -> Self {
        Self(Vec::from(value))
    }
}

impl<'a> From<&'a str> for ByteString {
    fn from(value: &'a str) -> Self {
        Self(Vec::from(value))
    }
}

impl From<String> for ByteString {
    fn from(value: String) -> Self {
        Self(Vec::from(value))
    }
}

impl From<Vec<u8>> for ByteString {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl serde::Serialize for ByteString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

struct BStringVisitor;

impl<'de> Visitor<'de> for BStringVisitor {
    type Value = ByteString;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("byte string")
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let capacity = cmp::min(visitor.size_hint().unwrap_or_default(), 4096);
        let mut bytes = Vec::with_capacity(capacity);

        while let Some(b) = visitor.next_element()? {
            bytes.push(b);
        }

        Ok(ByteString::from(bytes))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(ByteString::from(v))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ByteString::from(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ByteString::from(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ByteString::from(v))
    }
}

impl<'de> Deserialize<'de> for ByteString {
    fn deserialize<D>(deserializer: D) -> Result<ByteString, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_byte_buf(BStringVisitor)
    }
}

impl ByteString {
    /// Returns the inner vector.
    #[inline]
    #[must_use]
    pub fn into_vec(self) -> Vec<u8> {
        self.0
    }
}
