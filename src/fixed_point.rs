use std::ops::{Add, AddAssign, Sub, SubAssign};
// TODO: Implement use std::cmp::{Ord, PartialOrd};

/// Represents a fixed point datatype tailored to Chu-chu Rocket clones
/// running at 60fps.
/// We also want to run on microcontrollers, so we're restricting ourselves
/// to 16 bit data types.
/// It would be nicer to use a fractional part that aligned to a bit
/// boundary so that we could do simple bitshifting, but c'est la vie.
/// TODO: When const generics land, make the fractional part a generic parameter
/// TODO: If I can't avoid division when mapping to pixels, consider just using i16 with a
/// 360 scaling factor.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct FixedPoint {
    value: i8,
    fractional: i16,
}

impl FixedPoint {
    /// Creates a new fixed point value from the individual components
    /// The input range of fractional should be in the range [-359..359]
    /// TODO: Add assertions (or use BoundedInteger crate)
    ///
    /// #examples
    /// ```
    /// use shoko_rocket_rust::FixedPoint;
    /// let a = FixedPoint::new(1,  180); // 1.5
    /// let b = FixedPoint::new(1, -180); // 0.5
    /// let c = FixedPoint::new(0,  180); // Also 0.5!
    /// ```
    pub fn new(value: i8, fractional: i16) -> FixedPoint {
        FixedPoint { value, fractional }
    }

    /// Creates a new fixed point value from a floating point value
    ///
    /// #examples
    /// ```
    /// use shoko_rocket_rust::FixedPoint;
    /// let a = FixedPoint::from_float(0.5f32);
    /// ```
    pub fn from_float(value: f32) -> FixedPoint {
        let integral_part = value as i8;
        let remainder = value - value.trunc();
        if remainder > 0f32 {
            FixedPoint::new(integral_part, 0) + FixedPoint::new(0, (remainder * 360.0f32) as i16)
        } else {
            FixedPoint::new(integral_part, 0) - FixedPoint::new(0, (remainder * 360.0f32) as i16)
        }
    }

    /// Returns true if the change between two FixedPoint values results in a new
    /// integral part value
    ///
    /// #examples
    /// ```
    /// use shoko_rocket_rust::FixedPoint;
    /// let a = FixedPoint::new(1, 359);
    /// let b = FixedPoint::new(2, 0);
    ///
    /// assert_eq!(b.did_overflow(a), true);
    /// ```
    pub fn did_overflow(self, copy: Self) -> bool {
        self.value != copy.value
    }

    /// Maps the FixedPoint number onto an arbitrary 16 bit range.
    /// Note that values outside this range can be returned if the input
    /// values are themselves outside the from_min..from_max range
    ///
    /// Arguments:
    /// * `to_min`: The value that will be returned given an input of from_min
    /// * `to_max`: The value that will be returned given an input of from_max
    /// * `from_min`: The input mapping minimum
    /// * `from_max`: The input mapping maximum
    ///
    /// TODO: Add some assertions about intputs
    pub fn map_to_i16(
        self,
        from_min: FixedPoint,
        from_max: FixedPoint,
        to_min: i16,
        to_max: i16,
    ) -> i16 {
        // Integer components are worth 360 times that of the fractional ones, which requires
        // an additional 9 bits. We therefore need to use an i32 to accurately represent this
        // If we restrict the input range, we can probably do this in an i16
        let to_delta = (to_max - to_min) as i32;
        let from_min_scaled = from_min.fractional as i32 + from_min.value as i32 * 360;
        let from_max_scaled = from_max.fractional as i32 + from_max.value as i32 * 360;
        let from_delta_scaled = from_max_scaled - from_min_scaled;
        let self_scaled = self.fractional as i32 + self.value as i32 * 360;

        // Now everthing is in 360ths, perform the mapping
        // I use 17 bits to represent the scaled variables
        // I use 16 bits to represent the to/from
        // We're two bits short, so we should be careful with input ranges!
        (to_min as i32 + ((self_scaled - from_min_scaled) * to_delta) / from_delta_scaled) as i16

        // TBD:
        // Given I will be displaying on a 160x120 screen and I want a 12x9 grid, that means each
        // square is 13.3px. I need some leftover room so a 12px (leaving 16 px on right)
        // If I did this I could probably optimise this, as I could more or less take the input and
        // do a couple of comparisons on the remainder
    }

    /// Gets the integer part of the number
    /// 1.5 -> 1
    /// -1.5 -> -1
    pub fn integer_part(self) -> i8 {
        self.value
    }
}

/// Implements the Add trait for FixedPoint
/// #examples
/// ```
/// use shoko_rocket_rust::FixedPoint;
/// FixedPoint::new(1, 0) + FixedPoint::new(1, 0);
/// ```
impl Add for FixedPoint {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut result = self;
        result += other;
        result
    }
}

