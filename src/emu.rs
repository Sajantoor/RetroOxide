use crate::{cpu::cpu::CPU, rom::cartridge::Cartridge};

pub struct Context {
    is_running: bool,
    is_paused: bool,
    ticks: u64,
    cpu: CPU,
}

impl Context {
    pub fn new(cartridge: Cartridge) -> Self {
        Context {
            is_running: false,
            is_paused: false,
            ticks: 0,
            cpu: CPU::new(cartridge),
        }
    }

    pub fn start(&mut self) {
        self.is_running = true;
        self.is_paused = false;
        return self.run();
    }

    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    fn run(&mut self) {
        while self.is_running {
            if !self.is_paused {
                self.cpu.step();
                // delay
            } else {
                // sleep
            }

            self.ticks += 1;
        }
    }
}
