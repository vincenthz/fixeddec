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
mod exp;
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
        assert_eq!(v, FixedDec::<u32, 2>::new(1_41));

        let x0 = FixedDec::<u32, 3>::new(2_000);
        let v = x0.sqrt();
        assert_eq!(v, FixedDec::<u32, 3>::new(1_414));

        let x0 = FixedDec::<u32, 0>::new(125348);
        let v = x0.sqrt();
        assert_eq!(v, FixedDec::<u32, 0>::new(354));

        let x0 = FixedDec::<u32, 1>::new(125348_0);
        let v = x0.sqrt();
        assert_eq!(v, FixedDec::<u32, 1>::new(354_0));

        let x0 = FixedDec::<u32, 3>::new(125348_000);
        let v = x0.sqrt();
        assert_eq!(v, FixedDec::<u32, 3>::new(354_045));

        // wide backing type: the low bound starts ~5% under the root, which is 200+ million
        // raw units away here, so this only completes quickly with a bisecting search
        let x0 = FixedDec::<u64, 1>::new(1_600_000_000_000_000_000);
        let v = x0.sqrt();
        assert_eq!(v, FixedDec::<u64, 1>::new(4_000_000_000));

        // values small next to 10^P: the answer is only right if the candidate is compared at
        // double precision, as the whole square sits inside the digits square() discards
        let x0 = FixedDec::<u32, 3>::new(1);
        let v = x0.sqrt();
        assert_eq!(v, FixedDec::<u32, 3>::new(31));

        let x0 = FixedDec::<u32, 2>::new(3);
        let v = x0.sqrt();
        assert_eq!(v, FixedDec::<u32, 2>::new(17));

        let x0 = FixedDec::<u32, 3>::new(10);
        let v = x0.sqrt();
        assert_eq!(v, FixedDec::<u32, 3>::new(100));

        // the whole range is reachable: scaling the value by 10^P overflows T for all of these,
        // so the candidate can only be compared at double precision
        let v = FixedDec::<u8, 2>::MAX.sqrt();
        assert_eq!(v, FixedDec::<u8, 2>::new(159));

        let v = FixedDec::<u32, 0>::MAX.sqrt();
        assert_eq!(v, FixedDec::<u32, 0>::new(65535));

        let v = FixedDec::<u32, 3>::MAX.sqrt();
        assert_eq!(v, FixedDec::<u32, 3>::new(2_072_430));

        let v = FixedDec::<u64, 9>::new(19_000_000_000).sqrt();
        assert_eq!(v, FixedDec::<u64, 9>::new(4_358_898_943));

        let v = FixedDec::<u64, 9>::MAX.sqrt();
        assert_eq!(v, FixedDec::<u64, 9>::new(135_818_791_312_945));

        let v = FixedDec::<u128, 38>::MAX.sqrt();
        assert_eq!(
            v,
            FixedDec::<u128, 38>::new(184_467_440_737_095_516_159_999_999_999_999_999_999)
        );
    }

    #[test]
    fn exp() {
        // exact to the last digit wherever the guard digits absorb the truncated terms
        assert_eq!(
            FixedDec::<u32, 6>::new(0).exp(),
            Some(FixedDec::new(1_000_000))
        );
        assert_eq!(
            FixedDec::<u32, 6>::new(500_000).exp(),
            Some(FixedDec::new(1_648_721))
        );
        assert_eq!(
            FixedDec::<u32, 6>::new(1_000_000).exp(),
            Some(FixedDec::new(2_718_282))
        );
        assert_eq!(
            FixedDec::<u64, 9>::new(1_000_000_000).exp(),
            Some(FixedDec::new(2_718_281_828))
        );
        // e^57, whose leading digits would all be lost if the powers of e were built at the
        // precision asked for, as `e` itself is only 3 there
        assert_eq!(
            FixedDec::<u128, 0>::new(57).exp(),
            Some(FixedDec::new(5_685_719_999_335_932_222_640_349))
        );

        // above the range of the backing type at that precision
        assert_eq!(FixedDec::<u32, 6>::new(10_000_000).exp(), None); // e^10 = 22026.5
        assert_eq!(FixedDec::<u64, 9>::new(24_000_000_000).exp(), None); // e^24 = 2.6e10
        assert_eq!(FixedDec::<u128, 0>::new(89).exp(), None);

        // close to the top of the range the last digits carry the roundings of the whole squaring
        // chain, a relative error of about 1e-17 for a 19 digits backing type
        let v = FixedDec::<u64, 9>::new(23_600_000_000).exp().unwrap();
        let reference = 17_756_189_565_520_348_111; // e^23.6 rounded at 9 digits
        assert!(
            v.value().abs_diff(reference) < 1_000,
            "{v} is not within 1000 units of the last digit"
        );
    }

    #[test]
    fn exp_neg() {
        assert_eq!(
            FixedDec::<u32, 6>::new(0).exp_neg(),
            FixedDec::new(1_000_000)
        );
        assert_eq!(
            FixedDec::<u32, 6>::new(1_000_000).exp_neg(),
            FixedDec::new(367_879)
        );
        assert_eq!(
            FixedDec::<u32, 6>::new(2_500_000).exp_neg(),
            FixedDec::new(82_085)
        );
        assert_eq!(
            FixedDec::<u64, 18>::new(1_000_000_000_000_000_000).exp_neg(),
            FixedDec::new(367_879_441_171_442_322)
        );

        // e^-20 sits below the last digit at this precision
        assert_eq!(
            FixedDec::<u32, 6>::new(20_000_000).exp_neg(),
            FixedDec::new(0)
        );

        // and the result never leaves the range, whatever the value
        assert_eq!(FixedDec::<u32, 0>::MAX.exp_neg(), FixedDec::new(0));
        assert!(FixedDec::<u128, 38>::MAX.exp_neg() <= FixedDec::from_integral(1).unwrap());
    }

    #[test]
    fn powi() {
        assert_eq!(
            FixedDec::<u32, 4>::new(15_000).powi(0),
            Some(FixedDec::new(10_000))
        );
        assert_eq!(
            FixedDec::<u32, 4>::new(15_000).powi(3),
            Some(FixedDec::new(33_750))
        );
        assert_eq!(
            FixedDec::<u64, 9>::new(1_000_000_001).powi(10),
            Some(FixedDec::new(1_000_000_010))
        );
        // 9^9 needs 9 integral digits, more than this type has left at 4 fractional ones
        assert_eq!(FixedDec::<u32, 4>::new(90_000).powi(9), None);
    }
}