/// Implements the AddAssign trait for FixedPoint
/// #examples
/// ```
/// use shoko_rocket_rust::FixedPoint;
/// let mut a = FixedPoint::new(1, 0);
/// let b = FixedPoint::new(1, 0);
/// a += b;
/// ```
impl AddAssign for FixedPoint {
    fn add_assign(&mut self, other: Self) {
        // Add fractional parts. Assume they are in a valid range
        let mut fractional_sum = self.fractional + other.fractional;
        let mut integral_sum = self.value + other.value;
        if fractional_sum >= 360 {
            integral_sum += 1;
            fractional_sum -= 360;
        }

        self.value = integral_sum;
        self.fractional = fractional_sum;
    }
}

/// Implements the Sub trait for FixedPoint
/// #examples
/// ```
/// use shoko_rocket_rust::FixedPoint;
/// FixedPoint::new(1, 0) - FixedPoint::new(1, 0);
/// ```
impl Sub for FixedPoint {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        let mut result = self;
        result -= other;
        result
    }
}

/// Implements the SubAssign trait for FixedPoint
/// #examples
/// ```
/// use shoko_rocket_rust::FixedPoint;
/// let mut a = FixedPoint::new(1, 0);
/// let b = FixedPoint::new(1, 0);
/// a -= b;
/// ```
impl SubAssign for FixedPoint {
    fn sub_assign(&mut self, other: Self) {
        // Subtract fractional parts. Assume they are in a valid range
        let mut fractional_sum = self.fractional - other.fractional;
        let mut integral_sum = self.value - other.value;
        if fractional_sum <= -360 {
            integral_sum -= 1;
            fractional_sum += 360;
        }

        self.value = integral_sum;
        self.fractional = fractional_sum;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// GIVEN Two FixedPoint values of 1 and 0
    /// WHEN The two are added
    /// THEN The result is 1
    #[test]
    fn zero_plus_one() {
        let zero = FixedPoint::new(0, 0);
        let one = FixedPoint::new(1, 0);

        let sum = zero + one;
        assert_eq!(sum.value, 1i8);
        assert_eq!(sum.fractional, 0i16);
    }

    /// GIVEN A starting value of 0
    /// WHEN The minimum fractional part is added 732 times
    /// THEN The result is 2 + 12/360
    #[test]
    fn repeated_small_addition() {
        let mut sum = FixedPoint::new(0, 0);
        let step = FixedPoint::new(0, 1);

        for _x in 0..732 {
            sum += step;
        }

        assert_eq!(sum.value, 2i8);
        assert_eq!(sum.fractional, 12i16);
    }

    /// GIVEN Two FixedPoint values of 3.5 and 4 + 2/3
    /// WHEN The two are added
    /// THEN The result is 8 + 1/6
    #[test]
    fn fractional_overflow_on_addition() {
        let a = FixedPoint::new(3, 180);
        let b = FixedPoint::new(4, 240);
        let sum = a + b;

        assert_eq!(sum.value, 8i8);
        assert_eq!(sum.fractional, 60i16);
    }

    /// GIVEN A starting value of 0
    /// WHEN The minimum fractional part is subtracted 732 times
    /// THEN The result is -3 + 348/360
    #[test]
    fn repeated_small_subtraction() {
        let mut sum = FixedPoint::new(0, 0);
        let step = FixedPoint::new(0, 1);

        for _x in 0..732 {
            sum -= step;
        }

        assert_eq!(sum.value, -2i8);
        assert_eq!(sum.fractional, -12i16);
    }

    /// GIVEN Two FixedPoint values of 3.5 and 4 + 2/3
    /// WHEN The two are subtracted
    /// THEN The result is -2 + 5/6
    #[test]
    fn fractional_underflow_on_subtraction() {
        let a = FixedPoint::new(3, 180);
        let b = FixedPoint::new(4, 240);
        let sum = a - b;

        assert_eq!(sum.value, -1i8);
        assert_eq!(sum.fractional, -60i16);
    }

    /// GIVEN An initial staring position at zero
    /// WHEN Moved 50% of the way to the next whole value
    /// THEN did_overflow returns false
    /// AND WHEN Moved 100% of the way to the next whole value
    /// THEN did_overflow returns true
    #[test]
    fn did_overflow() {
        let start = FixedPoint::new(0, 0);
        let half = FixedPoint::new(0, 180);

        let add_half = start + half;
        let add_full = start + half + half;

        let sub_half = start - half;
        let sub_full = start - half - half;

        assert_eq!(start.did_overflow(start), false);
        assert_eq!(add_half.did_overflow(start), false);
        assert_eq!(add_full.did_overflow(start), true);
        assert_eq!(sub_half.did_overflow(start), false);
        assert_eq!(sub_full.did_overflow(start), true);
    }

    /// GIVEN an input mapping range of 0..12
    /// AND an output mapping range of 0..160
    /// WHEN 0 is mapped
    /// THEN 0 is output
    #[test]
    fn map_to_i16_at_min() {
        let from_min = FixedPoint::new(0, 0);
        let from_max = FixedPoint::new(12, 0);
        let to_min = 0;
        let to_max = 160;
        let output = from_min.map_to_i16(from_min, from_max, to_min, to_max);

        assert_eq!(to_min, output);
    }

    /// GIVEN an input mapping range of 0..12
    /// AND an output mapping range of 0..160
    /// WHEN 12 is mapped
    /// THEN 160 is output
    #[test]
    fn map_to_i16_at_max() {
        let from_min = FixedPoint::new(0, 0);
        let from_max = FixedPoint::new(12, 0);
        let to_min = 0;
        let to_max = 160;
        let output = from_max.map_to_i16(from_min, from_max, to_min, to_max);

        assert_eq!(to_max, output);
    }

    /// GIVEN an input mapping range of 0..12
    /// AND an output mapping range of 0..160
    /// WHEN 6 is mapped
    /// THEN 80 is output
    #[test]
    fn map_to_i16_in_middle() {
        let input = FixedPoint::new(6, 0);
        let from_min = FixedPoint::new(0, 0);
        let from_max = FixedPoint::new(12, 0);
        let to_min = 0;
        let to_max = 160;
        let output = input.map_to_i16(from_min, from_max, to_min, to_max);

        assert_eq!(80, output);
    }

    /// GIVEN an input mapping range of 0..1
    /// GIVEN an output mapping range of 0..100
    /// WHEN -0.5 is mapped
    /// THEN -50 is output
    #[test]
    fn map_to_i16_below_min() {
        let input = FixedPoint::new(0, -180);
        let from_min = FixedPoint::new(0, 0);
        let from_max = FixedPoint::new(1, 0);
        let to_min = 0;
        let to_max = 100;
        let output = input.map_to_i16(from_min, from_max, to_min, to_max);

        assert_eq!(-50, output);
    }

    /// GIVEN an input mapping range of 0..1
    /// GIVEN an output mapping range of 0..100
    /// WHEN 1.5 is mapped
    /// THEN 150 is output
    #[test]
    fn map_to_i16_above_max() {
        let input = FixedPoint::new(1, 180);
        let from_min = FixedPoint::new(0, 0);
        let from_max = FixedPoint::new(1, 0);
        let to_min = 0;
        let to_max = 100;
        let output = input.map_to_i16(from_min, from_max, to_min, to_max);

        assert_eq!(150, output);
    }

    /// GIVEN an input mapping range of 0..100
    /// GIVEN an output mapping range of 0..10
    /// WHEN [-10, 0, 50, 100, 110] are mapped
    /// THEN [-1, 0, 5, 10, 11] are output
    #[test]
    fn map_to_i16_over_small_range() {
        let from_min = FixedPoint::new(0, 0);
        let from_max = FixedPoint::new(100, 0);
        let to_min = 0;
        let to_max = 10;

        assert_eq!(-1, FixedPoint::new(-10, 0).map_to_i16(from_min, from_max, to_min, to_max));
        assert_eq!(0, FixedPoint::new(0, 0).map_to_i16(from_min, from_max, to_min, to_max));
        assert_eq!(5, FixedPoint::new(50, 0).map_to_i16(from_min, from_max, to_min, to_max));
        assert_eq!(10, FixedPoint::new(100, 0).map_to_i16(from_min, from_max, to_min, to_max));
        assert_eq!(11, FixedPoint::new(110, 0).map_to_i16(from_min, from_max, to_min, to_max));
    }

    /// GIVEN an input mapping range of 10..110
    /// GIVEN an output mapping range of 100, 1000
    /// WHEN [10, 60, 110] are mapped
    /// THEN [100, 550, 1000] are output
    #[test]
    fn map_to_i16_with_non_zero_input_min() {
        let from_min = FixedPoint::new(10, 0);
        let from_max = FixedPoint::new(110, 0);
        let to_min = 100;
        let to_max = 1000;

        assert_eq!(100, FixedPoint::new(10, 0).map_to_i16(from_min, from_max, to_min, to_max));
        assert_eq!(550, FixedPoint::new(60, 0).map_to_i16(from_min, from_max, to_min, to_max));
        assert_eq!(1000, FixedPoint::new(110, 0).map_to_i16(from_min, from_max, to_min, to_max));
    }
}
