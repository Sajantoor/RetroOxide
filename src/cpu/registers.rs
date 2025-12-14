use std::cell::Cell;

#[derive(Debug)]
pub struct Registers {
    pub a: Cell<u8>,
    pub b: Cell<u8>,
    pub c: Cell<u8>,
    pub d: Cell<u8>,
    pub e: Cell<u8>,
    pub f: Cell<u8>, // contains the flags (lower of the AF register)
    pub h: Cell<u8>,
    pub l: Cell<u8>,
    pub sp: Cell<u16>,
    pub pc: Cell<u16>,
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            // TODO: These values may not be correct depending on the game.
            // https://gbdev.io/pandocs/Power_Up_Sequence.html#cpu-registers
            a: Cell::new(0x11),
            b: Cell::new(0x01),
            c: Cell::new(0),
            d: Cell::new(0),
            e: Cell::new(0x08),
            f: Cell::new(0),
            h: Cell::new(0),
            l: Cell::new(0x7C),
            sp: Cell::new(0xFFFE),
            pc: Cell::new(0x0100),
        }
    }

    pub fn get_bc(&self) -> u16 {
        (self.b.get() as u16) << 8 | self.c.get() as u16
    }

    pub fn set_bc(&self, value: u16) {
        self.b.set(((value & 0xFF00) >> 8) as u8);
        self.c.set((value & 0xFF) as u8);
    }

    pub fn get_de(&self) -> u16 {
        (self.d.get() as u16) << 8 | self.e.get() as u16
    }

    pub fn set_de(&self, value: u16) {
        self.d.set(((value & 0xFF00) >> 8) as u8);
        self.e.set((value & 0xFF) as u8);
    }

    pub fn get_hl(&self) -> u16 {
        (self.h.get() as u16) << 8 | self.l.get() as u16
    }

    pub fn set_hl(&self, value: u16) {
        self.h.set(((value & 0xFF00) >> 8) as u8);
        self.l.set((value & 0xFF) as u8);
    }

    pub fn get_af(&self) -> u16 {
        (self.a.get() as u16) << 8 | self.f.get() as u16
    }

    pub fn set_af(&self, value: u16) {
        self.a.set(((value & 0xFF00) >> 8) as u8);
        self.f.set((value & 0xFF) as u8);
    }

    /**
     * Setters and getters for flags
     */

    /**
     * Z
     */
    pub fn set_zero_flag(&mut self, flag: bool) {
        // Mask the lower (unset) bits and then set the flag bit (bit 7)
        self.f.set((self.f.get() & 0x7F) | ((flag as u8) << 7));
    }

    pub fn get_zero_flag(&self) -> bool {
        // Mask to remove all other bits, check if it's non zero
        return (self.f.get() & 0x80) != 0;
    }

    /**
     * N
     */
    pub fn set_subtraction_flag(&mut self, flag: bool) {
        self.f.set((self.f.get() & 0xBF) | ((flag as u8) << 6));
    }

    pub fn get_subtraction_flag(&self) -> bool {
        return (self.f.get() & 0x40) != 0;
    }

    /**
     * H
     */
    pub fn set_half_carry_flag(&mut self, flag: bool) {
        self.f.set((self.f.get() & 0xDF) | ((flag as u8) << 5));
    }

    pub fn get_half_carry_flag(&self) -> bool {
        return (self.f.get() & 0x20) != 0;
    }

    /**
     * C
     */
    pub fn set_carry_flag(&mut self, flag: bool) {
        self.f.set((self.f.get() & 0xEF) | ((flag as u8) << 4));
    }

    pub fn get_carry_flag(&self) -> bool {
        return (self.f.get() & 0x1) != 0;
    }
}
