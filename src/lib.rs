/* 
 * An emulator for the Chip8 interpreted programming language. Written in Rust and 
 * compiled to WebAssembly!
 * 
 * Hosted here:
 *
 * TODO: Add link to a playable part of the website!
 * 
 * To get it running on your machine, you'd need to load the ROM into the Chip8's
 * memory through the get_memory() ptr. Then you'd need to display the 32 * 64 video
 * memory through the get_video() ptr. Afterwards, all that needs to be done is make
 * calls to the tick() method, and your off to the races!
 */

mod utils;

use wasm_bindgen::prelude::*;
use fixedbitset::FixedBitSet;
use js_sys;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const START_ADDR: u16 = 0x200;
const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const VIDEO_SIZE: usize = WIDTH * HEIGHT;
const MEMORY_SIZE: usize = 4096;

const FONT_SIZE: usize = 80;
const FONT_START_ADDR: usize = 0x50;

// Chip8 needs to find these in memory
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

// Reset memory function to get a memory with the loaded font set.
fn reset_memory() -> [u8;MEMORY_SIZE] {

    let mut memory_init = [0;MEMORY_SIZE];
    for i in 0..FONT_SIZE {
        memory_init[FONT_START_ADDR + i] = FONT_SET[i];
    }

    memory_init
}

#[wasm_bindgen]
pub struct CHIP8 {
    registers: [u8;16],
    memory: [u8;MEMORY_SIZE],
    index: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: FixedBitSet,
    video: [u8;VIDEO_SIZE],
    opcode: u16
}

