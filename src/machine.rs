use std::io;
use std::io::{Seek, SeekFrom, Write};

use crate::{ComputeStack, Heap, HeapError, Instruction, InvalidComparisonByte, InvalidInstruction, ObjectReference, ReadSeek, StackError};
use crate::infallible_division::InfallibleDivision;
use crate::possibly_ordering::PossiblyOrdering;

macro_rules! stack_pop {
    ($self: ident, u8) => {
        $self.stack.pop_u8()?
    };
    ($self: ident, u16) => {
        $self.stack.pop_u16()?
    };
    ($self: ident, u32) => {
        $self.stack.pop_u32()?
    };
    ($self: ident, u64) => {
        $self.stack.pop_u64()?
    };
    ($self: ident, i8) => {
        $self.stack.pop_i8()?
    };
    ($self: ident, i16) => {
        $self.stack.pop_i16()?
    };
    ($self: ident, i32) => {
        $self.stack.pop_i32()?
    };
    ($self: ident, i64) => {
        $self.stack.pop_i64()?
    };
    ($self: ident, f32) => {
        $self.stack.pop_f32()?
    };
    ($self: ident, f64) => {
        $self.stack.pop_f64()?
    };
}

macro_rules! stack_push {
    ($self: ident, u8, $value: expr) => {
        $self.stack.push_u8($value)?;
    };
    ($self: ident, u16, $value: expr) => {
        $self.stack.push_u16($value)?;
    };
    ($self: ident, u32, $value: expr) => {
        $self.stack.push_u32($value)?;
    };
    ($self: ident, u64, $value: expr) => {
        $self.stack.push_u64($value)?;
    };
    ($self: ident, i8, $value: expr) => {
        $self.stack.push_i8($value)?;
    };
    ($self: ident, i16, $value: expr) => {
        $self.stack.push_i16($value)?;
    };
    ($self: ident, i32, $value: expr) => {
        $self.stack.push_i32($value)?;
    };
    ($self: ident, i64, $value: expr) => {
        $self.stack.push_i64($value)?;
    };
    ($self: ident, f32, $value: expr) => {
        $self.stack.push_f32($value)?;
    };
    ($self: ident, f64, $value: expr) => {
        $self.stack.push_f64($value)?;
    };
}

macro_rules! mov_st_hp_instruction_impl {
    ($self: ident, $len: expr) => {
        let obj_ref = $self.stack_pop_object_reference()?;
        let start = $self.stack.pop_u64()? as usize;
        let data = $self.stack.pop_slice($len)?;
        let mut out = $self.heap.get_mut_data_slice(&obj_ref, start, $len)?;
        out.write(&*data)?;
    };
}

macro_rules! mov_hp_st_instruction_impl {
    ($self: ident, $len: expr) => {
        let obj_ref = $self.stack_pop_object_reference()?;
        let start = $self.stack.pop_u64()? as usize;
        let data = $self.heap.get_data_slice(&obj_ref, start, $len)?;
        $self.stack.push_slice(data)?;
    };
}

macro_rules! jump_instruction_impl {
    ($self: ident, $fun: ident) => {
        let cmp: PossiblyOrdering = $self.stack.pop_u8()?.try_into()?;
        let address = $self.stack.pop_u64()?;
        if cmp.$fun() {
            $self.instructions.seek(SeekFrom::Start(address))?;
        }
    }
}

macro_rules! compare_instruction_impl {
    ($self: ident, $typ: tt) => {
        let a = stack_pop!($self, $typ);
        let b = stack_pop!($self, $typ);
        let cmp: PossiblyOrdering = PartialOrd::partial_cmp(&a, &b).into();
        $self.stack.push_u8(cmp as u8)?;
    };
}

macro_rules! not_instruction_impl {
    ($self: ident, $typ: tt) => {
        let value = stack_pop!($self, $typ);
        stack_push!($self, $typ, !value);
    };
}

macro_rules! two_argument_instruction_impl {
    ($self: ident, $typ: tt, $op: tt) => {
        two_argument_instruction_impl!($self, $typ, $op, $typ);
    };
    ($self: ident, $in_type: tt, $op: tt, $result_type: tt) => {
        two_argument_instruction_impl!($self, $in_type $op $in_type = $result_type);
    };
    ($self: ident, $a_type: tt $op: tt $b_type: tt = $result_type: tt) => {
        let a = stack_pop!($self, $a_type);
        let b = stack_pop!($self, $b_type);
        stack_push!($self, $result_type, a $op b);
    };
}

