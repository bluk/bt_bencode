use super::Value;
use core::ops;
use serde_bytes::ByteBuf;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::string::String;
#[cfg(feature = "std")]
use std::string::String;

pub trait Index {
    fn index<'a>(&self, v: &'a Value) -> Option<&'a Value>;

    fn index_mut<'a>(&self, v: &'a mut Value) -> Option<&'a mut Value>;
}

impl Index for usize {
    fn index<'a>(&self, v: &'a Value) -> Option<&'a Value> {
        match v {
            Value::List(ref l) => l.get(*self),
            _ => None,
        }
    }

    fn index_mut<'a>(&self, v: &'a mut Value) -> Option<&'a mut Value> {
        match v {
            Value::List(ref mut l) => l.get_mut(*self),
            _ => None,
        }
    }
}

impl Index for str {
    fn index<'a>(&self, v: &'a Value) -> Option<&'a Value> {
        match v {
            Value::Dict(ref d) => d.get(&ByteBuf::from(self)),
            _ => None,
        }
    }

    fn index_mut<'a>(&self, v: &'a mut Value) -> Option<&'a mut Value> {
        match v {
            Value::Dict(ref mut d) => d.get_mut(&ByteBuf::from(self)),
            _ => None,
        }
    }
}

impl Index for String {
    fn index<'a>(&self, v: &'a Value) -> Option<&'a Value> {
        self[..].index(v)
    }

    fn index_mut<'a>(&self, v: &'a mut Value) -> Option<&'a mut Value> {
        self[..].index_mut(v)
    }
}

impl<'s, T: ?Sized> Index for &'s T
where
    T: Index,
{
    fn index<'a>(&self, val: &'a Value) -> Option<&'a Value> {
        (*self).index(val)
    }

    fn index_mut<'a>(&self, val: &'a mut Value) -> Option<&'a mut Value> {
        (*self).index_mut(val)
    }
}

impl<I> ops::Index<I> for Value
where
    I: Index,
{
    type Output = Value;

    fn index(&self, index: I) -> &Value {
        self.get(index).expect("invalid index")
    }
}

impl<I> ops::IndexMut<I> for Value
where
    I: Index,
{
    fn index_mut(&mut self, index: I) -> &mut Value {
        self.get_mut(index).expect("invalid index")
    }
}
