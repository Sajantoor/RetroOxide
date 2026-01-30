use crate::{
    bus::bus::Bus,
    ppu::{
        lcd::{Lcd, SCREEN_HEIGHT, SCREEN_WIDTH},
        tile::Tile,
    },
};

use std::collections::HashMap;

pub const BYTES_PER_TILE: u16 = 16;
const TILE_MAP_AREA_SIZE: usize = 0x0400;
// Tile maps are 1 byte indexes

const BUFFER_SIZE: usize = (SCREEN_HEIGHT as usize * SCREEN_WIDTH as usize) * 4; // 4 for RGBA
const NUM_TILES_HIGH: u16 = SCREEN_HEIGHT as u16 / 8;
const NUM_TILES_WIDTH: u16 = SCREEN_WIDTH as u16 / 8;

#[derive(Debug)]
pub struct PPU {}

impl PPU {
    pub fn new() -> Self {
        PPU {}
    }

    pub fn render_bg(&self, bus: &Bus, lcd: &Lcd) -> [u8; BUFFER_SIZE] {
        // to render the background, we must see which tile map area to use, then
        // parse the tile maps and then parse the tile data area
        if lcd.is_bg_window_enabled(bus) {
            [0; BUFFER_SIZE];
        }

        let buffer = [0; BUFFER_SIZE];

        let tile_map_start: u16 = lcd.get_bg_tile_map_area_start(bus);
        // Tile map stores the index of the tile to be displayed
        let mut tile_map: [u8; TILE_MAP_AREA_SIZE] = [0; TILE_MAP_AREA_SIZE];

        for i in 0..TILE_MAP_AREA_SIZE as u16 {
            let addr = tile_map_start + i;
            let tile_index = bus.read_byte(addr);
            tile_map[i as usize] = tile_index;
        }

        // Get the tiles we need from memory and parse them, store them in the set
        let mut tile_set = HashMap::new();
        // TODO: Indexing is a bit weird depending on what this value is
        let title_data_area_start = lcd.get_bg_window_tile_data_area_start(bus);
        for i in 0..TILE_MAP_AREA_SIZE {
            let tile_index = tile_map[i];
            if tile_set.contains_key(&tile_index) {
                continue;
            }

            let memory_index = (tile_index as u16) * BYTES_PER_TILE;
            let addr = title_data_area_start + memory_index;
            let tile = Tile::new(bus, addr);
            tile_set.insert(tile_index, tile);
        }

        // now piece together the tiles
        for ty in 0..NUM_TILES_HIGH {
            for tx in 0..NUM_TILES_WIDTH {
                let map_num = ty * 20 + tx;
                let tile_index = tile_map[map_num as usize];
                // we should have the tile, otherwise something went wrong, so safe to unwrap here and crash
                let tile = tile_set.get(&tile_index).unwrap();
                tile.print_tile();
            }
        }

        buffer
    }
}
