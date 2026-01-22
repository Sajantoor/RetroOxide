use crate::{mappers::mbc1::Mbc1, rom::cartridge::Cartridge};

pub trait Mapper {
    fn new(cartridge: &Cartridge) -> Self;
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}

pub fn get_mapper(cartridge: &Cartridge) -> impl Mapper {
    // Factory function to get the correct mapper, for now only Mbc1 is implemented
    Mbc1::new(cartridge)
}
