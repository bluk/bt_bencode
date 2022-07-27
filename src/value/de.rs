//! Deserializes from a [Value].

use super::{Number, Value};
use crate::error::Error;
use serde::de::{
    DeserializeOwned, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor,
};
use serde::forward_to_deserialize_any;
use serde_bytes::ByteBuf;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{borrow::Cow, collections::BTreeMap, vec};
use core::slice;
#[cfg(feature = "std")]
use std::{borrow::Cow, collections::BTreeMap, vec};

/// Deserializes an instance of `T` from a [Value].
///
/// # Errors
///
/// Deserialization can fail if the data is not valid, if the data cannot cannot be deserialized
/// into an instance of `T`, and other IO errors.
pub fn from_value<T>(value: Value) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    T::deserialize(value)
}

impl<'de> serde::Deserializer<'de> for Value {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::ByteStr(s) => visitor.visit_byte_buf(s.into_vec()),
            Value::Int(n) => match n {
                Number::Signed(s) => visitor.visit_i64(s),
                Number::Unsigned(u) => visitor.visit_u64(u),
            },
            Value::List(l) => {
                let len = l.len();

                let mut deserializer = ListDeserializer {
                    iter: l.into_iter(),
                };
                let seq = visitor.visit_seq(&mut deserializer)?;
                if deserializer.iter.len() == 0 {
                    Ok(seq)
                } else {
                    Err(serde::de::Error::invalid_length(
                        len,
                        &"expected more elements to be consumed in list",
                    ))
                }
            }
            Value::Dict(d) => {
                let len = d.len();
                let mut deserializer = DictDeserializer {
                    iter: d.into_iter(),
                    value: None,
                };
                let map = visitor.visit_map(&mut deserializer)?;
                if deserializer.iter.len() == 0 {
                    Ok(map)
                } else {
                    Err(serde::de::Error::invalid_length(
                        len,
                        &"expected more elements to be consumed in dict",
                    ))
                }
            }
        }
    }

    forward_to_deserialize_any! {
        bool f32 f64 unit unit_struct

        i8 i16 i32 i64
        u8 u16 u32 u64

        char str string bytes byte_buf

        seq map

        struct enum identifier ignored_any
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<'de> IntoDeserializer<'de, Error> for Value {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

struct ListDeserializer {
    iter: vec::IntoIter<Value>,
}

impl<'de> SeqAccess<'de> for ListDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct DictDeserializer {
    iter: <BTreeMap<ByteBuf, Value> as IntoIterator>::IntoIter,
    value: Option<Value>,
}

impl<'de> MapAccess<'de> for DictDeserializer {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let key_de = DictKey {
                    key: Cow::Owned(key),
                };
                seed.deserialize(key_de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct DictKey<'a> {
    key: Cow<'a, ByteBuf>,
}

impl<'de> serde::Deserializer<'de> for DictKey<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.key {
            Cow::Borrowed(bytes) => visitor.visit_borrowed_bytes(bytes),
            Cow::Owned(bytes) => visitor.visit_byte_buf(bytes.into_vec()),
        }
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> serde::Deserializer<'de> for &'de Value {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::ByteStr(bytes) => visitor.visit_borrowed_bytes(bytes),
            Value::Int(n) => match n {
                Number::Signed(s) => visitor.visit_i64(*s),
                Number::Unsigned(u) => visitor.visit_u64(*u),
            },
            Value::List(l) => {
                let len = l.len();

                let mut deserializer = ListRefDeserializer { iter: l.iter() };

                let seq = visitor.visit_seq(&mut deserializer)?;
                if deserializer.iter.len() == 0 {
                    Ok(seq)
                } else {
                    Err(serde::de::Error::invalid_length(
                        len,
                        &"expected more elements to be consumed in list",
                    ))
                }
            }
            Value::Dict(d) => {
                let len = d.len();
                let mut deserializer = DictRefDeserializer {
                    iter: d.iter(),
                    value: None,
                };

                let map = visitor.visit_map(&mut deserializer)?;
                if deserializer.iter.len() == 0 {
                    Ok(map)
                } else {
                    Err(serde::de::Error::invalid_length(
                        len,
                        &"expected more elements to be consumed in dict",
                    ))
                }
            }
        }
    }

    forward_to_deserialize_any! {
        bool f32 f64 unit unit_struct

        i8 i16 i32 i64
        u8 u16 u32 u64

        char str string bytes byte_buf

        seq map

        struct enum identifier ignored_any
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    #[inline]
    fn is_human_readable(&self) -> bool {
        false
    }
}

struct ListRefDeserializer<'a> {
    iter: slice::Iter<'a, Value>,
}

impl<'a> SeqAccess<'a> for ListRefDeserializer<'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'a>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct DictRefDeserializer<'a> {
    iter: <&'a BTreeMap<ByteBuf, Value> as IntoIterator>::IntoIter,
    value: Option<&'a Value>,
}

