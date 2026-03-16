//! Deserializes Bencode data.

use crate::error::{Error, ErrorKind, Result};
use crate::read::{self, Read, Ref};
use serde::de::{self, Expected, Unexpected};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::{io, vec::Vec};

/// Deserializes an instance of `T` from the bytes of an [`io::Read`] type.
///
/// The entire [`io::Read`] source is consumed, and it is an error if there is
/// trailing data. If trailing data is expected, then the [`Deserializer`]
/// should be constructed directly. See [`Deserializer::byte_offset()`] for an
/// example.
///
/// # Errors
///
/// Deserialization can fail if the data is not valid, if the data cannot be
/// deserialized into an instance of `T`, if there is trailing data, and other
/// IO errors.
#[cfg(feature = "std")]
pub fn from_reader<R, T>(r: R) -> Result<T>
where
    R: io::Read,
    T: de::DeserializeOwned,
{
    let mut de = Deserializer::new(read::IoRead::new(r));
    let value = T::deserialize(&mut de)?;
    de.end()?;
    Ok(value)
}

/// Deserializes an instance of `T` from a slice of bytes.
///
/// The entire slice of bytes is consumed, and it is an error if there is
/// trailing data. If trailing data is expected, then the [`Deserializer`]
/// should be constructed directly. See [`Deserializer::byte_offset()`] for an
/// example.
///
/// # Errors
///
/// Deserialization can fail if the data is not valid, if the data cannot be
/// deserialized into an instance of `T`, if there is trailing data, and other
/// IO errors.
pub fn from_slice<'a, T>(s: &'a [u8]) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    let mut de = Deserializer::new(read::SliceRead::new(s));
    let value = T::deserialize(&mut de)?;
    de.end()?;
    Ok(value)
}

#[derive(Debug)]
/// A `Bencode` Deserializer for types which implement [Deserialize][serde::de::Deserialize].
pub struct Deserializer<R> {
    read: R,
    /// Temporary buffer used to reduce allocations made
    buf: Vec<u8>,
}

