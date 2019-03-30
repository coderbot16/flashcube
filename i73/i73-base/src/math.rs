/// Performs linear interpolation between two values based on a 3rd control value.
/// This is the "imprecise" method, that can possibly take advantage of an FMA instruction.
/// `t` represents the percentage between the two variables: 0.5 means 50%, ie. the average.
/// Note: `t` may be outside of [0, 1], in which case it will continue the line.
#[inline]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
	a + t*(b - a)
}

/// Performs linear interpolation between two values based on a 3rd control value.
/// This is the "imprecise" method, that can possibly take advantage of an FMA instruction.
/// `t` represents the percentage between the two variables: 0.5 means 50%, ie. the average.
/// Note: `t` may be outside of [0, 1], in which case it will continue the line.
#[inline]
pub fn lerp_precise(a: f64, b: f64, t: f64) -> f64 {
	(1.0 - t)*a + t*b
}

/// Performs linear interpolation between two values based on a 3rd control value, represented as a
/// fraction. This is algebraically equivalent to `lerp(a, b, tn/td)`, however results might differ
/// due to a different operation order.
#[inline]
pub fn lerp_fraction(a: f64, b: f64, tn: f64, td: f64) -> f64 {
	a + (b - a) * tn/td
}

/// Ensures that `x` is within the range `[min, max]`. Large values change to `max`, and small
/// values become `min`.
#[inline]
pub fn clamp(x: f64, min: f64, max: f64) -> f64 {
	x.max(min).min(max)
}

/// Floors the input and then clamps it into the range of an i32. This is the equivalent of the
/// following Java code:
/// ```java
/// 	return (double)((int)Math.floor(x));
/// ```
#[inline]
pub fn floor_clamped(x: f64) -> f64 {
	const MAX_I32: f64 = 2147483647.0;
	const MIN_I32: f64 = -2147483648.0;

	clamp(x.floor(), MIN_I32, MAX_I32)
}