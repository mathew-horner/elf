use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::{env, fmt, io, process};

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("usage: elf <FILE>");
        process::exit(1);
    }
    match cli(&args[1]) {
        Ok(output) => println!("{output:#?}"),
        Err(error) => {
            println!("error: {error}");
            process::exit(1);
        }
    }
}

#[derive(Debug)]
#[expect(unused)]
struct Output {
    word_size: WordSize,
    endianness: Endianness,
    abi: Abi,
    abi_version: u8,
    type_: Type,
    // TODO: Encode all the known ISAs, there's just too many and I'm lazy right now.
    machine: u16,
    version: u32,
    entry_point: Address,
    program_header_address: Address,
    section_header_address: Address,
    flags: u32,
    header_size: u16,
    program_header_entry_size: u16,
    program_header_entry_count: u16,
    section_header_entry_size: u16,
    section_header_entry_count: u16,
    section_header_name_entry_idx: u16,
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "placeholder")
    }
}

#[derive(Debug)]
enum WordSize {
    Bits32,
    Bits64,
}

#[derive(Debug)]
#[expect(unused)]
enum Address {
    Bits32(u32),
    Bits64(u64),
}

#[derive(Clone, Copy, Debug)]
enum Endianness {
    Big,
    Little,
}

#[derive(Debug)]
enum Abi {
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
enum Type {
    Unknown,
    Relocatable,
    Executable,
    SharedObject,
    Core,
    Other(u16),
}

struct Reader {
    inner: BufReader<File>,
    endianness: Option<Endianness>,
}

impl Reader {
    fn new(file: File) -> Self {
        Self {
            inner: BufReader::new(file),
            endianness: None,
        }
    }

    fn bytes<const N: usize>(&mut self) -> Result<[u8; N], io::Error> {
        let mut bytes = [0; N];
        self.inner.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    fn bytes_dynamic(&mut self, count: usize) -> Result<Vec<u8>, io::Error> {
        let mut bytes = vec![0; count];
        self.inner.read_exact(&mut bytes)?;
        Ok(bytes)
    }

    fn byte(&mut self) -> Result<u8, io::Error> {
        Ok(self.bytes::<1>()?[0])
    }

    fn u16(&mut self) -> Result<u16, Box<dyn Error>> {
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

    fn u32(&mut self) -> Result<u32, Box<dyn Error>> {
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

    fn u64(&mut self) -> Result<u64, Box<dyn Error>> {
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

fn cli(path: impl AsRef<Path>) -> Result<Output, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = Reader::new(file);

    // Read ELF header.

    let magic_bytes = reader.bytes_dynamic(4)?;
    if magic_bytes.as_slice() != &[0x7F, 0x45, 0x4C, 0x46] {
        return Err("not an ELF file".into());
    }

    let word_size = match reader.byte()? {
        1 => WordSize::Bits32,
        2 => WordSize::Bits64,
        other => {
            return Err(
                format!("invalid e_ident[EI_CLASS] should have been 1, 2 but was {other}").into(),
            );
        }
    };

    let endianness = match reader.byte()? {
        1 => Endianness::Little,
        2 => Endianness::Big,
        other => {
            return Err(
                format!("invalid e_ident[EI_DATA] should have been 1, 2 but was {other}").into(),
            );
        }
    };

    // Register endianness with the Reader so it knows how to parse the later multi-byte fields.
    reader.endianness = Some(endianness);

    match reader.byte()? {
        1 => {}
        other => {
            return Err(
                format!("invalid e_ident[EI_VERSION] should have been 1 but was {other}").into(),
            );
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

    Ok(Output {
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
