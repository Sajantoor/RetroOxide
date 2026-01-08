pub fn test_bit(byte: u8, bit: u8) -> bool {
    // Check the value of byte at the specified bit
    let mask = 1 << bit;
    byte & mask != 0
}
