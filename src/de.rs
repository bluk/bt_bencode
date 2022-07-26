//! Deserializes Bencode data.

use crate::error::{Error, Result};
use crate::read::{self, Read};
use serde::de::{self, Expected, Unexpected};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::{io, vec::Vec};

/// Deserializes an instance of `T` from the bytes of an [`io::Read`] type.
///
/// # Errors
///
/// Deserialization can fail if the data is not valid, if the data cannot cannot be deserialized
/// into an instance of `T`, and other IO errors.
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
/// # Errors
///
/// Deserialization can fail if the data is not valid, if the data cannot cannot be deserialized
/// into an instance of `T`, and other IO errors.
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

impl<R> Deserializer<R>
where
    R: Read,
{
    /// Constructs a Deserializer from a readable source.
    pub fn new(read: R) -> Self {
        Deserializer {
            read,
            buf: Vec::default(),
        }
    }

    /// Should be called after a value from the source is deserialized to validate that the entire
    /// source was read.
    ///
    /// # Errors
    ///
    /// An error is returned if there are unconsumed bytes in the readable source.
    pub fn end(&mut self) -> Result<()> {
        match self.read.peek() {
            Some(r) => r.and(Err(Error::TrailingData)),
            None => Ok(()),
        }
    }

    fn on_end_seq(&mut self) -> Result<()> {
        match self.parse_peek()? {
            b'e' => {
                self.parse_next()?;
                Ok(())
            }
            _ => Err(Error::InvalidList),
        }
    }

    fn on_end_map(&mut self) -> Result<()> {
        match self.parse_peek()? {
            b'e' => {
                self.parse_next()?;
                Ok(())
            }
            _ => Err(Error::InvalidDict),
        }
    }

    fn unexpected_type_err(&mut self, exp: &dyn Expected) -> Result<Error> {
        match self.parse_peek()? {
            b'0'..=b'9' => {
                self.parse_bytes()?;
                Ok(de::Error::invalid_type(Unexpected::Bytes(&self.buf), exp))
            }
            b'i' => {
                self.parse_next()?;
                let num_str = self.parse_integer_str()?;
                if num_str.starts_with('-') {
                    Ok(de::Error::invalid_type(
                        Unexpected::Signed(num_str.parse()?),
                        exp,
                    ))
                } else {
                    Ok(de::Error::invalid_type(
                        Unexpected::Unsigned(num_str.parse()?),
                        exp,
                    ))
                }
            }
            b'l' => Ok(de::Error::invalid_type(Unexpected::Seq, exp)),
            b'd' => Ok(de::Error::invalid_type(Unexpected::Map, exp)),
            _ => Err(Error::ExpectedSomeValue),
        }
    }

    #[inline]
    fn parse_peek(&mut self) -> Result<u8> {
        match self.read.peek() {
            Some(r) => r,
            None => Err(Error::EofWhileParsingValue),
        }
    }

    #[inline]
    fn parse_next(&mut self) -> Result<u8> {
        match self.read.next() {
            Some(r) => r,
            None => Err(Error::EofWhileParsingValue),
        }
    }

    fn parse_integer_bytes(&mut self, is_pos: bool) -> Result<()> {
        self.buf.clear();
        if !is_pos {
            self.buf.push(b'-');
        }
        loop {
            match self.parse_next()? {
                b'e' => return Ok(()),
                n @ b'0'..=b'9' => self.buf.push(n),
                _ => return Err(Error::InvalidInteger),
            }
        }
    }

    fn parse_integer_str(&mut self) -> Result<&str> {
        match self.parse_peek()? {
            b'-' => {
                self.parse_next()?;
                self.parse_integer_bytes(false)?;
                Ok(core::str::from_utf8(&self.buf)?)
            }
            b'0'..=b'9' => {
                self.parse_integer_bytes(true)?;
                Ok(core::str::from_utf8(&self.buf)?)
            }
            _ => Err(Error::InvalidInteger),
        }
    }

    #[inline]
    fn parse_bytes_len(&mut self) -> Result<usize> {
        self.buf.clear();
        loop {
            match self.parse_next()? {
                b':' => {
                    return Ok(core::str::from_utf8(&self.buf)?.parse()?);
                }
                n @ b'0'..=b'9' => self.buf.push(n),
                _ => return Err(Error::InvalidByteStrLen),
            }
        }
    }

    fn parse_bytes(&mut self) -> Result<()> {
        let len = self.parse_bytes_len()?;
        self.buf.clear();
        self.buf.reserve(len);
        // TODO: Should have a method to read from a slice
        for _ in 0..len {
            self.buf
                .push(self.read.next().ok_or(Error::EofWhileParsingValue)??);
        }
        Ok(())
    }

    fn capture_byte_string_len(&mut self) -> Result<usize> {
        let start_idx = self.buf.len();
        loop {
            match self.parse_next()? {
                b':' => {
                    let len = core::str::from_utf8(&self.buf[start_idx..])?.parse()?;
                    self.buf.push(b':');
                    return Ok(len);
                }
                n @ b'0'..=b'9' => self.buf.push(n),
                _ => return Err(Error::InvalidByteStrLen),
            }
        }
    }

    fn capture_byte_string(&mut self) -> Result<()> {
        let len = self.capture_byte_string_len()?;
        self.buf.reserve(len);
        for _ in 0..len {
            self.buf
                .push(self.read.next().ok_or(Error::EofWhileParsingValue)??);
        }
        Ok(())
    }

    fn capture_integer(&mut self) -> Result<()> {
        self.buf
            .push(self.read.next().ok_or(Error::EofWhileParsingValue)??);

        match self.parse_peek()? {
            b'-' => {
                self.buf
                    .push(self.read.next().ok_or(Error::EofWhileParsingValue)??);
            }
            b'0'..=b'9' => {}
            _ => return Err(Error::InvalidInteger),
        }

        loop {
            match self.parse_next()? {
                b'e' => {
                    self.buf.push(b'e');
                    return Ok(());
                }
                n @ b'0'..=b'9' => self.buf.push(n),
                _ => return Err(Error::InvalidInteger),
            }
        }
    }

    fn capture_list(&mut self) -> Result<()> {
        self.buf
            .push(self.read.next().ok_or(Error::EofWhileParsingValue)??);

        loop {
            match self.parse_peek()? {
                b'e' => {
                    self.buf
                        .push(self.read.next().ok_or(Error::EofWhileParsingValue)??);
                    return Ok(());
                }
                b'0'..=b'9' => self.capture_byte_string()?,
                b'i' => self.capture_integer()?,
                b'l' => self.capture_list()?,
                b'd' => self.capture_dict()?,
                _ => return Err(Error::InvalidList),
            }
        }
    }

    fn capture_dict(&mut self) -> Result<()> {
        self.buf
            .push(self.read.next().ok_or(Error::EofWhileParsingValue)??);

        loop {
            match self.parse_peek()? {
                b'0'..=b'9' => self.capture_byte_string()?,
                b'e' => {
                    self.buf
                        .push(self.read.next().ok_or(Error::EofWhileParsingValue)??);
                    return Ok(());
                }
                _ => {
                    return Err(Error::InvalidDict);
                }
            }

            match self.parse_peek()? {
                b'0'..=b'9' => self.capture_byte_string()?,
                b'i' => self.capture_integer()?,
                b'l' => self.capture_list()?,
                b'd' => self.capture_dict()?,
                _ => {
                    return Err(Error::InvalidDict);
                }
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

macro_rules! forward_deserialize_signed_integer {
    ($method:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            self.deserialize_i64(visitor)
        }
    };
}

macro_rules! forward_deserialize_unsigned_integer {
    ($method:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            self.deserialize_u64(visitor)
        }
    };
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.parse_peek()? {
            b'0'..=b'9' => {
                self.parse_bytes()?;
                visitor.visit_bytes(&self.buf)
            }
            b'i' => {
                self.parse_next()?;
                let num_str = self.parse_integer_str()?;
                if num_str.starts_with('-') {
                    visitor.visit_i64(num_str.parse()?)
                } else {
                    visitor.visit_u64(num_str.parse()?)
                }
            }
            b'l' => {
                self.parse_next()?;
                let ret = visitor.visit_seq(SeqAccess::new(self));
                match (ret, self.on_end_seq()) {
                    (Ok(ret), Ok(())) => Ok(ret),
                    (Err(err), _) | (_, Err(err)) => Err(err),
                }
            }
            b'd' => {
                self.parse_next()?;
                let ret = visitor.visit_map(MapAccess::new(self));
                match (ret, self.on_end_map()) {
                    (Ok(ret), Ok(())) => Ok(ret),
                    (Err(err), _) | (_, Err(err)) => Err(err),
                }
            }
            _ => Err(Error::ExpectedSomeValue),
        }
    }

    forward_to_deserialize_any! {
        bool f32 f64 unit unit_struct

        struct enum identifier ignored_any
    }

    forward_deserialize_signed_integer!(deserialize_i8);
    forward_deserialize_signed_integer!(deserialize_i16);
    forward_deserialize_signed_integer!(deserialize_i32);

    #[inline]
    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.parse_peek()? {
            b'i' => {
                self.parse_next()?;
                let num_str = self.parse_integer_str()?;
                if num_str.starts_with('-') {
                    visitor.visit_i64(num_str.parse()?)
                } else {
                    visitor.visit_u64(num_str.parse()?)
                }
            }
            _ => Err(self.unexpected_type_err(&visitor)?),
        }
    }

    forward_deserialize_unsigned_integer!(deserialize_u8);
    forward_deserialize_unsigned_integer!(deserialize_u16);
    forward_deserialize_unsigned_integer!(deserialize_u32);

    #[inline]
    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.parse_peek()? {
            b'i' => {
                self.parse_next()?;
                let num_str = self.parse_integer_str()?;
                if num_str.starts_with('-') {
                    visitor.visit_i64(num_str.parse()?)
                } else {
                    visitor.visit_u64(num_str.parse()?)
                }
            }
            _ => Err(self.unexpected_type_err(&visitor)?),
        }
    }

    #[inline]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.parse_peek()? {
            b'0'..=b'9' => {
                self.parse_bytes()?;
                match core::str::from_utf8(&self.buf) {
                    Ok(s) => visitor.visit_str(s),
                    Err(_) => visitor.visit_bytes(&self.buf),
                }
            }
            _ => Err(self.unexpected_type_err(&visitor)?),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.parse_peek()? {
            b'0'..=b'9' => {
                self.parse_bytes()?;
                visitor.visit_bytes(&self.buf)
            }
            b'i' => {
                self.buf.clear();
                self.capture_integer()?;
                visitor.visit_bytes(&self.buf)
            }
            b'l' => {
                self.buf.clear();
                self.capture_list()?;
                visitor.visit_bytes(&self.buf)
            }
            b'd' => {
                self.buf.clear();
                self.capture_dict()?;
                visitor.visit_bytes(&self.buf)
            }
            _ => Err(self.unexpected_type_err(&visitor)?),
        }
    }

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

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.parse_peek()? {
            b'l' => {
                self.parse_next()?;
                let ret = visitor.visit_seq(SeqAccess::new(self));
                match (ret, self.on_end_seq()) {
                    (Ok(ret), Ok(())) => Ok(ret),
                    (Err(err), _) | (_, Err(err)) => Err(err),
                }
            }
            _ => Err(self.unexpected_type_err(&visitor)?),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.parse_peek()? {
            b'd' => {
                self.parse_next()?;
                let ret = visitor.visit_map(MapAccess::new(self));
                match (ret, self.on_end_map()) {
                    (Ok(ret), Ok(())) => Ok(ret),
                    (Err(err), _) | (_, Err(err)) => Err(err),
                }
            }
            _ => Err(self.unexpected_type_err(&visitor)?),
        }
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

#[derive(Debug)]
struct SeqAccess<'a, R> {
    de: &'a mut Deserializer<R>,
}

