use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

pub trait Number:
    Copy
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Mul<Output = Self>
    + MulAssign
    + Div<Output = Self>
    + DivAssign
    + alloc::fmt::Debug
    + alloc::fmt::Display
{
    fn ten_power(p: u32) -> Option<Self>;
    fn checked_add(self, rhs: Self) -> Option<Self>;
    fn checked_sub(self, rhs: Self) -> Option<Self>;
    fn checked_mul(self, rhs: Self) -> Option<Self>;
    fn checked_div(self, rhs: Self) -> Option<Self>;
    fn checked_rem(self, rhs: Self) -> Option<Self>;
}

macro_rules! number_impl {
    ($ty:ty, $($tt:tt)+) => {
        impl Number for $ty {
            fn ten_power(p: u32) -> Option<Self> {
                let (r, overflowed) = <$ty>::overflowing_pow(10, p);
                (!overflowed).then_some(r)
            }
            fn checked_add(self, rhs: $ty) -> Option<$ty> {
                self.checked_add(rhs)
            }
            fn checked_sub(self, rhs: $ty) -> Option<$ty> {
                self.checked_sub(rhs)
            }
            fn checked_mul(self, rhs: $ty) -> Option<$ty> {
                self.checked_mul(rhs)
            }
            fn checked_div(self, rhs: $ty) -> Option<$ty> {
                self.checked_div(rhs)
            }
            $($tt)+
        }
    };
}

macro_rules! number_unsigned_impl {
    ($ty:ty) => {
        number_impl!(
            $ty,
            fn checked_rem(self, rhs: $ty) -> Option<$ty> {
                self.checked_rem(rhs)
            }
        );
    };
}
macro_rules! number_signed_impl {
    ($ty:ty) => {
        number_impl!(
            $ty,
            fn checked_rem(self, rhs: $ty) -> Option<$ty> {
                self.abs().checked_rem(rhs)
            }
        );
    };
}

number_unsigned_impl!(u8);
number_unsigned_impl!(u16);
number_unsigned_impl!(u32);
number_unsigned_impl!(u64);
number_unsigned_impl!(u128);
number_signed_impl!(i8);
number_signed_impl!(i16);
number_signed_impl!(i32);
number_signed_impl!(i64);
number_signed_impl!(i128);
