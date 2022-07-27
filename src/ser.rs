//! Serializes Bencode data.

use crate::error::{Error, Result};
use serde::{ser, Serialize};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{collections::BTreeMap, vec::Vec};

#[cfg(feature = "std")]
use std::{collections::BTreeMap, io, vec::Vec};

#[cfg(feature = "std")]
use crate::write;

use crate::write::Write;

/// Serializes an instance of `T` into the writer `W` as `Bencode` data.
///
/// # Errors
///
/// Serialization can fail if `T`'s implementation of
/// [Serialize][serde::ser::Serialize] decides to fail, if `T` contains
/// unsupported types for serialization, or if `T` contains a map with
/// non-string keys.
#[cfg(feature = "std")]
#[inline]
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ?Sized + Serialize,
{
    let mut ser = Serializer::new(write::IoWrite::new(writer));
    value.serialize(&mut ser)?;
    Ok(())
}

/// Serializes an instance of `T` into a new [Vec] as `Bencode` data.
///
/// # Errors
///
/// Serialization can fail if `T`'s implemenation of
/// [Serialize][serde::ser::Serialize] decides to fail, if `T` contains
/// unsupported types for serialization, or if `T` contains a map with
/// non-string keys.
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let mut writer = Vec::new();
    let mut ser = Serializer::new(&mut writer);
    value.serialize(&mut ser)?;
    Ok(writer)
}

/// A `Bencode` Serializer for types which implement [Serialize][serde::ser::Serialize].
#[derive(Debug)]
pub struct Serializer<W> {
    writer: W,
}

impl<W> Serializer<W>
where
    W: Write,
{
    /// Constructs a Serializer with an [Write] target.
    pub fn new(writer: W) -> Self {
        Serializer { writer }
    }
}

impl<W> Serializer<W>
where
    W: Write,
{
    /// Returns the inner writer.
    ///
    /// Useful when the serializer is done and the writer is needed to write other data.
    #[inline]
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = ser::Impossible<(), Error>;
    type SerializeTupleStruct = ser::Impossible<(), Error>;
    type SerializeTupleVariant = ser::Impossible<(), Error>;
    type SerializeMap = SerializeMap<'a, W>;
    type SerializeStruct = SerializeMap<'a, W>;
    type SerializeStructVariant = ser::Impossible<(), Error>;

    #[inline]
    fn serialize_bool(self, _value: bool) -> Result<()> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<()> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<()> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<()> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<()> {
        self.writer.write_all(b"i")?;
        self.writer
            .write_all(itoa::Buffer::new().format(value).as_bytes())?;
        self.writer.write_all(b"e")?;
        Ok(())
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<()> {
        self.serialize_u64(u64::from(value))
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<()> {
        self.serialize_u64(u64::from(value))
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<()> {
        self.serialize_u64(u64::from(value))
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<()> {
        self.writer.write_all(b"i")?;
        self.writer
            .write_all(itoa::Buffer::new().format(value).as_bytes())?;
        self.writer.write_all(b"e")?;
        Ok(())
    }

    #[inline]
    fn serialize_f32(self, _value: f32) -> Result<()> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_f64(self, _value: f64) -> Result<()> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<()> {
        let mut buf = [0; 4];
        self.serialize_str(value.encode_utf8(&mut buf))
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<()> {
        self.writer
            .write_all(itoa::Buffer::new().format(value.len()).as_bytes())?;
        self.writer.write_all(b":")?;
        self.writer.write_all(value.as_bytes())
    }

    #[inline]
    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        self.writer
            .write_all(itoa::Buffer::new().format(value.len()).as_bytes())?;
        self.writer.write_all(b":")?;
        self.writer.write_all(value)
    }

    #[inline]
    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<()> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.writer.write_all(b"l")?;
        Ok(self)
    }

    #[inline]
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.writer.write_all(b"d")?;
        Ok(SerializeMap::new(self))
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::UnsupportedType)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'a, W> ser::SerializeSeq for &'a mut Serializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    #[inline]
    fn end(self) -> Result<()> {
        self.writer.write_all(b"e")?;
        Ok(())
    }
}

/// A serializer for writing map data.
#[doc(hidden)]
#[derive(Debug)]
pub struct SerializeMap<'a, W> {
    ser: &'a mut Serializer<W>,
    entries: BTreeMap<Vec<u8>, Vec<u8>>,
    current_key: Option<Vec<u8>>,
}

impl<'a, W> SerializeMap<'a, W>
where
    W: Write,
{
    #[inline]
    fn new(ser: &'a mut Serializer<W>) -> Self {
        SerializeMap {
            ser,
            entries: BTreeMap::new(),
            current_key: None,
        }
    }

    #[inline]
    fn end_map(&mut self) -> Result<()> {
        if self.current_key.is_some() {
            return Err(Error::KeyWithoutValue);
        }

        for (k, v) in &self.entries {
            ser::Serializer::serialize_bytes(&mut *self.ser, k.as_ref())?;
            self.ser.writer.write_all(v)?;
        }

        Ok(())
    }
}

impl<'a, W> ser::SerializeMap for SerializeMap<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.current_key.is_some() {
            return Err(Error::KeyWithoutValue);
        }
        self.current_key = Some(key.serialize(&mut MapKeySerializer {})?);
        Ok(())
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = self.current_key.take().ok_or(Error::ValueWithoutKey)?;
        let buf: Vec<u8> = Vec::new();
        let mut ser = Serializer::new(buf);
        value.serialize(&mut ser)?;
        self.entries.insert(key, ser.into_inner());
        Ok(())
    }

    #[inline]
    fn end(mut self) -> Result<()> {
        self.end_map()?;
        self.ser.writer.write_all(b"e")?;
        Ok(())
    }
}

