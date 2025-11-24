use std::cell::Cell;

use crate::cpu::registers::Registers;

pub struct CPU {
    registers: Registers,
    // T-Edge
    //     A single tick of the Game Boy's clock, from low to high, or high to low - 8,388,608 hz
    // T-Cycle (t)
    //     Two T-Edges - 4,194,304 hz
    // M-Cycle (m)
    //     Four T-Cycles - 1,048,576 hz
    cycles: Cell<usize>, // in M-cycles
}

/**
 * For each instruction, we need to emulate the function + addressing mode + cycles
*/
impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: Registers::new(),
            cycles: Cell::new(0),
        }
    }

    // instructions are prefix byte, opcode (byte), displacement byte, intermediate data
    pub fn handle_instruction(&mut self, opcode: u8) {
        // Referenced: http://archive.gbdev.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
        let x: u8 = opcode & 0xC0; // bits 7-6
        let y = opcode & 0x38; // bits 5-3
        let z = opcode & 0x07; // bits 2-0
        let p = opcode & 0x18; // bits 5-4
        let q = y & 1 == 0; // y modulo 2

        // fallback to an "invalid" instruction is NOP
        match x {
            // Relative jumps and assorted ops
            0 => match z {
                0 => match y {
                    0 => self.nop(),
                    // 1 => self.load()
                    2 => self.stop(),
                    3 => self.jr(0x01),

                    // 3 => self.jp(d),
                    4..7 => {
                        // self.jp_conditional(y, d);
                        // self.jr_conditional(y, 0x01);
                    }
                    _ => panic!("Y has range 4 - 7, got {:?}", y),
                },

                1 => {
                    match q {
                        // false => self.load(rp[p], nn);
                        // true => self.add(HL, rp[p])
                        _ => todo!(),
                    }
                }

                2 => {
                    // load instructions
                }

                3 => {
                    if q == true {
                        self.inc_16(p);
                    } else {
                        self.dec_16(p);
                    }
                }

                4 => self.inc(y),
                5 => self.dec(y),

                // // TODO: 0x1 is supposed to be an intermediate
                6 => load(self.get_register_from_table_r(y), 0x1),
                7 => {}

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
                    self.cycles.set(self.cycles.get() + 1);
                }
            }

            // Operate on accumulator and register/memory location
            2 => match y {
                0 => self.add(z),
                1 => self.adc(z),
                2 => self.sub(z),
                3 => self.sbc(z),
                4 => self.and(z),
                5 => self.xor(z),
                6 => self.or(z),
                7 => self.cp(z),
                _ => panic!(
                    "Should not be able to reach this value, y only has a range of 0-7, got {:?}",
                    y
                ),
            },

            3 => {}

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
                unimplemented!("Get value from memory at address HL");
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

    fn nop(&mut self) {
        self.cycles.set(self.cycles.get() + 1);
    }

    fn halt(&mut self) {
        unimplemented!();
        self.cycles.set(self.cycles.get() + 1);
    }

    fn stop(&self) {
        unimplemented!();
    }

    fn jr(&self, n: i16) {
        // get current pc count and add n
        let updated_pc = (self.registers.pc.get() as i16) + n;
        self.registers.pc.set(updated_pc as u16);
        self.cycles.set(self.cycles.get() + 3);
    }

    fn add_helper(&mut self, i: u8, should_carry: bool) {
        // Add the value in r8
        let value: u16 = (*self.get_register_from_table_r(i)).into();
        let a_value = self.registers.a.get();
        let carry_value: u16 = if should_carry {
            self.registers.get_carry_flag() as u16
        } else {
            0
        };

        let result: u16 = value + (a_value as u16) + carry_value;

        self.registers.set_carry_flag(result > 0xFF);
        self.registers.set_zero_flag(result == 0);
        self.registers.set_substraction_flag(false);
        // Check if there is a carry from bit 3 to bit 4 by masking the lower nibble and summing them.
        // Note: Carry_value is not masked since carry is 0-1
        self.registers.set_half_carry_flag(
            ((a_value & 0xF) + (value as u8 & 0xF) + (carry_value as u8)) > 0xF,
        );

        // Set value in A register by truncating the value, as does the truncation
        self.registers.a.set(result as u8);
        self.cycles.set(self.cycles.get() + 1);
    }

    fn add(&mut self, i: u8) {
        self.add_helper(i, false);
    }

    fn adc(&mut self, i: u8) {
        self.add_helper(i, true);
    }

    fn subtract_helper(&mut self, i: u8, should_carry: bool) {
        let value: u16 = (*self.get_register_from_table_r(i)).into();
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
        self.registers.set_substraction_flag(true);
        // Check if there is a borrow from bit 4 to bit 3 by masking the
        self.registers.set_half_carry_flag(
            ((a_value & 0xF) as i8 - (value as i8 & 0xF) - (carry_value as i8)) < 0,
        );

        self.registers.a.set(result as u8);
        self.cycles.set(self.cycles.get() + 1);
    }

    fn sub(&mut self, i: u8) {
        self.subtract_helper(i, false);
    }

    fn sbc(&mut self, i: u8) {
        self.subtract_helper(i, true);
    }

    fn and(&mut self, i: u8) {
        // AND A,r8
        // Set A to the bitwise AND between the value in r8 and A.
        let value = *self.get_register_from_table_r(i);
        let result = self.registers.a.get() & value;
        self.registers.a.set(result);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_substraction_flag(false);
        self.registers.set_half_carry_flag(true);
        self.registers.set_carry_flag(false);

        self.cycles.set(self.cycles.get() + 1);
    }

    fn xor(&mut self, i: u8) {
        // XOR A,r8
        // Set A to the bitwise XOR between the value in r8 and A.
        let value = *self.get_register_from_table_r(i);
        let result = self.registers.a.get() ^ value;
        self.registers.a.set(result);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_substraction_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(false);

        self.cycles.set(self.cycles.get() + 1);
    }

    fn or(&mut self, i: u8) {
        // OR A,r8
        // Set A to the bitwise OR between the value in r8 and A.
        let value = *self.get_register_from_table_r(i);
        let result = self.registers.a.get() | value;

        self.registers.a.set(result);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_substraction_flag(false);
        self.registers.set_half_carry_flag(false);
        self.registers.set_carry_flag(false);

        self.cycles.set(self.cycles.get() + 1);
    }

    fn cp(&mut self, i: u8) {
        // compare the value in A with the value in r8.
        // This subtracts the value in r8 from A and sets flags accordingly, but discards the result.
        let value = *self.get_register_from_table_r(i);

        let result: i8 = (self.registers.a.get() as i8) - (value as i8);

        self.registers.set_zero_flag(result == 0);
        self.registers.set_substraction_flag(true);
        self.registers.set_half_carry_flag(
            ((self.registers.a.get() & 0xF) as i8) - ((value & 0xF) as i8) < 0,
        );
        self.registers.set_carry_flag(result < 0);

        self.cycles.set(self.cycles.get() + 1);
    }

    fn dec(&mut self, i: u8) {
        let pointer = self.get_register_from_table_r(i);
        *pointer = pointer.wrapping_add(1);
        self.cycles.set(self.cycles.get() + 1);
    }

    fn inc(&mut self, i: u8) {
        let pointer = self.get_register_from_table_r(i);
        *pointer = pointer.wrapping_add(1);
        self.cycles.set(self.cycles.get() + 1);
    }

    fn dec_16(&self, i: u8) {
        let value = self.get_register_from_table_rp(i);
        self.set_register_from_table_rp(i, value.wrapping_sub(1));
        self.cycles.set(self.cycles.get() + 2);
    }

    fn inc_16(&self, i: u8) {
        let value = self.get_register_from_table_rp(i);
        self.set_register_from_table_rp(i, value.wrapping_add(1));
        self.cycles.set(self.cycles.get() + 2);
    }
}

fn load(dest: &mut u8, source: u8) {
    *dest = source;
}
