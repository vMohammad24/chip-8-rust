use std::fs;
use crate::fonts::{FONT_SET, FONT_START_ADDR};

mod fonts;
pub mod display;

const START_ADDRESS: u16 = 0x200;

#[derive(Debug)]
pub struct Chip8 {
    memory: [u8; 4096],
    pub display: [u32; display::WIDTH * display::HEIGHT],
    pc: u16,
    index: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,

    registers: [u8; 16],
}

impl Chip8 {
    pub fn new() -> Chip8 {
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
}

enum OpCode {

}