impl<'a, W> ser::SerializeStruct for SerializeMap<'a, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = key.serialize(&mut MapKeySerializer {})?;

        let buf: Vec<u8> = Vec::new();
        let mut ser = Serializer::new(buf);
        value.serialize(&mut ser)?;
        self.entries.insert(key, ser.into_inner());
        Ok(())
    }

    #[inline]
    fn end(mut self) -> Result<()> {
        self.end_map()?;
        self.ser.writer.write_all(b"e")?;
        Ok(())
    }
}

struct MapKeySerializer;

impl<'a> ser::Serializer for &'a mut MapKeySerializer {
    type Ok = Vec<u8>;
    type Error = Error;

    type SerializeSeq = ser::Impossible<Vec<u8>, Error>;
    type SerializeTuple = ser::Impossible<Vec<u8>, Error>;
    type SerializeTupleStruct = ser::Impossible<Vec<u8>, Error>;
    type SerializeTupleVariant = ser::Impossible<Vec<u8>, Error>;
    type SerializeMap = ser::Impossible<Vec<u8>, Error>;
    type SerializeStruct = ser::Impossible<Vec<u8>, Error>;
    type SerializeStructVariant = ser::Impossible<Vec<u8>, Error>;

    fn serialize_bool(self, _value: bool) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_i8(self, _value: i8) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_i16(self, _value: i16) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_i32(self, _value: i32) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_i64(self, _value: i64) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_u8(self, _value: u8) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_u16(self, _value: u16) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_u32(self, _value: u32) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_u64(self, _value: u64) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_f32(self, _value: f32) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_f64(self, _value: f64) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_char(self, value: char) -> Result<Vec<u8>> {
        let mut buf = [0; 4];
        self.serialize_str(value.encode_utf8(&mut buf))
    }

    fn serialize_str(self, value: &str) -> Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::with_capacity(value.len());
        buf.extend_from_slice(value.as_bytes());
        Ok(buf)
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Vec<u8>> {
        let mut buf: Vec<u8> = Vec::with_capacity(value.len());
        buf.extend_from_slice(value);
        Ok(buf)
    }

