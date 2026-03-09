use crate::{
    bus::{
        bus::Bus,
        interrupt_flags::{self, InterruptType},
    },
    ppu::ppu::PPU,
    utils::test_bit,
};

const LCD_CONTROL_REGISTER: u16 = 0xFF40;
const LDC_STATUS_REGISTER: u16 = 0xFF41;
const LCD_Y_CORD_REGISTER: u16 = 0xFF44;
const LCD_Y_CORD_COMPARE_REGISTER: u16 = 0xFF45;
const BG_PALETTE: u16 = 0xFF47;

pub(crate) const BUFFER_SIZE: usize = (SCREEN_HEIGHT as usize * SCREEN_WIDTH as usize) * 4; // 4 for RGBA
pub(crate) const SCREEN_HEIGHT: u8 = 160;
pub(crate) const SCREEN_WIDTH: u8 = 144;

const SCAN_LINES: u8 = 154;
const VISIBLE_SCAN_LINES: u8 = SCREEN_WIDTH;

const SCAN_LINE_TIME: usize = 114; // 456 dots per scanline, 4 dots per M cycle
const HBLANK_TIME: usize = 204 / 4;
const OAM_READ_TIME: usize = 80 / 4;
const VRAM_READ_TIME: usize = 172 / 4;

#[derive(Debug, PartialEq, Eq, num_enum::IntoPrimitive)]
#[repr(u8)]
enum LcdControl {
    LcdEnable = 7,
    WindowTitleMapArea = 6,
    WindowEnable = 5,
    BgWindowTitleArea = 4,
    BgTileMapArea = 3,
    ObjSize = 2,
    ObjEnable = 1,
    BgWindowEnable = 0,
}

enum LcdStatus {
    // bit 7 is unused
    LycIntSelect = 6,
    Mode2IntSelect = 5,
    Mode1IntSelect = 4,
    Mode0IntSelect = 3,
    LycEqLy = 2,
    // bit 0 - 1 select the LDC mode
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum LcdMode {
    HBlank = 0,
    VBlank = 1,
    OAMRead = 2,
    VRAMRead = 3,
}

#[derive(Debug)]
pub struct Lcd {
    cycles: usize,
    ppu: PPU,
}

impl Lcd {
    pub fn new() -> Self {
        Lcd {
            cycles: 0,
            ppu: PPU::new(),
        }
    }

    pub fn update_graphics(&mut self, bus: &mut Bus, cycles: usize) -> Option<[u8; BUFFER_SIZE]> {
        if !self.is_lcd_enabled(bus) {
            self.cycles = 0;
            bus.write_byte(LCD_Y_CORD_REGISTER, 0);
            self.set_lcd_mode(bus, LcdMode::OAMRead);
            return None;
        }

        self.update_ldc_status(bus, cycles);

        let current_scan_line = self.get_current_scanline(bus);

        if current_scan_line == VISIBLE_SCAN_LINES {
            let buffer = self.ppu.render_bg(bus, &self);
            return Some(buffer);
        }

        return None;
    }

    fn get_current_scanline(&self, bus: &Bus) -> u8 {
        return bus.read_byte(LCD_Y_CORD_REGISTER);
    }

    fn update_ldc_status(&mut self, bus: &mut Bus, cycles: usize) {
        self.cycles += cycles;

        let current_mode = self.get_lcd_mode(bus);
        let updated_mode = self.update_mode(bus, &current_mode);
        //  TODO: check this every line instead
        let is_coincidence = self.set_lyc_equal_ly(bus);
        if current_mode != updated_mode
            && self.is_interrupt_requested(bus, &updated_mode, is_coincidence)
        {
            interrupt_flags::request_interrupt(bus, InterruptType::LCDStat);
        }
    }

    fn update_mode(&mut self, bus: &mut Bus, current_mode: &LcdMode) -> LcdMode {
        let current_line_ptr = bus.get_pointer(LCD_Y_CORD_REGISTER);
        match current_mode {
            LcdMode::VBlank => {
                if self.cycles >= SCAN_LINE_TIME {
                    self.cycles -= SCAN_LINE_TIME;
                    *current_line_ptr += 1;

                    if *current_line_ptr == SCAN_LINES {
                        *current_line_ptr = 0;
                        self.set_lcd_mode(bus, LcdMode::OAMRead);
                        return LcdMode::OAMRead;
                    }
                    // else remain in vertical blank
                }
            }
            LcdMode::HBlank => {
                if self.cycles >= HBLANK_TIME {
                    self.cycles -= HBLANK_TIME;
                    *current_line_ptr += 1;

                    if *current_line_ptr == VISIBLE_SCAN_LINES - 1 {
                        self.set_lcd_mode(bus, LcdMode::VBlank);
                        interrupt_flags::request_interrupt(bus, InterruptType::VBlank);
                        return LcdMode::VBlank;
                    } else {
                        self.set_lcd_mode(bus, LcdMode::OAMRead);
                        return LcdMode::OAMRead;
                    }
                }
            }
            LcdMode::OAMRead => {
                if self.cycles >= OAM_READ_TIME {
                    self.cycles -= OAM_READ_TIME;
                    self.set_lcd_mode(bus, LcdMode::VRAMRead);
                    return LcdMode::VRAMRead;
                }
            }
            LcdMode::VRAMRead => {
                if self.cycles >= VRAM_READ_TIME {
                    self.cycles -= VRAM_READ_TIME;
                    self.set_lcd_mode(bus, LcdMode::HBlank);
                    return LcdMode::HBlank;
                }
            }
        }

        return current_mode.clone();
    }

