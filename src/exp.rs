//! Exponentiation of a fixed point decimal
//!
//! The value is split into its integral and fractional parts, `x = i + f`. The fractional part
//! becomes the Taylor series of `e^f`, whose terms `f^n / n!` fall below the precision quickly: 13
//! of them at 9 digits, 20 at 18, 34 at 38. The integral part becomes a power of `e` by repeated
//! squaring.
//!
//! Neither part is computed at the precision that was asked for. The series is evaluated at
//! [`Number::E_PRECISION`], the highest precision the backing type can hold a value around `e` at,
//! and the powers of `e` as a mantissa below one paired with a count of tens taken out of it, which
//! leaves the whole width of the type for its digits. Everything in between is a guard digit, and
//! only the last step scales the result down to the requested precision. Computing a power of `e`
//! at the requested precision instead would lose its leading digits to it: at precision 0, `e`
//! itself is nothing but 3.

use crate::number::{Number, ten, ten_power};

/// Multiply two values scaled by `scale`, biasing the digits below the scale by `bias`
///
/// A `bias` of zero truncates them, half the scale rounds them to nearest. The product is formed at
/// double precision, so only a result above the range of `T` returns None.
pub(crate) fn mul_scaled<T: Number>(a: T, b: T, scale: T, bias: T) -> Option<T> {
    let (low, high) = a.widening_mul(b);
    let (low, high) = match low.checked_add(bias) {
        Some(low) => (low, high),
        // the bias pushed the low half out of range: what is left of it is the part past the range
        None => (low - (T::MAX - bias) - T::ONE, high.checked_add(T::ONE)?),
    };
    T::widening_div(low, high, scale)
}

/// Raise `base` to the power of `exponent` by repeated squaring, at the given scale
pub(crate) fn pow_scaled<T: Number>(base: T, exponent: T, scale: T, bias: T) -> Option<T> {
    let two = T::ONE + T::ONE;
    let mut acc = scale;
    let mut base = base;
    let mut exponent = exponent;

    while exponent > T::ZERO {
        if exponent.checked_rem(two)? != T::ZERO {
            acc = mul_scaled(acc, base, scale, bias)?;
        }
        exponent /= two;
        // squaring the base once more than the exponent asks for could overflow for nothing
        if exponent > T::ZERO {
            base = mul_scaled(base, base, scale, bias)?;
        }
    }
    Some(acc)
}

/// Move a value from the precision `from` to the precision `to`, rounding to nearest
pub(crate) fn rescale<T: Number>(value: T, from: u32, to: u32) -> Option<T> {
    match to.cmp(&from) {
        core::cmp::Ordering::Equal => Some(value),
        core::cmp::Ordering::Greater => {
            ten_power::<T>(to - from).and_then(|m| value.checked_mul(m))
        }
        core::cmp::Ordering::Less => {
            let divisor = ten_power::<T>(from - to)?;
            let quotient = value / divisor;
            let remainder = value - quotient * divisor;
            // `remainder + remainder >= divisor`, without the doubling leaving the range
            if remainder >= divisor - remainder {
                quotient.checked_add(T::ONE)
            } else {
                Some(quotient)
            }
        }
    }
}

/// Multiply two mantissas below one, renormalizing the product back into `[0.1, 1)`
///
/// Returns the mantissa and the count of tens taken out of it, which is 0 or -1. The renormalizing
/// case scales the double precision product down by one ten less, rather than scaling the result of
/// the first attempt back up, which would leave its last digit at zero.
fn mul_normalized<T: Number>(a: T, b: T, scale: T, tenth: T) -> Option<(T, i32)> {
    let two = T::ONE + T::ONE;
    let product = mul_scaled(a, b, scale, scale / two)?;
    if product >= tenth {
        Some((product, 0))
    } else {
        let smaller = scale / ten::<T>();
        Some((mul_scaled(a, b, smaller, smaller / two)?, -1))
    }
}

/// `e^f` for a fractional part `f` at or below one, at the given scale
///
/// The result is in `[1, e]`, which fits by the definition of [`Number::E_PRECISION`], and so do
/// all the partial sums leading to it.
fn exp_fractional<T: Number>(f: T, scale: T) -> Option<T> {
    let half = scale / (T::ONE + T::ONE);
    let mut acc = scale;
    let mut term = scale;
    let mut n = T::ZERO;

    loop {
        n += T::ONE;
        // f is at or below one, so each term is at or below the one before it and never overflows
        term = mul_scaled(term, f, scale, half)? / n;
        if term == T::ZERO {
            return Some(acc);
        }
        acc += term;
    }
}

