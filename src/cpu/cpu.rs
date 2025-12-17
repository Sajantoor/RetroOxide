use std::cell::Cell;

use crate::bus::Bus;
use crate::cpu::registers::Registers;
use crate::rom::cartridge::Cartridge;

#[derive(Debug)]
pub struct CPU {
    registers: Registers,
    // T-Edge
    //     A single tick of the Game Boy's clock, from low to high, or high to low - 8,388,608 hz
    // T-Cycle (t)
    //     Two T-Edges - 4,194,304 hz
    // M-Cycle (m)
    //     Four T-Cycles - 1,048,576 hz
    cycles: Cell<usize>, // in M-cycle
    bus: Bus,
    ime_flag: bool,
}

/**
 * For each instruction, we need to emulate the function + addressing mode + cycles
*/
impl CPU {
    pub fn new(cartridge: Cartridge) -> Self {
        CPU {
            registers: Registers::new(),
            cycles: Cell::new(0),
            bus: Bus::new(cartridge),
            ime_flag: false, // IME is unset (interrupts are disabled) when the game starts running.
        }
    }

    fn print_state(&self) {
        println!(
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            self.registers.a.get(),
            self.registers.f.get(),
            self.registers.b.get(),
            self.registers.c.get(),
            self.registers.d.get(),
            self.registers.e.get(),
            self.registers.h.get(),
            self.registers.l.get(),
            self.registers.sp.get(),
            self.registers.pc.get(),
            self.bus.read_byte(self.registers.pc.get()),
            self.bus.read_byte(self.registers.pc.get() + 1),
            self.bus.read_byte(self.registers.pc.get() + 2),
            self.bus.read_byte(self.registers.pc.get() + 3)
        );
    }

    pub fn step(&mut self) {
        self.print_state();
        let opcode = self.next_byte();
        self.handle_instruction(opcode);
    }

    fn next_byte(&self) -> u8 {
        let pc = self.registers.pc.get();
        let byte = self.bus.read_byte(pc);
        self.registers.pc.set(pc + 1);
        return byte;
    }

    fn next_word(&self) -> u16 {
        let pc = self.registers.pc.get();
        let word = self.bus.read_word(pc);
        self.registers.pc.set(pc + 2);
        return word;
    }