    fn set_lyc_equal_ly(&self, bus: &mut Bus) -> bool {
        let lyc = bus.read_byte(LCD_Y_CORD_COMPARE_REGISTER);
        let ly = bus.read_byte(LCD_Y_CORD_REGISTER);
        let equal = lyc == ly;

        let mut byte = bus.read_byte(LDC_STATUS_REGISTER);
        if equal {
            byte |= 1 << (LcdStatus::LycEqLy as u8);
        } else {
            byte &= !(1 << (LcdStatus::LycEqLy as u8));
        }

        bus.write_byte(LDC_STATUS_REGISTER, byte);
        return equal;
    }

    fn set_lcd_mode(&self, bus: &mut Bus, mode: LcdMode) {
        let mut byte = bus.read_byte(LDC_STATUS_REGISTER);
        byte &= !0x03; // Clear the last two bits
        byte |= mode as u8;
        bus.write_byte(LDC_STATUS_REGISTER, byte);
    }

    fn is_interrupt_requested(&self, bus: &Bus, mode: &LcdMode, is_coincidence: bool) -> bool {
        let byte = bus.read_byte(LDC_STATUS_REGISTER);
        if is_coincidence {
            if test_bit(byte, LcdStatus::LycIntSelect as u8) {
                return true;
            }
        }

        return match mode {
            LcdMode::HBlank => test_bit(byte, LcdStatus::Mode0IntSelect as u8),
            LcdMode::VBlank => test_bit(byte, LcdStatus::Mode1IntSelect as u8),
            LcdMode::OAMRead => test_bit(byte, LcdStatus::Mode2IntSelect as u8),
            LcdMode::VRAMRead => false,
        };
    }

    fn read_from_lcd_control_register(&self, bus: &Bus) -> u8 {
        bus.read_byte(LCD_CONTROL_REGISTER)
    }

    fn is_lcd_enabled(&self, bus: &Bus) -> bool {
        let byte = self.read_from_lcd_control_register(bus);
        return test_bit(byte, LcdControl::LcdEnable.into());
    }

    fn is_window_enabled(&self, bus: &Bus) -> bool {
        let byte = self.read_from_lcd_control_register(bus);
        return test_bit(byte, LcdControl::WindowEnable.into());
    }

    pub fn get_bg_window_tile_data_area_start(&self, bus: &Bus) -> u16 {
        let byte = self.read_from_lcd_control_register(bus);
        let value = test_bit(byte, LcdControl::BgWindowTitleArea.into());
        if value { 0x8000 } else { 0x8800 }
    }

    pub fn get_bg_tile_map_area_start(&self, bus: &Bus) -> u16 {
        let byte = self.read_from_lcd_control_register(bus);
        let value = test_bit(byte, LcdControl::BgTileMapArea.into());
        if value { 0x9C00 } else { 0x9800 }
    }

    pub fn is_bg_window_enabled(&self, bus: &Bus) -> bool {
        let byte = self.read_from_lcd_control_register(bus);
        return test_bit(byte, LcdControl::BgWindowEnable.into());
    }

    fn get_lcd_mode(&self, bus: &Bus) -> LcdMode {
        let byte = bus.read_byte(LDC_STATUS_REGISTER);
        let status = 0x03 & byte;
        return match status {
            0 => LcdMode::HBlank,
            1 => LcdMode::VBlank,
            2 => LcdMode::OAMRead,
            3 => LcdMode::VRAMRead,
            _ => panic!("Invalid LCD mode"),
        };
    }

    pub fn get_background_palette(&self, bus: &Bus) -> [u8; 4] {
        let byte = bus.read_byte(BG_PALETTE);
        return self.decode_palette(byte);
    }

    fn decode_palette(&self, byte: u8) -> [u8; 4] {
        // 	            7	6	5	4	3	2	1	0
        // Color for...	ID 3	ID 2	ID 1	ID 0
        return [
            byte & 0x03,        // bits 1 and 2
            (byte & 0x0C) >> 2, // bits 3 and 4, shifted
            (byte & 0x0C) >> 4, // bits 5 and 6, shifted
            (byte & 0xC0) >> 6, // bits 7 and 8, shifted
        ];
    }
}
