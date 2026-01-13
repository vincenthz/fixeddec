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
//! let a: FixedDec<u32, 2> = FixedDec::new(12345);
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
//#![no_std]

extern crate alloc;

pub mod constants;
mod number;
mod signed;
mod unsigned;

pub use signed::FixedSDec;
pub use unsigned::{FixedDec, fdec32, fdec64, fdec128};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integral_fractional() {
        let x1: FixedDec<u32, 3> = FixedDec::new(1000);
        let x2: FixedDec<u32, 4> = FixedDec::new(1000);
        let x3: FixedDec<u32, 3> = FixedDec::new(1234);

        assert_eq!(x1.integral(), 1);
        assert_eq!(x1.fractional(), 0);

        assert_eq!(x2.integral(), 0);
        assert_eq!(x2.fractional(), 1000);

        assert_eq!(x3.integral(), 1);
        assert_eq!(x3.fractional(), 234);
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
    fn squaring() {
        // 0.1000 * 0.1000 = 0.0100
        let x1: FixedDec<u32, 4> = FixedDec::new(1000);
        assert_eq!(x1.square().unwrap(), FixedDec::new(100));

        // 0.10 * 0.10 = 0.01
        let x1: FixedDec<u32, 2> = FixedDec::new(10);
        assert_eq!(x1.square().unwrap(), FixedDec::new(1));

        // 0.1 * 0.1 = 0.0
        let x1: FixedDec<u32, 1> = FixedDec::new(1);
        assert_eq!(x1.square().unwrap(), FixedDec::new(0));

        // 2.3456 * 2.3456 = 5.5018
        let x1: FixedDec<u32, 4> = FixedDec::new(23456);
        assert_eq!(x1.square().unwrap(), FixedDec::new(55018));
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

    #[test]
    fn floating64() {
        let x0 = FixedDec::<u32, 0>::new(1234);
        let x1 = FixedDec::<u32, 3>::new(1234);
        let x2 = FixedDec::<u32, 3>::new(10234);
        let x3 = FixedDec::<u32, 4>::new(10234);
        let x4 = FixedDec::<u32, 4>::new(12340);

        let s = ["1234", "1.234", "10.234", "1.0234", "1.234"];

        assert_eq!(alloc::format!("{}", x0.as_f64()), s[0]);
        assert_eq!(alloc::format!("{}", x1.as_f64()), s[1]);
        assert_eq!(alloc::format!("{}", x2.as_f64()), s[2]);
        assert_eq!(alloc::format!("{}", x3.as_f64()), s[3]);
        assert_eq!(alloc::format!("{}", x4.as_f64()), s[4]);
    }

    #[test]
    fn sqrt() {
        let x0 = FixedDec::<u32, 2>::new(2_00);
        let v = x0.sqrt();
        assert_eq!(v, Some(FixedDec::<u32, 2>::new(1_41)));

        let x0 = FixedDec::<u32, 3>::new(2_000);
        let v = x0.sqrt();
        assert_eq!(v, Some(FixedDec::<u32, 3>::new(1_414)));

        let x0 = FixedDec::<u32, 0>::new(125348);
        let v = x0.sqrt();
        assert_eq!(v, Some(FixedDec::<u32, 0>::new(354)));

        let x0 = FixedDec::<u32, 1>::new(125348_0);
        let v = x0.sqrt();
        assert_eq!(v, Some(FixedDec::<u32, 1>::new(354_0)));

        let x0 = FixedDec::<u32, 3>::new(125348_000);
        let v = x0.sqrt();
        // due to precision overflow during multiplication it is None instead of the 354_045
        assert_eq!(v, None);
    }
}
