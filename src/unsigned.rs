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

    /// Multiply two value at the given precision, truncating the digits below it
    ///
    /// The product is formed at double precision, so only a result that does not fit at this
    /// precision returns None, not an intermediate.
    pub fn multiply(self, other: Self) -> Option<Self> {
        let (low, high) = self.0.widening_mul(other.0);
        T::widening_div(low, high, ten_power_const::<T, P>()).map(Self)
    }

    /// Square the value at the same precision as the callee, truncating the digits below it
    ///
    /// As with [`FixedDec::multiply`], only a result that does not fit at this precision returns
    /// None.
    pub fn square(self) -> Option<Self> {
        self.multiply(self)
    }

    /// Square the value at double precision
    ///
    /// The output precision need to be twice the precision of the original value
    ///
    pub fn square_precise<const O: u32>(self) -> Option<FixedDec<T, O>> {
        const { assert!(O == P * 2) };
        self.0.checked_mul(self.0).map(FixedDec)
    }

    /// Raise the value to the power of an integral exponent, truncating the digits below the
    /// precision
    ///
    /// ```
    /// # use fixeddec::FixedDec;
    /// let x = FixedDec::<u32, 4>::new(1_5000);
    /// assert_eq!(x.powi(3), Some(FixedDec::<u32, 4>::new(3_3750)));
    /// ```
    ///
    /// If the result doesn't fit in the backing type, then None is returned
    pub fn powi(self, exponent: T) -> Option<Self> {
        let scale = ten_power_const::<T, P>();
        crate::exp::pow_scaled(self.0, exponent, scale, T::ZERO).map(Self)
    }

    /// Calculate `e` raised to the power of the value
    ///
    /// ```
    /// # use fixeddec::FixedDec;
    /// let one = FixedDec::<u32, 6>::from_integral(1).unwrap();
    /// assert_eq!(one.exp(), Some(FixedDec::<u32, 6>::new(2_718282)));
    /// ```
    ///
    /// Both halves of the computation are carried at more digits than the precision asked for, so
    /// the result is exact to the last digit over most of the range, and near the top of it carries
    /// the roundings of the repeated squaring: the measured worst case is a relative error of
    /// 2e-8 with a `u32` backing type, 9e-18 with a `u64` and 3e-36 with a `u128`.
    ///
    /// The result leaves the backing type at a small value, e.g. anything above 23.6 at
    /// `<u64, 9>`, and None is returned then. Within the relative error above of that limit the
    /// overflow itself is decided at that same accuracy, so a value just past it can come back as a
    /// result close to [`FixedDec::MAX`] rather than as None.
    pub fn exp(self) -> Option<Self> {
        crate::exp::exp(self.integral(), self.fractional(), P).map(Self)
    }

    /// Calculate `e` raised to the power of the value negated, i.e. `e^-x`
    ///
    /// ```
    /// # use fixeddec::FixedDec;
    /// let one = FixedDec::<u32, 6>::from_integral(1).unwrap();
    /// assert_eq!(one.exp_neg(), FixedDec::<u32, 6>::new(367_879));
    /// ```
    ///
    /// Unlike [`FixedDec::exp`] this cannot fail, as the result is always in `(0, 1]`. A value
    /// large enough for `e^-x` to fall below the precision gives zero.
    ///
    /// The whole computation stays below one, which leaves it the full width of the backing type
    /// and makes the result exact to the last digit at any precision short of that width. At that
    /// last precision there are no digits left over to absorb the rounding of each term, and the
    /// last digit can then be one out, which also means the result is not monotone there.
    pub fn exp_neg(self) -> Self {
        // every step of the negated case is at or below one, so nothing on the way there can leave
        // the range of the backing type and the unwrap cannot trigger
        Self(crate::exp::exp_neg(self.integral(), self.fractional(), P).unwrap())
    }

    /// Calculate the square root of the value, truncated towards zero
    ///
    /// The result is the largest value that squares back to at most the original value, so it is
    /// exact when the value is a perfect square and one unit of precision short of the real root
    /// otherwise. It is defined for every value, as the root of the largest value representable
    /// at a given precision is always representable at that same precision.
    ///
    /// ```
    /// # use fixeddec::FixedDec;
    /// // sqrt(2) with 3 fractional digits
    /// let two = FixedDec::<u32, 3>::new(2_000);
    /// assert_eq!(two.sqrt(), FixedDec::<u32, 3>::new(1_414));
    /// ```
    ///
    pub fn sqrt(self) -> Self {
        let exponent = ten_power_const::<T, P>();

        // double precision product, with the halves ordered high first so that comparing two of
        // them compares the products they stand for
        let wide_mul = |a: T, b: T| {
            let (low, high) = a.widening_mul(b);
            (high, low)
        };

        // the value brought to the precision a candidate's square lands at, i.e. `self * 10^P`,
        // kept at double precision so that the square of any candidate up to T::MAX can be
        // compared against it exactly. comparing against square() instead would see the truncated
        // quotient only, and accept any candidate whose square overshoots within the P discarded
        // digits, which for values small next to 10^P is most of the range.
        let target = wide_mul(self.0, exponent);

        // calculate a low bound for the possible candidate.
        // no precision adjustment is done, so quite a few digits are lost here using isqrt() for the value and the exponent
        let raw_root = self.0.isqrt();
        let exponent_root = exponent.isqrt();

        // isqrt() truncates both factors, so this bound never sits above the root
        let low = raw_root * exponent_root;

        // rounding each factor up instead gives a bound strictly above the root, as isqrt()
        // undershoots each of them by less than one. it is not always representable, and T::MAX
        // then serves just as well: no power of ten reaches T::MAX, so `self * 10^P < T::MAX²`.
        let high = (raw_root + T::ONE)
            .checked_mul(exponent_root + T::ONE)
            .unwrap_or(T::MAX);

        // the digits lost in the bounds can leave them a long way apart, so the gap is closed by
        // bisection instead of one raw unit at a time: the number of steps is then proportional to
        // the width of T rather than to the magnitude of the result.
        let two = T::ONE + T::ONE;
        let mut lo = low;
        let mut hi = high;

        // invariant: lo² <= self * 10^P < hi²
        while hi - lo > T::ONE {
            let mid = lo + (hi - lo) / two;
            if wide_mul(mid, mid) > target {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        Self(lo)
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
