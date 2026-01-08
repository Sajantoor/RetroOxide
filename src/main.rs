mod bus;
mod cpu;
mod emu;
mod mappers;
mod rom;
mod utils;

use crate::rom::cartridge::Cartridge;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        panic!("Not enough arguments provided");
    }

    let file_path = args.get(1).unwrap();

    let cartridge = Cartridge::new(&file_path)?;

    let mut context = emu::Context::new(cartridge);
    context.start();

    Ok(())
}
