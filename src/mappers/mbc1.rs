use crate::mappers::mapper::Mapper;
use crate::rom::cartridge::Cartridge;

#[derive(Debug)]
pub(crate) struct Mbc1 {
    rom_data: Vec<u8>,
    is_ram_enabled: bool,
}

impl Mapper for Mbc1 {
    fn new(cartridge: &Cartridge) -> Self {
        Mbc1 {
            rom_data: cartridge.get_data().to_vec(),
            is_ram_enabled: false,
        }
    }

    fn read(&self, addr: u16) -> u8 {
        let addr = addr as usize;

        match addr {
            0x0000..0x4000 => self.rom_data[addr],
            // TODO: Rom banking here
            0x4000..0x8000 => self.rom_data[addr],
            0xA000..0xC000 => {
                if self.is_ram_enabled {
                    unimplemented!("RAM is not implemented")
                } else {
                    0xFF
                }
            }
            _ => panic!("Out of bank range"),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        let addr = addr as usize;
        match addr {
            0x0000..0x2000 => {
                let lower_nibble = value & 0x0F;
                // Enable ram is the lower nibble is $A, any other value disables the ram
                self.is_ram_enabled = lower_nibble == 0x0A;
            }
            0x2000..0x4000 => {
                unimplemented!("ROM bank is unimplemented");
            }
            0x4000..0x6000 => {
                if self.is_ram_enabled {
                } else {
                    0xFF;
                }
            }
            0x6000..0x8000 => {
                unimplemented!("Bank mode select is unimplemented");
            }
            _ => panic!("out of bank range"),
        }
    }
}