impl<'a> MapAccess<'a> for DictRefDeserializer<'a> {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'a>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let key_de = DictKey {
                    key: Cow::Borrowed(key),
                };
                seed.deserialize(key_de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'a>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    use alloc::{string::String, vec, vec::Vec};
    #[cfg(feature = "std")]
    use std::{string::String, vec, vec::Vec};

    #[test]
    fn test_deserialize_string() -> Result<()> {
        let v = Value::ByteStr(ByteBuf::from(String::from("spam")));
        let s: String = from_value(v)?;
        assert_eq!("spam", s);
        Ok(())
    }

    #[test]
    fn test_deserialize_byte_str() -> Result<()> {
        let v = Value::ByteStr(ByteBuf::from(String::from("spam")));
        let b: ByteBuf = from_value(v)?;
        assert_eq!(ByteBuf::from(String::from("spam")), b);
        Ok(())
    }

    #[test]
    fn test_deserialize_integer_1() -> Result<()> {
        let v = Value::Int(Number::Unsigned(3));
        let i: u64 = from_value(v)?;
        assert_eq!(i, 3);
        Ok(())
    }

    #[test]
    fn test_deserialize_integer_2() -> Result<()> {
        let v = Value::Int(Number::Signed(-3));
        let i: i64 = from_value(v)?;
        assert_eq!(i, -3);
        Ok(())
    }

    #[test]
    fn test_deserialize_integer_3() -> Result<()> {
        let v = Value::Int(Number::Unsigned(0));
        let i: u64 = from_value(v)?;
        assert_eq!(i, 0);
        Ok(())
    }

    #[test]
    fn test_deserialize_list() -> Result<()> {
        let v = Value::List(vec![
            Value::ByteStr(ByteBuf::from(String::from("spam"))),
            Value::ByteStr(ByteBuf::from(String::from("eggs"))),
        ]);
        let v: Vec<String> = from_value(v)?;
        assert_eq!(v, vec!["spam", "eggs"]);
        Ok(())
    }

    #[test]
    fn test_deserialize_dict_1() -> Result<()> {
        let mut m = BTreeMap::new();
        m.insert(
            ByteBuf::from(String::from("cow")),
            Value::ByteStr(ByteBuf::from(String::from("moo"))),
        );
        m.insert(
            ByteBuf::from(String::from("spam")),
            Value::ByteStr(ByteBuf::from(String::from("eggs"))),
        );
        let d = Value::Dict(m);
        let d: BTreeMap<String, String> = from_value(d)?;

        let mut expected = BTreeMap::new();
        expected.insert(String::from("cow"), String::from("moo"));
        expected.insert(String::from("spam"), String::from("eggs"));
        assert_eq!(d, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_dict_1_borrowed_value() -> Result<()> {
        use serde::Deserialize;

        let mut m = BTreeMap::new();
        m.insert(
            ByteBuf::from(String::from("cow")),
            Value::ByteStr(ByteBuf::from(String::from("moo"))),
        );
        m.insert(
            ByteBuf::from(String::from("spam")),
            Value::ByteStr(ByteBuf::from(String::from("eggs"))),
        );
        let d = Value::Dict(m);
        let d = BTreeMap::<&str, &str>::deserialize(&d)?;

        let mut expected = BTreeMap::new();
        expected.insert("cow", "moo");
        expected.insert("spam", "eggs");
        assert_eq!(d, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_dict_2() -> Result<()> {
        let mut m = BTreeMap::new();
        m.insert(
            ByteBuf::from(String::from("spam")),
            Value::List(vec![
                Value::ByteStr(ByteBuf::from(String::from("a"))),
                Value::ByteStr(ByteBuf::from(String::from("b"))),
            ]),
        );
        let d = Value::Dict(m);
        let d: BTreeMap<String, Vec<String>> = from_value(d)?;

        let mut expected = BTreeMap::new();
        expected.insert(
            String::from("spam"),
            vec![String::from("a"), String::from("b")],
        );
        assert_eq!(d, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_dict_2_borrowed_value() -> Result<()> {
        use serde::Deserialize;

        let mut m = BTreeMap::new();
        m.insert(
            ByteBuf::from(String::from("spam")),
            Value::List(vec![
                Value::ByteStr(ByteBuf::from(String::from("a"))),
                Value::ByteStr(ByteBuf::from(String::from("b"))),
            ]),
        );
        let d = Value::Dict(m);
        let d = BTreeMap::<&str, Vec<&str>>::deserialize(&d)?;

        let mut expected = BTreeMap::new();
        expected.insert("spam", vec!["a", "b"]);
        assert_eq!(d, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_dict_2_borrowed_value_as_bytes() -> Result<()> {
        use serde::Deserialize;

        let mut m = BTreeMap::new();
        m.insert(
            ByteBuf::from(String::from("spam")),
            Value::List(vec![
                Value::ByteStr(ByteBuf::from(String::from("a"))),
                Value::ByteStr(ByteBuf::from(String::from("b"))),
            ]),
        );
        let d = Value::Dict(m);
        let d = BTreeMap::<&str, Vec<&[u8]>>::deserialize(&d)?;

        let mut expected = BTreeMap::new();
        expected.insert("spam", vec!["a".as_bytes(), "b".as_bytes()]);
        assert_eq!(d, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_dict_3() -> Result<()> {
        let mut m = BTreeMap::new();
        m.insert(
            ByteBuf::from(String::from("spam")),
            Value::List(vec![
                Value::ByteStr(ByteBuf::from(String::from("a"))),
                Value::ByteStr(ByteBuf::from(String::from("b"))),
            ]),
        );
        let d = Value::Dict(m);
        let d: BTreeMap<String, Value> = from_value(d)?;

        let mut expected = BTreeMap::new();
        expected.insert(
            String::from("spam"),
            Value::List(vec![
                Value::ByteStr(ByteBuf::from(String::from("a"))),
                Value::ByteStr(ByteBuf::from(String::from("b"))),
            ]),
        );
        assert_eq!(d, expected);
        Ok(())
    }

    #[test]
    fn test_deserialize_dict_4() -> Result<()> {
        let mut m = BTreeMap::new();
        m.insert(
            ByteBuf::from(String::from("spam")),
            Value::List(vec![
                Value::ByteStr(ByteBuf::from(String::from("a"))),
                Value::ByteStr(ByteBuf::from(String::from("b"))),
            ]),
        );
        let d = Value::Dict(m);
        let d: BTreeMap<String, Vec<Value>> = from_value(d)?;

        let mut expected = BTreeMap::new();
        expected.insert(
            String::from("spam"),
            vec![
                Value::ByteStr(ByteBuf::from(String::from("a"))),
                Value::ByteStr(ByteBuf::from(String::from("b"))),
            ],
        );
        assert_eq!(d, expected);
        Ok(())
    }
}