    // instructions are prefix byte, opcode (byte), displacement byte, intermediate data
    pub fn handle_instruction(&mut self, opcode: u8) {
        // Referenced: http://archive.gbdev.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
        let x = (opcode >> 6) & 0x03; // bits 7-6
        let y = (opcode >> 3) & 0x07; // bits 5-3
        let z = opcode & 0x07; // bits 2-0
        let p = y >> 1 & 0x03; // bits 5-4
        let q = (opcode >> 3 & 0x01) == 1; // bit 3

        // fallback to an "invalid" instruction is NOP
        match x {
            // Relative jumps and assorted ops
            0 => match z {
                0 => match y {
                    0 => self.nop(),
                    1 => {
                        // LD (nn), SP
                        let nn = self.next_word();
                        let value = self.registers.sp.get();
                        self.bus.write_word(nn, value);
                        self.increment_cycles(5);
                    }
                    2 => self.stop(),
                    3 => {
                        let d: i8 = self.next_byte() as i8;
                        self.jr(d);
                    }
                    4..=7 => {
                        let d: i8 = self.next_byte() as i8;
                        self.jr_conditional(d, y - 4);
                    }
                    _ => panic!("Y has range 4 - 7, got {:?}", y),
                },

                1 => {
                    match q {
                        false => {
                            // Load instruction:
                            let nn = self.next_word();
                            self.set_register_from_table_rp(p, nn);
                        }
                        true => {
                            // Add instruction
                            let value = self.get_register_from_table_rp(p);
                            self.add_hl(value);
                        }
                    }
                }

                2 => {
                    // Load instructions involving A and (BC), (DE), (HL+), (HL-)
                    let pointer: u16;
                    match p {
                        0 => pointer = self.registers.get_bc(),
                        1 => pointer = self.registers.get_de(),
                        2 => {
                            pointer = self.registers.get_hl();
                            self.registers.set_hl(pointer.wrapping_add(1));
                        }
                        3 => {
                            pointer = self.registers.get_hl();
                            self.registers.set_hl(pointer.wrapping_sub(1));
                        }
                        _ => panic!("P has range 0 - 3, got {:?}", p),
                    }

                    match q {
                        false => {
                            let source = self.registers.a.get();
                            self.bus.write_byte(pointer, source);
                            self.increment_cycles(2);
                        }
                        true => {
                            let value = self.bus.read_byte(pointer);
                            self.registers.a.set(value);
                            self.increment_cycles(2);
                        }
                    }
                }

                3 => {
                    if q == false {
                        self.inc_16(p)
                    } else {
                        self.dec_16(p)
                    }
                }

                4 => self.inc(y),
                5 => self.dec(y),
                6 => {
                    let n = self.next_byte();
                    load(self.get_register_from_table_r(y), n);
                    self.increment_cycles(2);
                }
                7 => match y {
                    0 => self.rlca(),
                    1 => self.rrca(),
                    2 => self.rla(),
                    3 => self.rra(),
                    4 => self.daa(),
                    5 => self.cpl(),
                    6 => self.scf(),
                    7 => self.ccf(),
                    _ => panic!("Y has range 0 - 7, got {:?}", y),
                },

                _ => panic!("Z has range 0 - 7, got {:?}", z),
            },

            // Load and halt instructions
            1 => {
                if y == 6 && z == 6 {
                    return self.halt();
                } else {
                    let z = *(self.get_register_from_table_r(z));
                    let y = self.get_register_from_table_r(y);
                    load(y, z);
                    self.increment_cycles(1);
                }
            }

            // Operate on accumulator and register/memory location
            2 => {
                let value = *self.get_register_from_table_r(z);
                match y {
                    0 => self.add(value),
                    1 => self.adc(value),
                    2 => self.sub(value),
                    3 => self.sbc(value),
                    4 => self.and(value),
                    5 => self.xor(value),
                    6 => self.or(value),
                    7 => self.cp(value),
                    _ => panic!(
                        "Should not be able to reach this value, y only has a range of 0-7, got {:?}",
                        y
                    ),
                }
            }

            3 => match z {
                0 => match y {
                    0..4 => self.ret_conditional(y),
                    4 => {
                        // LD (0xFF00 + n), A
                        let n = self.next_byte();
                        let addr = 0xFF00 + (n as u16);
                        self.load_to_memory(addr, self.registers.a.get());
                    }
                    5 => {
                        let d = self.next_byte() as i8;
                        self.add_sp(d);
                    }
                    6 => {
                        // LD A, (0xFF00 + n)
                        let n = self.next_byte();
                        let addr = 0xFF00 + (n as u16);
                        let value = self.bus.read_byte(addr);
                        self.registers.a.set(value);
                        self.increment_cycles(2);
                    }
                    7 => {
                        // LD HL, SP + d
                        let n = self.next_byte() as i8 as i16;
                        let sp = self.registers.sp.get() as i16;
                        let value = sp.wrapping_add(n);

                        self.registers.set_hl(value as u16);
                        self.increment_cycles(3);

                        // Set flags
                        self.registers
                            .set_half_carry_flag(((sp & 0xF) + (n & 0xF)) > 0xF);
                        self.registers
                            .set_carry_flag(((sp & 0xFF) + (n & 0xFF)) > 0xFF);
                        self.registers.set_zero_flag(false);
                        self.registers.set_subtraction_flag(false);
                    }
                    _ => panic!("Y has range 0 - 7, got {:?}", y),
                },
                1 => {
                    if !q {
                        self.pop(p);
                    } else {
                        match p {
                            0 => self.ret(),
                            1 => self.reti(),
                            2 => self.jp(self.registers.get_hl()),
                            3 => {
                                // LD SP,HL
                                let hl = self.registers.get_hl();
                                self.registers.sp.set(hl);
                                self.increment_cycles(2);
                            }
                            _ => panic!("P has range 0 - 4 got {:?}", p),
                        }
                    }
                }

                2 => match y {
                    0..4 => {
                        let nn = self.next_word();
                        self.jp_cc(nn, y);
                    }
                    4 => {
                        // LD (oxFF00 + C), A
                        let addr = (self.registers.c.get() as u16) + 0xFF00;
                        self.load_to_memory(addr, self.registers.a.get());
                    }
                    5 => {
                        // LD (nn), A
                        let nn = self.next_word();
                        self.load_to_memory(nn, self.registers.a.get());
                    }
                    6 => {
                        // LD A, (0xFF00 + C)
                        let addr = (self.registers.c.get() as u16) + 0xFF00;
                        let value = self.bus.read_byte(addr);
                        self.registers.a.set(value);
                        self.increment_cycles(2);
                    }
                    7 => {
                        // LD A, (nn)
                        let addr = self.next_word();
                        let value = self.bus.read_byte(addr);
                        self.registers.a.set(value);
                        self.increment_cycles(3);
                    }
                    _ => panic!("Y has range 0 - 7, got {:?}", y),
                },
                3 => match y {
                    0 => {
                        let nn = self.next_word();
                        self.jp(nn)
                    }
                    1 => self.handle_cb_prefix(),
                    2..6 => self.nop(),
                    6 => self.di(),
                    7 => self.ei(),
                    _ => self.nop(),
                },
                4 => match y {
                    0..4 => {
                        let nn = self.next_word();
                        self.call_conditional(nn, y)
                    }
                    4..7 => self.nop(),
                    _ => panic!("Y has range 0 - 7, got {:?}", y),
                },
                5 => {
                    if !q {
                        let value = self.get_register_from_table_rp2(p);
                        self.push(value)
                    } else if p == 0 {
                        let nn = self.next_word();
                        self.call(nn)
                    }
                }
                6 => {
                    let n = self.next_byte();
                    match y {
                        0 => self.add(n),
                        1 => self.adc(n),
                        2 => self.sub(n),
                        3 => self.sbc(n),
                        4 => self.and(n),
                        5 => self.xor(n),
                        6 => self.or(n),
                        7 => self.cp(n),
                        _ => panic!(
                            "Should not be able to reach this value, y only has a range of 0-7, got {:?}",
                            y
                        ),
                    }
                }
                7 => {
                    self.rst(y as u16 * 8);
                }
                _ => {
                    panic!("Z has range 0 - 7, got {:?}", z);
                }
            },

            _ => {
                panic!("Invalid x value: {:?}", x);
            }
        }

        self.nop();
    }