    fn serialize_unit(self) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Vec<u8>> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_none(self) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Vec<u8>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<ser::Impossible<Vec<u8>, Error>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_tuple(self, _size: usize) -> Result<ser::Impossible<Vec<u8>, Error>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<ser::Impossible<Vec<u8>, Error>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<ser::Impossible<Vec<u8>, Error>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<ser::Impossible<Vec<u8>, Error>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<ser::Impossible<Vec<u8>, Error>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<ser::Impossible<Vec<u8>, Error>> {
        Err(Error::UnsupportedType)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_bytes::ByteBuf;

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    use alloc::{format, string::String, vec};
    #[cfg(feature = "std")]
    use std::string::String;

    #[test]
    fn test_serialize_bool() {
        assert!(matches!(to_vec(&true), Err(Error::UnsupportedType)));
    }

    #[test]
    fn test_serialize_isize() {
        let value: isize = 2;
        assert_eq!(to_vec(&value).unwrap(), String::from("i2e").into_bytes());
        let value: isize = -2;
        assert_eq!(to_vec(&value).unwrap(), String::from("i-2e").into_bytes());
    }

    #[test]
    fn test_serialize_i8() {
        let value: i8 = 2;
        assert_eq!(to_vec(&value).unwrap(), String::from("i2e").into_bytes());
        let value: i8 = -2;
        assert_eq!(to_vec(&value).unwrap(), String::from("i-2e").into_bytes());
    }

    #[test]
    fn test_serialize_i16() {
        let value: i16 = 2;
        assert_eq!(to_vec(&value).unwrap(), String::from("i2e").into_bytes());
        let value: i16 = -2;
        assert_eq!(to_vec(&value).unwrap(), String::from("i-2e").into_bytes());
    }

    #[test]
    fn test_serialize_i32() {
        let value: i32 = 2;
        assert_eq!(to_vec(&value).unwrap(), String::from("i2e").into_bytes());
        let value: i32 = -2;
        assert_eq!(to_vec(&value).unwrap(), String::from("i-2e").into_bytes());
    }

    #[test]
    fn test_serialize_i64() {
        let value: i64 = 2;
        assert_eq!(to_vec(&value).unwrap(), String::from("i2e").into_bytes());
        let value: i64 = -2;
        assert_eq!(to_vec(&value).unwrap(), String::from("i-2e").into_bytes());
    }

    #[test]
    fn test_serialize_usize() {
        let value: usize = 2;
        assert_eq!(to_vec(&value).unwrap(), String::from("i2e").into_bytes());
    }

    #[test]
    fn test_serialize_u8() {
        let value: u8 = 2;
        assert_eq!(to_vec(&value).unwrap(), String::from("i2e").into_bytes());
    }

    #[test]
    fn test_serialize_u16() {
        let value: u16 = 2;
        assert_eq!(to_vec(&value).unwrap(), String::from("i2e").into_bytes());
    }

    #[test]
    fn test_serialize_u32() {
        let value: u32 = 2;
        assert_eq!(to_vec(&value).unwrap(), String::from("i2e").into_bytes());
    }

    #[test]
    fn test_serialize_u64() {
        let value: u64 = 2;
        assert_eq!(to_vec(&value).unwrap(), String::from("i2e").into_bytes());
    }

    #[test]
    fn test_serialize_u64_greater_than_i64_max() {
        let value: u64 = (i64::max_value() as u64) + 1;
        assert_eq!(to_vec(&value).unwrap(), format!("i{}e", value).into_bytes());
    }

    #[test]
    fn test_serialize_f32() {
        let value: f32 = 2.0;
        assert!(matches!(to_vec(&value), Err(Error::UnsupportedType)));
    }

    #[test]
    fn test_serialize_f64() {
        let value: f64 = 2.0;
        assert!(matches!(to_vec(&value), Err(Error::UnsupportedType)));
    }

    #[test]
    fn test_serialize_char() {
        let value: char = 'a';
        assert_eq!(to_vec(&value).unwrap(), String::from("1:a").into_bytes());
    }

    #[test]
    fn test_serialize_str() {
        let value: &str = "Hello world!";
        assert_eq!(
            to_vec(&value).unwrap(),
            String::from("12:Hello world!").into_bytes()
        );
    }

    #[test]
    fn test_serialize_empty_str() {
        let value: &str = "";
        assert_eq!(to_vec(&value).unwrap(), String::from("0:").into_bytes());
    }

    #[test]
    fn test_serialize_bytes() {
        let value = ByteBuf::from(String::from("123").into_bytes());
        assert_eq!(to_vec(&&value).unwrap(), String::from("3:123").into_bytes());
    }

    #[test]
    fn test_serialize_unit() {
        assert!(matches!(to_vec(&()), Err(Error::UnsupportedType)));
    }

    #[test]
    fn test_serialize_none() {
        let value: Option<i64> = None;
        assert!(matches!(to_vec(&value), Err(Error::UnsupportedType)));
    }

    #[test]
    fn test_serialize_some() {
        let value: Option<i64> = Some(2);
        assert_eq!(to_vec(&value).unwrap(), String::from("i2e").into_bytes());
    }

    #[test]
    fn test_serialize_unit_struct() {
        use serde::Serializer;

        let mut writer = Vec::new();
        assert!(matches!(
            super::Serializer::new(&mut writer).serialize_unit_struct("Nothing"),
            Err(Error::UnsupportedType)
        ));
    }

    #[test]
    fn test_serialize_unit_variant() {
        use serde::Serializer;

        let mut writer = Vec::new();
        assert!(matches!(
            super::Serializer::new(&mut writer).serialize_unit_variant("Nothing", 0, "Case"),
            Err(Error::UnsupportedType)
        ));
    }

    #[test]
    fn test_serialize_newtype_struct() {
        use serde::Serializer;

        let mut writer = Vec::new();

        assert!(super::Serializer::new(&mut writer)
            .serialize_newtype_struct("Nothing", &2)
            .is_ok());

        assert_eq!(String::from_utf8(writer).unwrap(), "i2e");
    }

    #[test]
    fn test_serialize_newtype_variant() {
        use serde::Serializer;

        let mut writer = Vec::new();
        assert!(matches!(
            super::Serializer::new(&mut writer).serialize_unit_variant("Nothing", 0, "Case"),
            Err(Error::UnsupportedType)
        ));
    }

    #[test]
    fn test_serialize_seq() {
        let value: Vec<u8> = vec![1, 2, 3];
        assert_eq!(
            to_vec(&&value).unwrap(),
            String::from("li1ei2ei3ee").into_bytes()
        );
    }

    #[test]
    fn test_serialize_seq_empty() {
        let value: Vec<u8> = vec![];
        assert_eq!(to_vec(&&value).unwrap(), String::from("le").into_bytes());
    }

    #[test]
    fn test_serialize_tuple() {
        use serde::Serializer;

        let mut writer = Vec::new();
        assert!(matches!(
            super::Serializer::new(&mut writer).serialize_tuple(0),
            Err(Error::UnsupportedType)
        ));
    }

    #[test]
    fn test_serialize_tuple_struct() {
        use serde::Serializer;

        let mut writer = Vec::new();
        assert!(matches!(
            super::Serializer::new(&mut writer).serialize_tuple_struct("Tuple Struct", 2),
            Err(Error::UnsupportedType)
        ));
    }

    #[test]
    fn test_serialize_tuple_variant() {
        use serde::Serializer;

        let mut writer = Vec::new();
        assert!(matches!(
            super::Serializer::new(&mut writer).serialize_tuple_variant(
                "Tuple Variant",
                2,
                "Case",
                1
            ),
            Err(Error::UnsupportedType)
        ));
    }

    #[test]
    fn test_serialize_struct_variant() {
        use serde::Serializer;

        let mut writer = Vec::new();
        assert!(matches!(
            super::Serializer::new(&mut writer).serialize_struct_variant(
                "Struct Variant",
                2,
                "Case",
                1
            ),
            Err(Error::UnsupportedType)
        ));
    }

    #[test]
    fn test_serialize_struct() {
        use serde_derive::Serialize;

        #[derive(Serialize)]
        struct Test {
            int: u32,
            s: String,
        }

        let test = Test {
            int: 3,
            s: String::from("Hello, World!"),
        };
        assert_eq!(
            to_vec(&test).unwrap(),
            String::from("d3:inti3e1:s13:Hello, World!e").into_bytes()
        );
    }
}
