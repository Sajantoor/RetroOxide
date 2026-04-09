use crate::{
    bus::timer::Timer,
    cpu::cpu::CPU,
    ppu::lcd::{BUFFER_SIZE, Lcd},
    rom::cartridge::Cartridge,
};

#[derive(Debug)]
pub struct Context {
    is_running: bool,
    is_paused: bool,
    cpu: CPU,
    timer: Timer,
    lcd: Lcd,
}

impl Context {
    pub fn new(cartridge: Cartridge) -> Self {
        Context {
            is_running: false,
            is_paused: false,
            cpu: CPU::new(&cartridge),
            timer: Timer::new(),
            lcd: Lcd::new(),
        }
    }

    pub fn start(&mut self) {
        self.is_running = true;
        self.is_paused = false;
    }

    pub fn stop(&mut self) {
        // TODO: Save the game here
        self.is_running = false;
    }

    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    pub fn is_running(&self) -> bool {
        return self.is_running && !self.is_paused;
    }

    pub fn step(&mut self) -> Option<[u8; BUFFER_SIZE]> {
        if !self.is_running() {
            return None;
        }

        let cycle_diff = self.cpu.step();
        self.timer.update_timer(&mut self.cpu.bus, cycle_diff);
        let buffer = self.lcd.update_graphics(&mut self.cpu.bus, cycle_diff);
        self.cpu.handle_interrupts();
        return buffer;
    }
}