    /**
         *
        Table "r"
        8-bit registers
        Index	0	1	2	3	4	5	6	    7
        Value	B	C	D	E	H	L	(HL)	A
    */
    pub fn get_register_from_table_r(&mut self, i: u8) -> &mut u8 {
        match i {
            0 => self.registers.b.get_mut(),
            1 => self.registers.c.get_mut(),
            2 => self.registers.d.get_mut(),
            3 => self.registers.e.get_mut(),
            4 => self.registers.h.get_mut(),
            5 => self.registers.l.get_mut(),
            6 => {
                // (HL), cycles need to go up by 1 as well
                self.increment_cycles(1);
                let pointer = self.bus.get_pointer(self.registers.get_hl());
                return pointer;
            }
            7 => self.registers.a.get_mut(),
            _ => panic!(
                "This should be unreachable since i has a 4 bit range, but got: {:?}",
                i
            ),
        }
    }

    /**
        Table "rp"

        Register pairs featuring SP
        Index	0	1	2	3
        Value	BC	DE	HL	SP
    */
    pub fn get_register_from_table_rp(&self, i: u8) -> u16 {
        match i {
            0 => self.registers.get_bc(),
            1 => self.registers.get_de(),
            2 => self.registers.get_hl(),
            3 => self.registers.sp.get(),
            _ => panic!(
                "This should be unreachable since i has a 4 bit range, but got: {:?}",
                i
            ),
        }
    }

    pub fn set_register_from_table_rp(&self, i: u8, value: u16) {
        match i {
            0 => self.registers.set_bc(value),
            1 => self.registers.set_de(value),
            2 => self.registers.set_hl(value),
            3 => self.registers.sp.set(value),
            _ => panic!(
                "This should be unreachable since i has a 4 bit range, but got: {:?}",
                i
            ),
        }
    }

    pub fn get_register_from_table_rp2(&self, i: u8) -> u16 {
        match i {
            0..3 => self.get_register_from_table_rp(i),
            3 => self.registers.get_af(),
            _ => panic!(
                "This should be unreachable since i has a 4 bit range, but got: {:?}",
                i
            ),
        }
    }

