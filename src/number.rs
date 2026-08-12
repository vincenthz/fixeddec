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

    /// Divide the double precision value made of the `(low, high)` halves by `divisor`, truncating
    ///
    /// This is the counterpart of [`Number::widening_mul`], and undoes it: scaling a product back
    /// down never has to give up on an intermediate that left the range of a single value.
    ///
    /// Returns None if the quotient does not fit in a single value, or if the divisor is zero.
    ///
    fn widening_div(low: Self, high: Self, divisor: Self) -> Option<Self>;

    /// Euler's number at [`Number::E_PRECISION`]
    const E: Self;

    /// The reciprocal of Euler's number, at the last precision of [`Number::TEN_POWER`]
    ///
    /// Being below one it needs no integral digit, and so has room for a digit more than
    /// [`Number::E`] on the types where the two precisions differ.
    ///
    const E_RECIP: Self;

    /// The precision [`Number::E`] is given at
    ///
    /// This is the highest precision at which this type can still hold `e`, which is one digit
    /// short of its maximum precision when the integral digit does not fit alongside. Series that
    /// converge to a value around one are evaluated here, leaving the digits between this
    /// precision and the requested one as guard digits.
    ///
    const E_PRECISION: u32;

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
    ($ty:ty, $ity:ty, e($eprec:expr, $e:expr, $erecip:expr), $power10:expr, $($tt:tt)+) => {
        impl Number for $ty {
            const MIN : $ty = <$ty>::MIN;
            const MAX : $ty = <$ty>::MAX;
            const ZERO : $ty = 0;
            const ONE : $ty = 1;
            const TEN_POWER : &'static [$ty] = &$power10;
            const E : $ty = $e;
            const E_RECIP : $ty = $erecip;
            const E_PRECISION : u32 = $eprec;
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
    ($ty:ty, $ity:ty, wide $wty:ty, e($eprec:expr, $e:expr, $erecip:expr), $power10:expr) => {
        number_unsigned_impl!(
            $ty,
            $ity,
            e($eprec, $e, $erecip),
            $power10,
            fn widening_mul(self, rhs: $ty) -> ($ty, $ty) {
                let product = (self as $wty) * (rhs as $wty);
                (product as $ty, (product >> <$ty>::BITS) as $ty)
            }
            fn widening_div(low: $ty, high: $ty, divisor: $ty) -> Option<$ty> {
                if divisor == 0 {
                    return None;
                }
                let dividend = ((high as $wty) << <$ty>::BITS) | (low as $wty);
                let quotient = dividend / (divisor as $wty);
                if quotient > <$ty>::MAX as $wty {
                    None
                } else {
                    Some(quotient as $ty)
                }
            }
        );
    };
    // no primitive is twice as wide as $ty, so the product is assembled from $half sized limbs
    ($ty:ty, $ity:ty, halves $half:ty, e($eprec:expr, $e:expr, $erecip:expr), $power10:expr) => {
        number_unsigned_impl!(
            $ty,
            $ity,
            e($eprec, $e, $erecip),
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
            fn widening_div(low: $ty, high: $ty, divisor: $ty) -> Option<$ty> {
                // a high half at or above the divisor already divides into more than one value
                if divisor == 0 || high >= divisor {
                    return None;
                }
                // schoolbook long division taking one bit of the low half at a time. the
                // remainder is kept below the divisor, so doubling it can only reach one bit
                // past the type, which is what testing the top bit beforehand recovers.
                let mut remainder = high;
                let mut quotient: $ty = 0;
                for bit in (0..<$ty>::BITS).rev() {
                    let overshoot = remainder >> (<$ty>::BITS - 1) != 0;
                    remainder = (remainder << 1) | ((low >> bit) & 1);
                    if overshoot || remainder >= divisor {
                        remainder = remainder.wrapping_sub(divisor);
                        quotient |= 1 << bit;
                    }
                }
                Some(quotient)
            }
        );
    };
    ($ty:ty, $ity:ty, e($eprec:expr, $e:expr, $erecip:expr), $power10:expr, $($tt:tt)+) => {
        number_impl!(
            $ty,
            $ity,
            e($eprec, $e, $erecip),
            $power10,
            fn checked_rem(self, rhs: $ty) -> Option<$ty> {
                self.checked_rem(rhs)
            }
            $($tt)+
        );
    };
}
number_unsigned_impl!(u8, i8, wide u16, e(1, 27, 36), [1, 10, 100]);
number_unsigned_impl!(u16, i16, wide u32, e(4, 27_182, 3_678), [1, 10, 100, 1000, 10000]);
number_unsigned_impl!(
    u32,
    i32,
    wide u64,
    e(9, 2_718_281_828, 367_879_441),
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
    e(
        18,
        2_718_281_828_459_045_235,
        3_678_794_411_714_423_215
    ),
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
    e(
        38,
        2_718_281_828_459_045_235_360_287_471_352_662_497_75,
        36_787_944_117_144_232_159_552_377_016_146_086_744
    ),
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

    // (a, b, high half, low half) of the product: widening_mul returns the halves the other
    // way round, and widening_div takes them apart again
    const U8: &[(u8, u8, u8, u8)] = &[(255, 255, 254, 1), (200, 3, 2, 88), (10, 10, 0, 100)];

    const U16: &[(u16, u16, u16, u16)] = &[(65535, 65535, 65534, 1), (1000, 1000, 15, 16960)];

    const U64: &[(u64, u64, u64, u64)] = &[
        (
            18446744073709551615,
            18446744073709551615,
            18446744073709551614,
            1,
        ),
        (4294967296, 4294967296, 1, 0),
        (
            10000000000000000000,
            18446744073709551615,
            9999999999999999999,
            8446744073709551616,
        ),
    ];

    const U128: &[(u128, u128, u128, u128)] = &[
        (0, 340282366920938463463374607431768211455, 0, 0),
        (
            1,
            340282366920938463463374607431768211455,
            0,
            340282366920938463463374607431768211455,
        ),
        (
            18446744073709551615,
            18446744073709551617,
            0,
            340282366920938463463374607431768211455,
        ),
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

    #[test]
    fn widening_mul() {
        for (a, b, high, low) in U8 {
            assert_eq!(Number::widening_mul(*a, *b), (*low, *high), "{a} * {b}");
        }
        for (a, b, high, low) in U16 {
            assert_eq!(Number::widening_mul(*a, *b), (*low, *high), "{a} * {b}");
        }
        for (a, b, high, low) in U64 {
            assert_eq!(Number::widening_mul(*a, *b), (*low, *high), "{a} * {b}");
        }
        // u128 has no wider primitive to lean on, so its limbs and both of its carries are
        // exercised here: squaring the maximum carries out of the middle limbs, and the case
        // just above it carries out of the low one
        for (a, b, high, low) in U128 {
            assert_eq!(Number::widening_mul(*a, *b), (*low, *high), "{a} * {b}");
            // the halves are symmetric in the operands
            assert_eq!(Number::widening_mul(*b, *a), (*low, *high), "{b} * {a}");
            // and the low half is the wrapping product
            assert_eq!(a.wrapping_mul(*b), *low, "{a} * {b}");
        }
    }

    #[test]
    fn widening_div() {
        // dividing a product by one of its factors gives the other one back, whatever the
        // magnitude of the intermediate
        macro_rules! roundtrip {
            ($vectors:expr) => {
                for (a, b, high, low) in $vectors {
                    if *b != 0 {
                        assert_eq!(
                            Number::widening_div(*low, *high, *b),
                            Some(*a),
                            "({high},{low}) / {b}"
                        );
                    }
                    if *a != 0 {
                        assert_eq!(
                            Number::widening_div(*low, *high, *a),
                            Some(*b),
                            "({high},{low}) / {a}"
                        );
                    }
                }
            };
        }
        roundtrip!(U8);
        roundtrip!(U16);
        roundtrip!(U64);
        roundtrip!(U128);

        // digits below the quotient are truncated, not rounded
        assert_eq!(Number::widening_div(9u8, 0, 10), Some(0));
        assert_eq!(Number::widening_div(199u8, 0, 100), Some(1));
        assert_eq!(
            Number::widening_div(0u128, 1, 3),
            Some(113427455640312821154458202477256070485)
        );

        // a quotient of its own needs to fit in a single value
        assert_eq!(Number::widening_div(0u8, 1, 1), None);
        assert_eq!(Number::widening_div(0u8, 1, 2), Some(128));
        // 255 * 65536 + 255 divides into 65537, one past the type, but into 65280 by 256
        assert_eq!(Number::widening_div(255u16, 255, 255), None);
        assert_eq!(Number::widening_div(255u16, 255, 256), Some(65280));
        assert_eq!(Number::widening_div(0u64, u64::MAX, u64::MAX), None);
        assert_eq!(
            Number::widening_div(u64::MAX, u64::MAX - 1, u64::MAX),
            Some(u64::MAX)
        );
        assert_eq!(Number::widening_div(0u128, u128::MAX, u128::MAX), None);

        // and a zero divisor has no quotient at all
        assert_eq!(Number::widening_div(1u8, 0, 0), None);
        assert_eq!(Number::widening_div(1u128, 0, 0), None);
    }
}
