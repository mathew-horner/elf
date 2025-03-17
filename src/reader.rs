use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, Read};

use crate::header::Endianness;

pub struct Reader {
    inner: BufReader<File>,

    /// This represents the endianness of the underlying ELF file.
    ///
    /// It is initialized to `None` because we read the endianness from the ELF header, so
    /// operations that don't require the endianness to be determined (such as [`Reader::bytes`],
    /// etc.) are always okay to use, but operations that do require it to be determined (such as
    /// [`Reader::u16`], [`Reader::u32`], etc.) cannot be used until the endianness is determined
    /// and stored here.
    pub(crate) endianness: Option<Endianness>,
}

impl Reader {
    pub fn new(file: File) -> Self {
        Self {
            inner: BufReader::new(file),
            endianness: None,
        }
    }

    /// Read `N` bytes; used in situations where a statically sized array is needed.
    pub fn bytes<const N: usize>(&mut self) -> Result<[u8; N], io::Error> {
        let mut bytes = [0; N];
        self.inner.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    /// Read `count` bytes; used in situations where a statically sized array is unnecessary.
    pub fn bytes_dynamic(&mut self, count: usize) -> Result<Vec<u8>, io::Error> {
        let mut bytes = vec![0; count];
        self.inner.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    /// Read one byte from the file.
    pub fn byte(&mut self) -> Result<u8, io::Error> {
        Ok(self.bytes::<1>()?[0])
    }

    /// Read two bytes from the file and interpret them as one `u16`.
    pub fn u16(&mut self) -> Result<u16, Box<dyn Error>> {
        let Some(endianness) = self.endianness else {
            return Err("tried to read u16 before endianness was defined, this is a bug!".into());
        };
        let bytes = self.bytes::<2>()?;
        let u16 = match endianness {
            Endianness::Little => u16::from_le_bytes(bytes),
            Endianness::Big => u16::from_be_bytes(bytes),
        };
        Ok(u16)
    }

    /// Read four bytes from the file and interpret them as one `u32`.
    pub fn u32(&mut self) -> Result<u32, Box<dyn Error>> {
        let Some(endianness) = self.endianness else {
            return Err("tried to read u32 before endianness was defined, this is a bug!".into());
        };
        let bytes = self.bytes::<4>()?;
        let u32 = match endianness {
            Endianness::Little => u32::from_le_bytes(bytes),
            Endianness::Big => u32::from_be_bytes(bytes),
        };
        Ok(u32)
    }

    /// Read eight bytes from the file and interpret them as one `u64`.
    pub fn u64(&mut self) -> Result<u64, Box<dyn Error>> {
        let Some(endianness) = self.endianness else {
            return Err("tried to read u64 before endianness was defined, this is a bug!".into());
        };
        let bytes = self.bytes::<8>()?;
        let u64 = match endianness {
            Endianness::Little => u64::from_le_bytes(bytes),
            Endianness::Big => u64::from_be_bytes(bytes),
        };
        Ok(u64)
    }
}
