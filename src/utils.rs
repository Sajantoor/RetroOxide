pub fn test_bit(byte: u8, bit: u8) -> bool {
    // Check the value of byte at the specified bit
    let mask = 1 << bit;
    byte & mask != 0
}

pub fn set_bit(byte: u8, bit: u8) -> u8 {
    byte | 1 << bit
}

pub fn clear_bit(byte: u8, bit: u8) -> u8 {
    let mask = !(1 << bit);
    byte & mask
}
