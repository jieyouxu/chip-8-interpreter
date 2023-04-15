use raylib::input::key_from_i32;
use raylib::prelude::*;

use std::collections::HashMap;
use std::time::{Duration, Instant};

const RECT_LEN: usize = 20;
const GRID_WIDTH: usize = 64;
const GRID_HEIGHT: usize = 32;
const TIME_STEP_MS: f64 = 100.0;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VariableRegister {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    VA,
    VB,
    VC,
    VD,
    VE,
    VF,
}

impl From<u8> for VariableRegister {
    fn from(value: u8) -> Self {
        match value {
            0x0 => Self::V0,
            0x1 => Self::V1,
            0x2 => Self::V2,
            0x3 => Self::V3,
            0x4 => Self::V4,
            0x5 => Self::V5,
            0x6 => Self::V6,
            0x7 => Self::V7,
            0x8 => Self::V8,
            0x9 => Self::V9,
            0xA => Self::VA,
            0xB => Self::VB,
            0xC => Self::VC,
            0xD => Self::VD,
            0xE => Self::VE,
            0xF => Self::VF,
            _ => panic!("invalid register"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Instruction {
    ClearScreen,
    Jump(u16),
    JumpWithOffset(u16),
    Call(u16),
    Pop,
    SetRegImm(VariableRegister, u8),
    AddRegImm(VariableRegister, u8),
    SetIdxRegImm(u16),
    Display {
        x: VariableRegister,
        y: VariableRegister,
        n: u8,
    },
    SkipIfEqImm(VariableRegister, u8),
    SkipIfNeqImm(VariableRegister, u8),
    SkipIfEqReg(VariableRegister, VariableRegister),
    SkipIfNeqReg(VariableRegister, VariableRegister),
    Set(VariableRegister, VariableRegister),
    BinOr(VariableRegister, VariableRegister),
    BinAnd(VariableRegister, VariableRegister),
    Xor(VariableRegister, VariableRegister),
    Add(VariableRegister, VariableRegister),
    SubtractLR(VariableRegister, VariableRegister),
    SubtractRL(VariableRegister, VariableRegister),
    ShiftLeft(VariableRegister, VariableRegister),
    ShiftRight(VariableRegister, VariableRegister),
    Random(VariableRegister, u8),
    SkipIfKeyPressed(VariableRegister),
    SkipIfKeyNotPressed(VariableRegister),
    GetDelayTimer(VariableRegister),
    SetDelayTimer(VariableRegister),
    SetSoundTimer(VariableRegister),
    GetKey(VariableRegister),
    Font(VariableRegister),
    BinDecConversion(VariableRegister),
    Store(u8),
    Load(u8),
    AddToIndex(VariableRegister),
}

pub fn main() {
    let rom = std::fs::read("roms/ibm-logo.ch8").expect("failed to read ROM at given path");

    let mut memory = vec![0u8; 4096];

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
    ];

    memory[0x0..fonts.len()].copy_from_slice(&fonts);
    memory[0x200..(0x200 + rom.len())].copy_from_slice(&rom);

    let mut index_register = 0u16;

    let mut variable_registers = HashMap::with_capacity(16);
    variable_registers.insert(VariableRegister::V0, 0u8);
    variable_registers.insert(VariableRegister::V1, 0);
    variable_registers.insert(VariableRegister::V2, 0);
    variable_registers.insert(VariableRegister::V3, 0);
    variable_registers.insert(VariableRegister::V4, 0);
    variable_registers.insert(VariableRegister::V5, 0);
    variable_registers.insert(VariableRegister::V6, 0);
    variable_registers.insert(VariableRegister::V7, 0);
    variable_registers.insert(VariableRegister::V8, 0);
    variable_registers.insert(VariableRegister::V9, 0);
    variable_registers.insert(VariableRegister::VA, 0);
    variable_registers.insert(VariableRegister::VB, 0);
    variable_registers.insert(VariableRegister::VC, 0);
    variable_registers.insert(VariableRegister::VD, 0);
    variable_registers.insert(VariableRegister::VE, 0);
    variable_registers.insert(VariableRegister::VF, 0);

    let mut program_counter = 0x200u16;

    let mut display = [false; GRID_WIDTH * GRID_HEIGHT];
    let mut stack: Vec<u16> = Vec::new();

    let (mut rl, thread) = raylib::init()
        .size(
            (GRID_WIDTH * RECT_LEN) as i32,
            (GRID_HEIGHT * RECT_LEN) as i32,
        )
        .title("CHIP-8 Interpreter")
        .build();

    let mut key_downs = HashMap::with_capacity(16);
    key_downs.insert(KeyboardKey::KEY_ONE, false);
    key_downs.insert(KeyboardKey::KEY_TWO, false);
    key_downs.insert(KeyboardKey::KEY_THREE, false);
    key_downs.insert(KeyboardKey::KEY_FOUR, false);
    key_downs.insert(KeyboardKey::KEY_Q, false);
    key_downs.insert(KeyboardKey::KEY_W, false);
    key_downs.insert(KeyboardKey::KEY_E, false);
    key_downs.insert(KeyboardKey::KEY_R, false);
    key_downs.insert(KeyboardKey::KEY_A, false);
    key_downs.insert(KeyboardKey::KEY_S, false);
    key_downs.insert(KeyboardKey::KEY_D, false);
    key_downs.insert(KeyboardKey::KEY_F, false);
    key_downs.insert(KeyboardKey::KEY_Z, false);
    key_downs.insert(KeyboardKey::KEY_X, false);
    key_downs.insert(KeyboardKey::KEY_C, false);
    key_downs.insert(KeyboardKey::KEY_V, false);

    let mut prev_time = Instant::now();
    let mut delay_timer = Duration::from_millis(0);
    let mut sound_timer = Duration::from_millis(0);

    while !rl.window_should_close() {
        for (key, is_pressed) in &mut key_downs {
            *is_pressed = rl.is_key_pressed(*key);
        }

        let current_time = Instant::now();
        let delta = current_time - prev_time;

        delay_timer = delay_timer.saturating_sub(delta);
        sound_timer = sound_timer.saturating_sub(delta);

        if delta > std::time::Duration::from_millis(TIME_STEP_MS as u64) {
            let ins = fetch(&mut memory, &mut program_counter);
            let ins = decode(ins);
            println!("{:X?}", ins);
            execute(
                &mut memory,
                &mut display,
                &mut program_counter,
                &mut stack,
                &mut variable_registers,
                &mut index_register,
                &mut key_downs,
                &mut delay_timer,
                &mut sound_timer,
                ins,
            );

            prev_time = current_time;
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        draw_grid(&display, &mut d);
    }
}

fn draw_grid(display: &[bool; GRID_WIDTH * GRID_HEIGHT], d: &mut RaylibDrawHandle) {
    let mut gx = 0;
    let mut gy = 0;
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            match display[(y * GRID_WIDTH + x) as usize] {
                true => d.draw_rectangle(
                    (gx * RECT_LEN) as i32,
                    (gy * RECT_LEN) as i32,
                    RECT_LEN as i32,
                    RECT_LEN as i32,
                    Color::WHITE,
                ),
                false => d.draw_rectangle(
                    (gx * RECT_LEN) as i32,
                    (gy * RECT_LEN) as i32,
                    RECT_LEN as i32,
                    RECT_LEN as i32,
                    Color::BLACK,
                ),
            }
            gx += 1;
        }
        gx = 0;
        gy += 1;
    }
}

fn fetch(memory: &mut [u8], program_counter: &mut u16) -> u16 {
    let ins = u16::from_be_bytes([
        memory[*program_counter as usize],
        memory[(*program_counter + 1) as usize],
    ]);
    *program_counter += 2;
    ins
}

fn decode(ins: u16) -> Instruction {
    let first_nibble = ins >> 12 & 0x0F;
    match first_nibble {
        0x00 => {
            let second_nibble = ins >> 8 & 0x0F;
            match second_nibble {
                0x00 => {
                    let second_byte = ins & 0xFF;
                    match second_byte {
                        0xE0 => Instruction::ClearScreen,
                        0xEE => Instruction::Pop,
                        _ => unimplemented!("unknown instruction"),
                    }
                }
                _ => unimplemented!("unknown instruction"),
            }
        }
        0x01 => {
            let imm = ins & 0xFFF;
            Instruction::Jump(imm)
        }
        0x02 => {
            let imm = ins & 0xFFF;
            Instruction::Call(imm)
        }
        0x03 => {
            let reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
            let imm = (ins & 0xFF) as u8;
            Instruction::SkipIfEqImm(reg, imm)
        }
        0x04 => {
            let reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
            let imm = (ins & 0xFF) as u8;
            Instruction::SkipIfNeqImm(reg, imm)
        }
        0x05 => {
            let x_reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
            let y_reg = VariableRegister::from((ins >> 4 & 0x0F) as u8);
            if ins & 0x0F == 0x0 {
                Instruction::SkipIfEqReg(x_reg, y_reg)
            } else {
                unimplemented!("unknown instruction")
            }
        }
        0x06 => {
            let imm = (ins & 0xFF) as u8;
            let reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
            Instruction::SetRegImm(reg, imm)
        }
        0x07 => {
            let imm = (ins & 0xFF) as u8;
            let reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
            Instruction::AddRegImm(reg, imm)
        }
        0x08 => {
            // Logical and arithmetic instructions
            match ins & 0x0F {
                0x00 => {
                    let x_reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
                    let y_reg = VariableRegister::from((ins >> 4 & 0x0F) as u8);
                    Instruction::Set(x_reg, y_reg)
                }
                0x01 => {
                    let x_reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
                    let y_reg = VariableRegister::from((ins >> 4 & 0x0F) as u8);
                    Instruction::BinOr(x_reg, y_reg)
                }
                0x02 => {
                    let x_reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
                    let y_reg = VariableRegister::from((ins >> 4 & 0x0F) as u8);
                    Instruction::BinAnd(x_reg, y_reg)
                }
                0x03 => {
                    let x_reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
                    let y_reg = VariableRegister::from((ins >> 4 & 0x0F) as u8);
                    Instruction::Xor(x_reg, y_reg)
                }
                0x04 => {
                    let x_reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
                    let y_reg = VariableRegister::from((ins >> 4 & 0x0F) as u8);
                    Instruction::Add(x_reg, y_reg)
                }
                0x05 => {
                    let x_reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
                    let y_reg = VariableRegister::from((ins >> 4 & 0x0F) as u8);
                    Instruction::SubtractLR(x_reg, y_reg)
                }
                0x06 => {
                    let x_reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
                    let y_reg = VariableRegister::from((ins >> 4 & 0x0F) as u8);
                    Instruction::ShiftRight(x_reg, y_reg)
                }
                0x07 => {
                    let x_reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
                    let y_reg = VariableRegister::from((ins >> 4 & 0x0F) as u8);
                    Instruction::SubtractRL(x_reg, y_reg)
                }
                0x0E => {
                    let x_reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
                    let y_reg = VariableRegister::from((ins >> 4 & 0x0F) as u8);
                    Instruction::ShiftLeft(x_reg, y_reg)
                }
                _ => unimplemented!("unknown instruction"),
            }
        }
        0x09 => {
            let x_reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
            let y_reg = VariableRegister::from((ins >> 4 & 0x0F) as u8);
            if ins & 0x0F == 0x0 {
                Instruction::SkipIfNeqReg(x_reg, y_reg)
            } else {
                unimplemented!("unknown instruction")
            }
        }
        0x0A => {
            let imm = ins & 0xFFF;
            Instruction::SetIdxRegImm(imm)
        }
        0x0B => {
            let imm = ins & 0xFFF;
            Instruction::JumpWithOffset(imm)
        }
        0x0C => {
            let reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
            let imm = (ins & 0xFF) as u8;
            Instruction::Random(reg, imm)
        }
        0x0D => {
            let x_reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
            let y_reg = VariableRegister::from((ins >> 12 & 0x0F) as u8);
            let imm = (ins & 0x0F) as u8;
            Instruction::Display {
                x: x_reg,
                y: y_reg,
                n: imm,
            }
        }
        0x0E => {
            let reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
            match ins & 0xFF {
                0x9E => Instruction::SkipIfKeyPressed(reg),
                0xA1 => Instruction::SkipIfKeyNotPressed(reg),
                _ => unimplemented!("unknown instruction"),
            }
        }
        0x0F => {
            let reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
            match ins & 0xFF {
                0x07 => Instruction::GetDelayTimer(reg),
                0x0A => Instruction::GetKey(reg),
                0x15 => Instruction::SetDelayTimer(reg),
                0x18 => Instruction::SetSoundTimer(reg),
                0x1E => Instruction::AddToIndex(reg),
                0x29 => Instruction::Font(reg),
                0x33 => Instruction::BinDecConversion(reg),
                0x55 => {
                    let imm = (ins >> 8 & 0x0F) as u8;
                    Instruction::Store(imm)
                }
                0x65 => {
                    let imm = (ins >> 8 & 0x0F) as u8;
                    Instruction::Load(imm)
                }
                _ => {
                    println!("unknown instruction: {:X}", ins);
                    unimplemented!("unknown instruction")
                }
            }
        }
        _ => unimplemented!("unknown instruction"),
    }
}

fn execute(
    memory: &mut [u8],
    display: &mut [bool],
    program_counter: &mut u16,
    stack: &mut Vec<u16>,
    variable_registers: &mut HashMap<VariableRegister, u8>,
    index_register: &mut u16,
    key_downs: &mut HashMap<KeyboardKey, bool>,
    delay_timer: &mut Duration,
    sound_timer: &mut Duration,
    ins: Instruction,
) {
    match ins {
        Instruction::ClearScreen => {
            for pixel in display {
                *pixel = false;
            }
        }
        Instruction::Jump(loc) => {
            *program_counter = loc;
        }
        Instruction::JumpWithOffset(offset) => {
            // This follows the COSMAC VIP interpreter to jump to address `NNN` plus value in
            // register V0.
            let addr = *variable_registers.get(&VariableRegister::V0).unwrap() as u16;
            let addr = addr + offset;
            *program_counter = addr;
        }
        Instruction::Pop => {
            let Some(loc) = stack.pop() else {
                panic!("invalid pop: missing return address from stack");
            };
            *program_counter = loc;
        }
        Instruction::Call(loc) => {
            stack.push(*program_counter);
            *program_counter = loc;
        }
        Instruction::SetRegImm(reg, imm) => {
            variable_registers.entry(reg).and_modify(|v| *v = imm);
        }
        Instruction::AddRegImm(reg, imm) => {
            variable_registers
                .entry(reg)
                .and_modify(|v| *v = (*v).wrapping_add(imm));
        }
        Instruction::SetIdxRegImm(imm) => {
            *index_register = imm;
        }
        Instruction::Display { x, y, n } => {
            let x = (variable_registers.get(&x).unwrap() & (GRID_WIDTH - 1) as u8) as u64;
            let y = (variable_registers.get(&y).unwrap() & (GRID_HEIGHT - 1) as u8) as u64;
            variable_registers
                .entry(VariableRegister::VF)
                .and_modify(|v| *v = 0);
            println!(
                "Display({:X?}, {:X?}): {:X?}",
                x,
                y,
                &memory[(*index_register as usize)..((*index_register + n as u16) as usize)]
            );
            for j in 0u64..n as u64 {
                if y + j >= GRID_HEIGHT as u64 {
                    break;
                }

                let sprite_byte = memory[(*index_register + j as u16) as usize];
                for i in 0u64..8 {
                    if x + i >= GRID_WIDTH as u64 {
                        break;
                    }

                    let sprite_pixel = (sprite_byte >> (7 - i)) & 0x1;
                    if sprite_pixel == 1 && display[((y + j) * 64 + (x + i)) as usize] {
                        display[((y + j) * 64 + (x + i)) as usize] = false;
                        variable_registers
                            .entry(VariableRegister::VF)
                            .and_modify(|v| *v = 1);
                    } else if sprite_pixel == 1 && !display[((y + j) * 64 + (x + i)) as usize] {
                        display[((y + j) * 64 + (x + i)) as usize] = true;
                    }
                }
            }
        }
        Instruction::SkipIfEqImm(reg, imm) => {
            if *variable_registers.get(&reg).unwrap() == imm {
                *program_counter += 2;
            }
        }
        Instruction::SkipIfNeqImm(reg, imm) => {
            if *variable_registers.get(&reg).unwrap() != imm {
                *program_counter += 2;
            }
        }
        Instruction::SkipIfEqReg(x_reg, y_reg) => {
            if variable_registers.get(&x_reg).unwrap() == variable_registers.get(&y_reg).unwrap() {
                *program_counter += 2;
            }
        }
        Instruction::SkipIfNeqReg(x_reg, y_reg) => {
            if variable_registers.get(&x_reg).unwrap() != variable_registers.get(&y_reg).unwrap() {
                *program_counter += 2;
            }
        }
        Instruction::Set(x_reg, y_reg) => {
            let val = *variable_registers.get(&y_reg).unwrap();
            variable_registers.entry(x_reg).and_modify(|v| *v = val);
        }
        Instruction::BinOr(x_reg, y_reg) => {
            let x_val = *variable_registers.get(&x_reg).unwrap();
            let y_val = *variable_registers.get(&y_reg).unwrap();
            variable_registers
                .entry(x_reg)
                .and_modify(|v| *v = x_val | y_val);
        }
        Instruction::BinAnd(x_reg, y_reg) => {
            let x_val = *variable_registers.get(&x_reg).unwrap();
            let y_val = *variable_registers.get(&y_reg).unwrap();
            variable_registers
                .entry(x_reg)
                .and_modify(|v| *v = x_val & y_val);
        }
        Instruction::Xor(x_reg, y_reg) => {
            let x_val = *variable_registers.get(&x_reg).unwrap();
            let y_val = *variable_registers.get(&y_reg).unwrap();
            variable_registers
                .entry(x_reg)
                .and_modify(|v| *v = x_val ^ y_val);
        }
        Instruction::Add(x_reg, y_reg) => {
            let x_val = *variable_registers.get(&x_reg).unwrap();
            let y_val = *variable_registers.get(&y_reg).unwrap();
            match x_val.checked_add(y_val) {
                None => {
                    variable_registers
                        .entry(VariableRegister::VF)
                        .and_modify(|v| *v = 1);
                    variable_registers
                        .entry(x_reg)
                        .and_modify(|v| *v = (x_val + y_val) & 0xFF);
                }
                Some(val) => {
                    variable_registers
                        .entry(VariableRegister::VF)
                        .and_modify(|v| *v = 0);
                    variable_registers.entry(x_reg).and_modify(|v| *v = val);
                }
            }
        }
        Instruction::SubtractLR(x_reg, y_reg) => {
            let v1 = *variable_registers.get(&x_reg).unwrap();
            let v2 = *variable_registers.get(&y_reg).unwrap();
            if v1 > v2 {
                variable_registers
                    .entry(VariableRegister::VF)
                    .and_modify(|v| *v = 1);
                variable_registers
                    .entry(x_reg)
                    .and_modify(|v| *v = v1 - v2 & 0xFF);
            } else {
                variable_registers
                    .entry(VariableRegister::VF)
                    .and_modify(|v| *v = 0);
                variable_registers
                    .entry(x_reg)
                    .and_modify(|v| *v = v1.wrapping_sub(v2) & 0xFF);
            }
        }
        Instruction::SubtractRL(x_reg, y_reg) => {
            let v1 = *variable_registers.get(&y_reg).unwrap();
            let v2 = *variable_registers.get(&x_reg).unwrap();
            if v1 > v2 {
                variable_registers
                    .entry(VariableRegister::VF)
                    .and_modify(|v| *v = 1);
                variable_registers
                    .entry(x_reg)
                    .and_modify(|v| *v = v1 - v2 & 0xFF);
            } else {
                variable_registers
                    .entry(VariableRegister::VF)
                    .and_modify(|v| *v = 0);
                variable_registers
                    .entry(x_reg)
                    .and_modify(|v| *v = v1.wrapping_sub(v2) & 0xFF);
            }
        }
        Instruction::ShiftLeft(x_reg, _y_reg) => {
            // We follow CHIP-48 and SUPER-CHIP so that shifts happen to VX in place and ignores Y
            // completely.
            let mut val = *variable_registers.get(&x_reg).unwrap();
            let vf = val & 0x80;
            val <<= 1;
            variable_registers
                .entry(VariableRegister::VF)
                .and_modify(|v| *v = vf);
            variable_registers.entry(x_reg).and_modify(|v| *v = val);
        }
        Instruction::ShiftRight(x_reg, _y_reg) => {
            // We follow CHIP-48 and SUPER-CHIP so that shifts happen to VX in place and ignores Y
            // completely.
            let mut val = *variable_registers.get(&x_reg).unwrap();
            let vf = val & 0x01;
            val >>= 1;
            variable_registers
                .entry(VariableRegister::VF)
                .and_modify(|v| *v = vf);
            variable_registers.entry(x_reg).and_modify(|v| *v = val);
        }
        Instruction::Random(reg, imm) => {
            use rand::Rng;
            let r: u8 = rand::thread_rng().gen();
            let r = r & imm;
            variable_registers.entry(reg).and_modify(|v| *v = r);
        }
        Instruction::SkipIfKeyPressed(reg) => {
            let key = *variable_registers.get(&reg).unwrap();
            let key = key_from_i32(key as i32).unwrap();
            if *key_downs.get(&key).unwrap() {
                *program_counter += 2;
            }
        }
        Instruction::SkipIfKeyNotPressed(reg) => {
            let key = *variable_registers.get(&reg).unwrap();
            let key = key_from_i32(key as i32).unwrap();
            if !*key_downs.get(&key).unwrap() {
                *program_counter += 2;
            }
        }
        Instruction::GetDelayTimer(reg) => {
            let delay_timer = delay_timer.as_millis() as u8;
            variable_registers
                .entry(reg)
                .and_modify(|v| *v = delay_timer);
        }
        Instruction::SetDelayTimer(reg) => {
            let val = *variable_registers.get(&reg).unwrap();
            *delay_timer = Duration::from_millis(val as u64);
        }
        Instruction::SetSoundTimer(reg) => {
            let val = *variable_registers.get(&reg).unwrap();
            *sound_timer = Duration::from_millis(val as u64);
        }
        Instruction::GetKey(reg) => {
            for (key, is_pressed) in key_downs {
                if *is_pressed {
                    let key = *key as u32 as u8;
                    variable_registers.entry(reg).and_modify(|v| *v = key);
                    return;
                }
            }

            *program_counter -= 2;
        }
        Instruction::Font(reg) => {
            let addr = *variable_registers.get(&reg).unwrap();
            *index_register = addr as u16;
        }
        Instruction::BinDecConversion(reg) => {
            let val = *variable_registers.get(&reg).unwrap();
            let d3 = val % 10;
            let d2 = val / 10 % 10;
            let d1 = val / 100;

            let addr = *index_register as usize;
            memory[addr] = d1;
            memory[addr + 1] = d2;
            memory[addr + 2] = d3;
        }
        Instruction::Store(x) => {
            let variable_registers = variable_registers.iter().collect::<Vec<_>>();
            let addr = *index_register as usize;
            for i in 0..usize::min(x as usize, variable_registers.len()) {
                let (_, val) = variable_registers[i];
                memory[addr + i] = *val;
            }
        }
        Instruction::Load(x) => {
            let mut variable_registers = variable_registers
                .iter()
                .map(|(k, v)| (*k, *v))
                .collect::<Vec<_>>();
            let addr = *index_register as usize;
            for i in 0..usize::min(x as usize, variable_registers.len()) {
                variable_registers[i].1 = memory[addr + i];
            }
        }
        Instruction::AddToIndex(reg) => {
            // We use AMIGA interpreter's behavior of setting VF to 1 if I overflows from 0x0FFF to
            // above 0x1000.
            let offset = *variable_registers.get(&reg).unwrap();
            if *index_register + offset as u16 > 0x0FFF {
                variable_registers
                    .entry(VariableRegister::VF)
                    .and_modify(|v| *v = 1);
                *index_register += offset as u16;
            } else {
                variable_registers
                    .entry(VariableRegister::VF)
                    .and_modify(|v| *v = 0);
                *index_register += offset as u16;
            }
        }
    }
}
