mod cartridge;

use crate::cartridge::Cartridge;

fn main() {
    let cartridge = Cartridge::from_file("./roms/Tetris.gb");
    println!("{:?}", cartridge.unwrap().rom_header);
}