    pub fn set_register_from_table_rp2(&self, i: u8, value: u16) {
        match i {
            0 => self.registers.set_bc(value),
            1 => self.registers.set_de(value),
            2 => self.registers.set_hl(value),
            3 => self.registers.set_af(value),
            _ => panic!(
                "This should be unreachable since i has a 4 bit range, but got: {:?}",
                i
            ),
        }
    }

    fn nop(&mut self) {
        self.increment_cycles(1);
    }

    fn halt(&mut self) {
        unimplemented!();
        self.increment_cycles(1);
    }

    fn stop(&self) {
        unimplemented!();
    }

    fn jr(&self, n: i8) {
        let displacement: i16 = n as i16;
        // get current pc count and add n
        let updated_pc = (self.registers.pc.get() as i16) + displacement;
        self.registers.pc.set(updated_pc as u16);
        self.increment_cycles(3);
    }

    fn check_condition(&self, condition: u8) -> bool {
        match condition {
            0 => {
                // Check if not zero (NZ)
                !self.registers.get_zero_flag()
            }
            1 => {
                // Check if zero (Z)
                self.registers.get_zero_flag()
            }
            2 => {
                // Check if not carry (NC)
                !self.registers.get_carry_flag()
            }
            3 => {
                // Check if carry (C)
                self.registers.get_carry_flag()
            }
            _ => panic!("Condition invalid {:?}", condition),
        }
    }

    fn jr_conditional(&self, d: i8, condition: u8) {
        if self.check_condition(condition) {
            self.jr(d);
        } else {
            self.increment_cycles(2);
        }
    }

    fn jp_cc(&mut self, addr: u16, condition: u8) {
        if self.check_condition(condition) {
            self.jp(addr);
        } else {
            self.increment_cycles(3);
        }
    }

    fn jp(&mut self, addr: u16) {
        self.registers.pc.set(addr);
        self.increment_cycles(4);
    }

    fn call(&mut self, addr: u16) {
        let cycles = self.cycles.get();
        // pushes the next pc onto the stack, then jumps to the addr
        self.push(self.registers.pc.get());
        self.jp(addr);
        // TODO: some wasted instructions
        // since push and jp set cycles, updating here
        self.cycles.set(cycles + 6);
    }

    fn rst(&mut self, vec: u16) {
        // call address vec
        let cycles = self.cycles.get();
        self.call(vec);
        self.cycles.set(cycles + 4);
    }

    fn call_conditional(&mut self, addr: u16, condition: u8) {
        if self.check_condition(condition) {
            self.call(addr);
        } else {
            self.increment_cycles(3);
        }
    }

    fn add_helper(&mut self, value: u8, should_carry: bool) {
        let value: u16 = value.into();
        let a_value = self.registers.a.get();
        let carry_value: u16 = if should_carry {
            self.registers.get_carry_flag() as u16
        } else {
            0
        };

        let result: u16 = value + (a_value as u16) + carry_value;

        self.registers.set_carry_flag(result > 0xFF);
        // Check if there is a carry from bit 3 to bit 4 by masking the lower nibble and summing them.
        self.registers.set_half_carry_flag(
            ((a_value & 0xF) + (value as u8 & 0xF) + (carry_value as u8)) > 0xF,
        );
        self.registers.set_subtraction_flag(false);
        self.registers.set_zero_flag(result & 0xFF == 0);

        // Set value in A register by truncating the value, as does the truncation
        self.registers.a.set(result as u8);
        self.increment_cycles(1);
    }

    fn add(&mut self, value: u8) {
        self.add_helper(value, false);
    }

    fn adc(&mut self, value: u8) {
        self.add_helper(value, true);
    }

    fn add_hl(&mut self, i: u16) {
        let value = self.registers.get_hl();
        let result: u32 = value as u32 + i as u32;

        self.registers.set_carry_flag(result > 0xFFFF);
        self.registers
            .set_half_carry_flag(((value & 0x0FFF) + (i & 0x0FFF)) > 0x0FFF);
        self.registers.set_subtraction_flag(false);
        self.increment_cycles(2);

        self.registers.set_hl(result as u16);
    }

