use crate::display::{HEIGHT, WIDTH};
use crate::fonts::{FONT_SET, FONT_START_ADDR};
use std::fs;
const START_ADDRESS: u16 = 0x200;

#[derive(PartialEq)]
enum ScreenMode {
    LoRes,
    HiRes,
}

impl ScreenMode {
    fn scale_factor(&self) -> usize {
        match self {
            ScreenMode::LoRes => 2,
            ScreenMode::HiRes => 1,
        }
    }
}

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
    last_key_pressed: Option<u8>,

    quirks: Quirks,
    screen_mode: ScreenMode,
}

impl Default for Chip8 {
    fn default() -> Self {
        let mut memory = [0; 4096];

        for (i, byte) in FONT_SET.iter().enumerate() {
            memory[(FONT_START_ADDR as usize) + i] = *byte;
        }

        Chip8 {
            memory,
            display: [0x000000; WIDTH * HEIGHT],
            pc: START_ADDRESS,
            index: 0,
            stack: Vec::with_capacity(16),
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; 16],
            keypad: [false; 16],
            last_key_pressed: None,
            quirks: Quirks::default(),
            screen_mode: ScreenMode::LoRes,
        }
    }
}
impl Chip8 {
    pub fn load_rom(&mut self, filename: &str) {
        let file = fs::read(filename).expect("Rom should exist");

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

        // maybe less efficient, but saves so many casts
        let digit1 = ((op & 0xF000) >> 12) as usize;
        let digit2 = ((op & 0x0F00) >> 8) as usize;
        let digit3 = ((op & 0x00F0) >> 4) as usize;
        let digit4 = (op & 0x000F) as usize;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0xE, 0) => self.display = [0x000000; HEIGHT * WIDTH],

            // return from routine
            (0, 0, 0xE, 0xE) => {
                let addr = self.stack.pop().expect("Stack should contain address");
                self.pc = addr;
            }
            (0, 0, 0xF, 0xF) => self.screen_mode = ScreenMode::HiRes,
            (0, 0, 0xF, 0xE) => {
                self.screen_mode = ScreenMode::LoRes;
            }
            (0, 0, 0xC, n) => {
                //scroll down by n pixels
                let mut new_display = [0x0; WIDTH * HEIGHT];
                let movement = n * self.screen_mode.scale_factor();

                for row in movement..HEIGHT {
                    for pixel in 0..WIDTH {
                        let idx = (row * HEIGHT) + pixel;
                        new_display[idx] = self.display[(row - movement * HEIGHT) + pixel]
                    }
                }

                self.display = new_display
            }
            (0, 0, 0xF, 0xB) => {
                let mut new_display = [0x0; WIDTH * HEIGHT];
                let movement = 4 * self.screen_mode.scale_factor();

                for row in 0..HEIGHT {
                    for pixel in movement..WIDTH {
                        let idx = (row * HEIGHT) + pixel;
                        new_display[idx] = self.display[(row * HEIGHT) + pixel - movement]
                    }
                }

                self.display = new_display
            }
            (0, 0, 0xF, 0xC) => {
                let mut new_display = [0x0; WIDTH * HEIGHT];
                let movement = 4 * self.screen_mode.scale_factor();

                for row in 0..HEIGHT {
                    for pixel in 0..(WIDTH - movement) {
                        let idx = (row * HEIGHT) + pixel;
                        new_display[idx] = self.display[(row * HEIGHT) + pixel - movement]
                    }
                }

                self.display = new_display
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
                if self.registers[v_a] == value {
                    self.pc += 2;
                }
            }
            (4, v_a, _, _) => {
                let value = (op & 0xFF) as u8;
                if self.registers[v_a] != value {
                    self.pc += 2;
                }
            }
            (5, x, y, 0) => {
                if self.registers[x] == self.registers[y] {
                    self.pc += 2;
                }
            }
            (9, x, y, 0) => {
                if self.registers[x] != self.registers[y] {
                    self.pc += 2;
                }
            }

            (6, v_a, _, _) => {
                self.registers[v_a] = (op & 0xFF) as u8;
            }
            (7, v_a, _, _) => {
                (self.registers[v_a], _) = self.registers[v_a].overflowing_add((op & 0xFF) as u8);
            }

