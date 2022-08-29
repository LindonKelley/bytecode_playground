use std::cmp::Ordering;

/// `PossiblyOrdering` is semantically equivalent to [`Option`]<[`Ordering`]>, with some nicer
/// functional uses such as being able to call [`is_eq`](PossiblyOrdering::is_eq) on instances
/// of this type without necessitating a check for a `None`
///
/// `PossiblyOrdering` also defines the following discriminants for it's variants:\
/// 0 = Unordered\
/// 1 = Less\
/// 2 = Equal\
/// 3 = Greater
/// 
/// these are used for converting an instance into a byte, or a (valid) byte into an
/// instance of `PossiblyOrdering`
#[repr(u8)]
pub enum PossiblyOrdering {
    Unordered = 0,
    Less = 1,
    Equal = 2,
    Greater = 3
}

impl PossiblyOrdering {
    //noinspection RsSelfConvention
    pub const fn is_eq(self) -> bool {
        matches!(self, PossiblyOrdering::Equal)
    }

    //noinspection RsSelfConvention
    pub const fn is_ne(self) -> bool {
        !matches!(self, PossiblyOrdering::Equal)
    }

    //noinspection RsSelfConvention
    pub const fn is_lt(self) -> bool {
        matches!(self, PossiblyOrdering::Less)
    }

    //noinspection RsSelfConvention
    pub const fn is_gt(self) -> bool {
        matches!(self, PossiblyOrdering::Greater)
    }

    //noinspection RsSelfConvention
    pub const fn is_le(self) -> bool {
        !matches!(self, PossiblyOrdering::Greater)
    }

    //noinspection RsSelfConvention
    pub const fn is_ge(self) -> bool {
        !matches!(self, PossiblyOrdering::Less)
    }
}

impl From<Option<Ordering>> for PossiblyOrdering {
    fn from(ord: Option<Ordering>) -> Self {
        match ord {
            None => PossiblyOrdering::Unordered,
            Some(Ordering::Less) => PossiblyOrdering::Less,
            Some(Ordering::Equal) => PossiblyOrdering::Equal,
            Some(Ordering::Greater) => PossiblyOrdering::Greater
        }
    }
}

impl From<Ordering> for PossiblyOrdering {
    fn from(ord: Ordering) -> Self {
        match ord {
            Ordering::Less => PossiblyOrdering::Less,
            Ordering::Equal => PossiblyOrdering::Equal,
            Ordering::Greater => PossiblyOrdering::Greater
        }
    }
}

#[derive(Debug)]
pub struct InvalidComparisonByte(pub u8);

impl TryFrom<u8> for PossiblyOrdering {
    type Error = InvalidComparisonByte;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PossiblyOrdering::Unordered),
            1 => Ok(PossiblyOrdering::Less),
            2 => Ok(PossiblyOrdering::Equal),
            3 => Ok(PossiblyOrdering::Greater),
            v => Err(InvalidComparisonByte(v))
        }
    }
}
