use crate::error::Error;

use core::fmt::{self, Debug};
use serde::de::{self, Deserialize, DeserializeSeed, Deserializer, MapAccess, Unexpected, Visitor};
use serde::forward_to_deserialize_any;
use serde::ser::{SerializeStruct, Serializer};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{borrow::ToOwned, vec::Vec};
#[cfg(feature = "std")]
use std::vec::Vec;

/// Reference to a range of bytes encompassing a single valid BENCODE value in the
/// input data.
///
/// A `RawValue` can be used to defer parsing parts of a payload until later,
/// or to avoid parsing it at all in the case that part of the payload just
/// needs to be transferred verbatim into a different output object.
///
/// When serializing, a value of this type will retain its original formatting
/// and will not be minified or pretty-printed.
#[cfg_attr(docsrs, doc(cfg(feature = "raw_value")))]
#[repr(transparent)]
pub struct RawValue {
    bencode: Vec<u8>,
}

/// The private token used to identify RawValue in serde's struct protocol.
///
/// Serializers and deserializers that encounter a struct named with this token
/// activate raw passthrough: the bytes are written/read verbatim instead of
/// being parsed as a structured bencode value.
pub const TOKEN: &str = "$bt_bencode::private::RawValue";

impl RawValue {
    /// Constructs a [`RawValue`] by copying the given bencode-encoded byte slice.
    ///
    /// The caller is responsible for ensuring `s` contains a single, complete,
    /// valid bencode value.  No validation is performed.
    pub fn from_slice(s: &[u8]) -> Self {
        RawValue {
            bencode: s.to_owned(),
        }
    }

    /// Constructs a [`RawValue`] by taking ownership of the given bencode-encoded [`Vec<u8>`].
    ///
    /// The caller is responsible for ensuring `s` contains a single, complete,
    /// valid bencode value.  No validation is performed.
    pub fn from_vec(s: Vec<u8>) -> Self {
        RawValue { bencode: s }
    }

    /// Returns the raw bencode bytes held by this value.
    pub fn get(&self) -> &[u8] {
        &self.bencode
    }
}

impl Debug for RawValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("RawValue")
            .field(&format_args!("{:0x?}", &self.bencode))
            .finish()
    }
}

impl serde::Serialize for RawValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct(TOKEN, 1)?;
        s.serialize_field(TOKEN, &RawBytes(&self.bencode))?;
        s.end()
    }
}

struct RawBytes<'a>(&'a [u8]);

impl serde::Serialize for RawBytes<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.0)
    }
}

impl<'de> Deserialize<'de> for RawValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RawVisitor;

        impl<'de> Visitor<'de> for RawVisitor {
            type Value = RawValue;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "any valid BENCODE value")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let value = visitor.next_key::<RawKey>()?;
                if value.is_none() {
                    return Err(de::Error::invalid_type(Unexpected::Map, &self));
                }
                visitor.next_value_seed(RawFromSlice)
            }
        }

        deserializer.deserialize_newtype_struct(TOKEN, RawVisitor)
    }
}

struct RawKey;

impl<'de> Deserialize<'de> for RawKey {
    fn deserialize<D>(deserializer: D) -> Result<RawKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("raw value")
            }

            fn visit_str<E>(self, s: &str) -> Result<(), E>
            where
                E: de::Error,
            {
                if s == TOKEN {
                    Ok(())
                } else {
                    Err(de::Error::custom("unexpected raw value"))
                }
            }
        }

        deserializer.deserialize_identifier(FieldVisitor)?;
        Ok(RawKey)
    }
}

/// A [`DeserializeSeed`] / [`Visitor`] that reconstructs a [`RawValue`] from
/// a sequence of raw bytes delivered by [`OwnedRawDeserializer`].
pub struct RawFromSlice;

impl<'de> DeserializeSeed<'de> for RawFromSlice {
    type Value = RawValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(self)
    }
}

impl<'de> Visitor<'de> for RawFromSlice {
    type Value = RawValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("raw value")
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(RawValue::from_vec(v))
    }
}

struct RawKeyDeserializer;

impl<'de> Deserializer<'de> for RawKeyDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(TOKEN)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 char str string seq
        bytes byte_buf map struct option unit newtype_struct ignored_any
        unit_struct tuple_struct tuple enum identifier
    }
}

struct RawValueDeserializer(Vec<u8>);

impl<'de> Deserializer<'de> for RawValueDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_byte_buf(self.0)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64 char str string seq
        bytes byte_buf map struct option unit newtype_struct ignored_any
        unit_struct tuple_struct tuple enum identifier
    }
}

/// A serde [`MapAccess`] that presents a single key/value entry whose key is
/// the private [`TOKEN`] and whose value is the raw bencode byte buffer.
pub struct OwnedRawDeserializer {
    /// The raw bencode bytes.  Set to `Some` initially; taken on the first
    /// [`MapAccess::next_value_seed`] call.
    pub raw_value: Option<Vec<u8>>,
}

impl<'de> MapAccess<'de> for OwnedRawDeserializer {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: DeserializeSeed<'de>,
    {
        if self.raw_value.is_none() {
            return Ok(None);
        }
        seed.deserialize(RawKeyDeserializer).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(RawValueDeserializer(self.raw_value.take().unwrap()))
    }
}
