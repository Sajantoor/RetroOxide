use crate::mappers::mapper::Mapper;
use crate::rom::cartridge::Cartridge;

#[derive(Debug)]
pub(crate) struct NoMbc {
    rom_data: Vec<u8>,
}

impl Mapper for NoMbc {
    fn new(cartridge: &Cartridge) -> Self {
        NoMbc {
            rom_data: cartridge.get_data().to_vec(),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        let addr = addr as usize;

        match addr {
            0x0000..0x8000 => self.rom_data[addr],
            _ => panic!("Out of bank range"),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        // rom is read only
    }
}
