use crate::{bus::timer::Timer, cpu::cpu::CPU, ppu::lcd::Lcd, rom::cartridge::Cartridge};

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
        return self.run();
    }

    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    pub fn step(&mut self) {
        let cycle_diff = self.cpu.step();
        self.timer.update_timer(&mut self.cpu.bus, cycle_diff);
        self.lcd.update_graphics(&mut self.cpu.bus, cycle_diff);
        self.cpu.handle_interrupts();
    }

    fn run(&mut self) {
        while self.is_running {
            if !self.is_paused {
                self.step();
                // delay
            } else {
                // sleep
            }
        }
    }
}
