mod cartridge;

use crate::cartridge::Cartridge;

fn main() {
    let cartridge_result = Cartridge::from_file("./roms/Tetris.gb");
    let cartridge = cartridge_result.unwrap();
    println!("{:?}", cartridge.rom_header);
    println!("{:?}", cartridge.validate_header_checksum())
}
