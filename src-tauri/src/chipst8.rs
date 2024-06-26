use std::{
    sync::{mpsc::Sender, Arc},
    thread,
    time::Duration,
};

use parking_lot::Mutex;
use rand::prelude::*;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const FONT_ADDRESS: usize = 0x050;
const FONT_SIZE: usize = 80;

pub type Display = [[bool; SCREEN_WIDTH]; SCREEN_HEIGHT];

pub struct Chipst8 {
    memory: [u8; 4096],
    pc: u16,
    i: u16,
    v: [u8; 16],
    stack: Vec<u16>,
    sound_timer: Arc<Mutex<u8>>,
    delay_timer: Arc<Mutex<u8>>,
    display: Display,
    keys: [bool; 16],
    is_running: bool,
    display_tx: Sender<Display>,
    micros_per_cycle: u64,
    wait_for_key_up: bool,
}

impl Chipst8 {
    pub fn new(display_tx: Sender<Display>, beep_tx: Sender<bool>) -> Chipst8 {
        let fonts = [
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
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ] as [u8; FONT_SIZE];

        let mut memory = [0; 4096];
        memory[FONT_ADDRESS..FONT_ADDRESS + FONT_SIZE].copy_from_slice(&fonts);

        let delay_timer = Arc::new(Mutex::new(0));
        let sound_timer = Arc::new(Mutex::new(0));
        let beep_tx = Arc::new(beep_tx);

        {
            let delay_timer = delay_timer.clone();
            thread::spawn(move || loop {
                thread::sleep(Duration::from_millis(16));
                let mut timer = delay_timer.lock();
                if *timer > 0 {
                    *timer -= 1;
                }
            })
        };

        {
            let sound_timer = sound_timer.clone();
            let beep_tx = beep_tx.clone();
            thread::spawn(move || loop {
                thread::sleep(Duration::from_millis(16));
                let mut timer = sound_timer.lock();
                let mut beep = false;
                if *timer > 0 {
                    beep = true;
                    *timer -= 1;
                }
                match (*beep_tx).send(beep) {
                    Ok(_) => (),
                    Err(e) => eprintln!("beep: {e}"),
                }
            })
        };

        Chipst8 {
            memory,
            i: 0,
            v: [0; 16],
            stack: Vec::new(),
            sound_timer,
            delay_timer,
            display: [[false; SCREEN_WIDTH]; SCREEN_HEIGHT],
            pc: 0x200,
            keys: [false; 16],
            is_running: false,
            display_tx,
            micros_per_cycle: 300,
            wait_for_key_up: false,
        }
    }

