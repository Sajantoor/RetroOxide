// TAC register (FF07)
// Value overflows goes back to the value in TMA register (FPF06)

use crate::{
    bus::{
        bus::Bus,
        interrupt_flags::{self, InterruptType},
    },
    utils::{self},
};

const DIVIDER_REGISTER: u16 = 0xFF04;
const TIMA_REGISTER: u16 = 0xFF05; // timer counter register
const TMA_REGISTER: u16 = 0xFF06; // timer modulo 
const TAC_REGISTER: u16 = 0xFF07; // timer control 

#[derive(Debug)]
pub struct Timer {
    counter: i32,
    divider_counter: i32,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            counter: 0,
            divider_counter: 0,
        }
    }

    pub fn update_timer(&mut self, bus: &mut Bus, cycles: usize) {
        let cycles = cycles.try_into().unwrap();

        self.update_divider_register(bus, cycles);

        if !self.is_clock_enabled(bus) {
            return;
        }

        self.counter -= cycles;

        if self.counter < 0 {
            let timer_value = bus.read_byte(TIMA_REGISTER);
            if timer_value == 0xFF {
                let tma_value = bus.read_byte(TMA_REGISTER);
                bus.write_byte(TIMA_REGISTER, tma_value);
                interrupt_flags::request_interrupt(bus, InterruptType::Timer);
            } else {
                bus.write_byte(TIMA_REGISTER, timer_value + 1);
            }

            self.select_clock(bus);
        }
    }

    fn is_clock_enabled(&self, bus: &mut Bus) -> bool {
        //  2nd bit defines if it's enabled or not
        let byte = bus.read_byte(TAC_REGISTER);
        return utils::test_bit(byte, 2);
    }

    fn select_clock(&mut self, bus: &mut Bus) {
        let value = self.get_clock_frequency(bus);

        match value {
            0 => self.counter = 256,
            1 => self.counter = 4,
            2 => self.counter = 16,
            3 => self.counter = 64,
            _ => unreachable!("Invalid clock select value"),
        }
    }

    fn get_clock_frequency(&self, bus: &Bus) -> u8 {
        let byte = bus.read_byte(TAC_REGISTER);
        // first 2 bits determine the clock select
        let mask = 0x03;
        let value = byte & mask;
        return value;
    }

    fn update_divider_register(&mut self, bus: &mut Bus, cycles: i32) {
        self.divider_counter += cycles;
        if self.divider_counter >= 0xFF {
            self.divider_counter = self.divider_counter - 0xFF;
            // TODO: This is a hack since we're not allowed to write to the divider register directly
            let byte = bus.get_pointer(DIVIDER_REGISTER);
            *byte = byte.wrapping_add(1);
        }
    }
}
