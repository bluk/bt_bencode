//! Serializes into a [Value].

use super::{Number, Value};
use crate::error::{Error, Result};
use serde::{ser, Serialize};
use serde_bytes::ByteBuf;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{collections::BTreeMap, vec::Vec};
#[cfg(feature = "std")]
use std::{collections::BTreeMap, vec::Vec};

/// Serializes an instance of `T` into a [Value].
///
/// # Errors
///
/// Serialization can fail if `T`'s implementation of
/// [Serialize][serde::ser::Serialize] decides to fail, if `T` contains
/// unsupported types for serialization, or if `T` contains a map with
/// non-string keys.
#[inline]
pub fn to_value<T>(value: &T) -> Result<Value>
where
    T: ?Sized + Serialize,
{
    value.serialize(Serializer)
}

#[derive(Debug)]
struct Serializer;

impl ser::Serializer for Serializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SerializeList;
    type SerializeTuple = ser::Impossible<Self::Ok, Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Error>;
    type SerializeMap = SerializeDict;
    type SerializeStruct = SerializeDict;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Error>;

    #[inline]
    fn serialize_bool(self, _value: bool) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Self::Ok> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Self::Ok> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Self::Ok> {
        self.serialize_i64(i64::from(value))
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<Self::Ok> {
        Ok(Value::Int(Number::Signed(value)))
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Self::Ok> {
        self.serialize_u64(u64::from(value))
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Self::Ok> {
        self.serialize_u64(u64::from(value))
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Self::Ok> {
        self.serialize_u64(u64::from(value))
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        Ok(Value::Int(Number::Unsigned(value)))
    }

    #[inline]
    fn serialize_f32(self, _value: f32) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_f64(self, _value: f64) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        let mut buf = [0; 4];
        self.serialize_str(value.encode_utf8(&mut buf))
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        Ok(Value::ByteStr(ByteBuf::from(value)))
    }

    #[inline]
    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        Ok(Value::ByteStr(ByteBuf::from(value)))
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
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
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::UnsupportedType)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeList {
            list: Vec::with_capacity(len.unwrap_or(0)),
        })
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
        Ok(SerializeDict {
            dict: BTreeMap::new(),
            current_key: None,
        })
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

struct SerializeList {
    list: Vec<Value>,
}

impl ser::SerializeSeq for SerializeList {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.list.push(to_value(value)?);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok> {
        Ok(Value::List(self.list))
    }
}

#[derive(Debug)]
struct SerializeDict {
    dict: BTreeMap<ByteBuf, Value>,
    current_key: Option<ByteBuf>,
}

impl ser::SerializeMap for SerializeDict {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if self.current_key.is_some() {
            return Err(Error::KeyWithoutValue);
        }
        self.current_key = Some(key.serialize(&mut DictKeySerializer)?);
        Ok(())
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = self.current_key.take().ok_or(Error::ValueWithoutKey)?;
        let value = to_value(value)?;
        self.dict.insert(key, value);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok> {
        Ok(Value::Dict(self.dict))
    }
}

impl ser::SerializeStruct for SerializeDict {
    type Ok = Value;
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = key.serialize(&mut DictKeySerializer)?;
        let value = to_value(value)?;
        self.dict.insert(key, value);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<Self::Ok> {
        Ok(Value::Dict(self.dict))
    }
}

#[derive(Debug)]
struct DictKeySerializer;

impl ser::Serializer for &mut DictKeySerializer {
    type Ok = ByteBuf;
    type Error = Error;

    type SerializeSeq = ser::Impossible<ByteBuf, Error>;
    type SerializeTuple = ser::Impossible<ByteBuf, Error>;
    type SerializeTupleStruct = ser::Impossible<ByteBuf, Error>;
    type SerializeTupleVariant = ser::Impossible<ByteBuf, Error>;
    type SerializeMap = ser::Impossible<ByteBuf, Error>;
    type SerializeStruct = ser::Impossible<ByteBuf, Error>;
    type SerializeStructVariant = ser::Impossible<ByteBuf, Error>;

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_i8(self, _value: i8) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_i16(self, _value: i16) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_i32(self, _value: i32) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_i64(self, _value: i64) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_u8(self, _value: u8) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_u16(self, _value: u16) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_u32(self, _value: u32) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_u64(self, _value: u64) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        let mut buf = [0; 4];
        self.serialize_str(value.encode_utf8(&mut buf))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        Ok(ByteBuf::from(value))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        Ok(ByteBuf::from(value))
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Self::Ok> {
        Err(Error::UnsupportedType)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<ser::Impossible<Self::Ok, Error>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_tuple(self, _size: usize) -> Result<ser::Impossible<Self::Ok, Error>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<ser::Impossible<Self::Ok, Error>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<ser::Impossible<Self::Ok, Error>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<ser::Impossible<Self::Ok, Error>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<ser::Impossible<Self::Ok, Error>> {
        Err(Error::UnsupportedType)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<ser::Impossible<Self::Ok, Error>> {
        Err(Error::UnsupportedType)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_bytes::ByteBuf;

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    use alloc::{string::String, vec};
    #[cfg(feature = "std")]
    use std::{string::String, vec};

    #[test]
    fn test_serialize_bool() {
        assert!(matches!(to_value(&true), Err(Error::UnsupportedType)));
    }

    #[test]
    fn test_serialize_isize() {
        let value: isize = 2;
        assert_eq!(to_value(&value).unwrap(), Value::Int(Number::Signed(2)));
        let value: isize = -2;
        assert_eq!(to_value(&value).unwrap(), Value::Int(Number::Signed(-2)));
    }

    #[test]
    fn test_serialize_i8() {
        let value: i8 = 2;
        assert_eq!(to_value(&value).unwrap(), Value::Int(Number::Signed(2)));
        let value: i8 = -2;
        assert_eq!(to_value(&value).unwrap(), Value::Int(Number::Signed(-2)));
    }

    #[test]
    fn test_serialize_i16() {
        let value: i16 = 2;
        assert_eq!(to_value(&value).unwrap(), Value::Int(Number::Signed(2)));
        let value: i16 = -2;
        assert_eq!(to_value(&value).unwrap(), Value::Int(Number::Signed(-2)));
    }

    #[test]
    fn test_serialize_i32() {
        let value: i32 = 2;
        assert_eq!(to_value(&value).unwrap(), Value::Int(Number::Signed(2)));
        let value: i32 = -2;
        assert_eq!(to_value(&value).unwrap(), Value::Int(Number::Signed(-2)));
    }

    #[test]
    fn test_serialize_i64() {
        let value: i64 = 2;
        assert_eq!(to_value(&value).unwrap(), Value::Int(Number::Signed(2)));
        let value: i64 = -2;
        assert_eq!(to_value(&value).unwrap(), Value::Int(Number::Signed(-2)));
    }

    #[test]
    fn test_serialize_usize() {
        let value: usize = 2;
        assert_eq!(to_value(&value).unwrap(), Value::Int(Number::Unsigned(2)));
    }

    #[test]
    fn test_serialize_u8() {
        let value: u8 = 2;
        assert_eq!(to_value(&value).unwrap(), Value::Int(Number::Unsigned(2)));
    }

    #[test]
    fn test_serialize_u16() {
        let value: u16 = 2;
        assert_eq!(to_value(&value).unwrap(), Value::Int(Number::Unsigned(2)));
    }

    #[test]
    fn test_serialize_u32() {
        let value: u32 = 2;
        assert_eq!(to_value(&value).unwrap(), Value::Int(Number::Unsigned(2)));
    }

    #[test]
    fn test_serialize_u64() {
        let value: u64 = 2;
        assert_eq!(to_value(&value).unwrap(), Value::Int(Number::Unsigned(2)));
    }

    #[test]
    fn test_serialize_u64_greater_than_i64_max() {
        let value: u64 = (i64::max_value() as u64) + 1;
        assert_eq!(
            to_value(&value).unwrap(),
            Value::Int(Number::Unsigned(value))
        );
    }

    #[test]
    fn test_serialize_f32() {
        let value: f32 = 2.0;
        assert!(matches!(to_value(&value), Err(Error::UnsupportedType)));
    }

    #[test]
    fn test_serialize_f64() {
        let value: f64 = 2.0;
        assert!(matches!(to_value(&value), Err(Error::UnsupportedType)));
    }

    #[test]
    fn test_serialize_char() {
        let value: char = 'a';
        assert_eq!(
            to_value(&value).unwrap(),
            Value::ByteStr(ByteBuf::from("a"))
        );
    }

    #[test]
    fn test_serialize_str() {
        let value: &str = "Hello world!";
        assert_eq!(
            to_value(&value).unwrap(),
            Value::ByteStr(ByteBuf::from(value))
        );
    }

    #[test]
    fn test_serialize_empty_str() {
        let value: &str = "";
        assert_eq!(
            to_value(&value).unwrap(),
            Value::ByteStr(ByteBuf::from(value))
        );
    }

    #[test]
    fn test_serialize_bytes() {
        let value = ByteBuf::from(String::from("123").into_bytes());
        assert_eq!(to_value(&&value).unwrap(), Value::ByteStr(value));
    }

    #[test]
    fn test_serialize_unit() {
        assert!(matches!(to_value(&()), Err(Error::UnsupportedType)));
    }

    #[test]
    fn test_serialize_none() {
        let value: Option<i64> = None;
        assert!(matches!(to_value(&value), Err(Error::UnsupportedType)));
    }

    #[test]
    fn test_serialize_some() {
        let value: Option<i64> = Some(2);
        assert_eq!(to_value(&value).unwrap(), Value::Int(Number::Signed(2)));
    }

    #[test]
    fn test_serialize_unit_struct() {
        use serde::Serializer;

        assert!(matches!(
            Serializer.serialize_unit_struct("Nothing"),
            Err(Error::UnsupportedType)
        ));
    }

    #[test]
    fn test_serialize_unit_variant() {
        use serde::Serializer;

        assert!(matches!(
            Serializer.serialize_unit_variant("Nothing", 0, "Case"),
            Err(Error::UnsupportedType)
        ));
    }

    #[test]
    fn test_serialize_newtype_struct() {
        use serde::Serializer;

        assert!(Serializer.serialize_newtype_struct("Nothing", &2).is_ok());
    }

    #[test]
    fn test_serialize_newtype_variant() {
        use serde::Serializer;

        assert!(matches!(
            Serializer.serialize_unit_variant("Nothing", 0, "Case"),
            Err(Error::UnsupportedType)
        ));
    }

    #[test]
    fn test_serialize_seq() {
        let value: Vec<u8> = vec![1, 2, 3];
        assert_eq!(
            to_value(&&value).unwrap(),
            Value::List(vec![
                Value::Int(Number::Unsigned(1)),
                Value::Int(Number::Unsigned(2)),
                Value::Int(Number::Unsigned(3)),
            ])
        );
    }

    #[test]
    fn test_serialize_seq_empty() {
        let value: Vec<u8> = vec![];
        assert_eq!(to_value(&&value).unwrap(), Value::List(Vec::new()));
    }

    #[test]
    fn test_serialize_tuple() {
        use serde::Serializer;

        assert!(matches!(
            Serializer.serialize_tuple(0),
            Err(Error::UnsupportedType)
        ));
    }

    #[test]
    fn test_serialize_tuple_struct() {
        use serde::Serializer;

        assert!(matches!(
            Serializer.serialize_tuple_struct("Tuple Struct", 2),
            Err(Error::UnsupportedType)
        ));
    }

    #[test]
    fn test_serialize_tuple_variant() {
        use serde::Serializer;

        assert!(matches!(
            Serializer.serialize_tuple_variant("Tuple Variant", 2, "Case", 1),
            Err(Error::UnsupportedType)
        ));
    }

    #[test]
    fn test_serialize_struct_variant() {
        use serde::Serializer;

        assert!(matches!(
            Serializer.serialize_struct_variant("Struct Variant", 2, "Case", 1),
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
        let mut expected = BTreeMap::new();
        expected.insert(
            ByteBuf::from(String::from("int")),
            Value::Int(Number::Unsigned(3)),
        );
        expected.insert(
            ByteBuf::from(String::from("s")),
            Value::ByteStr(ByteBuf::from(String::from("Hello, World!"))),
        );

        assert_eq!(to_value(&test).unwrap(), Value::Dict(expected));
    }
}
