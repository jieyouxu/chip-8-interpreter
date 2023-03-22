use std::collections::HashMap;

use crossbeam_channel::{after, select, tick};

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
    Call(u16),
    Pop,
    SetRegImm(VariableRegister, u16),
    AddRegImm(VariableRegister, u16),
    SetIdxRegImm(u16),
    Display {
        x: VariableRegister,
        y: VariableRegister,
        n: u16,
    },
}

pub fn main() {
    let rom = include_bytes!("../roms/ibm-logo.ch8");

    let instructions_per_second = 700;
    let instr_delay_ms = 1000.0 / instructions_per_second as f64;
    let instr_delay_ms = std::time::Duration::from_millis(instr_delay_ms.floor() as u64);
    println!("delay_ms: {:?}", instr_delay_ms);

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
    memory[0x200..(0x200 + rom.len())].copy_from_slice(rom);

    let mut index_register = 0u16;

    let mut variable_registers = HashMap::with_capacity(16);
    variable_registers.insert(VariableRegister::V0, 0);
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

    let mut display = vec![false; 64 * 32];
    let mut stack: Vec<u16> = Vec::new();

    let mut delay_timer = 0u8;
    let mut sound_timer = 0u8;
    let timer_delay_ms = std::time::Duration::from_millis((1000.0f64 / 60.0).floor() as u64);

    let timer_ticker = tick(timer_delay_ms);
    let timeout = after(std::time::Duration::from_millis(30));

    let instr_ticker = tick(instr_delay_ms);

    loop {
        select! {
            recv(timer_ticker) -> _ => {
                delay_timer = delay_timer.saturating_sub(1);
                sound_timer = sound_timer.saturating_sub(1);
            },
            recv(instr_ticker) -> _ => {
                let ins = fetch(&mut memory, &mut program_counter);
                let ins = decode(ins);
                println!("{:X?}", ins);
                execute(&mut memory, &mut display, &mut program_counter, &mut stack, &mut variable_registers, &mut index_register, ins);
                println!("variable_registers: {:#X?}", &variable_registers);
                println!("index_register: {:X?}", &index_register);
                println!("program_counter: {:X?}", &program_counter);
                render(&display);
            },
            recv(timeout) -> _ => break,
        }
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
        0x06 => {
            let imm = ins & 0xFF;
            let reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
            Instruction::SetRegImm(reg, imm)
        }
        0x07 => {
            let imm = ins & 0xFF;
            let reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
            Instruction::AddRegImm(reg, imm)
        }
        0x0A => {
            let imm = ins & 0xFFF;
            Instruction::SetIdxRegImm(imm)
        }
        0x0D => {
            let x_reg = VariableRegister::from((ins >> 8 & 0x0F) as u8);
            let y_reg = VariableRegister::from((ins >> 12 & 0x0F) as u8);
            let imm = ins & 0x0F;
            Instruction::Display {
                x: x_reg,
                y: y_reg,
                n: imm,
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
    variable_registers: &mut HashMap<VariableRegister, u16>,
    index_register: &mut u16,
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
            let x = variable_registers.get(&x).unwrap() & 63;
            let y = variable_registers.get(&y).unwrap() & 31;
            variable_registers
                .entry(VariableRegister::VF)
                .and_modify(|v| *v = 0);
            println!(
                "DISPLAY({:X?}, {:X?}): {:X?}",
                x,
                y,
                &memory[(*index_register as usize)..((*index_register + n) as usize)]
            );
            for j in 0..n {
                if y + j >= 32 {
                    break;
                }

                let sprite_byte = memory[(*index_register + j) as usize];
                println!("sprite_byte: {:X}", sprite_byte);
                for i in 0..8 {
                    if x + i >= 64 {
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
    }
}

fn render(display: &[bool]) {
    for y in 0..32 {
        for x in 0..64 {
            match display[(y * 64 + x) as usize] {
                true => print!("█"),
                false => print!("."),
            }
        }
        print!("\n");
    }
}
