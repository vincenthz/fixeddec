use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::number::{Number, ten, ten_power, ten_power_const};

/// u32 (32 bits) aliases to FixedDec
#[allow(non_camel_case_types)]
pub type fdec32<const P: u32> = FixedDec<u32, P>;

/// u64 (64 bits) aliases to FixedDec
#[allow(non_camel_case_types)]
pub type fdec64<const P: u32> = FixedDec<u64, P>;

/// u128 (128 bits) aliases to FixedDec
#[allow(non_camel_case_types)]
pub type fdec128<const P: u32> = FixedDec<u128, P>;

/// A integral number with a precision of fractional digits
///
/// At P=0, it is a normal unsigned integer with no fractional part
///
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct FixedDec<T: Number, const P: u32>(pub(crate) T);

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
        const { assert!(T::TEN_POWER.len() > P as usize) }
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
        let prec = ten_power_const::<T, P>();
        t.checked_mul(prec).map(Self)
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
        self.0.checked_div(ten_power_const::<T, P>()).unwrap()
    }

    /// Return the fractional part of this decimal
    ///
    /// ```
    /// use fixeddec::FixedDec;
    /// let f = FixedDec::<u32, 3>::new(1_234);
    /// assert_eq!(f.fractional(), 234);
    /// ```
    pub fn fractional(self) -> T {
        self.0.checked_rem(ten_power_const::<T, P>()).unwrap()
    }

    /// Convert a fixed decimal into a f32 (with potential data loss)
    pub fn as_f32(self) -> f32
    where
        f32: From<T>,
    {
        f32::from(self.0) / f32::from(ten_power_const::<T, P>())
    }

    /// Convert a fixed decimal into a f64 (with potential data loss)
    pub fn as_f64(self) -> f64
    where
        f64: From<T>,
    {
        f64::from(self.0) / f64::from(ten_power_const::<T, P>())
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
        let ten = ten();
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

    /// Multiply two value at the given precision
    pub fn multiply(self, other: Self) -> Option<Self> {
        let r = self.0.checked_mul(other.0)?;
        let div = ten_power_const::<T, P>();
        Some(Self(r / div))
    }

    /// Square the value at the same precision as the callee
    pub fn square(self) -> Option<Self> {
        let r = self.0.checked_mul(self.0)?;
        let div = ten_power_const::<T, P>();
        Some(Self(r / div))
    }

    /// Square the value at double precision
    ///
    /// The output precision need to be twice the precision of the original value
    ///
    pub fn square_precise<const O: u32>(self) -> Option<FixedDec<T, O>> {
        const { assert!(O == P * 2) };
        self.0.checked_mul(self.0).map(FixedDec)
    }

    /// Calculate the square root of the value
    pub fn sqrt(self) -> Option<Self> {
        // calculate a low bound for the possible candidate.
        // no precision adjustment is done, so quite a few digits are lost here using isqrt() for the value and the exponent
        let raw_root = self.0.isqrt();
        let exponent = ten_power_const::<T, P>();
        let exponent_root = exponent.isqrt();

        let low = Self(raw_root * exponent_root);
        let one = T::ONE;

        if low.square() == Some(self) {
            return Some(low);
        }

        // we adjust the candidate incrementally/naively until it is above the initial value we are calculating the value
        let mut candidate = low;
        loop {
            let next = candidate + Self(one);
            let Some(v) = next.square() else {
                return None;
            };
            if v > self {
                return Some(candidate);
            } else {
                candidate = next;
            }
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
