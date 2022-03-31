//! Possible crate errors.

use serde::{de, ser};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{
    format,
    string::{self, String, ToString},
};
#[cfg(feature = "std")]
use std::{
    error, format, io,
    string::{self, String, ToString},
};

use core::{
    fmt::{self, Display},
    num, result,
};

/// Alias for a `Result` with a `bt_bencode::Error` error type.
pub type Result<T> = result::Result<T, Error>;

/// All possible crate errors.
#[derive(Debug)]
pub enum Error {
    Deserialize(String),
    EofWhileParsingValue,
    ExpectedSomeValue,
    FromUtf8Error(string::FromUtf8Error),
    InvalidByteStrLen,
    InvalidInteger,
    InvalidDict,
    InvalidList,
    #[cfg(feature = "std")]
    IoError(io::Error),
    KeyMustBeAByteStr,
    KeyWithoutValue,
    ParseIntError(num::ParseIntError),
    Serialize(String),
    TrailingData,
    UnsupportedType,
    ValueWithoutKey,
}

#[cfg(feature = "std")]
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Deserialize(_) => None,
            Error::EofWhileParsingValue => None,
            Error::ExpectedSomeValue => None,
            Error::FromUtf8Error(err) => Some(err),
            Error::InvalidByteStrLen => None,
            Error::InvalidInteger => None,
            Error::InvalidDict => None,
            Error::InvalidList => None,
            #[cfg(feature = "std")]
            Error::IoError(err) => Some(err),
            Error::KeyMustBeAByteStr => None,
            Error::KeyWithoutValue => None,
            Error::ParseIntError(err) => Some(err),
            Error::Serialize(_) => None,
            Error::TrailingData => None,
            Error::UnsupportedType => None,
            Error::ValueWithoutKey => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Deserialize(str) => f.write_str(str),
            Error::EofWhileParsingValue => f.write_str("eof while parsing value"),
            Error::ExpectedSomeValue => f.write_str("expected some value"),
            Error::FromUtf8Error(err) => Display::fmt(&*err, f),
            Error::InvalidByteStrLen => f.write_str("invalid byte string length"),
            Error::InvalidInteger => f.write_str("invalid integer"),
            Error::InvalidDict => f.write_str("invalid dictionary"),
            Error::InvalidList => f.write_str("invalid list"),
            #[cfg(feature = "std")]
            Error::IoError(err) => Display::fmt(&*err, f),
            Error::KeyMustBeAByteStr => f.write_str("key must be a byte string"),
            Error::KeyWithoutValue => f.write_str("key without value"),
            Error::ParseIntError(err) => Display::fmt(&*err, f),
            Error::Serialize(str) => f.write_str(str),
            Error::TrailingData => f.write_str("trailing data error"),
            Error::UnsupportedType => f.write_str("unsupported type"),
            Error::ValueWithoutKey => f.write_str("value without key"),
        }
    }
}

#[cfg(feature = "std")]
impl From<Error> for io::Error {
    fn from(other: Error) -> Self {
        match other {
            Error::IoError(e) => e,
            _ => io::Error::from(io::ErrorKind::Other),
        }
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(other: string::FromUtf8Error) -> Self {
        Error::FromUtf8Error(other)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(other: num::ParseIntError) -> Self {
        Error::ParseIntError(other)
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Deserialize(msg.to_string())
    }

    fn invalid_type(unexp: de::Unexpected<'_>, exp: &dyn de::Expected) -> Self {
        Error::Deserialize(format!(
            "unexpected type error. invalid_type={}, expected_type={}",
            unexp, exp
        ))
    }
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
impl de::StdError for Error {}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Serialize(msg.to_string())
    }
}