macro_rules! shift_instruction_impl {
    ($self: ident, $typ: tt, $op: tt) => {
        two_argument_instruction_impl!($self, $typ $op u8 = $typ);
    }
}

macro_rules! convert_instruction_impl {
    ($self: ident, $from: tt -> $to: tt) => {
        let v = stack_pop!($self, $from);
        stack_push!($self, $to, v as $to);
    };
}

macro_rules! div_rem_instruction_impl {
    ($self: ident, $typ: tt) => {
        let a = stack_pop!($self, $typ);
        let b = stack_pop!($self, $typ);
        stack_push!($self, $typ, <$typ>::infallible_div(a, b));
        stack_push!($self, $typ, <$typ>::infallible_rem(a, b));
    };
}

#[derive(Debug)]
pub enum MachineError {
    IO(io::Error),
    UnknownInstruction(u8),
    InvalidInstruction(InvalidInstruction),
    EndOfInstructions,
    IncompleteInstruction(u8),
    Stack(StackError),
    Heap(HeapError)
}

impl From<io::Error> for MachineError {
    fn from(e: io::Error) -> Self {
        MachineError::IO(e)
    }
}

impl From<StackError> for MachineError {
    fn from(e: StackError) -> Self {
        MachineError::Stack(e)
    }
}

impl From<HeapError> for MachineError {
    fn from(e: HeapError) -> Self {
        MachineError::Heap(e)
    }
}

impl From<InvalidComparisonByte> for MachineError {
    fn from(e: InvalidComparisonByte) -> Self {
        InvalidInstruction::from(e).into()
    }
}

impl From<InvalidInstruction> for MachineError {
    fn from(e: InvalidInstruction) -> Self {
        MachineError::InvalidInstruction(e)
    }
}

pub struct Machine {
    pub(crate) instructions: Box<dyn ReadSeek>,
    pub(crate) stack: Box<dyn ComputeStack>,
    pub(crate) heap: Heap
}

impl Machine {
    pub fn step(&mut self) -> Result<(), (MachineError, Option<Instruction>)> {
        let instruction = Instruction::from_reader(&mut self.instructions)
            .map_err(|e| (e, None))?;
        println!("executing {:?}", instruction);
        self.execute(instruction)
            .map_err(|e| (e, Some(instruction)))?;
        Ok(())
    }

