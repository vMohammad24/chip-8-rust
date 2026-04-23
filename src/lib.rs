use crate::fonts::{FONT_SET, FONT_START_ADDR};
use std::fs;

pub mod display;
mod fonts;

const START_ADDRESS: u16 = 0x200;

#[derive(Debug)]
pub struct Chip8 {
    memory: [u8; 4096],
    pub display: [u32; display::WIDTH * display::HEIGHT],
    pc: u16,
    index: u16,
    stack: Vec<u16>,
    pub delay_timer: u8,
    sound_timer: u8,

    registers: [u8; 16],
}

impl Default for Chip8 {
    fn default() -> Self {
        let mut memory = [0; 4096];

        for (i, byte) in FONT_SET.iter().enumerate() {
            memory[(FONT_START_ADDR as usize) + i] = *byte;
        }

        Chip8 {
            memory,
            display: [0x000000; 64 * 32],
            pc: START_ADDRESS,
            index: 0,
            stack: vec![],
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; 16],
        }
    }
}
impl Chip8 {
    pub fn load_rom(&mut self, filename: &str) {
        let file = fs::read(filename).expect("File to load should exist");

        for (i, byte) in file.iter().enumerate() {
            self.memory[(START_ADDRESS as usize) + i] = *byte;
        }
    }

    pub fn decrease_timers(&mut self) {
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            //TODO: beep?
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    pub fn tick(&mut self) {
        let op = (self.memory[self.pc as usize] as u16 * 256)
            + (self.memory[self.pc as usize + 1] as u16);
        self.pc += 2;

        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0xE, 0) => self.display = [0x000000; 64 * 32],

            // return from routine
            (0, 0, 0xE, 0xE) => {
                let addr = self.stack.pop().expect("Stack should contain address");
                self.pc = addr;
            }
            // Jump
            (1, _, _, _) => {
                let addr = op & 0xFFF;
                self.pc = addr;
            }
            // Subroutines
            (2, _, _, _) => {
                let addr = op & 0xFFF;
                self.stack.push(self.pc);

                self.pc = addr;
            }
            // Skip Conditionally
            (3, v_a, _, _) => {
                let value = (op & 0xFF) as u8;
                if self.registers[v_a as usize] == value {
                    self.pc += 2;
                }
            }
            (4, v_a, _, _) => {
                let value = (op & 0xFF) as u8;
                if self.registers[v_a as usize] != value {
                    self.pc += 2;
                }
            }
            (5, x, y, 0) => {
                if self.registers[x as usize] == self.registers[y as usize] {
                    self.pc += 2;
                }
            }
            (6, v_a, _, _) => {
                self.registers[v_a as usize] = (op & 0xFF) as u8;
            }
            (7, v_a, _, _) => {
                (self.registers[v_a as usize], _) =
                    self.registers[v_a as usize].overflowing_add((op & 0xFF) as u8);
            }
            (0xA, _, _, _) => {
                let val = op & 0xFFF;
                self.index = val;
            }
            (0xD, x, y, n) => {
                let x = self.registers[x as usize] % display::WIDTH as u8;
                let y = self.registers[y as usize] % display::HEIGHT as u8;

                for row in 0..n {
                    let sprite_data = self.memory[(self.index + row) as usize];

                    for i in 0..8 {
                        let sprite_bit = sprite_data & (0x80 >> i);
                        let idx = ((y as usize + row as usize) * display::WIDTH)
                            + (x as usize + i as usize);
                        let display_bit = self.display[idx];

                        if display_bit == 0xFFFFFF && sprite_bit != 0 {
                            self.display[idx] = 0x000000;

                            self.registers[0xF] = 0x1;
                        } else if sprite_bit != 0x0 && display_bit == 0x0 {
                            self.display[idx] = 0xFFFFFF;
                        }
                    }
                }
            }
            (_, _, _, _) => panic!("Unimplemented opcode: {:x}", op),
        }
    }
}