impl<'a, R> Deserializer<R>
where
    R: Read<'a>,
{
    /// Constructs a Deserializer from a readable source.
    pub fn new(read: R) -> Self {
        Deserializer {
            read,
            buf: Vec::default(),
        }
    }

    /// Returns the byte offset in the underlying readable source.
    ///
    /// For most use cases, the entire source should be consumed with no
    /// trailing data (e.g. a metainfo file should not have extra data after the
    /// bencoded data).
    ///
    /// If there is expected trailing data, then it may be helpful to know how
    /// much data was read.
    ///
    /// # Example
    ///
    /// ```
    /// use serde::Deserialize as _;
    /// use bt_bencode::Deserializer;
    ///
    /// let bytes: &[u8] = b"4:spameggs";
    /// let mut de = Deserializer::from_slice(bytes);
    /// let value: &str = <&str>::deserialize(&mut de)?;
    /// assert_eq!(value, "spam");
    ///
    /// // Do not call `de.end()` which checks for trailing data
    ///
    /// assert_eq!(de.byte_offset(), 6);
    /// assert_eq!(b"eggs", &bytes[de.byte_offset()..]);
    ///
    /// # Ok::<_, bt_bencode::Error>(())
    /// ```
    pub fn byte_offset(&self) -> usize {
        self.read.byte_offset()
    }

    /// Should be called after a value from the source is deserialized to
    /// validate that the entire source was read.
    ///
    /// If trailing data is expected, do not call this method. It may be
    /// beneficial to know how much data was read. See
    /// [`Deserializer::byte_offset()`].
    ///
    /// # Errors
    ///
    /// An error is returned if there are unconsumed bytes in the readable
    /// source.
    pub fn end(&mut self) -> Result<()> {
        match self.read.peek() {
            Some(r) => r.and(Err(Error::new(
                ErrorKind::TrailingData,
                self.read.byte_offset(),
            ))),
            None => Ok(()),
        }
    }

    fn on_end_seq(&mut self) -> Result<()> {
        match self.parse_peek()? {
            b'e' => {
                self.parse_next()?;
                Ok(())
            }
            _ => Err(Error::new(ErrorKind::InvalidList, self.read.byte_offset())),
        }
    }

    fn on_end_map(&mut self) -> Result<()> {
        match self.parse_peek()? {
            b'e' => {
                self.parse_next()?;
                Ok(())
            }
            _ => Err(Error::new(ErrorKind::InvalidDict, self.read.byte_offset())),
        }
    }

    fn unexpected_type_err(&mut self, exp: &dyn Expected) -> Result<Error> {
        match self.parse_peek()? {
            b'0'..=b'9' => {
                self.buf.clear();
                let bytes = self.read.parse_byte_str(&mut self.buf)?;
                Ok(de::Error::invalid_type(Unexpected::Bytes(&bytes), exp))
            }
            b'i' => {
                self.parse_next()?;
                let (is_positive, num) = self.parse_integer()?;
                if is_positive {
                    Ok(de::Error::invalid_type(Unexpected::Unsigned(num), exp))
                } else {
                    use core::convert::TryFrom;

                    Ok(de::Error::invalid_type(
                        Unexpected::Signed(-i64::try_from(num).map_err(|_| {
                            Error::new(ErrorKind::InvalidInteger, self.read.byte_offset())
                        })?),
                        exp,
                    ))
                }
            }
            b'l' => Ok(de::Error::invalid_type(Unexpected::Seq, exp)),
            b'd' => Ok(de::Error::invalid_type(Unexpected::Map, exp)),
            _ => Err(Error::new(
                ErrorKind::ExpectedSomeValue,
                self.read.byte_offset(),
            )),
        }
    }

    #[inline]
    fn parse_peek(&mut self) -> Result<u8> {
        self.read
            .peek()
            .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.read.byte_offset()))?
    }

    #[inline]
    fn parse_next(&mut self) -> Result<u8> {
        self.read
            .next()
            .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.read.byte_offset()))?
    }

    fn parse_integer(&mut self) -> Result<(bool, u64)> {
        let peek = self
            .read
            .peek()
            .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??;
        let is_positive = if peek == b'-' {
            self.read
                .next()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??;
            false
        } else {
            true
        };

        let peek = self
            .read
            .peek()
            .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??;
        match peek {
            b'0'..=b'9' => {}
            _ => {
                return Err(Error::new(ErrorKind::InvalidInteger, self.byte_offset()));
            }
        }

        let mut value: u64 = 0;

        loop {
            match self
                .read
                .next()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
            {
                b'e' => {
                    return Ok((is_positive, value));
                }
                n @ b'0'..=b'9' => {
                    value = value
                        .checked_mul(10)
                        .ok_or_else(|| Error::new(ErrorKind::InvalidInteger, self.byte_offset()))?;
                    value = value
                        .checked_add(u64::from(n - b'0'))
                        .ok_or_else(|| Error::new(ErrorKind::InvalidInteger, self.byte_offset()))?;
                }
                _ => return Err(Error::new(ErrorKind::InvalidInteger, self.byte_offset())),
            }
        }
    }
}

#[cfg(feature = "std")]
impl<R> Deserializer<read::IoRead<R>>
where
    R: io::Read,
{
    /// Constructs a Deserializer from an [`std::io::Read`][std::io::Read] source.
    #[must_use]
    pub fn from_reader(reader: R) -> Self {
        Deserializer::new(read::IoRead::new(reader))
    }
}

impl<'a> Deserializer<read::SliceRead<'a>> {
    /// Constructs a Deserializer from a `&[u8]`.
    #[must_use]
    pub fn from_slice(bytes: &'a [u8]) -> Self {
        Deserializer::new(read::SliceRead::new(bytes))
    }
}

