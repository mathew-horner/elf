//! This module implements the ability to read an ELF file header.

use std::error::Error;

use crate::reader::Reader;

#[derive(Debug)]
#[expect(unused)]
pub struct Header {
    #[rustfmt::skip]
    // e_ident[EI_MAG0] -> e_ident[EI_MAG3] is omitted because it will always be the same sequence
    // of bytes in valid headers, so there is no need to store it here.

    /// Whether the file is in 32- or 64-bit format.
    ///
    /// field: `e_ident[EI_CLASS]`
    pub word_size: WordSize,

    /// Whether the file uses little or big endianness.
    ///
    /// field: `e_ident[EI_DATA]`
    pub endianness: Endianness,

    #[rustfmt::skip]
    // TODO: Should we care about e_ident[EI_VERSION]? Will it ever be different from e_version?
    // Will either ever not be 1?

    /// The target operating system ABI.
    ///
    /// field: `e_ident[EI_OSABI]`
    pub abi: Abi,

    /// The target operating system ABI version.
    ///
    /// field: `e_ident[EI_ABIVERSION]`
    pub abi_version: u8,

    /// The object file type.
    ///
    /// field: `e_type`
    pub type_: Type,

    /// The target instruction set architecture.
    ///
    /// field: `e_machine`
    pub machine: u16, // TODO: Encode all the known ISAs, there's just too many and I'm lazy right now.

    /// The ELF version.
    ///
    /// field: `e_version`
    pub version: u32,

    /// The memory address where the entry point for the process is located.
    ///
    /// field: `e_entry`
    pub entry_point: Address,

    /// Pointer to the start of the program header table.
    ///
    /// field: `e_phoff`
    pub program_header_address: Address,

    /// Pointer to the start of the section header table.
    ///
    /// field: `e_shoff`
    pub section_header_address: Address,

    /// Flags; target architecture dependent.
    ///
    /// field: `e_flags`
    pub flags: u32,

    /// The size of this header.
    ///
    /// field: `e_ehsize`
    pub header_size: u16,

    /// The size of entries in the program header table.
    ///
    /// field: `e_phentsize`
    pub program_header_entry_size: u16,

    /// The number of entries in the program header table.
    ///
    /// field: `e_phnum`
    pub program_header_entry_count: u16,

    /// The size of entries in the section header table.
    ///
    /// field: `e_shentsize`
    pub section_header_entry_size: u16,

    /// The number of entries in the section header table.
    ///
    /// field: `e_shnum`
    pub section_header_entry_count: u16,

    /// The index of the section header table entry which defines section names.
    ///
    /// field: `e_shstrndx`
    pub section_header_name_entry_idx: u16,
}

impl Header {
    pub fn read(reader: &mut Reader) -> Result<Self, Box<dyn Error>> {
        let magic_bytes = reader.bytes_dynamic(4)?;
        if magic_bytes.as_slice() != &[0x7F, 0x45, 0x4C, 0x46] {
            return Err("not an ELF file".into());
        }

        let word_size = match reader.byte()? {
            1 => WordSize::Bits32,
            2 => WordSize::Bits64,
            other => {
                return Err(format!(
                    "invalid e_ident[EI_CLASS] should have been 1, 2 but was {other}"
                )
                .into());
            }
        };

        let endianness = match reader.byte()? {
            1 => Endianness::Little,
            2 => Endianness::Big,
            other => {
                return Err(format!(
                    "invalid e_ident[EI_DATA] should have been 1, 2 but was {other}"
                )
                .into());
            }
        };

        // Register endianness with the Reader so it knows how to parse the later multi-byte fields.
        reader.endianness = Some(endianness);

        match reader.byte()? {
            1 => {}
            other => {
                return Err(format!(
                    "invalid e_ident[EI_VERSION] should have been 1 but was {other}"
                )
                .into());
            }
        };

        let abi = match reader.byte()? {
            0x00 => Abi::SystemV,
            0x01 => Abi::HPUX,
            0x02 => Abi::NetBSD,
            0x03 => Abi::Linux,
            0x04 => Abi::GNUHurd,
            0x06 => Abi::Solaris,
            0x07 => Abi::AIX,
            0x08 => Abi::IRIX,
            0x09 => Abi::FreeBSD,
            0x0A => Abi::Tru64,
            0x0B => Abi::NovellModesto,
            0x0C => Abi::OpenBSD,
            0x0D => Abi::OpenVMS,
            0x0E => Abi::NonStopKernel,
            0x0F => Abi::AROS,
            0x10 => Abi::FenixOS,
            0x11 => Abi::NuxiCloudABI,
            0x12 => Abi::OpenVOS,
            other => {
                return Err(format!("invalid e_ident[EI_OSABI] {other}").into());
            }
        };

        let abi_version = reader.byte()?;

        // The standard defines 7 bytes of padding at the end of the identifier.
        reader.bytes_dynamic(7)?;

        let type_ = match reader.u16()? {
            0x00 => Type::Unknown,
            0x01 => Type::Relocatable,
            0x02 => Type::Executable,
            0x03 => Type::SharedObject,
            0x04 => Type::Core,
            valid @ 0xFE00 | valid @ 0xFEFF | valid @ 0xFF00 | valid @ 0xFFFF => Type::Other(valid),
            invalid => {
                return Err(format!("invalid e_type {invalid}").into());
            }
        };

        let machine = reader.u16()?;
        let version = reader.u32()?;

        let entry_point;
        let program_header_address;
        let section_header_address;

        match word_size {
            WordSize::Bits32 => {
                entry_point = Address::Bits32(reader.u32()?);
                program_header_address = Address::Bits32(reader.u32()?);
                section_header_address = Address::Bits32(reader.u32()?);
            }
            WordSize::Bits64 => {
                entry_point = Address::Bits64(reader.u64()?);
                program_header_address = Address::Bits64(reader.u64()?);
                section_header_address = Address::Bits64(reader.u64()?);
            }
        };

        let flags = reader.u32()?;
        let header_size = reader.u16()?;
        let program_header_entry_size = reader.u16()?;
        let program_header_entry_count = reader.u16()?;
        let section_header_entry_size = reader.u16()?;
        let section_header_entry_count = reader.u16()?;
        let section_header_name_entry_idx = reader.u16()?;

        Ok(Self {
            word_size,
            endianness,
            abi,
            abi_version,
            type_,
            machine,
            version,
            entry_point,
            program_header_address,
            section_header_address,
            flags,
            header_size,
            program_header_entry_size,
            program_header_entry_count,
            section_header_entry_size,
            section_header_entry_count,
            section_header_name_entry_idx,
        })
    }
}

#[derive(Debug)]
pub enum WordSize {
    Bits32,
    Bits64,
}

#[derive(Debug)]
#[expect(unused)]
pub enum Address {
    Bits32(u32),
    Bits64(u64),
}

#[derive(Clone, Copy, Debug)]
pub enum Endianness {
    Big,
    Little,
}

#[derive(Debug)]
pub enum Abi {
    SystemV,
    HPUX,
    NetBSD,
    Linux,
    GNUHurd,
    Solaris,
    AIX,
    IRIX,
    FreeBSD,
    Tru64,
    NovellModesto,
    OpenBSD,
    OpenVMS,
    NonStopKernel,
    AROS,
    FenixOS,
    NuxiCloudABI,
    OpenVOS,
}

#[derive(Debug)]
#[expect(unused)]
pub enum Type {
    Unknown,
    Relocatable,
    Executable,
    SharedObject,
    Core,
    Other(u16),
}
