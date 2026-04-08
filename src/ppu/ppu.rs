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

const BACKGROUND_SIZE: usize = 256; // the background is 256x256 pixels, but only 160x144 is visible at a time
const LAYER_WIDTH: usize = BACKGROUND_SIZE / TILE_SIZE;

const BG_TILE_DATA_AREA_1_BASE_POINTER: u16 = 0x9000;

#[derive(Debug)]
pub struct PPU {}

impl PPU {
    pub fn new() -> Self {
        PPU {}
    }

    pub fn render(&self, bus: &Bus, lcd: &Lcd) -> [u8; BUFFER_SIZE] {
        let mut buffer = [0; BUFFER_SIZE];
        self.render_background(bus, lcd, &mut buffer);
        self.render_window(bus, lcd, &mut buffer);
        return buffer;
    }

    fn render_background(&self, bus: &Bus, lcd: &Lcd, buffer: &mut [u8; BUFFER_SIZE]) {
        if !lcd.is_bg_enabled(bus) {
            buffer.fill(0xF);
            return;
        }

        // to render the background, we must see which tile map area to use, then
        // parse the tile maps and then parse the tile data area
        let tile_map = self.get_background_tile_map(bus, lcd);
        let tile_set = self.get_background_window_tile_set(bus, lcd, &tile_map);
        let palette = lcd.get_background_window_palette(bus);
        let (x_offset, y_offset) = lcd.get_background_scroll(bus);

        // now piece together the tiles
        for py in 0..SCREEN_HEIGHT as usize {
            let y = (py + y_offset as usize) % BACKGROUND_SIZE;
            let tile_index_y = y % TILE_SIZE;

            for px in 0..SCREEN_WIDTH as usize {
                let x = (px + x_offset as usize) % BACKGROUND_SIZE;
                let map_num = (y / TILE_SIZE) * LAYER_WIDTH + (x / TILE_SIZE);
                let tile_index = tile_map[map_num];
                // we should have the tile, otherwise something went wrong, so safe to unwrap here and crash
                let tile = tile_set.get(&tile_index).unwrap();

                let tile_index_x = x % TILE_SIZE;
                let row = tile.get_row(tile_index_y);

                let value = row[tile_index_x];
                let palette_index = palette[value as usize];
                let colour = SYSTEM_PALETTE[palette_index as usize];
                self.copy_colour_into_buffer(buffer, &colour, px, py);
            }
        }
    }

    fn render_window(&self, bus: &Bus, lcd: &Lcd, buffer: &mut [u8; BUFFER_SIZE]) {
        if !lcd.is_window_enabled(bus) {
            return;
        }

        let tile_map = self.get_window_tile_map(bus, lcd);
        let tile_set = self.get_background_window_tile_set(bus, lcd, &tile_map);
        let palette = lcd.get_background_window_palette(bus);
        let (x_offset, y_offset) = lcd.get_window_position(bus);

        for py in (y_offset as usize)..(SCREEN_HEIGHT as usize) {
            let tile_index_y = py % TILE_SIZE;

            for px in (x_offset as usize)..(SCREEN_WIDTH as usize) {
                let tile_index_x = px % TILE_SIZE;
                let map_num = (py / TILE_SIZE) * LAYER_WIDTH + (px / TILE_SIZE);
                let tile_index = tile_map[map_num];
                let tile = tile_set.get(&tile_index).unwrap();

                let row = tile.get_row(tile_index_y);

                let value = row[tile_index_x];
                let palette_index = palette[value as usize];
                let colour = SYSTEM_PALETTE[palette_index as usize];
                self.copy_colour_into_buffer(buffer, &colour, px, py);
            }
        }
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

    fn get_background_window_tile_set(
        &self,
        bus: &Bus,
        lcd: &Lcd,
        tile_map: &[u8],
    ) -> HashMap<u8, Tile> {
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
        self.get_tile_map(bus, lcd, true)
    }

    fn get_window_tile_map(&self, bus: &Bus, lcd: &Lcd) -> [u8; TILE_MAP_AREA_SIZE] {
        self.get_tile_map(bus, lcd, false)
    }

    fn get_tile_map(&self, bus: &Bus, lcd: &Lcd, is_background: bool) -> [u8; TILE_MAP_AREA_SIZE] {
        let tile_map_start = if is_background {
            lcd.get_bg_tile_map_area_start(bus)
        } else {
            lcd.get_window_tile_map_area_start(bus)
        };
        // Tile map stores the index of the tile to be displayed
        let mut tile_map = [0; TILE_MAP_AREA_SIZE];

        for i in 0..TILE_MAP_AREA_SIZE as u16 {
            let addr = tile_map_start + i;
            let tile_index = bus.read_byte(addr);
            tile_map[i as usize] = tile_index;
        }

        return tile_map;
    }
}