    fn add_sp(&mut self, d: i8) {
        let sp = self.registers.sp.get();
        let result = (sp as i16).wrapping_add(d as i16);
        self.registers.sp.set(result as u16);

        self.registers
            .set_half_carry_flag(((sp & 0xF) + (d as u16 & 0xF)) > 0xF);
        self.registers
            .set_carry_flag(((sp & 0xFF) + (d as u16 & 0xFF)) > 0xFF);
        self.registers.set_zero_flag(false);
        self.registers.set_subtraction_flag(false);
        self.increment_cycles(4);
    }

    fn subtract_helper(&mut self, value: u8, should_carry: bool) {
        let value: u16 = value.into();
        let a_value = self.registers.a.get();
        let carry_value: u16 = if should_carry {
            self.registers.get_carry_flag() as u16
        } else {
            0
        };

        // TODO: I'm not sure if I can just cast it. I think I can.
        let result: i16 = (a_value as i16) - (value as i16) - (carry_value as i16);
        self.registers.set_carry_flag(result < 0);
        self.registers.set_zero_flag((result & 0xFF) == 0);
        self.registers.set_subtraction_flag(true);
        // Check if there is a borrow from bit 4 to bit 3 by masking the
        self.registers.set_half_carry_flag(
            ((a_value & 0xF) as i8 - (value as i8 & 0xF) - (carry_value as i8)) < 0,
        );

        self.registers.a.set(result as u8);
        self.increment_cycles(1);
    }

    fn sub(&mut self, value: u8) {
        self.subtract_helper(value, false);
    }

    fn sbc(&mut self, value: u8) {
        self.subtract_helper(value, true);
    }

    fn and(&mut self, value: u8) {
        // AND A,r8
        // Set A to the bitwise AND between the value in r8 and A.
        let result = self.registers.a.get() & value;
        self.registers.a.set(result);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtraction_flag(false);
        self.registers.set_half_carry_flag(true);
        self.registers.set_carry_flag(false);

        self.increment_cycles(1);
    }

    fn xor(&mut self, value: u8) {
        // XOR A,r8
        // Set A to the bitwise XOR between the value in r8 and A.
        let result = self.registers.a.get() ^ value;
        self.registers.a.set(result);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtraction_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(false);

        self.increment_cycles(1);
    }

    fn or(&mut self, value: u8) {
        // OR A,r8
        // Set A to the bitwise OR between the value in r8 and A.
        let result = self.registers.a.get() | value;

        self.registers.a.set(result);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtraction_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(false);

        self.increment_cycles(1);
    }

    fn cp(&mut self, value: u8) {
        // compare the value in A with the value in r8.
        // This subtracts the value in r8 from A and sets flags accordingly, but discards the result.
        let result = self.registers.a.get().wrapping_sub(value) as i8;

        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtraction_flag(true);
        self.registers.set_half_carry_flag(
            ((self.registers.a.get() & 0xF) as i8) - ((value & 0xF) as i8) < 0,
        );
        self.registers
            .set_carry_flag(self.registers.a.get() < value);

        self.increment_cycles(1);
    }

    fn dec(&mut self, i: u8) {
        let pointer = self.get_register_from_table_r(i);
        let value = pointer.wrapping_sub(1);
        *pointer = value;
        self.increment_cycles(1);

        // set flags
        self.registers.set_zero_flag(value == 0);
        self.registers.set_subtraction_flag(true);
        self.registers.set_half_carry_flag((value & 0x0F) == 0x0F);
    }

    fn inc(&mut self, i: u8) {
        let pointer = self.get_register_from_table_r(i);
        let value = pointer.wrapping_add(1);
        *pointer = value;
        self.increment_cycles(1);

        // set flags
        self.registers.set_zero_flag(value == 0);
        self.registers.set_subtraction_flag(false);
        self.registers.set_half_carry_flag((value & 0x0F) == 0x00);
    }

    fn dec_16(&mut self, i: u8) {
        let value = self.get_register_from_table_rp(i).wrapping_sub(1);
        self.set_register_from_table_rp(i, value);
        self.increment_cycles(2);
    }

    fn inc_16(&mut self, i: u8) {
        let value = self.get_register_from_table_rp(i).wrapping_add(1);
        self.set_register_from_table_rp(i, value);
        self.increment_cycles(2);
    }

    fn increment_cycles(&self, i: usize) {
        self.cycles.set(self.cycles.get() + i);
    }

