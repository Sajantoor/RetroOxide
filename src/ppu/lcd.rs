use crate::{
    bus::{
        bus::Bus,
        interrupt_flags::{self, InterruptType},
    },
    ppu::ppu::{BUFFER_SIZE, PPU},
    utils::test_bit,
};

const LCD_CONTROL_REGISTER: u16 = 0xFF40;
const LDC_STATUS_REGISTER: u16 = 0xFF41;
const LCD_Y_CORD_REGISTER: u16 = 0xFF44;
const LCD_Y_CORD_COMPARE_REGISTER: u16 = 0xFF45;
const BG_PALETTE: u16 = 0xFF47;

pub(crate) const SCREEN_HEIGHT: u8 = 160;
pub(crate) const SCREEN_WIDTH: u8 = 144;

const SCAN_LINES: u8 = 154;
const VISIBLE_SCAN_LINES: u8 = SCREEN_WIDTH;

const SCAN_LINE_TIME: u16 = 456; // 456 clock cycles 
const MODE_2_TIME: u16 = 80;
const MODE_3_TIME: u16 = 172;
const MODE_2_BOUNDS: u16 = SCAN_LINE_TIME - MODE_2_TIME;
const MODE_3_BOUNDS: u16 = MODE_2_BOUNDS - MODE_3_TIME;

#[derive(Debug, PartialEq, Eq, num_enum::IntoPrimitive)]
#[repr(u8)]
enum LcdControl {
    LdcEnable = 7,
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

#[derive(PartialEq, Eq)]
enum LcdMode {
    TransferringDataToLcdDriver = 3,
    SearchingSpritesAtt = 2,
    VBlank = 1,
    HBlank = 0,
}

#[derive(Debug)]
pub struct Lcd {
    scanline_counter: u16,
    ppu: PPU,
}

impl Lcd {
    pub fn new() -> Self {
        Lcd {
            scanline_counter: SCAN_LINE_TIME,
            ppu: PPU::new(),
        }
    }

    pub fn update_graphics(&mut self, bus: &mut Bus, cycles: usize) {
        self.update_ldc_status(bus);
        let cycles = cycles as u16;

        if !self.is_lcd_enabled(bus) {
            return;
        }

        self.scanline_counter = self.scanline_counter.saturating_sub(cycles);
        if self.scanline_counter > 0 {
            return;
        }

        // move to next scanline
        let current_line_ptr = bus.get_pointer(LCD_Y_CORD_REGISTER);
        let current_scan_line = *current_line_ptr;
        self.scanline_counter += SCAN_LINE_TIME;

        if current_scan_line == VISIBLE_SCAN_LINES {
            // Entered VBlank
            // draw the background here
            let buffer = self.ppu.render_bg(bus, &self);
            interrupt_flags::request_interrupt(bus, InterruptType::VBlank);
        } else if current_scan_line > SCAN_LINES {
            // Restart scanning from the top
            *current_line_ptr = 0;
        } else {
            *current_line_ptr += 1;
            // self.draw_scanline(bus);
        }
    }

    fn update_ldc_status(&mut self, bus: &mut Bus) {
        if !self.is_lcd_enabled(bus) {
            self.scanline_counter = 456;
            bus.write_byte(LCD_Y_CORD_REGISTER, 0);
            // reset status bits
            let status = bus.read_byte(LDC_STATUS_REGISTER) & 0xFC;
            bus.write_byte(LDC_STATUS_REGISTER, status);

            self.set_lcd_mode(bus, LcdMode::VBlank);
            return;
        }

        let current_line = bus.read_byte(LCD_Y_CORD_REGISTER);
        let current_mode = self.get_lcd_mode(bus);
        let mode: LcdMode;

        if current_line >= VISIBLE_SCAN_LINES {
            mode = LcdMode::VBlank;
        } else if self.scanline_counter >= MODE_2_BOUNDS {
            mode = LcdMode::SearchingSpritesAtt;
        } else if self.scanline_counter >= MODE_3_BOUNDS {
            mode = LcdMode::TransferringDataToLcdDriver;
        } else {
            mode = LcdMode::HBlank;
        }

        if mode != current_mode {
            let interrupt_requested = self.is_interrupt_requested(bus, &mode);
            if interrupt_requested {
                interrupt_flags::request_interrupt(bus, InterruptType::LCDStat);
            }
            self.set_lcd_mode(bus, mode);
        }

        let ly_compare = bus.read_byte(LCD_Y_CORD_COMPARE_REGISTER);
        if current_line == ly_compare {
            self.set_lyc_equal_ly(bus, true);

            if self.is_lyc_equal_ly_interrupt_enabled(bus) {
                interrupt_flags::request_interrupt(bus, InterruptType::LCDStat);
            }
        } else if self.is_lyc_equal_ly(bus) {
            self.set_lyc_equal_ly(bus, false);
        }
    }

    fn draw_scanline(&self, bus: &mut Bus) {
        todo!();
    }

    fn is_lyc_equal_ly_interrupt_enabled(&self, bus: &Bus) -> bool {
        let byte = bus.read_byte(LDC_STATUS_REGISTER);
        return test_bit(byte, LcdStatus::LycIntSelect as u8);
    }

    fn is_lyc_equal_ly(&self, bus: &Bus) -> bool {
        let byte = bus.read_byte(LDC_STATUS_REGISTER);
        return test_bit(byte, LcdStatus::LycEqLy as u8);
    }

    fn set_lyc_equal_ly(&self, bus: &mut Bus, equal: bool) {
        let mut byte = bus.read_byte(LDC_STATUS_REGISTER);
        if equal {
            byte |= 1 << (LcdStatus::LycEqLy as u8);
        } else {
            byte &= !(1 << (LcdStatus::LycEqLy as u8));
        }
        bus.write_byte(LDC_STATUS_REGISTER, byte);
    }

    fn set_lcd_mode(&self, bus: &mut Bus, mode: LcdMode) {
        let mut byte = bus.read_byte(LDC_STATUS_REGISTER);
        byte &= !0x03; // Clear the last two bits
        byte |= mode as u8;
        bus.write_byte(LDC_STATUS_REGISTER, byte);
    }

    fn is_interrupt_requested(&self, bus: &Bus, mode: &LcdMode) -> bool {
        let byte = bus.read_byte(LDC_STATUS_REGISTER);
        return match mode {
            LcdMode::HBlank => test_bit(byte, LcdStatus::Mode0IntSelect as u8),
            LcdMode::VBlank => test_bit(byte, LcdStatus::Mode1IntSelect as u8),
            LcdMode::SearchingSpritesAtt => test_bit(byte, LcdStatus::Mode2IntSelect as u8),
            LcdMode::TransferringDataToLcdDriver => false,
        };
    }

    fn read_from_lcd_control_register(&self, bus: &Bus) -> u8 {
        bus.read_byte(LCD_CONTROL_REGISTER)
    }

    fn is_lcd_enabled(&self, bus: &Bus) -> bool {
        let byte = self.read_from_lcd_control_register(bus);
        return test_bit(byte, LcdControl::LdcEnable.into());
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
            2 => LcdMode::SearchingSpritesAtt,
            3 => LcdMode::TransferringDataToLcdDriver,
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
