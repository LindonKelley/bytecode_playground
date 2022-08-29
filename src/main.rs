use std::io::{Cursor, Read, Seek};

use instruction::{Instruction, InvalidInstruction};

use crate::compute_heap::{Heap, HeapError, ObjectReference};
use crate::compute_stack::{ComputeStack, StackError};
use crate::possibly_ordering::InvalidComparisonByte;

mod infallible_division;
mod compute_stack;
mod possibly_ordering;
mod compute_heap;
mod instruction;
mod machine;

fn main() {
    use instruction::Instruction::*;
    use machine::{Machine, MachineError};
    let mut instructions = Vec::new();
    instructions
        .instruct(PSH_8(1.0f64.to_le_bytes()))
        .instruct(PSH_8(1.0f64.to_le_bytes()))
        .instruct(ADD_F_8)
        .instruct(CNV_F8_U8);
    println!("{:?}", instructions);
    let mut machine = Machine {
        instructions: Box::new(Cursor::new(instructions)),
        stack: Box::new(Vec::new()),
        heap: Heap::new()
    };
    loop {
        println!("{:?}", machine.stack);
        match machine.step() {
            Ok(_) => {}
            Err((MachineError::EndOfInstructions, _)) => {
                break
            }
            Err((e, instruction)) => {
                panic!("machine hit following error during instruction {:?}\n{:?}", instruction, e);
            }
        }
        if machine.stack.size() > 100 {
            println!("machine stack size exceeded 100, stopping");
            break;
        }
    }
}

pub trait InstructionReceiver {
    fn instruct(&mut self, instruction: Instruction) -> &mut Self;

    fn push_jump_marker(&mut self, location: &mut usize) -> &mut Self;

    fn assign_jump_marker(&mut self, location: usize, address: u64) -> &mut Self;

    fn assign_jump_marker_here(&mut self, location: usize) -> &mut Self;
}

