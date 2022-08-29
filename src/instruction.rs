use std::io;
use std::io::Read;

use crate::InvalidComparisonByte;
use crate::machine::MachineError;

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    PSH_1([u8; 1]),
    PSH_2([u8; 2]),
    PSH_4([u8; 4]),
    PSH_8([u8; 8]),

    POP_1,
    POP_2,
    POP_4,
    POP_8,

    /// Pops 8 bytes **`children_length`**\
    /// Pops 8 bytes **`data_length`**\
    /// Allocates an [Object] on the heap, with **`children_length`** slots for children, and **`data_length`** bytes for data\
    /// Pushes 8 bytes corresponding to reference of the allocated space, in related instructions, this will be referred to as an **`Object Reference`**
    ALLOC,


    COPY_REF,
    SET_CHILD,
    GET_CHILD,

    MOV_ST_HP_1, // pop 8 bytes as `address`, pop 1 byte as `var`, write `var` to heap address `address`
    MOV_ST_HP_2,
    MOV_ST_HP_4,
    MOV_ST_HP_8,

    MOV_HP_ST_1, // pop 8 bytes as `address`, push 1 byte value at heap address `address` onto the stack
    MOV_HP_ST_2,
    MOV_HP_ST_4,
    MOV_HP_ST_8,

    JSR, // pop 8 bytes as `address`, pushes the next instruction address onto the stack, jump to `address`
    RET, // pop 8 bytes as `address`, jump to `address`

    JMP_EQ, // pops 1 byte as `cmp`, pops 8 bytes as `address`, jump to `address` if `cmp` == 2
    JMP_NE,
    JMP_GE,
    JMP_GT,
    JMP_LE,
    JMP_LT,

    /// Pops 1 byte as **`a`**\
    /// Pops 1 byte as **`b`**\
    /// Pushes 1 byte corresponding to the [`PossiblyOrdering`] of **`a`** and **`b`**
    CMP_U_1,
    CMP_U_2,
    CMP_U_4,
    CMP_U_8,
    CMP_S_1, // pops 1 byte as `a`, pops 1 byte as `b`, pushes one of [-1, 0, +1] as 1 byte for less than, equal to, or greater than respectively
    CMP_S_2,
    CMP_S_4,
    CMP_S_8,
    CMP_F4, // pops 4 bytes as `a`, pops 4 bytes as `b`, pushes one of [-1, 0, +1] as 1 byte for less than, equal to, or greater than respectively, according to the totalOrder predicate as defined in IEEE 754 (2008 revision)
    CMP_F8, // pops 8 bytes as `a`, pops 8 bytes as `b`, pushes one of [-1, 0, +1] as 1 byte for less than, equal to, or greater than respectively, according to the totalOrder predicate as defined in IEEE 754 (2008 revision)

    NOT_1,
    NOT_2,
    NOT_4,
    NOT_8,
    AND_1,
    AND_2,
    AND_4,
    AND_8,
    OR_1,
    OR_2,
    OR_4,
    OR_8,
    XOR_1,
    XOR_2,
    XOR_4,
    XOR_8,
    SHL_1,
    SHL_2,
    SHL_4,
    SHL_8,
    SHR_1,
    SHR_2,
    SHR_4,
    SHR_8,
    SAR_1,
    SAR_2,
    SAR_4,
    SAR_8,

    ADD_1,
    ADD_2,
    ADD_4,
    ADD_8,
    SUB_1,
    SUB_2,
    SUB_4,
    SUB_8,
    MUL_1,
    MUL_2,
    MUL_4,
    MUL_8,
    DIV_REM_U_1,
    DIV_REM_U_2,
    DIV_REM_U_4,
    DIV_REM_U_8,
    DIV_REM_S_1,
    DIV_REM_S_2,
    DIV_REM_S_4,
    DIV_REM_S_8,

    ADD_F_4,
    ADD_F_8,
    SUB_F_4,
    SUB_F_8,
    MUL_F_4,
    MUL_F_8,
    DIV_F_4,
    DIV_F_8,
    REM_F_4,
    REM_F_8,

    CNV_U8_F4,
    CNV_U8_F8,
    CNV_S8_F4,
    CNV_S8_F8,

    CNV_F4_U8,
    CNV_F8_U8,
    CNV_F4_S8,
    CNV_F8_S8,

    CNV_F4_F8,
    CNV_F8_F4,

    CALL_EXT
}