    // Rotation instructions
    fn rlc(&mut self, register: &mut u8) {
        let value = *register;
        let carry = (value & 0x80) >> 7;
        // Shift left by 1 and set bit 0 to carry
        let result = (value << 1) | carry;
        *register = result;

        self.registers.set_carry_flag(carry == 1);
        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtraction_flag(false);
        self.registers.set_half_carry_flag(false);
    }

    fn rlca(&mut self) {
        let mut a = self.registers.a.get();
        self.rlc(&mut a);
        self.registers.a.set(a);
        self.registers.set_zero_flag(false);
        self.increment_cycles(1);
    }

    fn rrc(&mut self, register: &mut u8) {
        let value = *register;
        // Get the carry bit (bit 0)
        let new_carry = value & 0x01;
        // Shift right by 1 and set bit 7 to carry
        let result: u8 = (value >> 1) | (new_carry << 7);
        *register = result;

        self.registers.set_carry_flag(new_carry == 1);
        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtraction_flag(false);
        self.registers.set_half_carry_flag(false);
    }

    fn rrca(&mut self) {
        let mut a = self.registers.a.get();
        self.rrc(&mut a);
        self.registers.a.set(a);
        self.registers.set_zero_flag(false);
        self.increment_cycles(1);
    }

    fn rl(&mut self, register: &mut u8) {
        let value = *register;

        let carry = self.registers.get_carry_flag() as u8;
        let new_carry = (value & 0x80) >> 7;

        let result = (value << 1) | carry;
        *register = result;

        self.registers.set_carry_flag(new_carry == 1);
        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtraction_flag(false);
        self.registers.set_half_carry_flag(false);
    }

    fn rla(&mut self) {
        let mut a = self.registers.a.get();
        self.rl(&mut a);
        self.registers.a.set(a);
        self.registers.set_zero_flag(false);
        self.increment_cycles(1);
    }

    fn rr(&mut self, register: &mut u8) {
        let value = *register;

        let carry = self.registers.get_carry_flag() as u8;
        let new_carry = value & 0x01;

        let result = (value >> 1) | (carry << 7);
        *register = result;

        self.registers.set_carry_flag(new_carry == 1);
        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtraction_flag(false);
        self.registers.set_half_carry_flag(false);
    }

    fn rra(&mut self) {
        let mut a = self.registers.a.get();
        self.rr(&mut a);
        self.registers.a.set(a);
        self.registers.set_zero_flag(false);
        self.increment_cycles(1);
    }

    fn daa(&self) {
        unimplemented!("DAA is unimplemented");
    }

    fn cpl(&mut self) {
        let a = self.registers.a.get();
        self.registers.a.set(!a);
        self.registers.set_half_carry_flag(true);
        self.registers.set_subtraction_flag(true);
        self.increment_cycles(1);
    }

    fn scf(&mut self) {
        self.registers.set_carry_flag(true);
        self.registers.set_half_carry_flag(false);
        self.registers.set_subtraction_flag(false);
        self.increment_cycles(1);
    }

    fn ccf(&mut self) {
        self.registers
            .set_carry_flag(!self.registers.get_carry_flag());
        self.registers.set_half_carry_flag(false);
        self.registers.set_subtraction_flag(false);
        self.increment_cycles(1);
    }

    fn ret_conditional(&self, condition: u8) {
        if self.check_condition(condition) {
            self.ret();
            self.increment_cycles(1);
        } else {
            self.increment_cycles(2);
        }
    }

    fn ret(&self) {
        // basically pop pc
        let sp = self.registers.sp.get();
        let value = self.bus.read_word(sp);
        self.registers.sp.set(sp + 2);
        self.registers.pc.set(value);
        self.increment_cycles(4);
    }

    fn reti(&mut self) {
        self.ime_flag = true;
        self.ret();
    }

    fn pop(&self, i: u8) {
        // load the value in sp into the register
        let sp = self.registers.sp.get();
        let value = self.bus.read_word(sp);
        self.registers.sp.set(sp + 2);
        if i == 3 {
            // AF register, lower nibble of F is always 0
            self.set_register_from_table_rp2(i, value & 0xFFF0);
        } else {
            self.set_register_from_table_rp2(i, value);
        }

        self.increment_cycles(3);
    }

