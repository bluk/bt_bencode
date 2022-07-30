//! [Read] trait and helpers to read bytes for the deserializer.

use crate::error::{Error, ErrorKind, Result};
use core::ops::Deref;

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::{io, vec::Vec};

/// A reference to borrowed data.
///
/// The variant determines if the slice comes from a long lived source (e.g. an
/// existing byte array) or if it comes from a temporary buffer.
///
/// In the deserializer code, the different variants determine which visitor
/// method to call (e.g. `visit_borrowed_str` vs. `visit_str`).  Each variant
/// has a different lifetime which is what the compiler uses to ensure the data
/// will live long enough.
#[derive(Debug)]
pub enum Ref<'a, 'b, T>
where
    T: 'static + ?Sized,
{
    /// Reference from the original source of data.
    Source(&'a T),
    /// Reference from the given data buffer.
    Buffer(&'b T),
}

impl<'a, 'b, T> Deref for Ref<'a, 'b, T>
where
    T: 'static + ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match *self {
            Ref::Source(s) => s,
            Ref::Buffer(b) => b,
        }
    }
}

/// Trait used by the [`de::Deserializer`][crate::de::Deserializer] to read bytes.
pub trait Read<'a> {
    /// Consumes and returns the next read byte.
    fn next(&mut self) -> Option<Result<u8>>;

    /// Returns the next byte but does not consume.
    ///
    /// Repeated peeks (with no [next()][Read::next] call) should return the same byte.
    fn peek(&mut self) -> Option<Result<u8>>;

    /// Returns the position in the stream of bytes.
    fn byte_offset(&self) -> usize;

    /// Consumes and returns the next integer.
    ///
    /// The buffer can be used as a temporary buffer for storing any bytes which need to be read.
    /// The contents of the buffer is not guaranteed before or after the method is called.
    ///
    /// # Errors
    ///
    /// Errors include:
    ///
    /// - malformatted input
    /// - end of file
    fn parse_integer<'b>(&'b mut self, buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, str>>;

    /// Returns the next slice of data for the given length.
    ///
    /// If all of the data is already read and available to borrowed against,
    /// the returned result could be a reference to the original underlying
    /// data.
    ///
    /// If the data is not already available and needs to be buffered, the data
    /// could be added to the given buffer parameter and a borrowed slice from
    /// the buffer could be returned.
    ///
    /// # Errors
    ///
    /// Errors include:
    ///
    /// - malformatted input
    /// - end of file
    fn parse_byte_str<'b>(&'b mut self, buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, [u8]>>;

    /// Consumes and returns the next integer raw encoding.
    ///
    /// The buffer can be used as a temporary buffer for storing any bytes which need to be read.
    /// The contents of the buffer is not guaranteed before or after the method is called.
    ///
    /// # Errors
    ///
    /// Errors include:
    ///
    /// - malformatted input
    /// - end of file
    fn parse_raw_integer<'b>(&'b mut self, buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, [u8]>>;

    /// Consumes and returns the next byte string raw encoding.
    ///
    /// The buffer can be used as a temporary buffer for storing any bytes which need to be read.
    /// The contents of the buffer is not guaranteed before or after the method is called.
    ///
    /// # Errors
    ///
    /// Errors include:
    ///
    /// - malformatted input
    /// - end of file
    fn parse_raw_byte_str<'b>(&mut self, buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, [u8]>>;

    /// Consumes and returns the next list raw encoding.
    ///
    /// The buffer can be used as a temporary buffer for storing any bytes which need to be read.
    /// The contents of the buffer is not guaranteed before or after the method is called.
    ///
    /// # Errors
    ///
    /// Errors include:
    ///
    /// - malformatted input
    /// - end of file
    fn parse_raw_list<'b>(&'b mut self, buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, [u8]>>;

    /// Consumes and returns the next dictionary raw encoding.
    ///
    /// The buffer can be used as a temporary buffer for storing any bytes which need to be read.
    /// The contents of the buffer is not guaranteed before or after the method is called.
    ///
    /// # Errors
    ///
    /// Errors include:
    ///
    /// - malformatted input
    /// - end of file
    fn parse_raw_dict<'b>(&'b mut self, buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, [u8]>>;
}

/// A wrapper to implement this crate's [Read] trait for [`std::io::Read`] trait implementations.
#[cfg(feature = "std")]
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct IoRead<R>
where
    R: io::Read,
{
    iter: io::Bytes<R>,
    peeked_byte: Option<u8>,
    byte_offset: usize,
}

