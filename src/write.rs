//! `Write` trait and helpers to write bytes for the serializer.

#[cfg(feature = "std")]
use crate::error::Error;

#[cfg(feature = "std")]
use std::io;

use serde_bytes::ByteBuf;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::error::Result;

pub trait Write {
    fn write_all(&mut self, buf: &[u8]) -> Result<()>;
}

#[cfg(feature = "std")]
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
        self.writer.write_all(buf).map_err(|e| Error::IoError(e))
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