impl<'de, R: Read<'de>> de::Deserializer<'de> for &mut Deserializer<R> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.parse_peek()? {
            b'0'..=b'9' => {
                self.buf.clear();
                match self.read.parse_byte_str(&mut self.buf)? {
                    Ref::Source(bytes) => visitor.visit_borrowed_bytes(bytes),
                    Ref::Buffer(bytes) => visitor.visit_bytes(bytes),
                }
            }
            b'i' => {
                self.parse_next()?;

                let (is_positive, num) = self.parse_integer()?;
                if is_positive {
                    visitor.visit_u64(num)
                } else {
                    use core::convert::TryFrom;

                    visitor.visit_i64(-i64::try_from(num).map_err(|_| {
                        Error::new(ErrorKind::InvalidInteger, self.read.byte_offset())
                    })?)
                }
            }
            b'l' => {
                self.parse_next()?;
                let ret = visitor.visit_seq(SeqAccess { de: self });
                match (ret, self.on_end_seq()) {
                    (Ok(ret), Ok(())) => Ok(ret),
                    (Err(err), _) | (_, Err(err)) => Err(err),
                }
            }
            b'd' => {
                self.parse_next()?;
                let ret = visitor.visit_map(MapAccess { de: self });
                match (ret, self.on_end_map()) {
                    (Ok(ret), Ok(())) => Ok(ret),
                    (Err(err), _) | (_, Err(err)) => Err(err),
                }
            }
            _ => Err(Error::new(
                ErrorKind::ExpectedSomeValue,
                self.read.byte_offset(),
            )),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 unit unit_struct

        char str string

        seq tuple tuple_struct map

        struct enum identifier ignored_any
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // The hint is that the next value should be in the form of bytes.
        //
        // For a byte string value, the parsed byte string is returned (removing
        // the preceding length and `:`).
        //
        // If the next value is any other type, then capture the "raw" byte
        // representation of the value. For example, an integer value would
        // return the bytes for `i1234e` which includes the `i` and `e` encoding
        // bytes.
        //
        // The idea is to allow the capture of the raw representation of a field
        // as-is. The primary use case is to capture the `info` value in a
        // BitTorrent metainfo. The `info` value would be captured as-is without
        // parsing which allows the infohash to be generated according to the specification.
        match self.parse_peek()? {
            b'0'..=b'9' => {
                self.buf.clear();
                match self.read.parse_byte_str(&mut self.buf)? {
                    Ref::Source(bytes) => visitor.visit_borrowed_bytes(bytes),
                    Ref::Buffer(bytes) => visitor.visit_bytes(bytes),
                }
            }
            b'i' => {
                self.buf.clear();
                match self.read.parse_raw_integer(&mut self.buf)? {
                    Ref::Source(bytes) => visitor.visit_borrowed_bytes(bytes),
                    Ref::Buffer(bytes) => visitor.visit_bytes(bytes),
                }
            }
            b'l' => {
                self.buf.clear();
                match self.read.parse_raw_list(&mut self.buf)? {
                    Ref::Source(bytes) => visitor.visit_borrowed_bytes(bytes),
                    Ref::Buffer(bytes) => visitor.visit_bytes(bytes),
                }
            }
            b'd' => {
                self.buf.clear();
                match self.read.parse_raw_dict(&mut self.buf)? {
                    Ref::Source(bytes) => visitor.visit_borrowed_bytes(bytes),
                    Ref::Buffer(bytes) => visitor.visit_bytes(bytes),
                }
            }
            _ => Err(self.unexpected_type_err(&visitor)?),
        }
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

struct SeqAccess<'a, R> {
    de: &'a mut Deserializer<R>,
}

impl<'de, 'a, R: Read<'de> + 'a> de::SeqAccess<'de> for SeqAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.de.parse_peek()? {
            b'e' => Ok(None),
            _ => Ok(Some(seed.deserialize(&mut *self.de)?)),
        }
    }
}

struct MapAccess<'a, R> {
    de: &'a mut Deserializer<R>,
}

impl<'de, 'a, R: Read<'de> + 'a> de::MapAccess<'de> for MapAccess<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.de.parse_peek()? {
            b'0'..=b'9' => seed.deserialize(MapKey { de: &mut *self.de }).map(Some),
            b'e' => Ok(None),
            _ => Err(Error::new(
                ErrorKind::KeyMustBeAByteStr,
                self.de.read.byte_offset(),
            )),
        }
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct MapKey<'a, R> {
    de: &'a mut Deserializer<R>,
}

impl<'de, R> de::Deserializer<'de> for MapKey<'_, R>
where
    R: Read<'de>,
{
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_any(visitor)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 unit unit_struct seq tuple tuple_struct map
        char str string bytes byte_buf enum struct identifier ignored_any
    }
}

#[cfg(test)]
mod tests {
    use crate::ByteString;

    use super::*;
    use serde_derive::Deserialize;

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    use alloc::{collections::BTreeMap, string::String, vec};
    #[cfg(feature = "std")]
    use std::{collections::BTreeMap, string::String, vec};

