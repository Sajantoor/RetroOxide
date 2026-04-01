use crate::{
    bus::bus::Bus,
    ppu::{
        lcd::{BG_TILE_DATA_AREA_START_BANK_1, BUFFER_SIZE, Lcd, SCREEN_HEIGHT, SCREEN_WIDTH},
        palette::SYSTEM_PALETTE,
        tile::Tile,
    },
};

use std::collections::HashMap;

pub const BYTES_PER_TILE: u16 = 16;
const TILE_MAP_AREA_SIZE: usize = 0x0400;
const TILE_SIZE: usize = 8;
// Tile maps are 1 byte indexes

const NUM_TILES_HIGH: usize = SCREEN_HEIGHT as usize / TILE_SIZE;
const NUM_TILES_WIDTH: usize = SCREEN_WIDTH as usize / TILE_SIZE;
const BACKGROUND_SIZE: usize = 256; // the background is 256x256 pixels, but only 160x144 is visible at a time
const LAYER_WIDTH: usize = BACKGROUND_SIZE / TILE_SIZE;

const BG_TILE_DATA_AREA_1_BASE_POINTER: u16 = 0x9000;

#[derive(Debug)]
pub struct PPU {}

impl PPU {
    pub fn new() -> Self {
        PPU {}
    }

    pub fn render_bg(&self, bus: &Bus, lcd: &Lcd) -> [u8; BUFFER_SIZE] {
        if !lcd.is_bg_window_enabled(bus) {
            return [0xFF; BUFFER_SIZE];
        }

        // to render the background, we must see which tile map area to use, then
        // parse the tile maps and then parse the tile data area
        let mut buffer = [0; BUFFER_SIZE];
        let tile_map = self.get_background_tile_map(bus, lcd);
        let tile_set = self.get_background_tile_set(bus, lcd, &tile_map);
        let palette = lcd.get_background_palette(bus);

        // now piece together the tiles
        for ty in 0..NUM_TILES_HIGH {
            for tx in 0..NUM_TILES_WIDTH {
                let map_num = ty * LAYER_WIDTH + tx;
                let tile_index = tile_map[map_num];
                // we should have the tile, otherwise something went wrong, so safe to unwrap here and crash
                let tile = tile_set.get(&tile_index).unwrap();

                // Loop through the pixels in the tile, get the colour from the
                // palette and copy it into the buffer
                for y in 0..TILE_SIZE {
                    let row = tile.get_row(y);
                    let pixel_y = y + ty * TILE_SIZE;

                    for x in 0..TILE_SIZE {
                        let pixel_x = x + tx * TILE_SIZE;
                        let value = row[x];
                        let palette_index = palette[value as usize];
                        let colour = SYSTEM_PALETTE[palette_index as usize];
                        self.copy_colour_into_buffer(&mut buffer, &colour, pixel_x, pixel_y);
                    }
                }
            }
        }

        return buffer;
    }

    fn copy_colour_into_buffer(
        &self,
        buffer: &mut [u8; BUFFER_SIZE],
        colour: &[u8; 4],
        x: usize,
        y: usize,
    ) {
        let buffer_index = 4 * (y * SCREEN_WIDTH as usize + x);
        for i in 0..4 {
            buffer[buffer_index + i] = colour[i];
        }
    }

    fn get_background_tile_set(&self, bus: &Bus, lcd: &Lcd, tile_map: &[u8]) -> HashMap<u8, Tile> {
        // Get the tiles we need from memory and parse them, store them in the set
        let mut tile_set = HashMap::new();
        let tile_data_area_start = lcd.get_bg_window_tile_data_area_start(bus);
        let is_using_signed_addressing = tile_data_area_start == BG_TILE_DATA_AREA_START_BANK_1;

        for i in 0..TILE_MAP_AREA_SIZE {
            // Loop through the tile map, if we haven't already parsed the tile, grab it from memory and parse it
            let tile_index = tile_map[i];
            if tile_set.contains_key(&tile_index) {
                continue;
            }

            let addr = if is_using_signed_addressing {
                let memory_index = ((tile_index as i8) as i16 * BYTES_PER_TILE as i16) as u16;
                memory_index
                    .overflowing_add(BG_TILE_DATA_AREA_1_BASE_POINTER)
                    .0
            } else {
                let memory_index = (tile_index as u16) * BYTES_PER_TILE;
                tile_data_area_start + memory_index
            };

            let tile = Tile::new(bus, addr);
            tile_set.insert(tile_index, tile);
        }

        return tile_set;
    }

    fn get_background_tile_map(&self, bus: &Bus, lcd: &Lcd) -> [u8; TILE_MAP_AREA_SIZE] {
        let tile_map_start = lcd.get_bg_tile_map_area_start(bus);
        // Tile map stores the index of the tile to be displayed
        let mut tile_map = [0; TILE_MAP_AREA_SIZE];

        for i in 0..TILE_MAP_AREA_SIZE as u16 {
            let addr = tile_map_start + i;
            let tile_index: u8 = bus.read_byte(addr);
            tile_map[i as usize] = tile_index;
        }

        return tile_map;
    }
}
