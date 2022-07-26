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
    str::Utf8Error,
};

/// Alias for a [`Result`][std::result::Result] with a [`bt_bencode::Error`][Error] error type.
pub type Result<T> = result::Result<T, Error>;

/// All possible crate errors.
#[derive(Debug)]
pub enum Error {
    /// General deserialization error.
    ///
    /// Usually the error is due to mismatching types (e.g. a struct was expecting an u64 but the data had a string).
    Deserialize(String),
    /// End of file was encountered while parsing a value.
    EofWhileParsingValue,
    /// A value was expected but the deserializer did not find a valid bencoded value.
    ExpectedSomeValue,
    /// Error when decoding a byte string into a UTF-8 string.
    ///
    /// Usually the error is encountered when a struct field or a dictionary key is deserialized as a String but the byte string is not valid UTF-8.
    Utf8Error(Utf8Error),
    /// Error when decoding a byte string into a UTF-8 string.
    ///
    /// Usually the error is encountered when a struct field or a dictionary key is deserialized as a String but the byte string is not valid UTF-8.
    FromUtf8Error(string::FromUtf8Error),
    /// When deserializing a byte string, the length was not a valid number.
    InvalidByteStrLen,
    /// When deserializing an integer, the integer contained non-number characters.
    InvalidInteger,
    /// When deserializing a dictionary, the dictionary was not encoded correctly.
    ///
    /// Usually, the error is because a dictionary was not a byte string.
    InvalidDict,
    /// When deserializing a list, the list was not encoded correctly.
    ///
    /// Usually the error is because an invalid encoded item in the list was found.
    InvalidList,
    /// An I/O error.
    #[cfg(feature = "std")]
    IoError(io::Error),
    /// When deserializing, a dictionary key was found which was not a byte string.
    KeyMustBeAByteStr,
    /// A dictionary key was serialized but did not have a value for the key.
    KeyWithoutValue,
    /// Error when deserializing a number.
    ///
    /// If the number could not be parsed correctly. Either the number itself
    /// was invalid or the wrong type was used (a signed integer was encoded but
    /// a [u64] was the expected type).
    ParseIntError(num::ParseIntError),
    /// General serialization error.
    Serialize(String),
    /// Unparsed trailing data was detected
    TrailingData,
    /// An unsupported type was used.
    ///
    /// Usually the error is due to using unsupported types for keys (e.g. using
    /// an integer type instead of a ByteStr).
    UnsupportedType,
    /// A dictionary did not have a key but had a value.
    ///
    /// Should never occur.
    ValueWithoutKey,
}

#[cfg(feature = "std")]
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Deserialize(_)
            | Error::EofWhileParsingValue
            | Error::ExpectedSomeValue
            | Error::InvalidByteStrLen
            | Error::InvalidInteger
            | Error::InvalidDict
            | Error::InvalidList
            | Error::KeyMustBeAByteStr
            | Error::KeyWithoutValue
            | Error::Serialize(_)
            | Error::TrailingData
            | Error::UnsupportedType
            | Error::ValueWithoutKey => None,
            Error::Utf8Error(err) => Some(err),
            Error::FromUtf8Error(err) => Some(err),
            #[cfg(feature = "std")]
            Error::IoError(err) => Some(err),
            Error::ParseIntError(err) => Some(err),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Deserialize(str) | Error::Serialize(str) => f.write_str(str),
            Error::EofWhileParsingValue => f.write_str("eof while parsing value"),
            Error::ExpectedSomeValue => f.write_str("expected some value"),
            Error::Utf8Error(err) => Display::fmt(err, f),
            Error::FromUtf8Error(err) => Display::fmt(err, f),
            Error::InvalidByteStrLen => f.write_str("invalid byte string length"),
            Error::InvalidInteger => f.write_str("invalid integer"),
            Error::InvalidDict => f.write_str("invalid dictionary"),
            Error::InvalidList => f.write_str("invalid list"),
            #[cfg(feature = "std")]
            Error::IoError(err) => Display::fmt(err, f),
            Error::KeyMustBeAByteStr => f.write_str("key must be a byte string"),
            Error::KeyWithoutValue => f.write_str("key without value"),
            Error::ParseIntError(err) => Display::fmt(err, f),
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

impl From<Utf8Error> for Error {
    fn from(other: Utf8Error) -> Self {
        Error::Utf8Error(other)
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