/// `e^-f` for a fractional part `f` at or below one, at the given scale
///
/// The result is in `[1/e, 1]`. The series alternates, so its terms are summed in pairs: each pair
/// is a positive term followed by a smaller negative one, which keeps every intermediate inside the
/// unsigned range and leaves nothing for the accumulator to cancel against.
fn exp_neg_fractional<T: Number>(f: T, scale: T) -> Option<T> {
    let half = scale / (T::ONE + T::ONE);
    let mut acc = T::ZERO;
    let mut term = scale;
    let mut n = T::ZERO;

    loop {
        n += T::ONE;
        let subtracted = mul_scaled(term, f, scale, half)? / n;
        acc += term - subtracted;
        if subtracted == T::ZERO {
            return Some(acc);
        }

        n += T::ONE;
        term = mul_scaled(subtracted, f, scale, half)? / n;
        if term == T::ZERO {
            return Some(acc);
        }
    }
}

/// The widest precision the backing type has, at which a value below one keeps every digit
fn digits<T: Number>() -> u32 {
    T::TEN_POWER.len() as u32 - 1
}

/// `e^(integral + fractional)` at the precision `precision`, the parts being those of the value
///
/// Returns None when the result is above the range of `T` at this precision.
pub(crate) fn exp<T: Number>(integral: T, fractional: T, precision: u32) -> Option<T> {
    let inner = T::E_PRECISION;
    let inner_scale = ten_power::<T>(inner)?;

    // the fractional part rounded up to one is still a valid input to the series, and gives what
    // carrying it into the integral part would
    let f = rescale(fractional, precision, inner)?;
    let exp_f = exp_fractional(f, inner_scale)?;
    if integral == T::ZERO {
        return rescale(exp_f, inner, precision);
    }

    let digits = digits::<T>();
    let scale = ten_power::<T>(digits)?;
    let tenth = ten_power::<T>(digits - 1)?;
    // a mantissa is at least a tenth, so it stands for at least 10^(tens-1), and the type holds
    // less than 10^(digits+1) at precision zero
    let bound = digits as i32 - precision as i32 + 1;

    // one, and `e` itself, as mantissas paired with their count of tens
    let (mut mantissa, mut tens) = (tenth, 1i32);
    let (mut base, mut base_tens) = (rescale(T::E, inner + 1, digits)?, 1i32);

    let two = T::ONE + T::ONE;
    let mut exponent = integral;
    loop {
        if exponent.checked_rem(two)? != T::ZERO {
            let (product, adjust) = mul_normalized(mantissa, base, scale, tenth)?;
            mantissa = product;
            tens += base_tens + adjust;
        }
        exponent /= two;
        if exponent == T::ZERO {
            break;
        }
        let (square, adjust) = mul_normalized(base, base, scale, tenth)?;
        base = square;
        base_tens = base_tens * 2 + adjust;
        // whatever is left of the exponent multiplies the base into the result at least once, so
        // once the base alone is out of reach there is nothing left to compute
        if tens > bound || base_tens > bound {
            return None;
        }
    }

    // e^f normalized the same way, which keeps the product of the two below one
    let (mantissa, adjust) =
        mul_normalized(mantissa, rescale(exp_f, inner + 1, digits)?, scale, tenth)?;
    let tens = tens + 1 + adjust;

    // the mantissa is scaled by 10^digits and stands for 10^tens times that
    let delta = precision as i32 + tens - digits as i32;
    if delta >= 0 {
        ten_power::<T>(delta as u32).and_then(|m| mantissa.checked_mul(m))
    } else {
        rescale(mantissa, (-delta) as u32, 0)
    }
}

/// `e^-(integral + fractional)` at the precision `precision`
///
/// The result is in `(0, 1]`, so it always fits. Values too small to represent at this precision
/// come back as zero.
pub(crate) fn exp_neg<T: Number>(integral: T, fractional: T, precision: u32) -> Option<T> {
    // both factors are at or below one, so unlike the positive case the whole computation stays at
    // one scale, the widest the type has, and there is nothing to overflow along the way
    let digits = digits::<T>();
    let scale = ten_power::<T>(digits)?;
    let half = scale / (T::ONE + T::ONE);

    let f = rescale(fractional, precision, digits)?;
    let exp_f = exp_neg_fractional(f, scale)?;
    // 1/e is given at this very precision, so it comes in with all of its digits
    let exp_i = pow_scaled(T::E_RECIP, integral, scale, half)?;

    rescale(mul_scaled(exp_i, exp_f, scale, half)?, digits, precision)
}
