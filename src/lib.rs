//! A fixed-point decimal number type with generic precision.
//!
//! This module provides the `FixedDec<T, P>` type, a lightweight wrapper around
//! integral numbers that interprets them as fixed-point decimal values with `P` fractional digits.
//!
//! ## Overview
//!
//! `FixedDec` allows representing decimal numbers without using floating-point arithmetic,
//! which is useful in financial or deterministic computation contexts.
//!
//! The `P` const generic parameter defines the number of decimal places. For example:
//!
//! - `FixedDec(123, 0)` represents the integer `123`
//! - `FixedDec(123, 3)` represents the decimal `0.123`
//!
//! Internally, the value is stored as a raw integer of type `T`, and the decimal point is
//! applied logically according to the value of `P`.
//!
//! ## Type Parameters
//!
//! - `T`: The underlying integer type which is currently limited to rust builtin integer types (e.g., `i32`, `u64`)
//! - `P`: A compile-time constant specifying the number of fractional decimal digits.
//!
//! ## Use Cases
//!
//! This type is useful when you need:
//!
//! - Precise decimal arithmetic (e.g., for currencies or measurements).
//! - Consistent and deterministic behavior across platforms (unlike floats).
//! - Compile-time control over precision.
//!
//! ## Example
//!
//! ```rust
//! use fixeddec::FixedDec;
//!
//! let a: FixedDec<i32, 2> = FixedDec::new(12345);
//! assert_eq!(a.to_string(), "123.45");
//! ```
//!
#![no_std]

use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

extern crate alloc;

mod number;

use number::Number;

/// A integral number with a precision of fractional digits
///
/// At P=0, it is a normal integer with no fractional part
///
/// * `FixedDec(123, 0)` represent 123
/// * `FixedDec(123, 3)` = 0.123
///
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct FixedDec<T: Number, const P: u32>(T);

impl<T: Number, const P: u32> alloc::fmt::Debug for FixedDec<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}.{:0width$}",
            self.integral(),
            self.fractional(),
            width = P as usize
        )
    }
}

impl<T: Number, const P: u32> alloc::fmt::Display for FixedDec<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<T: Number, const P: u32> FixedDec<T, P> {
    /// Create a new FixedDec using the backing value already at the required precision
    ///
    /// ```
    /// use fixeddec::FixedDec;
    /// let f = FixedDec::<u32, 3>::new(1_234);
    /// ```
    pub fn new(t: T) -> Self {
        assert!(T::ten_power(P).is_some());
        Self(t)
    }

    /// Create a new FixedDec using the backing value as just the integral part
    ///
    /// ```
    /// use fixeddec::FixedDec;
    /// let f = FixedDec::<u32, 3>::from_integral(1_234).unwrap();
    /// assert_eq!(f.value(), 1_234_000);
    /// ```
    ///
    /// If the value represented with the fractional part overflow the backing part, returns None
    pub fn from_integral(t: T) -> Option<Self> {
        T::ten_power(P).and_then(|prec| t.checked_mul(prec).map(Self))
    }

    /// Try to change the precision of the value without changing the represented value
    pub fn set_precision<const O: u32>(self) -> Option<FixedDec<T, O>> {
        use core::cmp::Ordering;
        match P.cmp(&O) {
            Ordering::Equal => Some(FixedDec(self.0)),
            Ordering::Less => {
                let diff = P - O;
                T::ten_power(diff).and_then(|prec| self.0.checked_mul(prec).map(FixedDec))
            }
            Ordering::Greater => {
                let diff = O - P;
                T::ten_power(diff).and_then(|prec| self.0.checked_div(prec).map(FixedDec))
            }
        }
    }

    /// Try to convert the backing type of `FixedDec` from `T` to `U`
    pub fn try_into<U: Number>(self) -> Result<FixedDec<U, P>, <U as TryFrom<T>>::Error>
    where
        U: TryFrom<T>,
    {
        U::try_from(self.0).map(FixedDec)
    }

    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    pub fn checked_mul(self, rhs: T) -> Option<Self> {
        self.0.checked_mul(rhs).map(Self)
    }

    pub fn checked_div(self, rhs: T) -> Option<Self> {
        self.0.checked_div(rhs).map(Self)
    }

    pub fn checked_rem(self, rhs: T) -> Option<Self> {
        self.0.checked_rem(rhs).map(Self)
    }