    pub fn reset(&mut self) {
        self.pc = 0x200;
        self.i = 0;
        self.v = [0; 16];
        self.stack = Vec::new();
        self.display = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT];
        self.keys = [false; 16];
        self.is_running = false;
        self.wait_for_key_up = false;
        *self.delay_timer.lock() = 0;
        *self.sound_timer.lock() = 0;
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.reset();
        self.memory[0x200..0x200 + rom.len()].copy_from_slice(&rom);
        self.is_running = true;
    }

    pub fn speedup(&mut self) {
        if self.micros_per_cycle > 100 {
            self.micros_per_cycle -= 100;
        }
        println!("speed: {}", self.micros_per_cycle);
    }

    pub fn speeddown(&mut self) {
        if self.micros_per_cycle < 2000 {
            self.micros_per_cycle += 100;
        }
        println!("speed: {}", self.micros_per_cycle);
    }

    fn draw(&self) {
        match self.display_tx.send(self.display) {
            Ok(_) => return,
            Err(e) => eprintln!("miss a frame: {e}"),
        }
    }

    pub fn set_key(&mut self, key: usize, pressed: bool) {
        self.keys[key] = pressed;
        if !pressed {
            self.wait_for_key_up = false;
        }
    }

    pub fn cycle(&mut self) {
        if !self.is_running {
            thread::sleep(Duration::from_micros(self.micros_per_cycle));
            return;
        }

        if self.wait_for_key_up {
            thread::sleep(Duration::from_micros(self.micros_per_cycle));
            return;
        }

        let instruction = self.fetch();
        //println!("pc: {:04X} inst: {instruction:04X}", self.pc);
        self.execute(instruction);
        thread::sleep(Duration::from_micros(self.micros_per_cycle));
    }

    fn fetch(&mut self) -> u16 {
        let high_bits = self.memory[self.pc as usize] as u16;
        let low_bits = self.memory[(self.pc + 1) as usize] as u16;
        let instruction = (high_bits << 8) | low_bits;
        self.pc += 2;

        instruction
    }

    fn execute(&mut self, instruction: u16) {
        let byte3 = (instruction & 0xF000) >> 12;
        let byte2 = (instruction & 0x0F00) >> 8;
        let byte1 = (instruction & 0x00F0) >> 4;
        let byte0 = instruction & 0x000F;

        let x = byte2 as usize;
        let y = byte1 as usize;
        let nnn = instruction & 0xFFF;
        let nn = (instruction & 0xFF) as u8;
        let n = instruction & 0xF;

        match (byte3, byte2, byte1, byte0) {
            // 00E0
            (0, 0, 0xE, 0) => {
                self.display = [[false; SCREEN_WIDTH]; SCREEN_HEIGHT];
                self.draw();
            }
            // 00EE
            (0, 0, 0xE, 0xE) => match self.stack.pop() {
                Some(address) => self.pc = address,
                None => {
                    eprintln!("The subroutine stack has been empty!");
                    self.is_running = false;
                }
            },
            // 1NNN
            (1, _, _, _) => {
                self.pc = nnn;
            }
            // 2NNN
            (2, _, _, _) => {
                self.stack.push(self.pc);
                self.pc = nnn;
            }
            // 3XNN
            (3, _, _, _) => {
                if self.v[x] == nn {
                    self.pc += 2;
                }
            }
            // 4XNN
            (4, _, _, _) => {
                if self.v[x] != nn {
                    self.pc += 2;
                }
            }
            // 5XY0
            (5, _, _, _) => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            // 6XNN
            (6, _, _, _) => {
                self.v[x] = nn;
            }
            // 7XNN
            (7, _, _, _) => {
                self.v[x] = self.v[x].wrapping_add(nn);
            }
            // 8XY0
            (8, _, _, 0) => {
                self.v[x] = self.v[y];
            }
            // 8XY1
            (8, _, _, 1) => {
                self.v[x] |= self.v[y];
            }
            // 8XY2
            (8, _, _, 2) => {
                self.v[x] &= self.v[y];
            }
            // 8XY3
            (8, _, _, 3) => {
                self.v[x] ^= self.v[y];
            }
            // 8XY4
            (8, _, _, 4) => {
                let (res, carry) = self.v[x].overflowing_add(self.v[y]);
                self.v[x] = res;
                self.v[0xF] = if carry { 1 } else { 0 };
            }
            // 8XY5
            (8, _, _, 5) => {
                let (res, borrow) = self.v[x].overflowing_sub(self.v[y]);
                self.v[x] = res;
                self.v[0xF] = if borrow { 0 } else { 1 };
            }
            // 8XY6
            // be careful of instruction 8FF6
            (8, _, _, 6) => {
                let flag = self.v[y] & 0x1;
                self.v[x] = self.v[y] >> 1;
                self.v[0xF] = flag;
            }
            // 8XY7
            (8, _, _, 7) => {
                let (res, borrow) = self.v[y].overflowing_sub(self.v[x]);
                self.v[x] = res;
                self.v[0xF] = if borrow { 0 } else { 1 };
            }
            // 8XYE
            (8, _, _, 0xE) => {
                let flag = (self.v[y] >> 7) & 0x1;
                self.v[x] = self.v[y] << 1;
                self.v[0xF] = flag;
            }
            // 9XY0
            (9, _, _, 0) => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            // ANNN
            (0xA, _, _, _) => {
                self.i = nnn;
            }
            // BNNN
            (0xB, _, _, _) => {
                self.pc = nnn + self.v[0] as u16;
            }
            // CNNN
            (0xC, _, _, _) => {
                let randnum: u8 = random();
                self.v[x] = nn & randnum;
            }
            // DXYN
            (0xD, _, _, _) => {
                let mut py = ((self.v[y] as usize) % SCREEN_HEIGHT) as usize;
                self.v[0xF] = 0;

                let sprite = &self.memory[(self.i as usize)..((self.i + n) as usize)];
                for i in 0..n {
                    if py >= SCREEN_HEIGHT {
                        break;
                    }
                    let sprite_row = sprite[i as usize];
                    let mut px = ((self.v[x] as usize) % SCREEN_WIDTH) as usize;
                    for j in 0..8 {
                        if px >= SCREEN_WIDTH {
                            break;
                        }

                        let pixel = ((sprite_row >> (7 - j)) & 0x1) != 0;
                        if self.display[py][px] && pixel {
                            self.display[py][px] = false;
                            self.v[0xF] = 1;
                        } else if !self.display[py][px] && pixel {
                            self.display[py][px] = true;
                        }
                        px += 1;
                    }
                    py += 1;
                }

                self.draw();
            }
            // EX9E
            (0xE, _, 9, 0xE) => {
                let key = self.v[x];
                if self.keys[key as usize] {
                    self.pc += 2;
                }
            }
            // EXA1
            (0xE, _, 0xA, 1) => {
                let key = self.v[x];
                if !self.keys[key as usize] {
                    self.pc += 2;
                }

            }
            // FX07
            (0xF, _, 0, 7) => {
                self.v[x] = *self.delay_timer.lock();
            },
            // FX0A
            (0xF, _, 0, 0xA) => {
                for i in 0..16 {
                    if self.keys[i] {
                        self.v[x] = i as u8;
                        self.wait_for_key_up = true;
                        return;
                    }
                }
                self.pc -= 2;
            },
            // FX15
            (0xF, _, 1, 5) => {
                *self.delay_timer.lock() = self.v[x];
            },
            // FX18
            (0xF, _, 1, 8) => {
                *self.sound_timer.lock() = self.v[x]
            },
            // FX1E
            (0xF, _, 1, 0xE) => {
                self.i = self.i.wrapping_add(self.v[x] as u16);
            }
            // FX29
            (0xF, _, 2, 9) => {
                self.i = FONT_ADDRESS as u16 + self.v[x] as u16 * 5;
            }
            // FX33
            (0xF, _, 3, 3) => {
                let num = self.v[x];
                let d0 = num % 10;
                let d1 = (num % 100) / 10;
                let d2 = num / 100;

                let i = self.i as usize;

                self.memory[i] = d2;
                self.memory[i + 1] = d1;
                self.memory[i + 2] = d0;
            }
            // FX55
            (0xF, _, 5, 5) => {
                let i = self.i as usize;
                for idx in 0..x + 1 {
                    self.memory[i + idx] = self.v[idx];
                }
                self.i += x as u16 + 1;
            }
            // FX65
            (0xF, _, 6, 5) => {
                let i = self.i as usize;
                for idx in 0..x + 1 {
                    self.v[idx] = self.memory[i + idx];
                }
                self.i += x as u16 + 1;
            }
            _ => {
                eprintln!("Unsupported instruction!");
                self.is_running = false;
            }
        }
    }
}
