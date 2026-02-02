mod bus;
mod cpu;
mod emu;
mod mappers;
mod ppu;
mod rom;
mod ui;
mod utils;

use crate::rom::cartridge::Cartridge;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        panic!("Not enough arguments provided");
    }

    let file_path = args.get(1).unwrap();

    let cartridge = Cartridge::new(&file_path)?;

    let context = emu::Context::new(cartridge);
    let mut ui = ui::UI::new(context);
    ui.start();

    Ok(())
}
