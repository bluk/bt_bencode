//! Serializes Bencode data.

use crate::error::{Error, ErrorKind, Result};
use serde::{ser, Serialize};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{collections::BTreeMap, vec::Vec};

#[cfg(feature = "std")]
use std::{collections::BTreeMap, io, vec::Vec};

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
    let mut ser = Serializer::new(crate::write::IoWrite::new(writer));
    value.serialize(&mut ser)?;
    Ok(())
}

/// Serializes an instance of `T` into a new [Vec] as `Bencode` data.
///
/// # Errors
///
/// Serialization can fail if `T`'s implementation of
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
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = ser::Impossible<(), Error>;
    type SerializeMap = Compound<'a, W>;
    type SerializeStruct = Compound<'a, W>;
    type SerializeStructVariant = ser::Impossible<(), Error>;

    #[inline]
    fn serialize_bool(self, _value: bool) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
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
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    #[inline]
    fn serialize_f64(self, _value: f64) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
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
        Err(Error::with_kind(ErrorKind::UnsupportedType))
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
        Err(Error::with_kind(ErrorKind::UnsupportedType))
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
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    #[inline]
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.writer.write_all(b"l")?;
        Ok(self)
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    #[inline]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.writer.write_all(b"d")?;
        Ok(Compound::new_map(self))
    }

    #[inline]
    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        #[cfg(feature = "raw_value")]
        if name == crate::raw::TOKEN {
            return Ok(Compound::RawValue { ser: self });
        }
        let _ = name;
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
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<W> ser::SerializeSeq for &mut Serializer<W>
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

impl<W> ser::SerializeTuple for &mut Serializer<W>
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

impl<W> ser::SerializeTupleStruct for &mut Serializer<W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
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
pub enum Compound<'a, W> {
    Map {
        ser: &'a mut Serializer<W>,
        entries: BTreeMap<Vec<u8>, Vec<u8>>,
        current_key: Option<Vec<u8>>,
    },
    #[cfg(feature = "raw_value")]
    RawValue {
        ser: &'a mut Serializer<W>,
    },
}

impl<'a, W> Compound<'a, W>
where
    W: Write,
{
    #[inline]
    fn new_map(ser: &'a mut Serializer<W>) -> Self {
        Compound::Map {
            ser,
            entries: BTreeMap::new(),
            current_key: None,
        }
    }
}

impl<W> ser::SerializeMap for Compound<'_, W>
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
        match self {
            Compound::Map { current_key, .. } => {
                if current_key.is_some() {
                    return Err(Error::with_kind(ErrorKind::KeyWithoutValue));
                }
                *current_key = Some(key.serialize(&mut MapKeySerializer {})?);
                Ok(())
            }
            #[cfg(feature = "raw_value")]
            Compound::RawValue { .. } => unreachable!(),
        }
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self {
            Compound::Map {
                entries,
                current_key,
                ..
            } => {
                let key = current_key
                    .take()
                    .ok_or_else(|| Error::with_kind(ErrorKind::ValueWithoutKey))?;
                let buf: Vec<u8> = Vec::new();
                let mut ser = Serializer::new(buf); // TODO: optimize?
                value.serialize(&mut ser)?;
                entries.insert(key, ser.into_inner());
                Ok(())
            }
            #[cfg(feature = "raw_value")]
            Compound::RawValue { .. } => unreachable!(),
        }
    }

    #[inline]
    fn end(mut self) -> Result<()> {
        match self {
            Compound::Map {
                ref mut ser,
                entries,
                current_key,
            } => {
                if current_key.is_some() {
                    return Err(Error::with_kind(ErrorKind::KeyWithoutValue));
                }

                for (k, v) in entries {
                    ser::Serializer::serialize_bytes(&mut **ser, k.as_ref())?; // TODO: why
                    ser.writer.write_all(&v)?;
                }
                ser.writer.write_all(b"e")?;
                Ok(())
            }
            #[cfg(feature = "raw_value")]
            Compound::RawValue { .. } => unreachable!(),
        }
    }
}

