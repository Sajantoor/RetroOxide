mod cartridge;

use crate::cartridge::Cartridge;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        panic!("Not enough arguments provided");
    }

    let file_path = args.get(1).unwrap();

    let cartridge_result = Cartridge::from_file(&file_path);
    let cartridge = cartridge_result.unwrap();

    println!("{:?}", cartridge.rom_header);
    println!("{:?}", cartridge.validate_header_checksum())
}
