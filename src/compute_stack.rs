use std::fmt::Debug;

macro_rules! push_impl {
    ($name: ident, $typ: ty) => {
        #[inline(always)]
        fn $name(&mut self, value: $typ) -> StackResult<()> {
            self.push_slice(&value.to_le_bytes())
        }
    };
    ($(($name: ident, $typ: ty)),+) => {
        $(push_impl!($name, $typ);)+
    };
}

macro_rules! pop_impl {
    ($name: ident, $typ: ty, $bytes: literal) => {
        #[inline(always)]
        fn $name(&mut self) -> StackResult<$typ> {
            Ok(<$typ>::from_le_bytes((&*self.pop_slice($bytes)?).try_into().unwrap()))
        }
    };
    ($(($name: ident, $typ: ty, $bytes: literal)),+) => {
        $(pop_impl!($name, $typ, $bytes);)+
    };
}

pub trait ComputeStack: Debug {
    fn size(&self) -> usize;

    fn push_slice(&mut self, slice: &[u8]) -> StackResult<()>;

    push_impl! {
        (push_u8, u8), (push_u16, u16), (push_u32, u32), (push_u64, u64),
        (push_i8, i8), (push_i16, i16), (push_i32, i32), (push_i64, i64),
        (push_f32, f32), (push_f64, f64)
    }

    fn pop_slice(&mut self, length: usize) -> StackResult<Box<[u8]>>;

    pop_impl! {
        (pop_u8, u8, 1), (pop_u16, u16, 2), (pop_u32, u32, 4), (pop_u64, u64, 8),
        (pop_i8, i8, 1), (pop_i16, i16, 2), (pop_i32, i32, 4), (pop_i64, i64, 8),
        (pop_f32, f32, 4), (pop_f64, f64, 8)
    }

    fn remove_top(&mut self, length: usize) -> StackResult<()>;
}

impl ComputeStack for Vec<u8> {
    #[inline(always)]
    fn size(&self) -> usize {
        self.len()
    }

    fn push_slice(&mut self, slice: &[u8]) -> StackResult<()> {
        self.extend_from_slice(slice);
        Ok(())
    }

    // overriding since this is presumably faster than the default implementation
    fn push_u8(&mut self, value: u8) -> StackResult<()> {
        self.push(value);
        Ok(())
    }

    fn pop_slice(&mut self, length: usize) -> StackResult<Box<[u8]>> {
        let len = self.len();
        if len >= length {
            let tail = self.split_off(len - length);
            Ok(tail.into())
        } else {
            Err(StackError::Underflow)
        }
    }

    // overriding since this is presumably faster than the default implementation
    fn pop_u8(&mut self) -> StackResult<u8> {
        self.pop().ok_or(StackError::Underflow)
    }

    fn remove_top(&mut self, length: usize) -> StackResult<()> {
        let len = self.len();
        if len >= length {
            self.drain((len - length)..len);
            Ok(())
        } else {
            Err(StackError::Underflow)
        }
    }
}

#[derive(Debug)]
pub enum StackError {
    Underflow,
    Overflow
}

pub type StackResult<T> = Result<T, StackError>;