    #[test]
    fn test_deserialize_str() -> Result<()> {
        let s: &str = from_slice("4:spam".as_bytes())?;
        assert_eq!(s, "spam");
        Ok(())
    }

    #[test]
    fn test_deserialize_string() -> Result<()> {
        let s: String = from_slice("4:spam".as_bytes())?;
        assert_eq!(s, "spam");
        Ok(())
    }

    #[test]
    fn test_deserialize_integer_1() -> Result<()> {
        let input = "i3e";
        let i: u64 = from_slice(input.as_bytes())?;
        assert_eq!(i, 3);
        Ok(())
    }

    #[test]
    fn test_deserialize_integer_2() -> Result<()> {
        let input = "i-3e";
        let i: i64 = from_slice(input.as_bytes())?;
        assert_eq!(i, -3);
        Ok(())
    }

    #[test]
    fn test_deserialize_integer_3() -> Result<()> {
        let input = "i0e";
        let i: u64 = from_slice(input.as_bytes())?;
        assert_eq!(i, 0);
        Ok(())
    }

    #[test]
    fn test_deserialize_integer_4() -> Result<()> {
        let input = "i0e";
        let i: i64 = from_slice(input.as_bytes())?;
        assert_eq!(i, 0);
        Ok(())
    }

    #[test]
    fn test_deserialize_illegal_unsigned_int() {
        let input = "ie";
        let result: Result<u64> = from_slice(input.as_bytes());
        let error = result.unwrap_err();
        match error.kind() {
            ErrorKind::InvalidInteger => {}
            _ => panic!(),
        }
        assert_eq!(error.byte_offset(), 1);
    }

    #[test]
    fn test_deserialize_illegal_signed_int() {
        let input = "i-e";
        let result: Result<i64> = from_slice(input.as_bytes());
        let error = result.unwrap_err();
        match error.kind() {
            ErrorKind::InvalidInteger => {}
            _ => panic!(),
        }
        assert_eq!(error.byte_offset(), 2);
    }

    #[test]
    fn test_deserialize_list() -> Result<()> {
        let input = "l4:spam4:eggse";
        let v: Vec<String> = from_slice(input.as_bytes())?;
        assert_eq!(v, vec!["spam", "eggs"]);
        Ok(())
    }

    #[test]
    fn test_deserialize_list_str() -> Result<()> {
        let input = "l4:spam4:eggse";
        let v: Vec<&str> = from_slice(input.as_bytes())?;
        assert_eq!(v, vec!["spam", "eggs"]);
        Ok(())
    }

    #[test]
    fn test_deserialize_list_as_tuple() -> Result<()> {
        let input = "li123e4:eggse";
        let v: (i64, &str) = from_slice(input.as_bytes())?;
        assert_eq!(v, (123, "eggs"));
        Ok(())
    }

    #[test]
    fn test_deserialize_list_as_struct_tuple() -> Result<()> {
        #[derive(Debug, serde_derive::Deserialize, PartialEq, Eq)]
        struct S<'a>(i64, &'a str);

