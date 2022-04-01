//! [Write] trait and helpers to write bytes for the serializer.

#[cfg(feature = "std")]
use crate::error::Error;

#[cfg(feature = "std")]
use std::io;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::error::Result;

/// Trait used by the [`ser::Serializer`][crate::ser::Serializer] to write bytes.
pub trait Write {
    /// Writes all of the bytes.
    ///
    /// # Errors
    ///
    /// If the bytes could not be written, a Bencode error is returned.
    fn write_all(&mut self, buf: &[u8]) -> Result<()>;
}

/// A wrapper to implement this crate's [Write] trait for [`std::io::Write`] trait implementations.
#[cfg(feature = "std")]
#[derive(Debug)]
pub struct IoWrite<W>
where
    W: io::Write,
{
    writer: W,
}

#[cfg(feature = "std")]
impl<W> IoWrite<W>
where
    W: io::Write,
{
    /// Instantiates a new writer.
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

#[cfg(feature = "std")]
impl<W> Write for IoWrite<W>
where
    W: io::Write,
{
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.writer.write_all(buf).map_err(Error::IoError)
    }
}

impl Write for Vec<u8> {
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }
}

impl Write for &mut Vec<u8> {
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }
}