impl<'a, R: 'a> SeqAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> Self {
        SeqAccess { de }
    }
}

impl<'de, 'a, R: Read + 'a> de::SeqAccess<'de> for SeqAccess<'a, R> {
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

#[derive(Debug)]
struct MapAccess<'a, R> {
    de: &'a mut Deserializer<R>,
}

impl<'a, R: 'a> MapAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> Self {
        MapAccess { de }
    }
}

impl<'de, 'a, R: Read + 'a> de::MapAccess<'de> for MapAccess<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.de.parse_peek()? {
            b'0'..=b'9' => seed.deserialize(MapKey { de: &mut *self.de }).map(Some),
            b'e' => Ok(None),
            _ => Err(Error::KeyMustBeAByteStr),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

#[derive(Debug)]
struct MapKey<'a, R> {
    de: &'a mut Deserializer<R>,
}

impl<'de, 'a, R> de::Deserializer<'de> for MapKey<'a, R>
where
    R: Read,
{
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_bytes(visitor)
    }

    #[inline]
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_char(visitor)
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_str(visitor)
    }

    #[inline]
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_string(visitor)
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_bytes(visitor)
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.de.deserialize_byte_buf(visitor)
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
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 unit unit_struct seq tuple tuple_struct map
        enum struct identifier ignored_any
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_bytes::ByteBuf;
    use serde_derive::Deserialize;

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    use alloc::collections::BTreeMap;
    #[cfg(feature = "std")]
    use std::collections::BTreeMap;

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
    fn test_deserialize_list() -> Result<()> {
        let input = "l4:spam4:eggse";
        let v: Vec<String> = from_slice(input.as_bytes())?;
        assert_eq!(v, vec!["spam", "eggs"]);
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
    fn test_deserialize_dict_2() -> Result<()> {
        let input = "d4:spaml1:a1:bee";
        let m: BTreeMap<String, Vec<String>> = from_slice(input.as_bytes())?;
        let mut expected = BTreeMap::new();
        expected.insert(String::from("spam"), vec!["a".into(), "b".into()]);
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
    fn test_deserialize_integer_as_raw_bytes() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S(ByteBuf);

        let input = "i-1234e";
        let s: S = from_slice(input.as_bytes())?;
        let expected = S(ByteBuf::from(input.as_bytes().to_vec()));
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_list_as_raw_bytes() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S(ByteBuf);

        let input = "l4:spam4:eggse";
        let s: S = from_slice(input.as_bytes())?;
        let expected = S(ByteBuf::from(input.as_bytes().to_vec()));
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_map_value_as_raw_bytes() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S {
            spam: ByteBuf,
        }

        let input = "d4:spamd1:a1:bee";
        let s: S = from_slice(input.as_bytes())?;
        let expected = S {
            spam: ByteBuf::from(b"d1:a1:be".to_vec()),
        };
        assert_eq!(s, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_map_as_raw_bytes() -> Result<()> {
        #[derive(Debug, PartialEq, Deserialize)]
        struct S(ByteBuf);

        let input = "d4:spamd1:a1:bee";
        let s: S = from_slice(input.as_bytes())?;
        let expected = S(ByteBuf::from(input.as_bytes().to_vec()));
        assert_eq!(s, expected);
        Ok(())
    }
}