#[cfg(feature = "std")]
impl<R> IoRead<R>
where
    R: io::Read,
{
    /// Instantiates a new reader.
    pub fn new(reader: R) -> Self {
        IoRead {
            iter: reader.bytes(),
            peeked_byte: None,
            byte_offset: 0,
        }
    }
}

#[cfg(feature = "std")]
impl<'a, R> Read<'a> for IoRead<R>
where
    R: io::Read,
{
    #[inline]
    fn next(&mut self) -> Option<Result<u8>> {
        match self.peeked_byte.take() {
            Some(b) => {
                self.byte_offset += 1;
                Some(Ok(b))
            }
            None => match self.iter.next() {
                Some(Ok(b)) => {
                    self.byte_offset += 1;
                    Some(Ok(b))
                }
                Some(Err(err)) => Some(Err(Error::new(ErrorKind::Io(err), self.byte_offset()))),
                None => None,
            },
        }
    }

    #[inline]
    fn peek(&mut self) -> Option<Result<u8>> {
        match self.peeked_byte {
            Some(b) => Some(Ok(b)),
            None => match self.iter.next() {
                Some(Ok(b)) => {
                    self.peeked_byte = Some(b);
                    Some(Ok(b))
                }
                Some(Err(err)) => Some(Err(Error::new(ErrorKind::Io(err), self.byte_offset()))),
                None => None,
            },
        }
    }

    #[inline]
    fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    fn parse_integer<'b>(&'b mut self, buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, str>> {
        debug_assert!(buf.is_empty());

        let start_idx = buf.len();

        if self
            .peek()
            .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
            == b'-'
        {
            buf.push(b'-');
            self.next()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??;
        }

        loop {
            match self
                .next()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
            {
                b'e' => {
                    return Ok(Ref::Buffer(
                        core::str::from_utf8(&buf[start_idx..]).map_err(|error| {
                            Error::new(ErrorKind::Utf8Error(error), self.byte_offset())
                        })?,
                    ))
                }
                n @ b'0'..=b'9' => buf.push(n),
                _ => return Err(Error::new(ErrorKind::InvalidInteger, self.byte_offset())),
            }
        }
    }

    fn parse_byte_str<'b>(&'b mut self, buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, [u8]>> {
        debug_assert!(buf.is_empty());

        let len: usize;
        loop {
            match self
                .next()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
            {
                b':' => {
                    len = core::str::from_utf8(buf)
                        .map_err(|error| {
                            Error::new(ErrorKind::Utf8Error(error), self.byte_offset())
                        })?
                        .parse()
                        .map_err(|error| {
                            Error::new(ErrorKind::ParseIntError(error), self.byte_offset())
                        })?;
                    break;
                }
                n @ b'0'..=b'9' => buf.push(n),
                _ => return Err(Error::new(ErrorKind::InvalidByteStrLen, self.byte_offset())),
            }
        }

        buf.clear();
        buf.reserve(len);

        for _ in 0..len {
            buf.push(self.next().ok_or_else(|| {
                Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset())
            })??);
        }

        Ok(Ref::Buffer(&buf[..]))
    }

    fn parse_raw_integer<'b>(&'b mut self, buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, [u8]>> {
        let start_idx = buf.len();
        buf.push(
            self.next()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??,
        );

        match self
            .peek()
            .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
        {
            b'-' => {
                buf.push(self.next().ok_or_else(|| {
                    Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset())
                })??);
            }
            b'0'..=b'9' => {}
            _ => return Err(Error::new(ErrorKind::InvalidInteger, self.byte_offset())),
        }

        loop {
            match self
                .next()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
            {
                b'e' => {
                    buf.push(b'e');
                    return Ok(Ref::Buffer(&buf[start_idx..]));
                }
                n @ b'0'..=b'9' => buf.push(n),
                _ => return Err(Error::new(ErrorKind::InvalidInteger, self.byte_offset())),
            }
        }
    }

    fn parse_raw_byte_str<'b>(&mut self, buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, [u8]>> {
        let start_idx = buf.len();
        let len;
        loop {
            match self
                .next()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
            {
                b':' => {
                    len = core::str::from_utf8(&buf[start_idx..])
                        .map_err(|error| {
                            Error::new(ErrorKind::Utf8Error(error), self.byte_offset())
                        })?
                        .parse()
                        .map_err(|error| {
                            Error::new(ErrorKind::ParseIntError(error), self.byte_offset())
                        })?;
                    buf.push(b':');
                    break;
                }
                n @ b'0'..=b'9' => buf.push(n),
                _ => return Err(Error::new(ErrorKind::InvalidByteStrLen, self.byte_offset())),
            }
        }

        buf.reserve(len);
        for _ in 0..len {
            buf.push(self.next().ok_or_else(|| {
                Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset())
            })??);
        }
        Ok(Ref::Buffer(&buf[start_idx..]))
    }

    fn parse_raw_list<'b>(&'b mut self, buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, [u8]>> {
        let start_idx = buf.len();
        buf.push(
            self.next()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??,
        );

        loop {
            match self
                .peek()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
            {
                b'e' => {
                    buf.push(self.next().ok_or_else(|| {
                        Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset())
                    })??);
                    return Ok(Ref::Buffer(&buf[start_idx..]));
                }
                b'0'..=b'9' => {
                    self.parse_raw_byte_str(buf)?;
                }
                b'i' => {
                    self.parse_raw_integer(buf)?;
                }
                b'l' => {
                    self.parse_raw_list(buf)?;
                }
                b'd' => {
                    self.parse_raw_dict(buf)?;
                }
                _ => return Err(Error::new(ErrorKind::InvalidList, self.byte_offset())),
            }
        }
    }

    fn parse_raw_dict<'b>(&'b mut self, buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, [u8]>> {
        let start_idx = buf.len();
        buf.push(
            self.next()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??,
        );

        loop {
            match self
                .peek()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
            {
                b'0'..=b'9' => {
                    self.parse_raw_byte_str(buf)?;
                }
                b'e' => {
                    buf.push(self.next().ok_or_else(|| {
                        Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset())
                    })??);
                    return Ok(Ref::Buffer(&buf[start_idx..]));
                }
                _ => {
                    return Err(Error::new(ErrorKind::InvalidDict, self.byte_offset()));
                }
            }

            match self
                .peek()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
            {
                b'0'..=b'9' => {
                    self.parse_raw_byte_str(buf)?;
                }
                b'i' => {
                    self.parse_raw_integer(buf)?;
                }
                b'l' => {
                    self.parse_raw_list(buf)?;
                }
                b'd' => {
                    self.parse_raw_dict(buf)?;
                }
                _ => {
                    return Err(Error::new(ErrorKind::InvalidDict, self.byte_offset()));
                }
            }
        }
    }
}

