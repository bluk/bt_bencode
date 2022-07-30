//! Possible crate errors.

use serde::{de, ser};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
};
#[cfg(feature = "std")]
use std::{
    boxed::Box,
    error, format,
    string::{String, ToString},
};

use core::{
    fmt::{self, Display},
    num, result,
    str::Utf8Error,
};

/// Alias for a [`Result`][std::result::Result] with a [`bt_bencode::Error`][Error] error type.
pub type Result<T> = result::Result<T, Error>;

/// Errors during serialization and deserialization.
pub struct Error {
    inner: Box<ErrorImpl>,
}

impl Error {
    /// Constructs an error with the kind and the byte offset where the error
    /// was detected.
    ///
    /// A byte offset value of `0` indicates that the byte offset is either
    /// unknown or not relevant.
    #[must_use]
    #[inline]
    pub fn new(kind: ErrorKind, byte_offset: usize) -> Self {
        Self {
            inner: Box::new(ErrorImpl { kind, byte_offset }),
        }
    }

    #[must_use]
    #[inline]
    pub(crate) fn with_kind(kind: ErrorKind) -> Self {
        Self::new(kind, 0)
    }

    /// The kind of error encountered
    #[must_use]
    #[inline]
    pub fn kind(&self) -> &ErrorKind {
        &self.inner.kind
    }

    /// The byte offset where the error was detected.
    ///
    /// Usually, the byte offset is after the problem has been detected. For
    /// instance, if an integer is not encoded correctly like `i12ae`, the byte
    /// offset may be after the `a` byte is read.
    #[must_use]
    #[inline]
    pub fn byte_offset(&self) -> usize {
        self.inner.byte_offset
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl de::StdError for Error {
    #[cfg(feature = "std")]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.inner.kind.source()
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::with_kind(ErrorKind::Deserialize(msg.to_string()))
    }

    fn invalid_type(unexp: de::Unexpected<'_>, exp: &dyn de::Expected) -> Self {
        Error::with_kind(ErrorKind::Deserialize(format!(
            "unexpected type error. invalid_type={}, expected_type={}",
            unexp, exp
        )))
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::with_kind(ErrorKind::Serialize(msg.to_string()))
    }
}

#[cfg(feature = "std")]
impl From<Error> for std::io::Error {
    fn from(error: Error) -> Self {
        if let ErrorKind::Io(error) = error.inner.kind {
            return error;
        }
        std::io::Error::new(std::io::ErrorKind::Other, error.to_string())
    }
}

struct ErrorImpl {
    kind: ErrorKind,
    byte_offset: usize,
}

impl Display for ErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.byte_offset == 0 {
            Display::fmt(&self.kind, f)
        } else {
            write!(f, "{} at byte offset {}", self.kind, self.byte_offset)
        }
    }
}

impl fmt::Debug for ErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("byte_offset", &self.byte_offset)
            .finish()
    }
}

/// All possible crate errors.
#[allow(clippy::module_name_repetitions)]
// Should the type be non_exhaustive? Probably if this crate was version 1.0+ but would need to bump MSRV to 1.40.0
// #[non_exhaustive]
pub enum ErrorKind {
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
    #[cfg(feature = "std")]
    /// An I/O error.
    Io(std::io::Error),
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
impl error::Error for ErrorKind {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ErrorKind::Deserialize(_)
            | ErrorKind::EofWhileParsingValue
            | ErrorKind::ExpectedSomeValue
            | ErrorKind::InvalidByteStrLen
            | ErrorKind::InvalidInteger
            | ErrorKind::InvalidDict
            | ErrorKind::InvalidList
            | ErrorKind::KeyMustBeAByteStr
            | ErrorKind::KeyWithoutValue
            | ErrorKind::Serialize(_)
            | ErrorKind::TrailingData
            | ErrorKind::UnsupportedType
            | ErrorKind::ValueWithoutKey => None,
            ErrorKind::Utf8Error(err) => Some(err),
            ErrorKind::ParseIntError(err) => Some(err),
            #[cfg(feature = "std")]
            ErrorKind::Io(source) => Some(source),
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Deserialize(str) | ErrorKind::Serialize(str) => f.write_str(str),
            ErrorKind::EofWhileParsingValue => f.write_str("eof while parsing value"),
            ErrorKind::ExpectedSomeValue => f.write_str("expected some value"),
            ErrorKind::Utf8Error(err) => Display::fmt(err, f),
            ErrorKind::InvalidByteStrLen => f.write_str("invalid byte string length"),
            ErrorKind::InvalidInteger => f.write_str("invalid integer"),
            ErrorKind::InvalidDict => f.write_str("invalid dictionary"),
            ErrorKind::InvalidList => f.write_str("invalid list"),
            ErrorKind::KeyMustBeAByteStr => f.write_str("key must be a byte string"),
            ErrorKind::KeyWithoutValue => f.write_str("key without value"),
            ErrorKind::ParseIntError(err) => Display::fmt(err, f),
            ErrorKind::TrailingData => f.write_str("trailing data error"),
            ErrorKind::UnsupportedType => f.write_str("unsupported type"),
            ErrorKind::ValueWithoutKey => f.write_str("value without key"),
            #[cfg(feature = "std")]
            ErrorKind::Io(source) => Display::fmt(source, f),
        }
    }
}

impl fmt::Debug for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Deserialize(str) | ErrorKind::Serialize(str) => f.write_str(str),
            ErrorKind::EofWhileParsingValue => f.write_str("eof while parsing value"),
            ErrorKind::ExpectedSomeValue => f.write_str("expected some value"),
            ErrorKind::Utf8Error(err) => fmt::Debug::fmt(err, f),
            ErrorKind::InvalidByteStrLen => f.write_str("invalid byte string length"),
            ErrorKind::InvalidInteger => f.write_str("invalid integer"),
            ErrorKind::InvalidDict => f.write_str("invalid dictionary"),
            ErrorKind::InvalidList => f.write_str("invalid list"),
            ErrorKind::KeyMustBeAByteStr => f.write_str("key must be a byte string"),
            ErrorKind::KeyWithoutValue => f.write_str("key without value"),
            ErrorKind::ParseIntError(err) => fmt::Debug::fmt(err, f),
            ErrorKind::TrailingData => f.write_str("trailing data error"),
            ErrorKind::UnsupportedType => f.write_str("unsupported type"),
            ErrorKind::ValueWithoutKey => f.write_str("value without key"),
            #[cfg(feature = "std")]
            ErrorKind::Io(source) => fmt::Debug::fmt(source, f),
        }
    }
}