    /// Round at a specific precision
    pub fn round_at(self, prec: u32) -> Self {
        if prec >= P {
            self
        } else {
            // both the unwrap should not be possible to trigger since prec < P
            // will result in a valid ten's encoding and checked_rem.
            let wrap = T::ten_power(P - prec).unwrap();
            Self(self.0 - self.0.checked_rem(wrap).unwrap())
        }
    }

    /// Return the integral part of this decimal
    ///
    /// ```
    /// use fixeddec::FixedDec;
    /// let f = FixedDec::<u32, 3>::new(1_234);
    /// assert_eq!(f.integral(), 1);
    /// ```
    pub fn integral(self) -> T {
        T::ten_power(P)
            .and_then(|prec| self.0.checked_div(prec))
            .unwrap()
    }

    /// Return the fractional part of this decimal
    ///
    /// ```
    /// use fixeddec::FixedDec;
    /// let f = FixedDec::<u32, 3>::new(1_234);
    /// assert_eq!(f.fractional(), 234);
    /// ```
    pub fn fractional(self) -> T {
        T::ten_power(P)
            .and_then(|prec| self.0.checked_rem(prec))
            .unwrap()
    }

    /// Return the content value at the precision required
    ///
    /// ```
    /// use fixeddec::FixedDec;
    /// let f = FixedDec::<u32, 3>::new(1_234);
    /// assert_eq!(f.value(), 1_234);
    /// ```
    pub const fn value(self) -> T {
        self.0
    }
}

impl<T: Number, const P: u32> Add for FixedDec<T, P> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl<T: Number, const P: u32> AddAssign for FixedDec<T, P> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl<T: Number, const P: u32> Sub for FixedDec<T, P> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl<T: Number, const P: u32> SubAssign for FixedDec<T, P> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0
    }
}

impl<T: Number, const P: u32> Mul<T> for FixedDec<T, P> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl<T: Number, const P: u32> MulAssign<T> for FixedDec<T, P> {
    fn mul_assign(&mut self, rhs: T) {
        self.0 *= rhs
    }
}

impl<T: Number, const P: u32> Div<T> for FixedDec<T, P> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl<T: Number, const P: u32> DivAssign<T> for FixedDec<T, P> {
    fn div_assign(&mut self, rhs: T) {
        self.0 /= rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integral_fractional() {
        let x1: FixedDec<u32, 3> = FixedDec::new(1000);
        let x2: FixedDec<u32, 4> = FixedDec::new(1000);
        let x3: FixedDec<u32, 3> = FixedDec::new(1234);
        let i1: FixedDec<i32, 3> = FixedDec::new(-1234);

        assert_eq!(x1.integral(), 1);
        assert_eq!(x1.fractional(), 0);

        assert_eq!(x2.integral(), 0);
        assert_eq!(x2.fractional(), 1000);

        assert_eq!(x3.integral(), 1);
        assert_eq!(x3.fractional(), 234);

        assert_eq!(i1.integral(), -1);
        assert_eq!(i1.fractional(), 234);
    }

    #[test]
    fn formatting() {
        let x1: FixedDec<u32, 3> = FixedDec::new(1000);
        let x2: FixedDec<u32, 4> = FixedDec::new(1000);
        let x3: FixedDec<u32, 3> = FixedDec::new(1234);
        let x4: FixedDec<u32, 2> = FixedDec::new(123456);

        assert_eq!(alloc::format!("{}", x1), "1.000");
        assert_eq!(alloc::format!("{}", x2), "0.1000");
        assert_eq!(alloc::format!("{}", x3), "1.234");
        assert_eq!(alloc::format!("{}", x4), "1234.56");
    }

    #[test]
    fn rounding() {
        let x1: FixedDec<u32, 3> = FixedDec::new(1000);
        let x2: FixedDec<u32, 4> = FixedDec::new(1000);
        let x3: FixedDec<u32, 3> = FixedDec::new(1234);
        let x4: FixedDec<u32, 2> = FixedDec::new(123456);

        assert_eq!(x1.round_at(2), x1);
        assert_eq!(x2.round_at(2), x2);
        assert_eq!(x3.round_at(2), FixedDec::new(1230));
        assert_eq!(x4.round_at(1), FixedDec::new(123450));
        assert_eq!(x4.round_at(3), FixedDec::new(123456));
    }
}
