use crate::bus::bus::Bus;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InterruptType {
    VBlank = 0,
    LCDStat = 1,
    Timer = 2,
    Serial = 3,
    Joypad = 4,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum InterruptAddress {
    VBlank = 0x40,
    LCDStat = 0x48,
    Timer = 0x50,
    Serial = 0x58,
    Joypad = 0x60,
}

pub const INTERRUPT_ENABLE_ADDR: u16 = 0xFFFF;
pub const INTERRUPT_FLAG_ADDR: u16 = 0xFF0F;

pub fn get_requested_interrupt(interrupt_flags: u8) -> Option<InterruptType> {
    // Gets interrupts by priority
    for i in 0..5 {
        if (interrupt_flags & (1 << i)) != 0 {
            return Some(match i {
                0 => InterruptType::VBlank,
                1 => InterruptType::LCDStat,
                2 => InterruptType::Timer,
                3 => InterruptType::Serial,
                4 => InterruptType::Joypad,
                _ => unreachable!(),
            });
        }
    }

    return None;
}

pub fn request_interrupt(bus: &mut Bus, interrupt: InterruptType) {
    let current_interrupt_flags = bus.read_byte(INTERRUPT_ENABLE_ADDR);
    let updated_interrupt_flags = current_interrupt_flags | (1 << (interrupt as u8));
    bus.write_byte(INTERRUPT_FLAG_ADDR, updated_interrupt_flags);
}

pub fn acknowledge_interrupt(bus: &mut Bus, mut interrupt_flags: u8, interrupt: InterruptType) {
    // set the bit to zero
    interrupt_flags &= !(1 << (interrupt as u8));
    bus.write_byte(INTERRUPT_FLAG_ADDR, interrupt_flags);
}

pub fn get_interrupt_address(interrupt: InterruptType) -> InterruptAddress {
    match interrupt {
        InterruptType::VBlank => InterruptAddress::VBlank,
        InterruptType::LCDStat => InterruptAddress::LCDStat,
        InterruptType::Timer => InterruptAddress::Timer,
        InterruptType::Serial => InterruptAddress::Serial,
        InterruptType::Joypad => InterruptAddress::Joypad,
    }
}