    pub fn execute(&mut self, instruction: Instruction) -> Result<(), MachineError> {
        use Instruction::*;
        match instruction {
            PSH_1(value) => self.stack.push_slice(&value)?,
            PSH_2(value) => self.stack.push_slice(&value)?,
            PSH_4(value) => self.stack.push_slice(&value)?,
            PSH_8(value) => self.stack.push_slice(&value)?,
            POP_1 => self.stack.remove_top(1)?,
            POP_2 => self.stack.remove_top(2)?,
            POP_4 => self.stack.remove_top(4)?,
            POP_8 => self.stack.remove_top(8)?,
            ALLOC => {
                let children_length = self.stack.pop_u64()?;
                let data_length = self.stack.pop_u64()?;
                let obj_ref = self.heap.allocate(children_length as usize, data_length as usize)?;
                self.stack.push_u64(obj_ref.into())?;
            }
            COPY_REF => {
                // intentionally not using Machine::stack_pop_object_reference so that I can avoid
                // having to increment the stack references twice to make up for the one popped
                let obj_ref = ObjectReference::new_result(self.stack.pop_u64()?)?;
                self.heap.increment_stack_references(&obj_ref)?;
                self.stack.push_u64(obj_ref.clone().into())?;
                self.stack.push_u64(obj_ref.into())?;
            }
            SET_CHILD => {
                let parent = self.stack_pop_object_reference()?;
                let child_index = self.stack.pop_u64()? as usize;
                let child = self.stack_pop_nullable_object_reference()?;
                self.heap.set_child(&parent, child_index, child.as_ref())?;
            }
            GET_CHILD => {
                let parent = self.stack_pop_object_reference()?;
                let child_index = self.stack.pop_u64()? as usize;
                match self.heap.get_child(&parent, child_index)? {
                    None => self.stack.push_u64(0)?,
                    Some(child) => {
                        self.heap.increment_stack_references(&child)?;
                        self.stack.push_u64(child.into())?
                    }
                }
            }
            MOV_ST_HP_1 => {
                mov_st_hp_instruction_impl!(self, 1);
            }
            MOV_ST_HP_2 => {
                mov_st_hp_instruction_impl!(self, 2);
            }
            MOV_ST_HP_4 => {
                mov_st_hp_instruction_impl!(self, 4);
            }
            MOV_ST_HP_8 => {
                mov_st_hp_instruction_impl!(self, 8);
            }
            MOV_HP_ST_1 => {
                mov_hp_st_instruction_impl!(self, 1);
            }
            MOV_HP_ST_2 => {
                mov_hp_st_instruction_impl!(self, 2);
            }
            MOV_HP_ST_4 => {
                mov_hp_st_instruction_impl!(self, 4);
            }
            MOV_HP_ST_8 => {
                mov_hp_st_instruction_impl!(self, 8);
            }
            JSR => {
                let address = self.stack.pop_u64()?;
                let next_address = self.instructions.stream_position()? + 1;
                self.stack.push_u64(next_address)?;
                self.instructions.seek(SeekFrom::Start(address))?;
            }
            RET => {
                let address = self.stack.pop_u64()?;
                self.instructions.seek(SeekFrom::Start(address))?;
            }
            JMP_EQ => {
                jump_instruction_impl!(self, is_eq);
            }
            JMP_NE => {
                jump_instruction_impl!(self, is_ne);
            }
            JMP_GE => {
                jump_instruction_impl!(self, is_ge);
            }
            JMP_GT => {
                jump_instruction_impl!(self, is_gt);
            }
            JMP_LE => {
                jump_instruction_impl!(self, is_le);
            }
            JMP_LT => {
                jump_instruction_impl!(self, is_lt);
            }
            CMP_U_1 => {
                compare_instruction_impl!(self, u8);
            }
            CMP_U_2 => {
                compare_instruction_impl!(self, u16);
            }
            CMP_U_4 => {
                compare_instruction_impl!(self, u32);
            }
            CMP_U_8 => {
                compare_instruction_impl!(self, u64);
            }
            CMP_S_1 => {
                compare_instruction_impl!(self, i8);
            }
            CMP_S_2 => {
                compare_instruction_impl!(self, i16);
            }
            CMP_S_4 => {
                compare_instruction_impl!(self, i32);
            }
            CMP_S_8 => {
                compare_instruction_impl!(self, i64);
            }
            CMP_F4 => {
                compare_instruction_impl!(self, f32);
            }
            CMP_F8 => {
                compare_instruction_impl!(self, f64);
            }
            NOT_1 => {
                not_instruction_impl!(self, u8);
            }
            NOT_2 => {
                not_instruction_impl!(self, u16);
            }
            NOT_4 => {
                not_instruction_impl!(self, u32);
            }
            NOT_8 => {
                not_instruction_impl!(self, u64);
            }
            AND_1 => {
                two_argument_instruction_impl!(self, u8, &);
            }
            AND_2 => {
                two_argument_instruction_impl!(self, u16, &);
            }
            AND_4 => {
                two_argument_instruction_impl!(self, u32, &);
            }
            AND_8 => {
                two_argument_instruction_impl!(self, u64, &);
            }
            OR_1 => {
                two_argument_instruction_impl!(self, u8, |);
            }
            OR_2 => {
                two_argument_instruction_impl!(self, u16, |);
            }
            OR_4 => {
                two_argument_instruction_impl!(self, u32, |);
            }
            OR_8 => {
                two_argument_instruction_impl!(self, u64, |);
            }
            XOR_1 => {
                two_argument_instruction_impl!(self, u8, ^);
            }
            XOR_2 => {
                two_argument_instruction_impl!(self, u16, ^);
            }
            XOR_4 => {
                two_argument_instruction_impl!(self, u32, ^);
            }
            XOR_8 => {
                two_argument_instruction_impl!(self, u64, ^);
            }
            SHL_1 => {
                shift_instruction_impl!(self, u8, <<);
            }
            SHL_2 => {
                shift_instruction_impl!(self, u16, <<);
            }
            SHL_4 => {
                shift_instruction_impl!(self, u32, <<);
            }
            SHL_8 => {
                shift_instruction_impl!(self, u64, <<);
            }
            SHR_1 => {
                shift_instruction_impl!(self, u8, >>);
            }
            SHR_2 => {
                shift_instruction_impl!(self, u16, >>);
            }
            SHR_4 => {
                shift_instruction_impl!(self, u32, >>);
            }
            SHR_8 => {
                shift_instruction_impl!(self, u64, >>);
            }
            SAR_1 => {
                shift_instruction_impl!(self, i8, >>);
            }
            SAR_2 => {
                shift_instruction_impl!(self, i16, >>);
            }
            SAR_4 => {
                shift_instruction_impl!(self, u32, >>);
            }
            SAR_8 => {
                shift_instruction_impl!(self, u64, >>);
            }
            ADD_1 => {
                two_argument_instruction_impl!(self, u8, +);
            }
            ADD_2 => {
                two_argument_instruction_impl!(self, u16, +);
            }
            ADD_4 => {
                two_argument_instruction_impl!(self, u32, +);
            }
            ADD_8 => {
                two_argument_instruction_impl!(self, u64, +);
            }
            SUB_1 => {
                two_argument_instruction_impl!(self, u8, -);
            }
            SUB_2 => {
                two_argument_instruction_impl!(self, u16, -);
            }
            SUB_4 => {
                two_argument_instruction_impl!(self, u32, -);
            }
            SUB_8 => {
                two_argument_instruction_impl!(self, u64, -);
            }
            MUL_1 => {
                two_argument_instruction_impl!(self, u8, *);
            }
            MUL_2 => {
                two_argument_instruction_impl!(self, u16, *);
            }
            MUL_4 => {
                two_argument_instruction_impl!(self, u32, *);
            }
            MUL_8 => {
                two_argument_instruction_impl!(self, u64, *);
            }
            DIV_REM_U_1 => {
                div_rem_instruction_impl!(self, u8);
            }
            DIV_REM_U_2 => {
                div_rem_instruction_impl!(self, u16);
            }
            DIV_REM_U_4 => {
                div_rem_instruction_impl!(self, u32);
            }
            DIV_REM_U_8 => {
                div_rem_instruction_impl!(self, u64);
            }
            DIV_REM_S_1 => {
                div_rem_instruction_impl!(self, i8);
            }
            DIV_REM_S_2 => {
                div_rem_instruction_impl!(self, i16);
            }
            DIV_REM_S_4 => {
                div_rem_instruction_impl!(self, i32);
            }
            DIV_REM_S_8 => {
                div_rem_instruction_impl!(self, i64);
            }
            ADD_F_4 => {
                two_argument_instruction_impl!(self, f32, +);
            }
            ADD_F_8 => {
                two_argument_instruction_impl!(self, f64, +);
            }
            SUB_F_4 => {
                two_argument_instruction_impl!(self, f32, -);
            }
            SUB_F_8 => {
                two_argument_instruction_impl!(self, f64, -);
            }
            MUL_F_4 => {
                two_argument_instruction_impl!(self, f32, *);
            }
            MUL_F_8 => {
                two_argument_instruction_impl!(self, f64, *);
            }
            DIV_F_4 => {
                two_argument_instruction_impl!(self, f32, /);
            }
            DIV_F_8 => {
                two_argument_instruction_impl!(self, f64, /);
            }
            REM_F_4 => {
                two_argument_instruction_impl!(self, f64, %);
            }
            REM_F_8 => {
                two_argument_instruction_impl!(self, f64, %);
            }
            CNV_U8_F4 => {
                convert_instruction_impl!(self, u64 -> f32);
            }
            CNV_U8_F8 => {
                convert_instruction_impl!(self, u64 -> f64);
            }
            CNV_S8_F4 => {
                convert_instruction_impl!(self, i64 -> f32);
            }
            CNV_S8_F8 => {
                convert_instruction_impl!(self, i64 -> f64);
            }
            CNV_F4_U8 => {
                convert_instruction_impl!(self, f32 -> u64);
            }
            CNV_F8_U8 => {
                convert_instruction_impl!(self, f64 -> u64);
            }
            CNV_F4_S8 => {
                convert_instruction_impl!(self, f32 -> i64);
            }
            CNV_F8_S8 => {
                convert_instruction_impl!(self, f64 -> i64);
            }
            CNV_F4_F8 => {
                convert_instruction_impl!(self, f32 -> f64);
            }
            CNV_F8_F4 => {
                convert_instruction_impl!(self, f64 -> f32);
            }
            CALL_EXT => {
                todo!()
            }
        }
        Ok(())
    }

    fn stack_pop_nullable_object_reference(&mut self) -> Result<Option<ObjectReference>, MachineError> {
        match ObjectReference::new_option(self.stack.pop_u64()?) {
            None => Ok(None),
            Some(obj_ref) => {
                self.heap.decrement_stack_references(obj_ref.clone())?;
                Ok(Some(obj_ref))
            }
        }
    }

    fn stack_pop_object_reference(&mut self) -> Result<ObjectReference, MachineError> {
        let obj_ref = ObjectReference::new_result(self.stack.pop_u64()?)?;
        self.heap.decrement_stack_references(obj_ref.clone())?;
        Ok(obj_ref)
    }
}
