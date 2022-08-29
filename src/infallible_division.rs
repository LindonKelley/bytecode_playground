pub trait InfallibleDivision {
    fn infallible_div(a: Self, b: Self) -> Self;

    fn infallible_rem(a: Self, b: Self) -> Self;
}

macro_rules! impl_unchecked_division {
    ($t: ty) => {
        impl InfallibleDivision for $t {
            fn infallible_div(a: Self, b: Self) -> Self {
                if b != 0 {
                    a / b
                } else {
                    <$t>::MAX
                }
            }
        
            fn infallible_rem(a: Self, b: Self) -> Self {
                if b != 0 {
                    a % b
                } else {
                    a
                }
            }
        }
    };
    ($($n: ty),+) => {
        $(impl_unchecked_division!($n);)+
    };
}

impl_unchecked_division!{
    u8, u16, u32, u64,
    i8, i16, i32, i64
}

