use crate::error::Error;
use serde::{
    de::{Deserialize, MapAccess, SeqAccess, Visitor},
    ser::Serialize,
};
use serde_bytes::ByteBuf;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{collections::BTreeMap, fmt, str, str::FromStr, string::String, vec::Vec};
#[cfg(feature = "std")]
use std::{collections::BTreeMap, fmt, str, str::FromStr, string::String, vec::Vec};

/// A Bencoded number.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Number {
    Signed(i64),
    Unsigned(u64),
}

/// Represents valid untyped data.
///
/// It is useful when it is unknown what the data may contain (e.g. when different kinds of
/// messages can be received in a network packet).
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Value {
    /// A byte string.
    ///
    /// Encoded strings can contain non-UTF-8 bytes, so a byte string is used to represent
    /// "strings".
    ByteStr(ByteBuf),
    /// An integer which can be signed or unsigned.
    Int(Number),
    /// A list of values.
    List(Vec<Value>),
    /// A dictionary of values.
    Dict(BTreeMap<ByteBuf, Value>),
}

impl Value {
    pub fn as_byte_str(&self) -> Option<&ByteBuf> {
        match self {
            Value::ByteStr(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_byte_str_mut(&mut self) -> Option<&mut ByteBuf> {
        match self {
            Value::ByteStr(ref mut b) => Some(b),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::ByteStr(b) => str::from_utf8(b.as_slice()).ok(),
            _ => None,
        }
    }

    pub fn as_str_mut(&mut self) -> Option<&mut str> {
        match self {
            Value::ByteStr(ref mut b) => str::from_utf8_mut(b.as_mut_slice()).ok(),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Int(n) => match n {
                Number::Unsigned(n) => Some(*n),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(n) => match n {
                Number::Signed(n) => Some(*n),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::List(ref l) => Some(l),
            _ => None,
        }
    }

    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Value::List(ref mut l) => Some(l),
            _ => None,
        }
    }

    pub fn as_dict(&self) -> Option<&BTreeMap<ByteBuf, Value>> {
        match self {
            Value::Dict(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_dict_mut(&mut self) -> Option<&mut BTreeMap<ByteBuf, Value>> {
        match self {
            Value::Dict(ref mut d) => Some(d),
            _ => None,
        }
    }

    pub fn is_byte_str(&self) -> bool {
        self.as_byte_str().is_some()
    }

    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    pub fn is_u64(&self) -> bool {
        self.as_u64().is_some()
    }

    pub fn is_i64(&self) -> bool {
        self.as_i64().is_some()
    }

    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    pub fn is_dict(&self) -> bool {
        self.as_dict().is_some()
    }
}

impl From<i8> for Value {
    fn from(other: i8) -> Value {
        Value::Int(Number::Signed(other as i64))
    }
}

impl From<i16> for Value {
    fn from(other: i16) -> Value {
        Value::Int(Number::Signed(other as i64))
    }
}

impl From<i32> for Value {
    fn from(other: i32) -> Value {
        Value::Int(Number::Signed(other as i64))
    }
}

impl From<i64> for Value {
    fn from(other: i64) -> Value {
        Value::Int(Number::Signed(other))
    }
}

impl From<isize> for Value {
    fn from(other: isize) -> Value {
        Value::Int(Number::Signed(other as i64))
    }
}

impl From<u8> for Value {
    fn from(other: u8) -> Value {
        Value::Int(Number::Unsigned(other as u64))
    }
}

impl From<u16> for Value {
    fn from(other: u16) -> Value {
        Value::Int(Number::Unsigned(other as u64))
    }
}

impl From<u32> for Value {
    fn from(other: u32) -> Value {
        Value::Int(Number::Unsigned(other as u64))
    }
}

impl From<u64> for Value {
    fn from(other: u64) -> Value {
        Value::Int(Number::Unsigned(other))
    }
}

impl From<usize> for Value {
    fn from(other: usize) -> Value {
        Value::Int(Number::Unsigned(other as u64))
    }
}

impl FromStr for Value {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Value::ByteStr(ByteBuf::from(String::from(s))))
    }
}

impl<'a> From<&'a str> for Value {
    fn from(other: &'a str) -> Value {
        Value::ByteStr(ByteBuf::from(other))
    }
}

impl From<String> for Value {
    fn from(other: String) -> Value {
        Value::ByteStr(ByteBuf::from(other))
    }
}

impl<V: Into<Value>> From<Vec<V>> for Value {
    fn from(other: Vec<V>) -> Value {
        Value::List(other.into_iter().map(|v| v.into()).collect())
    }
}

impl<K: Into<ByteBuf>, V: Into<Value>> From<BTreeMap<K, V>> for Value {
    fn from(other: BTreeMap<K, V>) -> Value {
        Value::Dict(
            other
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }
}

impl<'de> Deserialize<'de> for Value {
    #[inline]
    fn deserialize<T>(deserializer: T) -> Result<Value, T::Error>
    where
        T: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("any valid Bencode value")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(Value::Int(Number::Signed(value)))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                Ok(Value::Int(Number::Unsigned(value)))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
                Ok(Value::ByteStr(ByteBuf::from(String::from(value))))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                Ok(Value::ByteStr(ByteBuf::from(value)))
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E> {
                Ok(Value::ByteStr(ByteBuf::from(value)))
            }

            fn visit_byte_buf<E>(self, value: Vec<u8>) -> Result<Self::Value, E> {
                Ok(Value::ByteStr(ByteBuf::from(value)))
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut list = Vec::new();
                while let Some(elem) = visitor.next_element()? {
                    list.push(elem);
                }
                Ok(Value::List(list))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut dict = BTreeMap::new();
                while let Some((key, value)) = visitor.next_entry()? {
                    dict.insert(key, value);
                }
                Ok(Value::Dict(dict))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::ByteStr(ref b) => b.serialize(serializer),
            Value::Int(i) => match i {
                Number::Signed(s) => s.serialize(serializer),
                Number::Unsigned(u) => u.serialize(serializer),
            },
            Value::List(l) => l.serialize(serializer),
            Value::Dict(d) => d.serialize(serializer),
        }
    }
}

mod de;
mod ser;

pub use de::from_value;
pub use ser::to_value;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    use alloc::vec;
    #[cfg(feature = "std")]
    use std::vec;

    #[test]
    fn test_deserialize_string() -> Result<()> {
        let input = "4:spam";
        let v: Value = crate::de::from_slice(input.as_bytes())?;
        assert_eq!(v, Value::ByteStr(ByteBuf::from(String::from("spam"))));
        Ok(())
    }

    #[test]
    fn test_deserialize_integer_1() -> Result<()> {
        let input = "i3e";
        let v: Value = crate::de::from_slice(input.as_bytes())?;
        assert_eq!(v, Value::Int(Number::Unsigned(3)));
        Ok(())
    }

    #[test]
    fn test_deserialize_integer_2() -> Result<()> {
        let input = "i-3e";
        let v: Value = crate::de::from_slice(input.as_bytes())?;
        assert_eq!(v, Value::Int(Number::Signed(-3)));
        Ok(())
    }

    #[test]
    fn test_deserialize_integer_3() -> Result<()> {
        let input = "i0e";
        let v: Value = crate::de::from_slice(input.as_bytes())?;
        assert_eq!(v, Value::Int(Number::Unsigned(0)));
        Ok(())
    }

    #[test]
    fn test_deserialize_list() -> Result<()> {
        let input = "l4:spam4:eggse";
        let v: Value = crate::de::from_slice(input.as_bytes())?;
        assert_eq!(
            v,
            Value::List(vec![
                Value::ByteStr(ByteBuf::from(String::from("spam"))),
                Value::ByteStr(ByteBuf::from(String::from("eggs"))),
            ])
        );
        Ok(())
    }

    #[test]
    fn test_deserialize_dict_1() -> Result<()> {
        let input = "d3:cow3:moo4:spam4:eggse";
        let v: Value = crate::de::from_slice(input.as_bytes())?;

        let mut expected = BTreeMap::new();
        expected.insert(
            ByteBuf::from(String::from("cow")),
            Value::ByteStr(ByteBuf::from(String::from("moo"))),
        );
        expected.insert(
            ByteBuf::from(String::from("spam")),
            Value::ByteStr(ByteBuf::from(String::from("eggs"))),
        );
        assert_eq!(v, Value::Dict(expected));
        Ok(())
    }

    #[test]
    fn test_deserialize_dict_2() -> Result<()> {
        let input = "d4:spaml1:a1:bee";
        let v: Value = crate::de::from_slice(input.as_bytes())?;
        let mut expected = BTreeMap::new();
        expected.insert(
            ByteBuf::from(String::from("spam")),
            Value::List(vec![
                Value::ByteStr(ByteBuf::from(String::from("a"))),
                Value::ByteStr(ByteBuf::from(String::from("b"))),
            ]),
        );
        assert_eq!(v, Value::Dict(expected));
        Ok(())
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_serialize_string() -> Result<()> {
        let expected = "4:spam";
        let v: Vec<u8> = crate::ser::to_vec(&Value::ByteStr(ByteBuf::from(String::from("spam"))))?;
        assert_eq!(v, expected.to_string().into_bytes());
        Ok(())
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_serialize_integer_1() -> Result<()> {
        let expected = "i3e";
        let v: Vec<u8> = crate::ser::to_vec(&Value::Int(Number::Unsigned(3)))?;
        assert_eq!(v, expected.to_string().into_bytes());
        Ok(())
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_serialize_integer_2() -> Result<()> {
        let expected = "i-3e";
        let v: Vec<u8> = crate::ser::to_vec(&Value::Int(Number::Signed(-3)))?;
        assert_eq!(v, expected.to_string().into_bytes());
        Ok(())
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_serialize_integer_3() -> Result<()> {
        let expected = "i0e";
        let v: Vec<u8> = crate::ser::to_vec(&Value::Int(Number::Unsigned(0)))?;
        assert_eq!(v, expected.to_string().into_bytes());
        Ok(())
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_serialize_list() -> Result<()> {
        let expected = "l4:spam4:eggse";
        let v: Vec<u8> = crate::ser::to_vec(&Value::List(vec![
            Value::ByteStr(ByteBuf::from(String::from("spam"))),
            Value::ByteStr(ByteBuf::from(String::from("eggs"))),
        ]))?;
        assert_eq!(v, expected.to_string().into_bytes());
        Ok(())
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_serialize_dict_1() -> Result<()> {
        let expected = "d3:cow3:moo4:spam4:eggse";
        let mut dict = BTreeMap::new();
        dict.insert(
            ByteBuf::from(String::from("cow")),
            Value::ByteStr(ByteBuf::from(String::from("moo"))),
        );
        dict.insert(
            ByteBuf::from(String::from("spam")),
            Value::ByteStr(ByteBuf::from(String::from("eggs"))),
        );
        let v: Vec<u8> = crate::ser::to_vec(&Value::Dict(dict))?;
        assert_eq!(v, expected.to_string().into_bytes());
        Ok(())
    }

    #[test]
    #[cfg(feature = "std")]
    fn test_serialize_dict_2() -> Result<()> {
        let expected = "d4:spaml1:a1:bee";
        let mut dict = BTreeMap::new();
        dict.insert(
            ByteBuf::from(String::from("spam")),
            Value::List(vec![
                Value::ByteStr(ByteBuf::from(String::from("a"))),
                Value::ByteStr(ByteBuf::from(String::from("b"))),
            ]),
        );
        let v: Vec<u8> = crate::ser::to_vec(&Value::Dict(dict))?;
        assert_eq!(v, expected.to_string().into_bytes());
        Ok(())
    }
}
