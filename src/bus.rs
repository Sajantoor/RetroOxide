pub struct Bus {
    // Temporary ram
    pub ram: [u8; 0x2000],
}

impl Bus {
    pub fn new() -> Self {
        Bus { ram: [0; 0x2000] }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    // Little endian read for word (2 bytes)
    pub fn read_word(&self, addr: u16) -> u16 {
        let low_byte = self.read_byte(addr) as u16;
        let high_byte = self.read_byte(addr + 1) as u16;
        (high_byte << 8) | low_byte
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        self.ram[addr as usize] = value;
    }

    // Little endian write for word (2 bytes)
    pub fn write_word(&mut self, addr: u16, value: u16) {
        let low_byte = (value & 0x00FF) as u8;
        let high_byte = (value >> 8) as u8;
        self.write_byte(addr, low_byte);
        self.write_byte(addr + 1, high_byte);
    }

    pub fn get_pointer(&mut self, addr: u16) -> &mut u8 {
        &mut self.ram[addr as usize]
    }
}