impl Instruction {
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, MachineError> {
        let discriminant = {
            let mut data = [0; 1];
            reader.read_exact(&mut data).map_err(|e| {
                match e.kind() {
                    io::ErrorKind::UnexpectedEof => MachineError::EndOfInstructions,
                    _ => e.into()
                }
            })?;
            data[0]
        };
        match discriminant {
            0 => {
                let mut data = [0; 1];
                reader.read_exact(&mut data).map_err(|e| {
                    match e.kind() {
                        io::ErrorKind::UnexpectedEof => MachineError::IncompleteInstruction(0),
                        _ => e.into()
                    }
                })?;
                Ok(Instruction::PSH_1(data))
            },
            1 => {
                let mut data = [0; 2];
                reader.read_exact(&mut data).map_err(|e| {
                    match e.kind() {
                        io::ErrorKind::UnexpectedEof => MachineError::IncompleteInstruction(1),
                        _ => e.into()
                    }
                })?;
                Ok(Instruction::PSH_2(data))
            },
            2 => {
                let mut data = [0; 4];
                reader.read_exact(&mut data).map_err(|e| {
                    match e.kind() {
                        io::ErrorKind::UnexpectedEof => MachineError::IncompleteInstruction(2),
                        _ => e.into()
                    }
                })?;
                Ok(Instruction::PSH_4(data))
            },
            3 => {
                let mut data = [0; 8];
                reader.read_exact(&mut data).map_err(|e| {
                    match e.kind() {
                        io::ErrorKind::UnexpectedEof => MachineError::IncompleteInstruction(3),
                        _ => e.into()
                    }
                })?;
                Ok(Instruction::PSH_8(data))
            },

            4 => Ok(Instruction::POP_1),
            5 => Ok(Instruction::POP_2),
            6 => Ok(Instruction::POP_4),
            7 => Ok(Instruction::POP_8),

            8 => Ok(Instruction::ALLOC),
            9 => Ok(Instruction::COPY_REF),
            10 => Ok(Instruction::SET_CHILD),
            11 => Ok(Instruction::GET_CHILD),

            12 => Ok(Instruction::MOV_ST_HP_1),
            13 => Ok(Instruction::MOV_ST_HP_2),
            14 => Ok(Instruction::MOV_ST_HP_4),
            15 => Ok(Instruction::MOV_ST_HP_8),

            16 => Ok(Instruction::MOV_HP_ST_1),
            17 => Ok(Instruction::MOV_HP_ST_2),
            18 => Ok(Instruction::MOV_HP_ST_4),
            19 => Ok(Instruction::MOV_HP_ST_8),

            20 => Ok(Instruction::JSR),
            21 => Ok(Instruction::RET),

            22 => Ok(Instruction::JMP_EQ),
            23 => Ok(Instruction::JMP_NE),
            24 => Ok(Instruction::JMP_GE),
            25 => Ok(Instruction::JMP_GT),
            26 => Ok(Instruction::JMP_LE),
            27 => Ok(Instruction::JMP_LT),


            28 => Ok(Instruction::CMP_U_1),
            29 => Ok(Instruction::CMP_U_2),
            30 => Ok(Instruction::CMP_U_4),
            31 => Ok(Instruction::CMP_U_8),
            32 => Ok(Instruction::CMP_S_1),
            33 => Ok(Instruction::CMP_S_2),
            34 => Ok(Instruction::CMP_S_4),
            35 => Ok(Instruction::CMP_S_8),
            36 => Ok(Instruction::CMP_F4),
            37 => Ok(Instruction::CMP_F8),

            38 => Ok(Instruction::NOT_1),
            39 => Ok(Instruction::NOT_2),
            40 => Ok(Instruction::NOT_4),
            41 => Ok(Instruction::NOT_8),
            42 => Ok(Instruction::AND_1),
            43 => Ok(Instruction::AND_2),
            44 => Ok(Instruction::AND_4),
            45 => Ok(Instruction::AND_8),
            46 => Ok(Instruction::OR_1),
            47 => Ok(Instruction::OR_2),
            48 => Ok(Instruction::OR_4),
            49 => Ok(Instruction::OR_8),
            50 => Ok(Instruction::XOR_1),
            51 => Ok(Instruction::XOR_2),
            52 => Ok(Instruction::XOR_4),
            53 => Ok(Instruction::XOR_8),
            54 => Ok(Instruction::SHL_1),
            55 => Ok(Instruction::SHL_2),
            56 => Ok(Instruction::SHL_4),
            57 => Ok(Instruction::SHL_8),
            58 => Ok(Instruction::SHR_1),
            59 => Ok(Instruction::SHR_2),
            60 => Ok(Instruction::SHR_4),
            61 => Ok(Instruction::SHR_8),
            62 => Ok(Instruction::SAR_1),
            63 => Ok(Instruction::SAR_2),
            64 => Ok(Instruction::SAR_4),
            65 => Ok(Instruction::SAR_8),

            66 => Ok(Instruction::ADD_1),
            67 => Ok(Instruction::ADD_2),
            68 => Ok(Instruction::ADD_4),
            69 => Ok(Instruction::ADD_8),
            70 => Ok(Instruction::SUB_1),
            71 => Ok(Instruction::SUB_2),
            72 => Ok(Instruction::SUB_4),
            73 => Ok(Instruction::SUB_8),
            74 => Ok(Instruction::MUL_1),
            75 => Ok(Instruction::MUL_2),
            76 => Ok(Instruction::MUL_4),
            77 => Ok(Instruction::MUL_8),
            78 => Ok(Instruction::DIV_REM_U_1),
            79 => Ok(Instruction::DIV_REM_U_2),
            80 => Ok(Instruction::DIV_REM_U_4),
            81 => Ok(Instruction::DIV_REM_U_8),
            82 => Ok(Instruction::DIV_REM_S_1),
            83 => Ok(Instruction::DIV_REM_S_2),
            84 => Ok(Instruction::DIV_REM_S_4),
            85 => Ok(Instruction::DIV_REM_S_8),

            86 => Ok(Instruction::ADD_F_4),
            87 => Ok(Instruction::ADD_F_8),
            88 => Ok(Instruction::SUB_F_4),
            89 => Ok(Instruction::SUB_F_8),
            90 => Ok(Instruction::MUL_F_4),
            91 => Ok(Instruction::MUL_F_8),
            92 => Ok(Instruction::DIV_F_4),
            93 => Ok(Instruction::DIV_F_8),
            94 => Ok(Instruction::REM_F_4),
            95 => Ok(Instruction::REM_F_8),

            96 => Ok(Instruction::CNV_U8_F4),
            97 => Ok(Instruction::CNV_U8_F8),
            98 => Ok(Instruction::CNV_S8_F4),
            99 => Ok(Instruction::CNV_S8_F8),

            100 => Ok(Instruction::CNV_F4_U8),
            101 => Ok(Instruction::CNV_F8_U8),
            102 => Ok(Instruction::CNV_F4_S8),
            103 => Ok(Instruction::CNV_F8_S8),

            104 => Ok(Instruction::CNV_F4_F8),
            105 => Ok(Instruction::CNV_F8_F4),

            106 => Ok(Instruction::CALL_EXT),

            n => Err(MachineError::UnknownInstruction(n))
        }

    }
}

#[derive(Debug)]
pub enum InvalidInstruction {
    /// A byte was popped for use as comparison, and was not one of \[0, 1, 2, 3\]
    InvalidComparisonByte(u8)
}

impl From<InvalidComparisonByte> for InvalidInstruction {
    fn from(e: InvalidComparisonByte) -> Self {
        InvalidInstruction::InvalidComparisonByte(e.0)
    }
}
