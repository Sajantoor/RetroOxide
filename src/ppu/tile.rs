// tiles are 8x8 squares

use crate::{bus::bus::Bus, ppu::ppu::BYTES_PER_TILE, utils::test_bit};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Tile {
    pixels: [[u8; 8]; 8], // Each pixel can have a value from 0-3 representing color index
}

impl Tile {
    pub fn new(bus: &Bus, addr: u16) -> Tile {
        Tile {
            pixels: Self::read(bus, addr),
        }
    }

    fn read(bus: &Bus, addr: u16) -> [[u8; 8]; 8] {
        let mut pixels = [[0; 8]; 8];
        // this needs to be changed to skip by 2
        for i in (0..BYTES_PER_TILE).step_by(2) {
            // the first byte specifies the least significant bit of the color ID
            // of each pixel, and the second byte specifies the most significant bit
            let row = (i / 2) as usize;

            let addr = addr + i;
            let least_significant_byte = bus.read_byte(addr);
            let most_significant_byte = bus.read_byte(addr + 1);

            // bits are flipped around, the most significant bit (left most) represents the
            // right most bit and vice versa
            for j in 0..8 {
                let least_significant_bit = test_bit(least_significant_byte, j) as u8;
                let most_significant_bit = test_bit(most_significant_byte, j) as u8;

                let pixel_colour_value = most_significant_bit << 1 | least_significant_bit;
                pixels[row][7 - (j as usize)] = pixel_colour_value;
            }
        }

        return pixels;
    }

    pub fn get_row(&self, row: usize) -> [u8; 8] {
        self.pixels[row]
    }
}
