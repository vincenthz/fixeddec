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
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + alloc::fmt::Debug
    + alloc::fmt::Display
    + 'static
{
    const TEN_POWER: &'static [Self];
    const MIN: Self;
    const MAX: Self;
    const ZERO: Self;
    const ONE: Self;

    type Signed;

    fn checked_add(self, rhs: Self) -> Option<Self>;
    fn checked_sub(self, rhs: Self) -> Option<Self>;
    fn checked_mul(self, rhs: Self) -> Option<Self>;
    fn checked_div(self, rhs: Self) -> Option<Self>;
    fn checked_rem(self, rhs: Self) -> Option<Self>;
    fn isqrt(self) -> Self;

    fn from_digit10(c: char) -> Option<Self>;
}

pub(crate) const fn ten<T: Number>() -> T {
    ten_power_const::<T, 1>()
}

pub(crate) const fn ten_power_const<T: Number, const P: u32>() -> T {
    const { assert!(T::TEN_POWER.len() > P as usize) };
    T::TEN_POWER[P as usize]
}

pub(crate) const fn ten_power<T: Number>(p: u32) -> Option<T> {
    if T::TEN_POWER.len() > p as usize {
        Some(T::TEN_POWER[p as usize])
    } else {
        None
    }
}

macro_rules! number_impl {
    ($ty:ty, $ity:ty, $power10:expr, $($tt:tt)+) => {
        impl Number for $ty {
            const MIN : $ty = <$ty>::MIN;
            const MAX : $ty = <$ty>::MAX;
            const ZERO : $ty = 0;
            const ONE : $ty = 1;
            const TEN_POWER : &'static [$ty] = &$power10;
            type Signed = $ity;

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
            fn from_digit10(c: char) -> Option<$ty> {
                // all rust integral type can represent number between 0-9
                c.to_digit(10).map(|i| i as $ty)
            }
            fn isqrt(self) -> Self {
                self.isqrt()
            }

            $($tt)+
        }
    };
}

macro_rules! number_unsigned_impl {
    ($ty:ty,$ity:ty,$power10:expr) => {
        number_impl!(
            $ty,
            $ity,
            $power10,
            fn checked_rem(self, rhs: $ty) -> Option<$ty> {
                self.checked_rem(rhs)
            }
        );
    };
}
number_unsigned_impl!(u8, i8, [1, 10, 100]);
number_unsigned_impl!(u16, i16, [1, 10, 100, 1000, 10000]);
number_unsigned_impl!(
    u32,
    i32,
    [
        1,
        10,
        100,
        1_000,
        10_000,
        100_000,
        1_000_000,
        10_000_000,
        100_000_000,
        1_000_000_000,
    ]
);
number_unsigned_impl!(
    u64,
    i64,
    [
        1,
        10,
        100,
        1_000,
        10_000,
        100_000,
        1_000_000,
        10_000_000,
        100_000_000,
        1_000_000_000,
        10_000_000_000,
        100_000_000_000,
        1_000_000_000_000,
        10_000_000_000_000,
        100_000_000_000_000,
        1_000_000_000_000_000,
        10_000_000_000_000_000,
        100_000_000_000_000_000,
        1_000_000_000_000_000_000,
        10_000_000_000_000_000_000,
    ]
);
number_unsigned_impl!(
    u128,
    i128,
    [
        1,
        10,
        100,
        1_000,
        10_000,
        100_000,
        1_000_000,
        10_000_000,
        100_000_000,
        1_000_000_000,
        10_000_000_000,
        100_000_000_000,
        1_000_000_000_000,
        10_000_000_000_000,
        100_000_000_000_000,
        1_000_000_000_000_000,
        10_000_000_000_000_000,
        100_000_000_000_000_000,
        1_000_000_000_000_000_000,
        10_000_000_000_000_000_000,
        100_000_000_000_000_000_000,
        1_000_000_000_000_000_000_000,
        10_000_000_000_000_000_000_000,
        100_000_000_000_000_000_000_000,
        1_000_000_000_000_000_000_000_000,
        10_000_000_000_000_000_000_000_000,
        100_000_000_000_000_000_000_000_000,
        1_000_000_000_000_000_000_000_000_000,
        10_000_000_000_000_000_000_000_000_000,
        100_000_000_000_000_000_000_000_000_000,
        1_000_000_000_000_000_000_000_000_000_000,
        10_000_000_000_000_000_000_000_000_000_000,
        100_000_000_000_000_000_000_000_000_000_000,
        1_000_000_000_000_000_000_000_000_000_000_000,
        10_000_000_000_000_000_000_000_000_000_000_000,
        100_000_000_000_000_000_000_000_000_000_000_000,
        1_000_000_000_000_000_000_000_000_000_000_000_000,
        10_000_000_000_000_000_000_000_000_000_000_000_000,
        100_000_000_000_000_000_000_000_000_000_000_000_000,
    ]
);
