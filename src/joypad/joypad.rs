use crate::utils::{clear_bit, set_bit, test_bit};

pub const JOYPAD_REGISTER: usize = 0xFF00;

//  A_RIGHT = 0,
//  B_LEFT = 1,
//  SELECT_UP = 2,
//  START_DOWN = 3,
//  SELECT_D_PAD = 4,

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Button {
    A = 0,
    B = 1,
    Select = 2,
    Start = 3,
    Right = 4,
    Left = 5,
    Up = 6,
    Down = 7,
}

#[derive(Debug)]
pub struct Joypad {
    is_buttons_selected: bool, // bit 5
    is_dpad_selected: bool,    // bit 4
    buttons_pressed: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            is_buttons_selected: false,
            is_dpad_selected: false,
            buttons_pressed: 0xFF,
        }
    }

    pub fn press_button(&mut self, button: Button, is_pressed: bool) -> bool {
        let bit = button as u8;

        // Note that, rather unconventionally for the Game Boy, a button being pressed
        // is seen as the corresponding bit being 0, not 1.
        if is_pressed {
            // if this is true, then the button was not pressed earlier, so we can request an interrupt
            let should_request_interrupt = test_bit(self.buttons_pressed, bit);
            self.buttons_pressed = clear_bit(self.buttons_pressed, bit);
            return should_request_interrupt;
        } else {
            self.buttons_pressed = set_bit(self.buttons_pressed, bit);
        }

        return false;
    }

    pub fn read(&self) -> u8 {
        if !self.is_buttons_selected && !self.is_dpad_selected {
            return 0x3F; // bit 5 and 4 are 1, all buttons are 1
        }

        if self.is_buttons_selected && self.is_dpad_selected {
            panic!("Both buttons and dpad is selected");
        }

        // bit 4 is 0
        if self.is_dpad_selected {
            // buttons are on the higher nibble of button_pressed, then select bit 5
            return ((self.buttons_pressed & 0xF0) >> 4) | (1 << 5);
        }

        // bit 5 is 0
        if self.is_buttons_selected {
            // buttons are on the lower nibble of buttons_pressed, then select bit 4
            return self.buttons_pressed & 0x0F | (1 << 4);
        }

        unreachable!("Joypad read");
    }

    pub fn write(&mut self, byte: u8) {
        self.is_buttons_selected = !test_bit(byte, 5);
        self.is_dpad_selected = !test_bit(byte, 4);
        // lower nibble is read-only and other bits are unused
    }
}
