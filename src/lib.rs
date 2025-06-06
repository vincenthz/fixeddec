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
//! - `FixedDec::<u32, 0>::new(123)` with internal value 123u32, represents the integer `123`
//! - `FixedDec::<u32, 3>::new(123)` with internal value 123u32, represents the decimal `0.123`
//!
//! Internally, the value is stored as a raw integer of type `T`, and the decimal point is
//! applied logically according to the value of `P`.
//!
//! ## Scientific Notation
//!
//! FixedDec precision translate into the scientific notation with negative exponent:
//!
//! `V*10^(-P) == FixedDec::<_, P>::new(V)`
//!
//! ## Unit
//!
//! For example this could be used to define SI suffixes:
//!
//! ```
//! # use fixeddec::FixedDec;
//! type Milli = FixedDec<u64, 3>;
//! type Micro = FixedDec<u64, 6>;
//! type Nano = FixedDec<u64, 9>;
//! ```
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
//!
//! // with float    : 0.1 + 0.2 = 0.30000000000000004
//! // with FixedDec : 0.1 + 0.2 = 0.3
//! let point_one = FixedDec::<u32, 1>::new(1);
//! let point_two = FixedDec::new(2);
//! let point_three = FixedDec::new(3);
//!
//! assert_eq!(point_one + point_two, point_three);
//! ```
//!
#![no_std]

use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

extern crate alloc;

pub mod constants;
mod number;

use number::{Number, ten_power};

/// A integral number with a precision of fractional digits
///
/// At P=0, it is a normal integer with no fractional part
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
    /// Minimum value representable by this type
    pub const MIN: Self = Self::new(T::MIN);

    /// Maximum value representable by this type
    pub const MAX: Self = Self::new(T::MAX);

    /// Create a new FixedDec using the backing value already at the required precision
    ///
    /// ```
    /// use fixeddec::FixedDec;
    /// let f = FixedDec::<u32, 3>::new(1_234);
    /// ```
    pub const fn new(t: T) -> Self {
        // similar to assert!(T::ten_power(P).is_some()); but const'able
        assert!(T::TEN_POWER.len() > P as usize);
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
        ten_power::<T>(P).and_then(|prec| t.checked_mul(prec).map(Self))
    }

    /// Try to change the precision of the value without changing the represented value
    ///
    /// If the demanded precision is smaller than the original precision, then silent truncating will happens:
    ///
    /// ```
    /// use fixeddec::FixedDec;
    /// let orig_value = FixedDec::<u32, 3>::new(123);
    /// let new_value = orig_value.set_precision::<2>();
    /// assert_eq!(new_value, Some(FixedDec::<u32, 2>::new(12)));
    /// ```
    ///
    /// ```
    /// use fixeddec::FixedDec;
    /// let orig_value = FixedDec::<u32, 3>::new(123);
    /// let new_value = orig_value.set_precision::<5>();
    /// assert_eq!(new_value, Some(FixedDec::<u32, 5>::new(12300)));
    /// ```
    ///
    pub fn set_precision<const O: u32>(self) -> Option<FixedDec<T, O>> {
        use core::cmp::Ordering;
        match P.cmp(&O) {
            Ordering::Equal => Some(FixedDec(self.0)),
            Ordering::Greater => {
                let diff = P - O;
                ten_power::<T>(diff).and_then(|prec| self.0.checked_div(prec).map(FixedDec))
            }
            Ordering::Less => {
                let diff = O - P;
                ten_power::<T>(diff).and_then(|prec| self.0.checked_mul(prec).map(FixedDec))
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

    /// Add two elements with checked result
    ///
    /// If the addition result doesn't fits in the type T, then None is returned
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    /// Subtract two elements with checked result
    ///
    /// If the subtraction result doesn't fits in the type T, then None is returned
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    /// Multiplication (Scaling) with checked result
    ///
    /// Note that operands are mixed between FixedDec and a scalar T, not another FixedDec.
    ///
    /// If the multiplication result doesn't fits in the type T, then None is returned
    pub fn checked_mul(self, rhs: T) -> Option<Self> {
        self.0.checked_mul(rhs).map(Self)
    }

    /// Division (Inverse Scaling) with checked result
    pub fn checked_div(self, rhs: T) -> Option<Self> {
        self.0.checked_div(rhs).map(Self)
    }

    /// Checked remainder. Computes self % rhs, returning None if rhs == 0.
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
            let wrap = ten_power::<T>(P - prec).unwrap();
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
        ten_power::<T>(P)
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
        ten_power::<T>(P)
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

    /// Parse a string containing a fractional number (e.g. "1.234")
    ///
    /// If the string doesn't contain any dot, then it interpreted as an integral number.
    pub fn from_str(s: &str) -> Option<Self> {
        let ten = ten_power(1).unwrap(); // safe all types have 10
        if let Some((i1, f1)) = s.split_once('.') {
            if !i1.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            if !f1.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }

            let mut acc = T::ZERO;

            // integral part
            for c in i1.chars() {
                let i = T::from_digit10(c)?;
                acc = acc.checked_mul(ten)?.checked_add(i)?;
            }

            // fractional part
            let sz_frac = f1.chars().count();
            for (depth, c) in f1.chars().enumerate() {
                if depth >= P as usize {
                    break;
                }
                let i = T::from_digit10(c)?;
                acc = acc.checked_mul(ten)?.checked_add(i)?;
            }

            if sz_frac < P as usize {
                let mul = ten_power(P - sz_frac as u32)?;
                acc = acc.checked_mul(mul)?;
            }

            Some(Self::new(acc))
        } else {
            // no fractional .
            if !s.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            let mut acc = T::ZERO;
            for c in s.chars() {
                let i = T::from_digit10(c)?;
                acc = acc.checked_mul(ten)?.checked_add(i)?;
            }
            Self::from_integral(acc)
        }
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

    #[test]
    fn from_str() {
        let x0 = FixedDec::<u32, 0>::new(1234);
        let x1 = FixedDec::<u32, 3>::new(1234);
        let x2 = FixedDec::<u32, 3>::new(10234);
        let x3 = FixedDec::<u32, 4>::new(10234);
        let x4 = FixedDec::<u32, 4>::new(12340);

        assert_eq!(FixedDec::from_str("1234"), Some(x0));
        assert_eq!(FixedDec::from_str("1.234"), Some(x1));
        assert_eq!(FixedDec::from_str("10.234"), Some(x2));
        assert_eq!(FixedDec::from_str("1.0234"), Some(x3));
        assert_eq!(FixedDec::from_str("1.02345"), Some(x3));
        assert_eq!(FixedDec::from_str("1.234"), Some(x4));
    }
}