/// A wrapper to implement this crate's [Read] trait for byte slices.
#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct SliceRead<'a> {
    slice: &'a [u8],
    byte_offset: usize,
}

impl<'a> SliceRead<'a> {
    /// Instantiates a new reader.
    #[must_use]
    pub fn new(slice: &'a [u8]) -> Self {
        SliceRead {
            slice,
            byte_offset: 0,
        }
    }
}

impl<'a> Read<'a> for SliceRead<'a> {
    #[inline]
    fn next(&mut self) -> Option<Result<u8>> {
        if self.byte_offset < self.slice.len() {
            let b = self.slice[self.byte_offset];
            self.byte_offset += 1;
            Some(Ok(b))
        } else {
            None
        }
    }

    #[inline]
    fn peek(&mut self) -> Option<Result<u8>> {
        if self.byte_offset < self.slice.len() {
            Some(Ok(self.slice[self.byte_offset]))
        } else {
            None
        }
    }

    #[inline]
    fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    #[inline]
    fn parse_integer<'b>(&'b mut self, _buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, str>> {
        let start_idx = self.byte_offset;

        match self
            .next()
            .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
        {
            b'-' | b'0'..=b'9' => loop {
                match self.next().ok_or_else(|| {
                    Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset())
                })?? {
                    b'0'..=b'9' => {}
                    b'e' => {
                        return Ok(Ref::Source(
                            core::str::from_utf8(&self.slice[start_idx..self.byte_offset - 1])
                                .map_err(|error| {
                                    Error::new(ErrorKind::Utf8Error(error), self.byte_offset())
                                })?,
                        ));
                    }
                    _ => return Err(Error::new(ErrorKind::InvalidInteger, self.byte_offset())),
                }
            },
            _ => Err(Error::new(ErrorKind::InvalidInteger, self.byte_offset())),
        }
    }

    #[inline]
    fn parse_byte_str<'b>(&'b mut self, _buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, [u8]>> {
        let start_idx = self.byte_offset;

        let len: usize;
        loop {
            match self
                .next()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
            {
                b':' => {
                    len = core::str::from_utf8(&self.slice[start_idx..self.byte_offset - 1])
                        .map_err(|error| {
                            Error::new(ErrorKind::Utf8Error(error), self.byte_offset())
                        })?
                        .parse()
                        .map_err(|error| {
                            Error::new(ErrorKind::ParseIntError(error), self.byte_offset())
                        })?;
                    break;
                }
                b'0'..=b'9' => {}
                _ => return Err(Error::new(ErrorKind::InvalidByteStrLen, self.byte_offset())),
            }
        }

        let start_idx = self.byte_offset;
        self.byte_offset += len;

        let slice_len = self.slice.len();
        if slice_len < self.byte_offset {
            self.byte_offset = slice_len;
            return Err(Error::new(
                ErrorKind::EofWhileParsingValue,
                self.byte_offset(),
            ));
        }

        Ok(Ref::Source(&self.slice[start_idx..self.byte_offset]))
    }

    fn parse_raw_integer<'b>(&'b mut self, _buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, [u8]>> {
        let start_idx = self.byte_offset;

        self.next()
            .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??;

        match self
            .peek()
            .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
        {
            b'-' => {
                self.next().ok_or_else(|| {
                    Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset())
                })??;
            }
            b'0'..=b'9' => {}
            _ => return Err(Error::new(ErrorKind::InvalidInteger, self.byte_offset())),
        }

        loop {
            match self
                .next()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
            {
                b'e' => {
                    return Ok(Ref::Source(&self.slice[start_idx..self.byte_offset]));
                }
                b'0'..=b'9' => {}
                _ => return Err(Error::new(ErrorKind::InvalidInteger, self.byte_offset())),
            }
        }
    }

    fn parse_raw_byte_str<'b>(&mut self, _buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, [u8]>> {
        let start_idx = self.byte_offset;

        let len: usize;
        loop {
            match self
                .next()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
            {
                b':' => {
                    len = core::str::from_utf8(&self.slice[start_idx..self.byte_offset - 1])
                        .map_err(|error| {
                            Error::new(ErrorKind::Utf8Error(error), self.byte_offset())
                        })?
                        .parse()
                        .map_err(|error| {
                            Error::new(ErrorKind::ParseIntError(error), self.byte_offset())
                        })?;
                    break;
                }
                b'0'..=b'9' => {}
                _ => return Err(Error::new(ErrorKind::InvalidByteStrLen, self.byte_offset())),
            }
        }
        self.byte_offset += len;

        let slice_len = self.slice.len();
        if slice_len < self.byte_offset {
            self.byte_offset = slice_len;
            return Err(Error::new(
                ErrorKind::EofWhileParsingValue,
                self.byte_offset(),
            ));
        }

        Ok(Ref::Source(&self.slice[start_idx..self.byte_offset]))
    }

    fn parse_raw_list<'b>(&'b mut self, buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, [u8]>> {
        let start_idx = self.byte_offset;

        self.next()
            .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??;

        loop {
            match self
                .peek()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
            {
                b'e' => {
                    self.next().ok_or_else(|| {
                        Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset())
                    })??;
                    return Ok(Ref::Source(&self.slice[start_idx..self.byte_offset]));
                }
                b'0'..=b'9' => {
                    self.parse_raw_byte_str(buf)?;
                }
                b'i' => {
                    self.parse_raw_integer(buf)?;
                }
                b'l' => {
                    self.parse_raw_list(buf)?;
                }
                b'd' => {
                    self.parse_raw_dict(buf)?;
                }
                _ => return Err(Error::new(ErrorKind::InvalidList, self.byte_offset())),
            }
        }
    }

    fn parse_raw_dict<'b>(&'b mut self, buf: &'b mut Vec<u8>) -> Result<Ref<'a, 'b, [u8]>> {
        let start_idx = self.byte_offset;

        self.next()
            .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??;

        loop {
            match self
                .peek()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
            {
                b'e' => {
                    self.next().ok_or_else(|| {
                        Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset())
                    })??;
                    return Ok(Ref::Source(&self.slice[start_idx..self.byte_offset]));
                }
                b'0'..=b'9' => {
                    self.parse_raw_byte_str(buf)?;
                }
                _ => {
                    return Err(Error::new(ErrorKind::InvalidDict, self.byte_offset()));
                }
            }

            match self
                .peek()
                .ok_or_else(|| Error::new(ErrorKind::EofWhileParsingValue, self.byte_offset()))??
            {
                b'0'..=b'9' => {
                    self.parse_raw_byte_str(buf)?;
                }
                b'i' => {
                    self.parse_raw_integer(buf)?;
                }
                b'l' => {
                    self.parse_raw_list(buf)?;
                }
                b'd' => {
                    self.parse_raw_dict(buf)?;
                }
                _ => {
                    return Err(Error::new(ErrorKind::InvalidDict, self.byte_offset()));
                }
            }
        }
    }
}