            // Math Stuff
            (8, x, y, inst) => match inst {
                0 => {
                    self.registers[x] = self.registers[y];
                }
                1 => {
                    self.registers[x] |= self.registers[y];
                }
                2 => {
                    self.registers[x] &= self.registers[y];
                }
                3 => {
                    self.registers[x] ^= self.registers[y];
                }
                4 => {
                    let overflow;
                    (self.registers[x], overflow) =
                        self.registers[x].overflowing_add(self.registers[y]);

                    self.registers[0xFusize] = if overflow { 0x1 } else { 0x0 };
                }
                // subtract
                5 | 7 => {
                    let overflow;
                    if inst == 5 {
                        (self.registers[x], overflow) =
                            self.registers[x].overflowing_sub(self.registers[y]);
                    } else {
                        (self.registers[x], overflow) =
                            self.registers[y].overflowing_sub(self.registers[x]);
                    }

                    self.registers[0xFusize] = if overflow { 0x0 } else { 0x1 };
                }
                // shift
                6 | 0xE => {
                    if self.quirks.shift_vxvy {
                        self.registers[x] = self.registers[y];
                    }

                    let sb;
                    if inst == 6 {
                        sb = self.registers[x] & 0x1;
                        self.registers[x] >>= 1;
                    } else {
                        sb = (self.registers[x] >> 7) & 0x1;
                        self.registers[x] <<= 1;
                    }
                    self.registers[0xF] = sb;
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
                self.registers[x] = rand & value;
            }

            (0xD, x, y, n) => {
                self.registers[0xF] = 0x0;
                let sf = self.screen_mode.scale_factor();

                let start_x = self.registers[x] % (WIDTH / sf) as u8;
                let start_y = self.registers[y] % (HEIGHT / sf) as u8;

                if n == 0 {
                    //TODO: draw 16x16 sprites
                }

                for row in 0..n {
                    if start_y as usize + row >= (HEIGHT / sf) {
                        break;
                    }

                    let sprite_data = self.memory[self.index as usize + row];
                    for bit in 0..8 {
                        if start_x + bit >= (WIDTH / sf) as u8 {
                            break;
                        }

                        let sprite_bit = sprite_data & (0x80 >> bit);

                        let x = (start_x as usize + bit as usize) * sf;
                        let y = (start_y as usize + row) * sf;

                        let idx = (y * WIDTH) + x;
                        let display_bit = self.display[idx];

                        if display_bit == 0xFFFFFF && sprite_bit != 0 {
                            for o_x in 0..sf {
                                for o_y in 0..sf {
                                    self.display[((y + o_y) * WIDTH) + (x + o_x)] = 0x000000;
                                }
                            }
                            self.registers[0xF] = 0x1;
                        } else if sprite_bit != 0x0 && display_bit == 0x0 {
                            for o_x in 0..sf {
                                for o_y in 0..sf {
                                    self.display[((y + o_y) * WIDTH) + (x + o_x)] = 0xFFFFFF;
                                }
                            }
                        }
                    }
                }
            }

            // Key handling
            (0xE, x, 9, 0xE) => {
                let key = self.registers[x];
                if self.keypad[key as usize] {
                    self.pc += 2;
                }
            }
            (0xE, x, 0xA, 0x1) => {
                let key = self.registers[x];
                if !self.keypad[key as usize] {
                    self.pc += 2;
                }
            }
            // timers
            (0xF, x, 0x0, 0x7) => {
                self.registers[x] = self.delay_timer;
            }
            (0xF, x, 0x1, 0x5) => {
                self.delay_timer = self.registers[x];
            }
            (0xF, x, 0x1, 0x8) => {
                self.sound_timer = self.registers[x];
            }

            (0xF, x, 0x1, 0xE) => {
                self.index += self.registers[x] as u16;
            }
            (0xF, x, 0x0, 0xA) => {
                if let Some(key) = self.last_key_pressed
                    && !self.keypad[key as usize]
                {
                    self.last_key_pressed = None;
                    self.registers[x] = key;
                } else {
                    if let Some(i) = self.keypad.iter().position(|&pressed| pressed) {
                        if self.quirks.key_wait_release {
                            self.last_key_pressed = Some(i as u8);
                        } else {
                            self.registers[x] = i as u8;
                            self.pc += 2; // cpu won't tick again so its fine
                        }
                    }

                    self.pc -= 2;
                }
            }
            (0xF, x, 0x2, 0x9) => {
                let char = self.registers[x] as u16;
                self.index = FONT_START_ADDR + (char * 5)
            }

            (0xF, x, 0x3, 0x3) => {
                let value = self.registers[x];
                let i = self.index as usize;

                self.memory[i] = value / 100;
                self.memory[i + 1] = (value / 10) % 10;
                self.memory[i + 2] = value % 10;
            }

            (0xF, x, 0x5, 0x5) => {
                for i in 0..=x {
                    self.memory[self.index as usize + i] = self.registers[i]
                }

                if self.quirks.increment_i_memory {
                    self.index += (x + 1) as u16;
                }
            }

            (0xF, x, 0x6, 0x5) => {
                for i in 0..=x {
                    self.registers[i] = self.memory[self.index as usize + i];
                }

                if self.quirks.increment_i_memory {
                    self.index += (x + 1) as u16;
                }
            }

            (_, _, _, _) => {
                panic!("Unimplemented opcode: {:x}", op)
            }
        }
    }
}

struct Quirks {
    shift_vxvy: bool,
    key_wait_release: bool,
    increment_i_memory: bool,
}

impl Default for Quirks {
    fn default() -> Self {
        Self {
            shift_vxvy: false,
            key_wait_release: false,
            increment_i_memory: true,
        }
    }
}
