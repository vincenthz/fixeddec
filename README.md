# FixedDec

A fixed-point decimal number type with generic precision.

This module provides the `FixedDec<T, P>` type, a lightweight wrapper around integral numbers that interprets them as fixed-point decimal values with `P` fractional digits.

## Overview

`FixedDec` allows representing decimal numbers without using floating-point
arithmetic, which is useful in financial or deterministic computation contexts.
The `P` const generic parameter defines the number of decimal places. For example:

 - `FixedDec::<u32, 0>::new(123)` represents the integer `123`
 - `FixedDec::<u32, 3>::new(123)` represents the decimal `0.123`

Internally, the value is stored as a raw integer of type `T`, and the decimal point is applied logically according to the value of `P`.
