//! Byte IpAddr which helps with the deserialization.

use core::{
    borrow::{Borrow, BorrowMut},
    fmt, net,
    ops::{Deref, DerefMut},
};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::format;

/// IpAddr serialize to and deserialize from big-endian bytes representation.
///
/// Due to a limitation within `serde` and Rust, a `IpAddr::V4` and `IpAddr::V6` will
/// serialize and deserialize as a list of individual byte elements. Serializing `IpAddr`
/// requires serialize enum which this bencode/library does not support yet.
///
/// # Examples
///
/// ```rust
/// use bt_bencode::ByteIpAddr;
/// use core::net;
///
/// let v4_bytes = [1,2,3,4];
/// let v6_bytes = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16];
/// let v4 = net::IpAddr::from(v4_bytes);
/// let v6 = net::IpAddr::from(v6_bytes);
/// let bip4 = ByteIpAddr::from(v4);
///
///
/// let encoded = bt_bencode::to_vec(&bip4)?;
/// assert_eq!(encoded, b"4:\x01\x02\x03\x04");
///
/// let decoded: ByteIpAddr = bt_bencode::from_slice(&encoded)?;
/// assert_eq!(decoded, v4.into());
///
/// let bip6 = ByteIpAddr::from(v6);
///
/// let encoded = bt_bencode::to_vec(&bip6)?;
/// assert_eq!(encoded, b"16:\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10");
///
/// let decoded: ByteIpAddr = bt_bencode::from_slice(&encoded)?;
/// assert_eq!(decoded, v6.into());
///
/// // test invalid bytes
/// assert!(bt_bencode::from_slice::<ByteIpAddr>(b"5:\x01\x02\x03\x04\x05").is_err());
///
/// # Ok::<(), bt_bencode::Error>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteIpAddr(net::IpAddr);

impl AsRef<net::IpAddr> for ByteIpAddr {
    fn as_ref(&self) -> &net::IpAddr {
        &self.0
    }
}

impl AsMut<net::IpAddr> for ByteIpAddr {
    fn as_mut(&mut self) -> &mut net::IpAddr {
        &mut self.0
    }
}

impl Borrow<net::IpAddr> for ByteIpAddr {
    fn borrow(&self) -> &net::IpAddr {
        &self.0
    }
}

impl BorrowMut<net::IpAddr> for ByteIpAddr {
    fn borrow_mut(&mut self) -> &mut net::IpAddr {
        &mut self.0
    }
}

impl fmt::Debug for ByteIpAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl Deref for ByteIpAddr {
    type Target = net::IpAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ByteIpAddr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for ByteIpAddr
where
    net::IpAddr: From<T>,
{
    fn from(value: T) -> Self {
        Self(net::IpAddr::from(value))
    }
}

impl serde::Serialize for ByteIpAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            net::IpAddr::V4(ip) => serializer.serialize_bytes(&ip.octets()),
            net::IpAddr::V6(ip) => serializer.serialize_bytes(&ip.octets()),
        }
    }
}

struct IpAddrVisitor;

impl<'de> Visitor<'de> for IpAddrVisitor {
    type Value = ByteIpAddr;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("byte string of length 4(ipv4) or 16(ipv6)")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v.len() {
            4 => Ok(ByteIpAddr(net::IpAddr::V4(net::Ipv4Addr::from([
                v[0], v[1], v[2], v[3],
            ])))),
            16 => Ok(ByteIpAddr(net::IpAddr::V6(net::Ipv6Addr::from([
                v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9], v[10], v[11], v[12],
                v[13], v[14], v[15],
            ])))),
            other => Err(de::Error::invalid_value(
                de::Unexpected::Str(&format!("get byte string {v:02x?} of length {other}")),
                &self,
            )),
        }
    }
}

impl<'de> Deserialize<'de> for ByteIpAddr {
    fn deserialize<D>(deserializer: D) -> Result<ByteIpAddr, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_byte_buf(IpAddrVisitor)
    }
}
