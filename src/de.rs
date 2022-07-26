//! Deserializes Bencode data.

use crate::error::{Error, Result};
use crate::read::{self, Read};
use serde::de::{self, Expected, Unexpected};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{string::String, vec, vec::Vec};
#[cfg(feature = "std")]
use std::{io, string::String, vec, vec::Vec};

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
}

impl<R> Deserializer<R>
where
    R: Read,
{
    /// Constructs a Deserializer from a readable source.
    pub fn new(read: R) -> Self {
        Deserializer { read }
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
                let bytes = self.parse_bytes()?;
                Ok(de::Error::invalid_type(Unexpected::Bytes(&bytes), exp))
            }
            b'i' => {
                self.parse_next()?;
                let num_str = self.parse_integer_string()?;
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

    fn parse_integer_bytes(&mut self, is_pos: bool) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        if !is_pos {
            result.push(b'-');
        }
        loop {
            match self.parse_next()? {
                b'e' => return Ok(result),
                n @ b'0'..=b'9' => result.push(n),
                _ => return Err(Error::InvalidInteger),
            }
        }
    }

    fn parse_integer_string(&mut self) -> Result<String> {
        match self.parse_peek()? {
            b'-' => {
                self.parse_next()?;
                Ok(String::from_utf8(self.parse_integer_bytes(false)?)?)
            }
            b'0'..=b'9' => Ok(String::from_utf8(self.parse_integer_bytes(true)?)?),
            _ => Err(Error::InvalidInteger),
        }
    }

    #[inline]
    fn parse_bytes_len(&mut self) -> Result<usize> {
        let mut result: Vec<u8> = Vec::new();
        loop {
            match self.parse_next()? {
                b':' => {
                    return Ok(String::from_utf8(result)?.parse()?);
                }
                n @ b'0'..=b'9' => result.push(n),
                _ => return Err(Error::InvalidByteStrLen),
            }
        }
    }

    fn parse_bytes(&mut self) -> Result<Vec<u8>> {
        let len = self.parse_bytes_len()?;
        let mut buf = vec![0u8; len];
        for i in &mut buf {
            *i = self.parse_next()?;
        }
        Ok(buf)
    }

    fn capture_byte_string_len(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let mut len_buf = Vec::new();
        loop {
            match self.parse_next()? {
                b':' => {
                    let len = String::from_utf8(len_buf.clone())?.parse()?;
                    buf.extend(len_buf);
                    buf.push(b':');
                    return Ok(len);
                }
                n @ b'0'..=b'9' => len_buf.push(n),
                _ => return Err(Error::InvalidByteStrLen),
            }
        }
    }

    fn capture_byte_string(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        let len = self.capture_byte_string_len(buf)?;
        buf.reserve(len);
        for _ in 0..len {
            buf.push(self.parse_next()?);
        }
        Ok(())
    }

    fn capture_integer(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        buf.push(self.parse_next()?);

        match self.parse_peek()? {
            b'-' => buf.push(self.parse_next()?),
            b'0'..=b'9' => {}
            _ => return Err(Error::InvalidInteger),
        }

        loop {
            match self.parse_next()? {
                b'e' => {
                    buf.push(b'e');
                    return Ok(());
                }
                n @ b'0'..=b'9' => buf.push(n),
                _ => return Err(Error::InvalidInteger),
            }
        }
    }

    fn capture_list(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        buf.push(self.parse_next()?);

        loop {
            match self.parse_peek()? {
                b'e' => {
                    buf.push(self.parse_next()?);
                    return Ok(());
                }
                b'0'..=b'9' => self.capture_byte_string(buf)?,
                b'i' => self.capture_integer(buf)?,
                b'l' => self.capture_list(buf)?,
                b'd' => self.capture_dict(buf)?,
                _ => return Err(Error::InvalidList),
            }
        }
    }

    fn capture_dict(&mut self, buf: &mut Vec<u8>) -> Result<()> {
        buf.push(self.parse_next()?);

        loop {
            match self.parse_peek()? {
                b'0'..=b'9' => self.capture_byte_string(buf)?,
                b'e' => {
                    buf.push(self.parse_next()?);
                    return Ok(());
                }
                _ => {
                    return Err(Error::InvalidDict);
                }
            }

            match self.parse_peek()? {
                b'0'..=b'9' => self.capture_byte_string(buf)?,
                b'i' => self.capture_integer(buf)?,
                b'l' => self.capture_list(buf)?,
                b'd' => self.capture_dict(buf)?,
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
                let bytes = self.parse_bytes()?;
                visitor.visit_byte_buf(bytes)
            }
            b'i' => {
                self.parse_next()?;
                let num_str = self.parse_integer_string()?;
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
                let num_str = self.parse_integer_string()?;
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
                let num_str = self.parse_integer_string()?;
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
        self.deserialize_string(visitor)
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.parse_peek()? {
            b'0'..=b'9' => {
                let bytes = self.parse_bytes()?;
                match String::from_utf8(bytes.clone()) {
                    Ok(s) => visitor.visit_string(s),
                    Err(_) => visitor.visit_byte_buf(bytes),
                }
            }
            _ => Err(self.unexpected_type_err(&visitor)?),
        }
    }

    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.parse_peek()? {
            b'0'..=b'9' => {
                let bytes = self.parse_bytes()?;
                visitor.visit_byte_buf(bytes)
            }
            b'i' => {
                let mut bytes = Vec::new();
                self.capture_integer(&mut bytes)?;
                visitor.visit_byte_buf(bytes)
            }
            b'l' => {
                let mut bytes = Vec::new();
                self.capture_list(&mut bytes)?;
                visitor.visit_byte_buf(bytes)
            }
            b'd' => {
                let mut bytes = Vec::new();
                self.capture_dict(&mut bytes)?;
                visitor.visit_byte_buf(bytes)
            }
            _ => Err(self.unexpected_type_err(&visitor)?),
        }
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