impl<W> ser::SerializeStruct for Compound<'_, W>
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
        match self {
            Compound::Map { entries, .. } => {
                let key = key.serialize(&mut MapKeySerializer {})?;

                let buf: Vec<u8> = Vec::new();
                let mut ser = Serializer::new(buf);
                value.serialize(&mut ser)?;
                entries.insert(key, ser.into_inner());
                Ok(())
            }
            #[cfg(feature = "raw_value")]
            Compound::RawValue { ser } => {
                if key == crate::raw::TOKEN {
                    value.serialize(&mut RawValueSerializer::new(&mut ser.writer))
                } else {
                    Err(Error::with_kind(ErrorKind::UnsupportedType))
                }
            }
        }
    }

    #[inline]
    fn end(self) -> Result<()> {
        match self {
            Compound::Map { .. } => ser::SerializeMap::end(self),
            #[cfg(feature = "raw_value")]
            Compound::RawValue { .. } => Ok(()),
        }
    }
}

struct MapKeySerializer;

impl ser::Serializer for &mut MapKeySerializer {
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
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_i8(self, _value: i8) -> Result<Vec<u8>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_i16(self, _value: i16) -> Result<Vec<u8>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_i32(self, _value: i32) -> Result<Vec<u8>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_i64(self, _value: i64) -> Result<Vec<u8>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_u8(self, _value: u8) -> Result<Vec<u8>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_u16(self, _value: u16) -> Result<Vec<u8>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_u32(self, _value: u32) -> Result<Vec<u8>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_u64(self, _value: u64) -> Result<Vec<u8>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_f32(self, _value: f32) -> Result<Vec<u8>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_f64(self, _value: f64) -> Result<Vec<u8>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
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
        Err(Error::with_kind(ErrorKind::UnsupportedType))
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
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Vec<u8>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Vec<u8>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_none(self) -> Result<Vec<u8>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Vec<u8>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<ser::Impossible<Vec<u8>, Error>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_tuple(self, _size: usize) -> Result<ser::Impossible<Vec<u8>, Error>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<ser::Impossible<Vec<u8>, Error>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<ser::Impossible<Vec<u8>, Error>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<ser::Impossible<Vec<u8>, Error>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<ser::Impossible<Vec<u8>, Error>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<ser::Impossible<Vec<u8>, Error>> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }
}

#[cfg(feature = "raw_value")]
struct RawValueSerializer<'a, W> {
    writer: &'a mut W,
}

#[cfg(feature = "raw_value")]
impl<'a, W> RawValueSerializer<'a, W>
where
    W: Write,
{
    pub(crate) fn new(writer: &'a mut W) -> Self {
        RawValueSerializer { writer }
    }
}

#[cfg(feature = "raw_value")]
impl<W> ser::Serializer for &mut RawValueSerializer<'_, W>
where
    W: Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = ser::Impossible<(), Error>;
    type SerializeTuple = ser::Impossible<(), Error>;
    type SerializeTupleStruct = ser::Impossible<(), Error>;
    type SerializeTupleVariant = ser::Impossible<(), Error>;
    type SerializeMap = ser::Impossible<(), Error>;
    type SerializeStruct = ser::Impossible<(), Error>;
    type SerializeStructVariant = ser::Impossible<(), Error>;

    fn serialize_bool(self, _value: bool) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_i8(self, _value: i8) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_i16(self, _value: i16) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_i32(self, _value: i32) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_i64(self, _value: i64) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_u8(self, _value: u8) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_u16(self, _value: u16) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_u32(self, _value: u32) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_u64(self, _value: u64) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_f32(self, _value: f32) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_f64(self, _value: f64) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_char(self, _value: char) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_str(self, value: &str) -> Result<()> {
        self.writer.write_all(value.as_bytes())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        self.writer.write_all(value)
    }

    fn serialize_unit(self) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_none(self) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<()> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::with_kind(ErrorKind::UnsupportedType))
    }
}

#[cfg(test)]
mod tests {
    use crate::ByteString;

    use super::*;

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    use alloc::{format, string::String, vec};
    #[cfg(feature = "std")]
    use std::string::String;

