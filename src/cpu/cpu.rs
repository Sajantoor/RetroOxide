use crate::cpu::registers::{self, Registers};

pub struct CPU {
    registers: Registers,
    // T-Edge
    //     A single tick of the Game Boy's clock, from low to high, or high to low - 8,388,608 hz
    // T-Cycle (t)
    //     Two T-Edges - 4,194,304 hz
    // M-Cycle (m)
    //     Four T-Cycles - 1,048,576 hz
    cycles: usize, // in M-cycles
}

/**
 * For each instruction, we need to emulate the function + addressing mode + cycles
*/
impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: Registers::new(),
            cycles: 0,
        }
    }

    // instructions are prefix byte, opcode (byte), displacement byte, intermediate data
    pub fn handle_instruction(mut self, opcode: u8) {
        // Referenced: http://archive.gbdev.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
        let x: u8 = opcode & 0xC0; // bits 7-6
        let y = opcode & 0x38; // bits 5-3
        let z = opcode & 0x07; // bits 2-0
        let p = opcode & 0x18; // bits 5-4
        let q = y & 1; // y modulo 2

        // fallback to an "invalid" instruction is NOP
        match x {
            //
            0 => {}

            // Load and halt instructions
            1 => {
                if y == 6 && z == 6 {
                    return self.halt();
                } else {
                    return self.load(y, z);
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

    fn nop(&mut self) {
        self.cycles += 1;
    }

    fn halt(&mut self) {
        unimplemented!();
        self.cycles += 1;
    }

    fn load(&mut self, y: u8, z: u8) {
        // Instruction: LD r[y], r[z]
        // copy the value in the register on the right, into the register in the left
        let value = self.registers.get_register_from_table_r(z);
        self.registers.set_register_from_table_r(y, value);
        self.cycles += 1;
    }

    fn add(&mut self, i: u8) {
        // Add the value in r8
        let value_in_register: u16 = self.registers.get_register_from_table_r(i).into();
        let a_value = self.registers.a;

        let result: u16 = value_in_register + (a_value as u16);

        self.registers.set_carry_flag(result > 0xFF);
        self.registers.set_zero_flag(result == 0);
        self.registers.set_substraction_flag(false);
        // Check if there is a carry from bit 3 to bit 4 by masking the lower nibble and summing them.
        self.registers
            .set_half_carry_flag(((a_value & 0xF) + (value_in_register as u8 & 0xF)) > 0xF);

        self.cycles += 1;
    }
}