        let input = "li123e4:eggse";
        let v: S<'_> = from_slice(input.as_bytes())?;
        assert_eq!(v, S(123, "eggs"));
        Ok(())
    }

    #[test]
    fn test_deserialize_dict_1() -> Result<()> {
        let input = "d3:cow3:moo4:spam4:eggse";
        let m: BTreeMap<String, String> = from_slice(input.as_bytes())?;
        let mut expected = BTreeMap::new();
        expected.insert(String::from("cow"), String::from("moo"));
        expected.insert(String::from("spam"), String::from("eggs"));
        assert_eq!(m, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_dict_1_str() -> Result<()> {
        let input = "d3:cow3:moo4:spam4:eggse";
        let m: BTreeMap<&str, &str> = from_slice(input.as_bytes())?;
        let mut expected = BTreeMap::new();
        expected.insert("cow", "moo");
        expected.insert("spam", "eggs");
        assert_eq!(m, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_dict_2() -> Result<()> {
        let input = "d4:spaml1:a1:bee";
        let m: BTreeMap<String, Vec<String>> = from_slice(input.as_bytes())?;
        let mut expected = BTreeMap::new();
        expected.insert(String::from("spam"), vec!["a".into(), "b".into()]);
        assert_eq!(m, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_dict_2_str() -> Result<()> {
        let input = "d4:spaml1:a1:bee";
        let m: BTreeMap<&str, Vec<&str>> = from_slice(input.as_bytes())?;
        let mut expected = BTreeMap::new();
        expected.insert("spam", vec!["a", "b"]);
        assert_eq!(m, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_struct() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S {
            spam: Vec<String>,
        }

        let input = "d4:spaml1:a1:bee";
        let s: S = from_slice(input.as_bytes())?;
        let expected = S {
            spam: vec!["a".into(), "b".into()],
        };
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_integer_as_raw_slice() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S<'a>(&'a [u8]);

        let input = "i-1234e";
        let s: S<'_> = from_slice(input.as_bytes())?;
        let expected = S(input.as_bytes());
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_list_as_raw_slice() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S<'a>(&'a [u8]);

        let input = "l4:spam4:eggse";
        let s: S<'_> = from_slice(input.as_bytes())?;
        let expected = S(input.as_bytes());
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_map_value_as_raw_slice() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S<'a> {
            spam: &'a [u8],
        }

        let input = "d4:spamd1:a1:bee";
        let s: S<'_> = from_slice(input.as_bytes())?;
        let expected = S { spam: b"d1:a1:be" };
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_map_as_raw_slice() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S<'a>(&'a [u8]);

        let input = "d4:spamd1:a1:bee";
        let s: S<'_> = from_slice(input.as_bytes())?;
        let expected = S(input.as_bytes());
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_integer_as_raw_bytes() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S(ByteString);

        let input = "i-1234e";
        let s: S = from_slice(input.as_bytes())?;
        let expected = S(ByteString::from(input.as_bytes().to_vec()));
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_list_as_raw_bytes() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S(ByteString);

        let input = "l4:spam4:eggse";
        let s: S = from_slice(input.as_bytes())?;
        let expected = S(ByteString::from(input.as_bytes().to_vec()));
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_map_value_as_raw_bytes() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S {
            spam: ByteString,
        }

        let input = "d4:spamd1:a1:bee";
        let s: S = from_slice(input.as_bytes())?;
        let expected = S {
            spam: ByteString::from(b"d1:a1:be".to_vec()),
        };
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_map_as_raw_bytes() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S(ByteString);

        let input = "d4:spamd1:a1:bee";
        let s: S = from_slice(input.as_bytes())?;
        let expected = S(ByteString::from(input.as_bytes().to_vec()));
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_some_struct() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S {
            inner: i32,
        }

        let input = "d5:inneri32ee";
        let s: Option<S> = from_slice(input.as_bytes())?;
        let expected = Some(S { inner: 32 });
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_none_struct() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S {
            inner: i32,
        }

        let input = "de";
        // let s: Option<S> = from_slice(input.as_bytes());
        let s: Result<Option<S>> = from_slice(input.as_bytes());
        println!("{:?}", s);
        let s = s.unwrap();
        let expected = None;
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_struct_some() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S {
            inner: Option<i32>,
        }

        let input = "d5:inneri32ee";
        let s: S = from_slice(input.as_bytes())?;
        let expected = S { inner: Some(32) };
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_struct_none_dict() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S {
            inner: Option<i32>,
        }

        let input = "de";
        let s: S = from_slice(input.as_bytes())?;
        let expected = S { inner: None };
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_struct_some_none() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S {
            inner: Option<T>,
        }

        #[derive(Debug, PartialEq, Deserialize)]
        struct T {
            innermost: Option<i32>,
        }

        let input = "d5:innerdee";
        let s: S = from_slice(input.as_bytes())?;
        let expected = S {
            inner: Some(T { innermost: None }),
        };
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_struct_some_some() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S {
            inner: Option<T>,
        }

        #[derive(Debug, PartialEq, Deserialize)]
        struct T {
            innermost: Option<i32>,
        }

        let input = "d5:innerd9:innermosti32eee";
        let s: S = from_slice(input.as_bytes())?;
        let expected = S {
            inner: Some(T {
                innermost: Some(32),
            }),
        };
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_inner_none() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S {
            inner: Option<T>,
        }

        #[derive(Debug, PartialEq, Deserialize)]
        struct T {
            innermost: Option<i32>,
        }

        let input = "de";
        let s: S = from_slice(input.as_bytes())?;
        let expected = S { inner: None };
        assert_eq!(s, expected);
        Ok(())
    }
}
