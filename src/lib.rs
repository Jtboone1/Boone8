mod utils;

use wasm_bindgen::prelude::*;
use js_sys;
use std::fs;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const START_ADDR: u16 = 0x200;
const FONT_SIZE: usize = 80;
const FONT_SET_STARTING_ADDR: usize = 0x50;

const FONT_SET: [u8;FONT_SIZE] = [
	0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
	0x20, 0x60, 0x20, 0x20, 0x70, // 1
	0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
	0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
	0x90, 0x90, 0xF0, 0x10, 0x10, // 4
	0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
	0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
	0xF0, 0x10, 0x20, 0x40, 0x40, // 7
	0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
	0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
	0xF0, 0x90, 0xF0, 0x90, 0x90, // A
	0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
	0xF0, 0x80, 0x80, 0x80, 0xF0, // C
	0xE0, 0x90, 0x90, 0x90, 0xE0, // D
	0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
	0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

#[wasm_bindgen]
pub struct CHIP8 {
    registers: [u8;16],
    memory: [u8;4096],
    index: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [bool;16],
    video: [u8;64 * 32],
    opcode: u16
}

#[wasm_bindgen]
impl CHIP8 {
    pub fn new() -> CHIP8 {

        let mut memory_init = [0;4096];
        for i in 0..FONT_SIZE {
            memory_init[FONT_SET_STARTING_ADDR + i] = FONT_SET[i];
        }

        CHIP8 {
            registers: [0;16],
            memory: memory_init,
            index: 0,
            pc: START_ADDR,
            stack: [0;16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false;16],
            video: [0;64*32],
            opcode: 0
        }
    } 

    pub fn load_rom(&mut self, filename: &str) {
        let contents = fs::read(filename).expect("Could not open file!");
        let filesize = fs::metadata(filename).expect("Could not read file metadata!").len();

        for i in 0..filesize as usize {
            self.memory[START_ADDR as usize + i] = contents[i];
        }
    }

    pub fn tick(&mut self) {
        self.opcode = (self.memory[self.pc as usize] as u16) << 8 | self.memory[(self.pc + 1) as usize] as u16;

        self.pc += 2;
        self.execute_opcode();

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn get_video(&self) -> *const u8 {
        self.video.as_ptr()
    }

    pub fn set_key_down(&mut self, key: usize) {
        self.keypad[key] = true;
    }

    pub fn set_key_up(mut self, key: usize) {
        self.keypad[key] = false;
    }
}

impl CHIP8 {
    fn generate_rand(&self) -> u8 {
        (js_sys::Math::random() * 255.0) as u8
    }

    fn execute_opcode(&mut self) {
        
        // Opcode nibbles for matching
        let op1 = (self.opcode & 0xF000) >> 12;
        let op2 = (self.opcode & 0x0F00) >> 8;
        let op3 = (self.opcode & 0x00F0) >> 4;
        let op4 = self.opcode & 0x000F;

        // Common parts of opcode
        let nnn = self.opcode & 0x0FFF;
        let kk = (self.opcode & 0x00F) as u8;
        let x = op2 as usize;
        let y = op3 as usize;
        let vx = self.registers[x];
        let vy = self.registers[y];

        match (op1, op2, op3, op4) {
            // 00E0: CLS
            (0x0, 0x0, 0xE, 0x0) => self.video = [0;32 * 64],
            // 00EE: RET
            (0x0, 0x0, 0xE, 0xE) => {
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
            },
            // 1nnn: JP addr
            (0x1, _, _, _) => self.pc = nnn,
            // 2nnn: CALL nnn
            (0x2, _, _, _) => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            },
            // 3xkk: SE Vx, kk
            (0x3, _, _, _) => if vx == kk { self.pc += 2 },
            // 4xkk: SNE Vx, kk
            (0x4, _, _, _) => if vx != kk { self.pc += 2 },
            // 5xy0: SE Vx, Vy
            (0x5, _, _, _) => if vx == vy { self.pc += 2 },
            // 6xkk: LD Vx, kk
            (0x6, _, _, _) => self.registers[x] = kk,
            // 7xkk: ADD Vx, kk
            (0x7, _, _, _) => self.registers[x] += kk,
            // 8xy0: LD Vx, Vy
            (0x8, _, _, 0x0) => self.registers[x] = vy,
            // 8xy1:  OR Vx, Vy
            (0x8, _, _, 0x1) => self.registers[x] |= vy,
            // 8xy2:  AND Vx, Vy
            (0x8, _, _, 0x2) => self.registers[x] &= vy,
            // 8xy3:  XOR Vx, Vy
            (0x8, _, _, 0x3) => self.registers[x] ^= vy,
            // 8xy4: ADD Vx, Vy
            (0x8, _, _, 0x4) => {
                let (n, overflow) = vx.overflowing_add(vy);

                if overflow { self.registers[0xF] = 1 }
                else { self.registers[0xF] = 0 }

                self.registers[x] = n;
            },
            // 8xy5: SUB Vx, Vy
            (8, _, _, 0x5) => {
                let (n, overflow) = vx.overflowing_sub(vy);

                if overflow { self.registers[0xF] = 0 }
                else { self.registers[0xF] = 1 }

                self.registers[x] = n;
            },
            // 8xy6: SHR Vx
            (0x8, _, _, 0x6) => {
                self.registers[0xF] = vx & 0x1;
                self.registers[x] >>= 1;
            },
            // 8xy7: SUBN Vx, Vy
            (0x8, _, _, 0x7) => {
                let (n, overflow) = vy.overflowing_sub(vx);

                if overflow { self.registers[0xF] = 0 }
                else { self.registers[0xF] = 1 }

                self.registers[x] = n;
            }
            // 8xyE: SHL Vx {, Vy}
            (0x8, _, _, 0xE) => {
                self.registers[0xF] = (vx & 0x80) >> 7;
                self.registers[x] <<= 1;
            }
            // 9xy0: SNE Vx, Vy
            (0x9, _, _, _) => if vx == vy { self.pc += 2 }
            // Annn: LD I, nnn
            (0xA, _, _, _) => self.index = nnn,
            // Bnnn: JP V0, nnn
            (0xB, _, _, _) => self.pc = self.registers[0x0] as u16 + nnn,
            // Cxkk: RND Vx, kk
            (0xC, _, _, _) => self.registers[x] = self.generate_rand() & kk,
            // Dxyn: DRW Vx, Vy, nibble
            (0xD, _, _, _) => {
                
            },
            // Ex9E: SKP Vx
            (0xE, _, 0x9, _) => if self.keypad[vx as usize] { self.pc += 2 }
            // ExA1: SKNP Vx
            (0xE, _, 0xA, _) => if !self.keypad[vx as usize] { self.pc += 2 }
            // Fx07: LD Vx, DT
            (0xF, _, _, 0x7) => self.registers[x] = self.delay_timer,
            // Fx0A: LD Vx, K
            (0xF, _, _, 0xA) => {
                let mut key_pressed = false;
                for i in 0..16 {
                    if self.keypad[i] { 
                        self.registers[x] = i as u8;
                        key_pressed = true;
                        break;
                    }
                }
                if !key_pressed { self.pc -= 2 }
            },
            // Fx15: LD DT, Vx
            (0xF, _, 0x1, 0x5) => self.delay_timer = vx,
            // Fx18: LD ST, Vx
            (0xF, _, _, 0x8) => self.sound_timer = vx,
            // Fx1E: ADD I, Vx
            (0xF, _, _, 0xE) => self.index += vx as u16,
            // Fx29: LD F, Vx
            (0xF, _, _, 0x9) => self.index = FONT_SET_STARTING_ADDR as u16 + (vx as u16 * 5),
            // Fx33: LD B, Vx
            (0xF, _, _, 0x3) => {
                self.memory[self.index as usize + 2] = vx % 10;
                self.memory[self.index as usize + 1] = (vx / 10) % 10;
                self.memory[self.index as usize] = (vx % 100) % 10;
            },
            // Fx55: LD [I], Vx
            (0xF, _, 0x5, _) => {
                for i in 0..x {
                    self.memory[self.index as usize + i] = self.registers[i]; 
                }
            }
            // Fx65: LD Vx, [I]
            (0xF, _, 0x6, _) => {
                for i in 0..x {
                    self.registers[i] = self.memory[self.index as usize + i];
                }
            }
            // Otherwise
            (_, _, _, _) => ()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rom_load() {
        let mut chip = CHIP8::new();
        chip.load_rom("chip8.ch8");

        let contents = fs::read("chip8.ch8").unwrap();
        let filesize = fs::metadata("chip8.ch8").expect("Could not read file metadata!").len();

        let start_index = START_ADDR as usize;
        let filesize_index = filesize as usize;
        assert_eq!(&chip.memory[start_index..(start_index + filesize_index)], &contents[0..filesize_index]);
    }
}