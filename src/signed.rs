use crate::{number::Number, unsigned::FixedDec};
use core::{
    cmp::Ordering,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Sign {
    Negative,
    Positive,
}

/*
impl Sign {
    pub fn multiply(self, other: Self) -> Self {
        // TODO replace by some xor
        match (self, other) {
            (Sign::Negative, Sign::Negative) => Self::Positive,
            (Sign::Negative, Sign::Positive) => Self::Negative,
            (Sign::Positive, Sign::Negative) => Self::Negative,
            (Sign::Positive, Sign::Positive) => Self::Positive,
        }
    }
}
*/

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedSDec<T: Number, const P: u32> {
    value: FixedDec<T, P>,
    sign: Sign,
}

impl<T: Number, const P: u32> FixedSDec<T, P> {
    /// Minimum value representable by this type
    pub const MIN: Self = FixedSDec {
        value: FixedDec::MAX,
        sign: Sign::Negative,
    };

    /// Maximum value representable by this type
    pub const MAX: Self = FixedSDec {
        value: FixedDec::MAX,
        sign: Sign::Positive,
    };

    /// Returns true if self is positive and false if the number is zero or negative.
    pub fn is_positive(self) -> bool {
        self.value.0 != T::ZERO && self.sign == Sign::Positive
    }

    /// Returns true if self is negative and false if the number is zero or negative.
    pub fn is_negative(self) -> bool {
        self.value.0 != T::ZERO && self.sign == Sign::Negative
    }
}

impl<T: Number, const P: u32> alloc::fmt::Debug for FixedSDec<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_negative() {
            write!(f, "-{:?}", self.value)
        } else {
            write!(f, "{:?}", self.value)
        }
    }
}

impl<T: Number, const P: u32> alloc::fmt::Display for FixedSDec<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.is_negative() {
            write!(f, "-{:?}", self.value)
        } else {
            write!(f, "{:?}", self.value)
        }
    }
}

impl<T: Number, const P: u32> Add for FixedSDec<T, P> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.sign == rhs.sign {
            Self {
                value: self.value + rhs.value,
                sign: self.sign,
            }
        } else {
            self - rhs
        }
    }
}

impl<T: Number, const P: u32> AddAssign for FixedSDec<T, P> {
    fn add_assign(&mut self, rhs: Self) {
        let r = *self + rhs;
        self.value = r.value;
        self.sign = r.sign;
    }
}

impl<T: Number, const P: u32> Sub for FixedSDec<T, P> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.sign == rhs.sign {
            match self.value.cmp(&rhs.value) {
                Ordering::Less => Self {
                    value: rhs.value - self.value,
                    sign: rhs.sign,
                },
                Ordering::Equal => Self {
                    value: FixedDec::new(T::ZERO),
                    sign: Sign::Positive,
                },
                Ordering::Greater => Self {
                    value: self.value - rhs.value,
                    sign: self.sign,
                },
            }
        } else {
            let sign = if self.value > rhs.value {
                self.sign
            } else {
                rhs.sign
            };
            Self {
                value: self.value + rhs.value,
                sign,
            }
        }
    }
}

impl<T: Number, const P: u32> SubAssign for FixedSDec<T, P> {
    fn sub_assign(&mut self, rhs: Self) {
        let r = *self - rhs;
        self.value = r.value;
        self.sign = r.sign;
    }
}

impl<T: Number, const P: u32> Mul<T> for FixedSDec<T, P> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let value = self.value * rhs;
        Self {
            sign: self.sign,
            value,
        }
    }
}

impl<T: Number, const P: u32> MulAssign<T> for FixedSDec<T, P> {
    fn mul_assign(&mut self, rhs: T) {
        let r = *self * rhs;
        self.value = r.value;
        self.sign = r.sign;
    }
}

impl<T: Number, const P: u32> Div<T> for FixedSDec<T, P> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self {
            value: self.value / rhs,
            sign: self.sign,
        }
    }
}

impl<T: Number, const P: u32> DivAssign<T> for FixedSDec<T, P> {
    fn div_assign(&mut self, rhs: T) {
        let r = *self / rhs;
        self.value = r.value;
        self.sign = r.sign;
    }
}