impl InstructionReceiver for Vec<u8> {
    fn instruct(&mut self, instruction: Instruction) -> &mut Self {
        match instruction {
            Instruction::PSH_1(value) => {
                self.push(0);
                self.extend_from_slice(&value);
            },
            Instruction::PSH_2(value) => {
                self.push(1);
                self.extend_from_slice(&value);
            },
            Instruction::PSH_4(value) => {
                self.push(2);
                self.extend_from_slice(&value);
            },
            Instruction::PSH_8(value) => {
                self.push(3);
                self.extend_from_slice(&value);
            },

            Instruction::POP_1 => self.push(4),
            Instruction::POP_2 => self.push(5),
            Instruction::POP_4 => self.push(6),
            Instruction::POP_8 => self.push(7),

            Instruction::ALLOC => self.push(8),
            Instruction::COPY_REF => self.push(9),
            Instruction::SET_CHILD => self.push(10),
            Instruction::GET_CHILD => self.push(11),

            Instruction::MOV_ST_HP_1 => self.push(12),
            Instruction::MOV_ST_HP_2 => self.push(13),
            Instruction::MOV_ST_HP_4 => self.push(14),
            Instruction::MOV_ST_HP_8 => self.push(15),

            Instruction::MOV_HP_ST_1 => self.push(16),
            Instruction::MOV_HP_ST_2 => self.push(17),
            Instruction::MOV_HP_ST_4 => self.push(18),
            Instruction::MOV_HP_ST_8 => self.push(19),

            Instruction::JSR => self.push(20),
            Instruction::RET => self.push(21),

            Instruction::JMP_EQ => self.push(22),
            Instruction::JMP_NE => self.push(23),
            Instruction::JMP_GE => self.push(24),
            Instruction::JMP_GT => self.push(25),
            Instruction::JMP_LE => self.push(26),
            Instruction::JMP_LT => self.push(27),


            Instruction::CMP_U_1 => self.push(28),
            Instruction::CMP_U_2 => self.push(29),
            Instruction::CMP_U_4 => self.push(30),
            Instruction::CMP_U_8 => self.push(31),
            Instruction::CMP_S_1 => self.push(32),
            Instruction::CMP_S_2 => self.push(33),
            Instruction::CMP_S_4 => self.push(34),
            Instruction::CMP_S_8 => self.push(35),
            Instruction::CMP_F4 => self.push(36),
            Instruction::CMP_F8 => self.push(37),

            Instruction::NOT_1 => self.push(38),
            Instruction::NOT_2 => self.push(39),
            Instruction::NOT_4 => self.push(40),
            Instruction::NOT_8 => self.push(41),
            Instruction::AND_1 => self.push(42),
            Instruction::AND_2 => self.push(43),
            Instruction::AND_4 => self.push(44),
            Instruction::AND_8 => self.push(45),
            Instruction::OR_1 => self.push(46),
            Instruction::OR_2 => self.push(47),
            Instruction::OR_4 => self.push(48),
            Instruction::OR_8 => self.push(49),
            Instruction::XOR_1 => self.push(50),
            Instruction::XOR_2 => self.push(51),
            Instruction::XOR_4 => self.push(52),
            Instruction::XOR_8 => self.push(53),
            Instruction::SHL_1 => self.push(54),
            Instruction::SHL_2 => self.push(55),
            Instruction::SHL_4 => self.push(56),
            Instruction::SHL_8 => self.push(57),
            Instruction::SHR_1 => self.push(58),
            Instruction::SHR_2 => self.push(59),
            Instruction::SHR_4 => self.push(60),
            Instruction::SHR_8 => self.push(61),
            Instruction::SAR_1 => self.push(62),
            Instruction::SAR_2 => self.push(63),
            Instruction::SAR_4 => self.push(64),
            Instruction::SAR_8 => self.push(65),

            Instruction::ADD_1 => self.push(66),
            Instruction::ADD_2 => self.push(67),
            Instruction::ADD_4 => self.push(68),
            Instruction::ADD_8 => self.push(69),
            Instruction::SUB_1 => self.push(70),
            Instruction::SUB_2 => self.push(71),
            Instruction::SUB_4 => self.push(72),
            Instruction::SUB_8 => self.push(73),
            Instruction::MUL_1 => self.push(74),
            Instruction::MUL_2 => self.push(75),
            Instruction::MUL_4 => self.push(76),
            Instruction::MUL_8 => self.push(77),
            Instruction::DIV_REM_U_1 => self.push(78),
            Instruction::DIV_REM_U_2 => self.push(79),
            Instruction::DIV_REM_U_4 => self.push(80),
            Instruction::DIV_REM_U_8 => self.push(81),
            Instruction::DIV_REM_S_1 => self.push(82),
            Instruction::DIV_REM_S_2 => self.push(83),
            Instruction::DIV_REM_S_4 => self.push(84),
            Instruction::DIV_REM_S_8 => self.push(85),

            Instruction::ADD_F_4 => self.push(86),
            Instruction::ADD_F_8 => self.push(87),
            Instruction::SUB_F_4 => self.push(88),
            Instruction::SUB_F_8 => self.push(89),
            Instruction::MUL_F_4 => self.push(90),
            Instruction::MUL_F_8 => self.push(91),
            Instruction::DIV_F_4 => self.push(92),
            Instruction::DIV_F_8 => self.push(93),
            Instruction::REM_F_4 => self.push(94),
            Instruction::REM_F_8 => self.push(95),

            Instruction::CNV_U8_F4 => self.push(96),
            Instruction::CNV_U8_F8 => self.push(97),
            Instruction::CNV_S8_F4 => self.push(98),
            Instruction::CNV_S8_F8 => self.push(99),

            Instruction::CNV_F4_U8 => self.push(100),
            Instruction::CNV_F8_U8 => self.push(101),
            Instruction::CNV_F4_S8 => self.push(102),
            Instruction::CNV_F8_S8 => self.push(103),

            Instruction::CNV_F4_F8 => self.push(104),
            Instruction::CNV_F8_F4 => self.push(105),

            Instruction::CALL_EXT => self.push(106),
        }
        self
    }

    fn push_jump_marker(&mut self, location: &mut usize) -> &mut Self {
        self.push(4);
        *location = self.len();
        self.extend_from_slice(&[0; 8]);
        self.instruct(Instruction::JSR);
        self
    }

    fn assign_jump_marker(&mut self, location: usize, address: u64) -> &mut Self {
        self[location..location + 8].copy_from_slice(&address.to_le_bytes());
        self
    }

    fn assign_jump_marker_here(&mut self, location: usize) -> &mut Self {
        let len = self.len();
        self.assign_jump_marker(location, len as u64)
    }
}

pub trait ReadSeek: Read + Seek {}

impl<T: Read + Seek> ReadSeek for T {}
