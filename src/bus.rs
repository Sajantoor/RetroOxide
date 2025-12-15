use crate::mappers::mapper::Mapper;
use crate::mappers::mbc1::Mbc1;
use crate::rom::cartridge::Cartridge;

#[derive(Debug)]
pub struct Bus {
    // 16KiB ROM bank 00
    // 16 KiB from cartridge, switchable banks
    mapper: Mbc1,
    // 8KiB Video RAM
    vram: [u8; 0x2000],
    // 8KiB External RAM
    ram: [u8; 0x2000],
    // 8KiB Work RAM
    wram: [u8; 0x2000],

    // 160 bytes Sprite Attribute Table
    oam: [u8; 0xA0],

    // I/O registers
    io_regs: [u8; 0x80],

    // High RAM
    hram: [u8; 0x7F],

    // Interrupt Enable Register
    ie_reg: u8,
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        Bus {
            mapper: Mbc1::new(cartridge),
            vram: [0; 0x2000],
            ram: [0; 0x2000],
            wram: [0; 0x2000],
            oam: [0; 0xA0],
            io_regs: [0; 0x80],
            hram: [0; 0x7F],
            ie_reg: 0,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        let index = addr as usize;
        match index {
            0x0000..0x8000 => self.mapper.read(addr),
            0x8000..=0x9FFF => self.vram[index - 0x8000],
            0xA000..=0xBFFF => self.ram[index - 0xA000],
            0xC000..=0xDFFF => self.wram[index - 0xC000],
            0xE000..=0xFDFF => unimplemented!("Echo RAM is not implemented"),
            0xFE00..=0xFE9F => self.oam[index - 0xFE00],
            0xFEA0..=0xFEFF => unimplemented!("Not usable memory area"),
            0xFF00..=0xFF7F => self.io_regs[index - 0xFF00],
            0xFF80..=0xFFFE => self.hram[index - 0xFF80],
            0xFFFF => self.ie_reg,
            _ => 0, // Unmapped memory returns 0
        }
    }

    // Little endian read for word (2 bytes)
    pub fn read_word(&self, addr: u16) -> u16 {
        let low_byte = self.read_byte(addr) as u16;
        let high_byte = self.read_byte(addr + 1) as u16;
        (high_byte << 8) | low_byte
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        let index = addr as usize;

        match index {
            0x0000..=0x7FFF => self.mapper.write(addr, value),
            0x8000..=0x9FFF => self.vram[index - 0x8000] = value,
            0xA000..=0xBFFF => self.ram[index - 0xA000] = value,
            0xC000..=0xDFFF => self.wram[index - 0xC000] = value,
            0xE000..=0xFDFF => {
                // Echo RAM, typically mirrors C000-DDFF
                unimplemented!("Echo RAM is not implemented");
            }
            0xFE00..=0xFE9F => self.oam[index - 0xFE00] = value,
            0xFEA0..=0xFEFF => {
                // Not usable memory area
                unimplemented!("Not usable memory area");
            }
            0xFF00..=0xFF7F => self.io_regs[index - 0xFF00] = value,
            0xFF80..=0xFFFE => self.hram[index - 0xFF80] = value,
            0xFFFF => self.ie_reg = value,
            _ => {
                unimplemented!("Writing to unmapped memory area");
            }
        }
    }

    // Little endian write for word (2 bytes)
    pub fn write_word(&mut self, addr: u16, value: u16) {
        let low_byte = (value & 0x00FF) as u8;
        let high_byte = (value >> 8) as u8;
        self.write_byte(addr, low_byte);
        self.write_byte(addr + 1, high_byte);
    }

    pub fn get_pointer(&mut self, addr: u16) -> &mut u8 {
        &mut self.ram[addr as usize]
    }
}
