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

    /// Multiply two values at double precision, as the `(low, high)` halves of the product
    ///
    /// Unlike [`Number::checked_mul`] this never loses anything: the product of two values of
    /// this type always fits in two of them. The halves are returned low first, as the equivalent
    /// standard library method (still unstable) does.
    ///
    fn widening_mul(self, rhs: Self) -> (Self, Self);

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
    // $wty is the primitive twice as wide as $ty, which can hold the whole product
    ($ty:ty, $ity:ty, wide $wty:ty, $power10:expr) => {
        number_unsigned_impl!(
            $ty,
            $ity,
            $power10,
            fn widening_mul(self, rhs: $ty) -> ($ty, $ty) {
                let product = (self as $wty) * (rhs as $wty);
                (product as $ty, (product >> <$ty>::BITS) as $ty)
            }
        );
    };
    // no primitive is twice as wide as $ty, so the product is assembled from $half sized limbs
    ($ty:ty, $ity:ty, halves $half:ty, $power10:expr) => {
        number_unsigned_impl!(
            $ty,
            $ity,
            $power10,
            fn widening_mul(self, rhs: $ty) -> ($ty, $ty) {
                const HALF: u32 = <$half>::BITS;
                let mask = <$half>::MAX as $ty;

                let (a_high, a_low) = (self >> HALF, self & mask);
                let (b_high, b_low) = (rhs >> HALF, rhs & mask);

                // the two middle limbs carry into the 3rd limb, and the low half of their sum
                // carries into the 2nd one
                let (middle, middle_carry) = (a_high * b_low).overflowing_add(a_low * b_high);
                let (low, low_carry) = (a_low * b_low).overflowing_add(middle << HALF);
                let high = a_high * b_high
                    + (middle >> HALF)
                    + ((middle_carry as $ty) << HALF)
                    + low_carry as $ty;
                (low, high)
            }
        );
    };
    ($ty:ty, $ity:ty, $power10:expr, $($tt:tt)+) => {
        number_impl!(
            $ty,
            $ity,
            $power10,
            fn checked_rem(self, rhs: $ty) -> Option<$ty> {
                self.checked_rem(rhs)
            }
            $($tt)+
        );
    };
}
number_unsigned_impl!(u8, i8, wide u16, [1, 10, 100]);
number_unsigned_impl!(u16, i16, wide u32, [1, 10, 100, 1000, 10000]);
number_unsigned_impl!(
    u32,
    i32,
    wide u64,
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
    wide u128,
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
    halves u64,
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

#[cfg(test)]
mod tests {
    use super::Number;

    #[test]
    fn widening_mul() {
        // (a, b, high half, low half): the call returns them the other way round
        const U8: &[(u8, u8, u8, u8)] = &[(255, 255, 254, 1), (200, 3, 2, 88), (10, 10, 0, 100)];
        for (a, b, high, low) in U8 {
            assert_eq!(Number::widening_mul(*a, *b), (*low, *high), "{a} * {b}");
        }

        const U16: &[(u16, u16, u16, u16)] = &[(65535, 65535, 65534, 1), (1000, 1000, 15, 16960)];
        for (a, b, high, low) in U16 {
            assert_eq!(Number::widening_mul(*a, *b), (*low, *high), "{a} * {b}");
        }

        const U64: &[(u64, u64, u64, u64)] = &[
            (18446744073709551615, 18446744073709551615, 18446744073709551614, 1),
            (4294967296, 4294967296, 1, 0),
            (
                10000000000000000000,
                18446744073709551615,
                9999999999999999999,
                8446744073709551616,
            ),
        ];
        for (a, b, high, low) in U64 {
            assert_eq!(Number::widening_mul(*a, *b), (*low, *high), "{a} * {b}");
        }

        // u128 has no wider primitive to lean on, so its limbs and both of its carries are
        // exercised here: squaring the maximum carries out of the middle limbs, and the third
        // case carries out of the low one
        const U128: &[(u128, u128, u128, u128)] = &[
            (0, 340282366920938463463374607431768211455, 0, 0),
            (1, 340282366920938463463374607431768211455, 0, 340282366920938463463374607431768211455),
            (18446744073709551615, 18446744073709551617, 0, 340282366920938463463374607431768211455),
            (18446744073709551616, 18446744073709551616, 1, 0),
            (
                170141183460469231731687303715884105728,
                170141183460469231731687303715884105728,
                85070591730234615865843651857942052864,
                0,
            ),
            (
                100000000000000000000000000000000000000,
                340282366920938463463374607431768211455,
                99999999999999999999999999999999999999,
                240282366920938463463374607431768211456,
            ),
            (
                340282366920938463463374607431768211455,
                340282366920938463463374607431768211455,
                340282366920938463463374607431768211454,
                1,
            ),
            (
                134351473349708985578604307872871789624,
                279465158934149700935886463558486303824,
                110339410744245830735791995057468488394,
                209751400881061728190397052390600880512,
            ),
            (
                21050564189233570673411765442743072437,
                16874368708971697930179439133948970191,
                1043883010674879136228496194239969333,
                172330358388033087645277110957767446619,
            ),
        ];
        for (a, b, high, low) in U128 {
            assert_eq!(Number::widening_mul(*a, *b), (*low, *high), "{a} * {b}");
            // the halves are symmetric in the operands
            assert_eq!(Number::widening_mul(*b, *a), (*low, *high), "{b} * {a}");
            // and the low half is the wrapping product
            assert_eq!(a.wrapping_mul(*b), *low, "{a} * {b}");
        }
    }
}
