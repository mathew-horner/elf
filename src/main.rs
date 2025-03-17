use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::{env, fmt, process};

use crate::header::Header;
use crate::reader::Reader;

mod header;
mod reader;

#[derive(Debug)]
#[expect(unused)]
struct Output {
    header: Header,
}

impl fmt::Display for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "placeholder")
    }
}

fn cli(path: impl AsRef<Path>) -> Result<Output, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = Reader::new(file);
    let header = Header::read(&mut reader)?;
    Ok(Output { header })
}

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
