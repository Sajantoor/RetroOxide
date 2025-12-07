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
                    1 => {
                        // TODO: Next argument
                        let nn: &mut u16 = &mut 0x01;
                        load_16(nn, self.registers.sp.get());
                    }
                    2 => self.stop(),
                    3 => self.jr(0x01),
                    4..7 => {
                        let d: i8 = 0x01;
                        self.jr_conditional(d, y - 4);
                    }
                    _ => panic!("Y has range 4 - 7, got {:?}", y),
                },

                1 => {
                    match q {
                        false => {
                            // Load instruction:
                            let nn: u16 = 0x01;
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

            3 => match z {
                0 => match y {
                    0..3 => self.ret_conditional(y),
                    // 4 => LD (0xFF00 + n), A
                    // 5 => ADD SP, d
                    // 6 => LD A, (0xFF00 + n)
                    // LD HL, SP + d
                    _ => panic!("Y has range 0 - 7, got {:?}", y),
                },
                1 => {
                    if !q {
                        // self.pop(rp2[p])
                    } else {
                        match p {
                            0 => self.ret(),
                            1 => self.reti(),
                            // 2 => self.jp(hl),
                            // 3 => LD SP, HL
                            _ => panic!("P has range 0 - 4 got {:?}", p),
                        }
                    }
                }

                2 => match y {
                    // 0..3 => self.jp_c(nn),
                    // 4 => LD (oxFF00 + C), A
                    // 5 => LD (nn), A
                    // 6 => LD A, (0xFF00 + C)
                    // 7 => LD A, (nn)
                    _ => panic!("Y has range 0 - 7, got {:?}", y),
                },
                3 => match y {
                    // 0 => self.jp(nn),
                    // 1 => CB prefix
                    // 6 => self.di()
                    // 7 => self.ei()
                    _ => self.nop(),
                },
                4 => match y {
                    // 0..3 => self.call_conditional(y, nn),
                    // 4..7 => self.nop(),
                    _ => panic!("Y has range 0 - 7, got {:?}", y),
                },
                5 => {
                    if !q {
                        // /self.push(rp2[p])
                    } else if p == 0 {
                        // self.call(nn)
                    }
                }

                6 => match y {
                    // TODO: Need to be replaced with N
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
                7 => {
                    // self.rst(y * 8);
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
        self.increment_cycles(1);
    }

    fn halt(&mut self) {
        unimplemented!();
        self.increment_cycles(1);
    }

    fn stop(&self) {
        unimplemented!();
    }

    fn jr(&self, n: i16) {
        // get current pc count and add n
        let updated_pc = (self.registers.pc.get() as i16) + n;
        self.registers.pc.set(updated_pc as u16);
        self.increment_cycles(3);
    }

    fn jr_conditional(&self, d: i8, condition: u8) {
        // Although relative jump instructions are traditionally shown with a 16-bit address for an operand, here they will take the form JR/DJNZ d, where d is the signed 8-bit displacement that follows (as this is how they are actually stored). The jump's final address is obtained by adding the displacement to the instruction's address plus 2.
        let displacement: i16 = d as i16 + 2;

        match condition {
            0 => {
                // Check if not zero (NZ)
                if !self.registers.get_zero_flag() {
                    self.jr(displacement);
                } else {
                    self.increment_cycles(2);
                }
            }
            1 => {
                // Check if zero (Z)
                if self.registers.get_zero_flag() {
                    self.jr(displacement);
                } else {
                    self.increment_cycles(2);
                }
            }
            2 => {
                // Check if not carry (NC)
                if !self.registers.get_carry_flag() {
                    self.jr(displacement);
                } else {
                    self.increment_cycles(2);
                }
            }
            3 => {
                // Check if carry (C)
                if self.registers.get_carry_flag() {
                    self.jr(displacement);
                } else {
                    self.increment_cycles(2);
                }
            }
            _ => panic!("JR invalid condition {:?}", condition),
        }
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
        self.registers.set_half_carry_flag(
            ((a_value & 0xF) + (value as u8 & 0xF) + (carry_value as u8)) > 0xF,
        );

        // Set value in A register by truncating the value, as does the truncation
        self.registers.a.set(result as u8);
        self.increment_cycles(1);
    }

    fn add(&mut self, i: u8) {
        self.add_helper(i, false);
    }

    fn adc(&mut self, i: u8) {
        self.add_helper(i, true);
    }

    fn add_hl(&mut self, i: u16) {
        unimplemented!();
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
        self.increment_cycles(1);
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

        self.increment_cycles(1);
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

        self.increment_cycles(1);
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

        self.increment_cycles(1);
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

        self.increment_cycles(1);
    }

    fn dec(&mut self, i: u8) {
        let pointer = self.get_register_from_table_r(i);
        *pointer = pointer.wrapping_add(1);
        self.increment_cycles(1);
    }

    fn inc(&mut self, i: u8) {
        let pointer = self.get_register_from_table_r(i);
        *pointer = pointer.wrapping_add(1);
        self.increment_cycles(1);
    }

    fn dec_16(&self, i: u8) {
        let value = self.get_register_from_table_rp(i);
        self.set_register_from_table_rp(i, value.wrapping_sub(1));
        self.increment_cycles(2);
    }

    fn inc_16(&self, i: u8) {
        let value = self.get_register_from_table_rp(i);
        self.set_register_from_table_rp(i, value.wrapping_add(1));
        self.increment_cycles(2);
    }

    fn increment_cycles(&self, i: usize) {
        self.cycles.set(self.cycles.get() + i);
    }

    // Rotation instructions
    fn rlca(&mut self) {
        let a = self.registers.a.get();
        // Get the carry bit (bit 7)
        let carry = (a & 0x80) >> 7;
        // Shift left by 1 and set bit 0 to carry
        let result = (a << 1) | carry;
        self.registers.a.set(result);

        self.registers.set_carry_flag(carry == 1);
        self.registers.set_zero_flag(false);
        self.registers.set_substraction_flag(false);
        self.registers.set_half_carry_flag(false);
        self.increment_cycles(1);
    }

    fn rrca(&mut self) {
        let a = self.registers.a.get();
        // Get the carry bit (bit 0)
        let carry = a & 0x01;
        // Shift right by 1 and set bit 7 to carry
        let result = (a >> 1) | (carry << 7);
        self.registers.a.set(result);

        self.registers.set_carry_flag(carry == 1);
        self.registers.set_zero_flag(false);
        self.registers.set_substraction_flag(false);
        self.registers.set_half_carry_flag(false);
        self.increment_cycles(1);
    }

    fn rla(&mut self) {
        let a = self.registers.a.get();

        let carry = self.registers.get_carry_flag() as u8;
        let new_carry = (a & 0x80) >> 7;

        let result = (a << 1) | carry;
        self.registers.a.set(result);

        self.registers.set_carry_flag(new_carry == 1);
        self.registers.set_zero_flag(false);
        self.registers.set_substraction_flag(false);
        self.registers.set_half_carry_flag(false);
        self.increment_cycles(1);
    }

    fn rra(&mut self) {
        let a = self.registers.a.get();

        let carry = self.registers.get_carry_flag() as u8;
        let new_carry = (a & 0x80) >> 7;

        let result = (a >> 1) | (carry << 7);
        self.registers.a.set(result);

        self.registers.set_carry_flag(new_carry == 1);
        self.registers.set_zero_flag(false);
        self.registers.set_substraction_flag(false);
        self.registers.set_half_carry_flag(false);
        self.increment_cycles(1);
    }

    fn daa(&self) {
        unimplemented!("DAA is unimplemented");
    }

    fn cpl(&mut self) {
        let a = self.registers.a.get();
        self.registers.a.set(a.wrapping_neg());
        self.registers.set_half_carry_flag(true);
        self.registers.set_substraction_flag(true);
        self.increment_cycles(1);
    }

    fn scf(&mut self) {
        self.registers.set_carry_flag(true);
        self.registers.set_half_carry_flag(false);
        self.registers.set_substraction_flag(false);
        self.increment_cycles(1);
    }

    fn ccf(&mut self) {
        self.registers
            .set_carry_flag(!self.registers.get_carry_flag());
        self.registers.set_half_carry_flag(false);
        self.registers.set_substraction_flag(false);
        self.increment_cycles(1);
    }

    fn ret_conditional(&self, condition: u8) {
        unimplemented!();
    }

    fn ret(&self) {
        // basically pop pc
        unimplemented!();
    }

    fn reti(&self) {
        // equivalent to executing EI then RET
        unimplemented!();
    }
}

fn load(dest: &mut u8, source: u8) {
    *dest = source;
}

fn load_16(dest: &mut u16, source: u16) {
    *dest = source;
}
