use crate::display::{HEIGHT, WIDTH};
use crate::fonts::{FONT_SET, FONT_START_ADDR};
use std::fs;
const START_ADDRESS: u16 = 0x200;

#[derive(Debug)]
pub struct Chip8 {
    memory: [u8; 4096],
    pub display: [u32; WIDTH * HEIGHT],
    pc: u16,
    index: u16,
    stack: Vec<u16>,

    pub delay_timer: u8,
    sound_timer: u8,

    registers: [u8; 16],
    pub keypad: [bool; 16],
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
            keypad: [false; 16],
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
            println!("!!BEEP!!")
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
            (9, x, y, 0) => {
                if self.registers[x as usize] != self.registers[y as usize] {
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

            // Math Stuff
            (8, x, y, inst) => match inst {
                0 => {
                    self.registers[x as usize] = self.registers[y as usize];
                }
                1 => {
                    self.registers[x as usize] |= self.registers[y as usize];
                }
                2 => {
                    self.registers[x as usize] &= self.registers[y as usize];
                }
                3 => {
                    self.registers[x as usize] ^= self.registers[y as usize];
                }
                4 => {
                    let overflow;
                    (self.registers[x as usize], overflow) =
                        self.registers[x as usize].overflowing_add(self.registers[y as usize]);

                    self.registers[0xFusize] = if overflow { 0x1 } else { 0x0 };
                }
                // subtract
                5 | 7 => {
                    let overflow;
                    if inst == 5 {
                        (self.registers[x as usize], overflow) =
                            self.registers[x as usize].overflowing_sub(self.registers[y as usize]);
                    } else {
                        (self.registers[x as usize], overflow) =
                            self.registers[y as usize].overflowing_sub(self.registers[x as usize]);
                    }

                    self.registers[0xFusize] = if overflow { 0x0 } else { 0x1 };
                }
                // shift
                6 | 0xE => {
                    // self.registers[x as usize] = self.registers[y as usize]; // TODO: configurable if this happens

                    if inst == 6 {
                        self.registers[0xF] = self.registers[x as usize] & 0x01;
                        self.registers[x as usize] >>= 1;
                    } else {
                        self.registers[0xF] = (self.registers[x as usize] >> 7) & 0x01;
                        self.registers[x as usize] <<= 1;
                    }
                }
                _ => {}
            },
            (0xA, _, _, _) => {
                let val = op & 0xFFF;
                self.index = val;
            }
            (0xB, _, _, _) => {
                let addr = op & 0xFFF;
                self.pc = addr + *self.registers.first().unwrap() as u16
            }
            (0xC, x, _, _) => {
                let value = (op & 0xFF) as u8;

                let rand: u8 = rand::random();
                self.registers[x as usize] = rand & value;
            }

            (0xD, x, y, n) => {
                self.registers[0xF] = 0x0;
                let x = self.registers[x as usize] % WIDTH as u8;
                let y = self.registers[y as usize] % HEIGHT as u8;
                for row in 0..n {
                    if y as u16 + row >= HEIGHT as u16 {
                        break
                    }
                    let sprite_data = self.memory[(self.index + row) as usize];

                    for i in 0..8 {
                        if x + i >= WIDTH as u8 {
                            break
                        }

                        let sprite_bit = sprite_data & (0x80 >> i);
                        let idx = ((y as usize + row as usize) * WIDTH) + (x as usize + i as usize);
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

            // Key handling
            (0xE, x, 9, 0xE) => {
                let key = self.registers[x as usize];
                if self.keypad[key as usize] {
                    self.pc += 2;
                }
            }
            (0xE, x, 0xA, 0x1) => {
                let key = self.registers[x as usize];
                if !self.keypad[key as usize] {
                    self.pc += 2;
                }
            }
            // timers
            (0xF, x, 0x0, 0x7) => {
                self.registers[x as usize] = self.delay_timer;
            }
            (0xF, x, 0x1, 0x5 | 0x8) => {
                let value = self.registers[x as usize];

                if (op & 0xF) == 0x5 {
                    self.delay_timer = value;
                } else {
                    self.sound_timer = value;
                }
            }

            (0xF, x, 0x1, 0xE) => {
                self.index += self.registers[x as usize] as u16;
            }
            (0xF, x, 0x0, 0xA) => {
                if let Some(i) = self.keypad.iter().position(|&pressed| pressed) {
                    self.registers[x as usize] = i as u8;
                } else {
                    self.pc -= 2;
                }
            }
            (0xF, x, 0x2, 0x9) => {
                let char = self.registers[x as usize] as u16;
                self.index = FONT_START_ADDR + (char * 5)
            },

            (0xF, x, 0x3, 0x3) => {
                let value = self.registers[x as usize];
                let i = self.index as usize;

                self.memory[i] = value / 100;
                self.memory[i + 1] = (value / 10) % 10;
                self.memory[i + 2] = value % 10;
            }

            (0xF, x, 0x5, 0x5) => {
                //TODO: toggle to increment i, for quirk behavior
                for i in 0..=x {
                    self.memory[(self.index + i) as usize] = self.registers[i as usize]
                }
            }

            (0xF, x, 0x6, 0x5) => {
                //TODO: toggle to increment i, for quirk behavior
                for i in 0..=x {
                    self.registers[i as usize] = self.memory[(self.index + i) as usize];
                }
            }

            (_, _, _, _) => {
                panic!("Unimplemented opcode: {:x}", op)
            }
        }
    }
}
