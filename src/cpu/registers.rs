pub struct Registers {
    pub a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8, // contains the flags (lower of the AF register)
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

impl Registers {
    pub fn new() -> Self {
        unimplemented!("Unimplemented new registers");
    }

    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    /**
         *
        Table "r"
        8-bit registers
        Index	0	1	2	3	4	5	6	    7
        Value	B	C	D	E	H	L	(HL)	A
    */
    pub fn get_register_from_table_r(&self, i: u8) -> u8 {
        match i {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => {
                // (HL), cycles need to go up by 1 as well
                unimplemented!("Get value from memory at address HL");
            }
            7 => self.a,
            _ => panic!(
                "This should be unreachable since i has a 4 bit range, but got: {:?}",
                i
            ),
        }
    }

    pub fn set_register_from_table_r(&mut self, i: u8, value: u8) {
        match i {
            0 => self.b = value,
            1 => self.c = value,
            2 => self.d = value,
            3 => self.e = value,
            4 => self.h = value,
            5 => self.l = value,
            6 => {
                // (HL)
                unimplemented!("Set value to memory at address HL");
            }
            7 => self.a = value,
            _ => panic!(
                "This should be unreachable since i has a 4 bit range, but got: {:?}",
                i
            ),
        };
    }

    /**
     * Setters and getters for flags
     */

    /**
     * Z
     */
    pub fn set_zero_flag(&mut self, flag: bool) {
        // Mask the lower (unset) bits and then set the flag bit (bit 7)
        self.f = (self.f & 0x7F) | ((flag as u8) << 7);
    }

    pub fn get_zero_flag(&self) -> bool {
        // Mask to remove all other bits, check if it's non zero
        return (self.f & 0x80) != 0;
    }

    /**
     * N
     */
    pub fn set_substraction_flag(&mut self, flag: bool) {
        self.f = (self.f & 0xBF) | ((flag as u8) << 6);
    }

    pub fn get_substraction_flag(&self) -> bool {
        return (self.f & 0x40) != 0;
    }

    /**
     * H
     */
    pub fn set_half_carry_flag(&mut self, flag: bool) {
        self.f = (self.f & 0xDF) | ((flag as u8) << 5);
    }

    pub fn get_half_carry_flag(&self) -> bool {
        return (self.f & 0x20) != 0;
    }

    /**
     * C
     */
    pub fn set_carry_flag(&mut self, flag: bool) {
        self.f = (self.f & 0xEF) | ((flag as u8) << 4);
    }

    pub fn get_carry_flag(&self) -> bool {
        return (self.f & 0x1) != 0;
    }
}