    fn push(&mut self, value: u16) {
        let mut sp = self.registers.sp.get();
        sp -= 2;
        self.bus.write_word(sp, value);
        self.registers.sp.set(sp);
        self.increment_cycles(4);
    }

    fn load_to_memory(&mut self, addr: u16, value: u8) {
        self.bus.write_byte(addr, value);
        self.increment_cycles(4);
    }

    fn di(&mut self) {
        self.ime_flag = false;
        self.increment_cycles(1);
    }

    fn ei(&mut self) {
        self.ime_flag = true;
        self.increment_cycles(1);
    }

    fn handle_cb_prefix(&mut self) {
        let opcode = self.next_byte();
        let x = (opcode >> 6) & 0x03; // bits 7-6
        let y = (opcode >> 3) & 0x07; // bits 5-3
        let z = opcode & 0x07; // bits 2-0

        let mut register = *self.get_register_from_table_r(z);

        match x {
            0 => {
                match y {
                    0 => {
                        self.rlc(&mut register);
                    }
                    1 => {
                        self.rrc(&mut register);
                    }
                    2 => {
                        self.rl(&mut register);
                    }
                    3 => {
                        self.rr(&mut register);
                    }
                    4 => {
                        self.sla(&mut register);
                    }
                    5 => {
                        self.sra(&mut register);
                    }
                    6 => {
                        self.swap(&mut register);
                    }
                    7 => {
                        self.srl(&mut register);
                    }
                    _ => panic!("Invalid CB prefix y value: {:?}", y),
                };
            }
            1 => {
                // BIT y, r[z]
                // test bit y in r[z], 0 is the rightmost bit, 7 is the leftmost bit
                let bit_mask = 1 << y;
                let is_bit_set = (register & bit_mask) != 0;

                self.registers.set_zero_flag(!is_bit_set);
                self.registers.set_subtraction_flag(false);
                self.registers.set_half_carry_flag(true);
                self.increment_cycles(2);
                // register is unchanged
                return;
            }
            2 => {
                // RES y, r[z]
                // reset bit y in r[z] to 0, 0 is the rightmost bit, 7 is the leftmost bit
                let bit_mask = !(1 << y);
                register &= bit_mask;
            }
            3 => {
                // SET y, r[z]
                // set bit y in r[z] to 1, 0 is the rightmost bit, 7 is the leftmost bit
                let bit_mask = 1 << y;
                register |= bit_mask;
            }
            _ => panic!("Invalid CB prefix x value: {:?}", x),
        }

        // All the above instructions did not increment cycles intentionally
        // will increment (HL) by 2 cycles intentionally, once above and once here
        *self.get_register_from_table_r(z) = register;
        self.increment_cycles(2);
    }

    fn sla(&mut self, register: &mut u8) {
        let value = *register;
        let new_carry = (value & 0x80) >> 7;

        let result = value << 1;
        *register = result;

        self.registers.set_carry_flag(new_carry == 1);
        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtraction_flag(false);
        self.registers.set_half_carry_flag(false);
    }

    fn sra(&mut self, register: &mut u8) {
        let value = *register;
        let new_carry = value & 0x01;

        let msb = value & 0x80; // preserve most significant bit
        let result: u8 = (value >> 1) | msb;
        *register = result;

        self.registers.set_carry_flag(new_carry == 1);
        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtraction_flag(false);
        self.registers.set_half_carry_flag(false);
    }

    fn swap(&mut self, register: &mut u8) {
        // swap the high and the low nibbles of the register
        let value = *register;
        let low = (value & 0xF0) >> 4; // take the high bits and shift them to low
        let high = (value & 0x0F) << 4; // take the low bits and shift them to high
        let result = high | low;
        *register = result;

        self.registers.set_carry_flag(false);
        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtraction_flag(false);
        self.registers.set_half_carry_flag(false);
    }

    fn srl(&mut self, register: &mut u8) {
        let value = *register;
        let new_carry = value & 0x01;

        let result: u8 = value >> 1;
        *register = result;

        self.registers.set_carry_flag(new_carry == 1);
        self.registers.set_zero_flag(result == 0);
        self.registers.set_subtraction_flag(false);
        self.registers.set_half_carry_flag(false);
    }
}

fn load(dest: &mut u8, source: u8) {
    *dest = source;
}