    macro_rules! assert_is_unsupported_type {
        ($e:expr) => {
            match $e {
                Ok(_) => unreachable!(),
                Err(error) => match error.kind() {
                    ErrorKind::UnsupportedType => {}
                    _ => panic!("wrong error type"),
                },
            }
        };
    }

    #[test]
    fn test_serialize_bool() {
        assert_is_unsupported_type!(to_vec(&true));
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
        assert_is_unsupported_type!(to_vec(&value));
    }

    #[test]
    fn test_serialize_f64() {
        let value: f64 = 2.0;
        assert_is_unsupported_type!(to_vec(&value));
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
        let value = ByteString::from(String::from("123").into_bytes());
        assert_eq!(to_vec(&&value).unwrap(), String::from("3:123").into_bytes());
    }

    #[test]
    fn test_serialize_unit() {
        assert_is_unsupported_type!(to_vec(&()));
    }

    #[test]
    fn test_serialize_none() {
        let value: Option<i64> = None;
        assert_is_unsupported_type!(to_vec(&value));
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
        assert_is_unsupported_type!(
            super::Serializer::new(&mut writer).serialize_unit_struct("Nothing")
        );
    }

    #[test]
    fn test_serialize_unit_variant() {
        use serde::Serializer;

        let mut writer = Vec::new();
        assert_is_unsupported_type!(
            super::Serializer::new(&mut writer).serialize_unit_variant("Nothing", 0, "Case")
        );
    }

    #[test]
    fn test_serialize_newtype_struct() {
        use serde::Serializer;

        let mut writer = Vec::new();

        super::Serializer::new(&mut writer)
            .serialize_newtype_struct("Nothing", &2)
            .unwrap();

        assert_eq!(String::from_utf8(writer).unwrap(), "i2e");
    }

    #[test]
    fn test_serialize_newtype_variant() {
        use serde::Serializer;

        let mut writer = Vec::new();
        assert_is_unsupported_type!(
            super::Serializer::new(&mut writer).serialize_unit_variant("Nothing", 0, "Case")
        );
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
        assert_eq!(
            to_vec(&(201, "spam")).unwrap(),
            String::from("li201e4:spame").into_bytes()
        );
    }

    #[test]
    fn test_serialize_tuple_struct() {
        #[derive(serde_derive::Serialize)]
        struct S<'a>(i64, &'a str);

        assert_eq!(
            to_vec(&S(201, "spam")).unwrap(),
            String::from("li201e4:spame").into_bytes()
        );
    }

    #[test]
    fn test_serialize_tuple_variant() {
        use serde::Serializer;

        let mut writer = Vec::new();
        assert_is_unsupported_type!(super::Serializer::new(&mut writer).serialize_tuple_variant(
            "Tuple Variant",
            2,
            "Case",
            1
        ));
    }

    #[test]
    fn test_serialize_struct_variant() {
        use serde::Serializer;

        let mut writer = Vec::new();
        assert_is_unsupported_type!(
            super::Serializer::new(&mut writer).serialize_struct_variant(
                "Struct Variant",
                2,
                "Case",
                1
            )
        );
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

    #[cfg(feature = "raw_value")]
    #[test]
    fn test_encode_raw_value_top_level() {
        use crate::RawValue;

        let raw = RawValue::from_slice(b"d3:fooi1e3:bar4:spam3:bazli2ei3eee");
        let encoded = to_vec(&raw).unwrap();
        assert_eq!(encoded, raw.get());
    }

    #[cfg(feature = "raw_value")]
    #[test]
    fn test_encode_raw_value_struct_field() {
        use crate::RawValue;
        use serde_derive::Serialize;

        #[derive(Serialize)]
        struct Envelope<'a> {
            payload: &'a RawValue,
        }

        let payload = RawValue::from_slice(b"d1:ai1e1:bi2ee");
        let encoded = to_vec(&Envelope { payload: &payload }).unwrap();
        assert_eq!(encoded, b"d7:payloadd1:ai1e1:bi2eee");
    }
}