// Our public API for JavaScript :D
#[wasm_bindgen]
impl CHIP8 {
    pub fn new() -> CHIP8 {
        utils::set_panic_hook();

        let chip8 = CHIP8 {
            registers: [0;16],
            memory: reset_memory(),
            index: 0,
            pc: START_ADDR,
            stack: [0;16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: FixedBitSet::with_capacity(16),
            video: [0;VIDEO_SIZE],
            opcode: 0
        };

        chip8
    } 

    // How we execute each cycle of the CPU
    pub fn tick(&mut self) {
        self.opcode = (self.memory[self.pc as usize] as u16) << 8 | self.memory[(self.pc + 1) as usize] as u16;

        // The Chip8 interprets the memory in two bytes,
        // so to perform the next action, we must increment
        // by two.
        self.pc += 2;
        self.execute_opcode();

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn reset(&mut self) {
        self.registers = [0;16];
        self.memory = reset_memory();
        self.index = 0;
        self.pc = START_ADDR;
        self.stack = [0;16];
        self.sp = 0;
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.keypad = FixedBitSet::with_capacity(16);
        self.video = [0;VIDEO_SIZE];
        self.opcode = 0
    }

    pub fn get_video(&self) -> *const u8 {
        self.video.as_ptr()
    }

    pub fn get_memory(&self) -> *const u8 {
        self.memory.as_ptr()
    }

    pub fn get_index(&self) -> u16 {
        self.index
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn get_registers(&self) -> *const u8 {
        self.registers.as_ptr()
    }

    pub fn get_sound_timer(&self) -> u8 {
        self.sound_timer
    }

    pub fn get_delay_timer(&self) -> u8 {
        self.delay_timer
    }

    pub fn get_opcode(&self) -> u16 {
        self.opcode
    }

    pub fn get_stack_ptr(&self) -> *const u16 {
        self.stack.as_ptr()
    }

    pub fn get_stack_index(&self) -> u8 {
        self.sp
    }

    pub fn set_key_down(&mut self, key: usize) {
        self.keypad.set(key, true)
    }

    pub fn set_key_up(&mut self, key: usize) {
        self.keypad.set(key, false)
    }
}

impl CHIP8 {

    // Rand crate doesn't have support for our wasm
    // target, so we just use the JavaScript's Math::Random
    // instead for the random instruction.
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
        let kk = (self.opcode & 0x00FF) as u8;
        let x = op2 as usize;
        let y = op3 as usize;
        let vx = self.registers[x];
        let vy = self.registers[y];

        // Logic for interpretting each opcode
        match (op1, op2, op3, op4) {
            // 00E0: CLS
            (0x0, 0x0, 0xE, 0x0) => self.video = [0;VIDEO_SIZE],
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
            (0x9, _, _, _) => if vx != vy { self.pc += 2 }
            // Annn: LD I, nnn
            (0xA, _, _, _) => self.index = nnn,
            // Bnnn: JP V0, nnn
            (0xB, _, _, _) => self.pc = self.registers[0x0] as u16 + nnn,
            // Cxkk: RND Vx, kk
            (0xC, _, _, _) => self.registers[x] = self.generate_rand() & kk,
            // Dxyn: DRW Vx, Vy, nibble
            (0xD, _, _, _) => {
                let n = self.opcode & 0x000F;
                let start = self.index as usize;
                let end = (self.index + n) as usize;

                for (row, &pixels) in self.memory[start..end].iter().enumerate() {
                    for col in 0..8 {
                        // Get pixel by shifting out the MSB and shifting it for each col
                        if pixels & 0x80 >> col > 0 {
                            let col = (vx as usize + col) % WIDTH;
                            let row = (vy as usize + row) % HEIGHT;

                            let idx = col + (row * WIDTH);
                            let current_pixel = self.video[idx];
                            
                            // If the pixel has collided
                            self.registers[0xF] = if current_pixel == 0xFF { 1 } else { 0 };

                            // Here's where we actually edit the video memory
                            self.video[idx] = current_pixel ^ 0xFF;
                        }
                    }
                }
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
            (0xF, _, _, 0x9) => self.index = FONT_START_ADDR as u16 + (vx as u16 * 5),
            // Fx33: LD B, Vx
            (0xF, _, _, 0x3) => {
                self.memory[self.index as usize + 2] = vx % 10;
                self.memory[self.index as usize + 1] = (vx / 10) % 10;
                self.memory[self.index as usize] = (vx / 100) % 10;
            },
            // Fx55: LD [I], Vx
            (0xF, _, 0x5, _) => {
                for i in 0..=x {
                    self.memory[self.index as usize + i] = self.registers[i]; 
                }
            }
            // Fx65: LD Vx, [I]
            (0xF, _, 0x6, _) => {
                for i in 0..=x {
                    self.registers[i] = self.memory[self.index as usize + i];
                }
            }
            // Otherwise
            (_, _, _, _) => ()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_opcode(opcode_test: u16) -> CHIP8 {
        let op1 = (opcode_test & 0xFF00) >> 8;
        let op2 = opcode_test & 0x00FF;

        let mut chip8 = CHIP8::new();
        chip8.pc = 0x200;
        chip8.memory[0x200] = op1 as u8;
        chip8.memory[0x201] = op2 as u8;

        chip8
    }

    #[test]
    fn test_00e0() {
        let mut test = setup_opcode(0x00E0);
        test.video = [1;VIDEO_SIZE];
        test.tick();
        assert_eq!(test.video, [0;VIDEO_SIZE]);
    }

    #[test]
    fn test_fx15() {
        let mut test = setup_opcode(0xF515);
        test.registers[0x5] = 0x2B;
        test.tick();
        assert_eq!(test.delay_timer, 0x2A);
    }

    #[test]
    fn test_7xkk() {
        let mut test = setup_opcode(0x7605);
        test.registers[0x6] = 0xE5;
        test.tick();
        assert_eq!(test.registers[0x6], 0xEA);
    }

    #[test]
    fn test_1nnn() {
        let mut test = setup_opcode(0x1333);
        test.tick(); 
        assert_eq!(test.pc, 0x333);
    }

    #[test]
    fn test_fx0a() {
        let mut test = setup_opcode(0xF70A);
        test.keypad.set(0x5, true);
        test.tick();
        assert_eq!(test.registers[0x7], 0x5);
    }
}
