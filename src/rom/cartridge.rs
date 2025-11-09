use crate::rom::rom_header::RomHeader;

use core::panic;
use std::fs;
use std::io;

#[derive(Debug)]
pub struct Cartridge {
    file_name: String,
    rom_size: u32,
    rom_data: Vec<u8>,
    pub rom_header: RomHeader,
}

impl Cartridge {
    pub fn from_file(path: &str) -> io::Result<Self> {
        let rom_data = fs::read(path)?;
        let rom_size = rom_data.len() as u32;

        let rom_header = RomHeader::parse(&rom_data)?;

        Ok(Self {
            file_name: path.to_string(),
            rom_size,
            rom_data,
            rom_header,
        })
    }

    pub fn read() {
        panic!("Not implemented yet!");
    }

    pub fn validate_header_checksum(&self) -> bool {
        let mut computed_checksum: u8 = 0;

        for i in 0x0134..(0x014C + 1) {
            computed_checksum = computed_checksum.wrapping_sub(self.rom_data[i] + 1);
        }

        if computed_checksum != self.rom_header.header_checksum {
            panic!("Header checksum is invalid!!!");
        }

        return true;
    }
}
