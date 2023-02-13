/*a Copyright

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

  http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

@file    range.rs
@brief   Part of SVG library
 */

//a Range
//tp Range
#[derive(Debug, Clone, Copy, PartialEq)]
/// This is a simple 'range' class for a single dimension
///
/// min <= max for a valid range; min > max indicates an empty range
pub struct Range {
    /// Minimum coordinate of the range
    min: f64,
    /// Maximum coordinate of the range
    max: f64,
}

//ti Display for Range
impl std::fmt::Display for Range {
    /// Display the [Range] as (min to max)
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} to {})", self.min, self.max)
    }
}

//tp Default for Range
impl std::default::Default for Range {
    fn default() -> Self {
        Self::none()
    }
}

//ti Range
impl Range {
    //fp new
    /// Create a new point from (min,max)
    #[inline]
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    //fp none
    /// Create a new empty range (0,0)
    #[inline]
    pub const fn none() -> Self {
        Self { min: 0., max: -1. }
    }

    //fp is_none
    /// Return true if the range is empty
    #[inline]
    pub fn is_none(&self) -> bool {
        self.min > self.max
    }

    //fp of_pts
    pub fn of_pts(a: f64, b: f64) -> Self {
        if a < b {
            Self::new(a, b)
        } else {
            Self::new(b, a)
        }
    }

    //mp size
    /// Return the size of the range
    #[inline]
    pub fn size(&self) -> f64 {
        if self.is_none() {
            0.
        } else {
            self.max - self.min
        }
    }

    //mp center
    /// Return the center of the range
    #[inline]
    pub fn center(&self) -> f64 {
        (self.max + self.min) / 2.0
    }

    //cp include
    /// Include a point into the range, exanding min or max if required
    pub fn include(mut self, x: f64) -> Self {
        if self.is_none() {
            self.min = x;
            self.max = x;
        } else {
            if x < self.min {
                self.min = x;
            }
            if x > self.max {
                self.max = x;
            }
        }
        self
    }

    //cp enlarge
    /// Enlarge by an amount
    pub fn enlarge(mut self, value: f64) -> Self {
        if !self.is_none() {
            self.min -= value;
            self.max += value;
        }
        self
    }

    //cp reduce
    /// Reduce by an amount
    pub fn reduce(mut self, value: f64) -> Self {
        if !self.is_none() {
            self.min += value;
            self.max -= value;
        }
        self
    }

    //cp union
    /// Consume the range, and find the union min and max of this with
    /// another, returning the new region
    pub fn union(mut self, other: &Range) -> Self {
        if other.is_none() {
            self
        } else if self.is_none() {
            self.min = other.min;
            self.max = other.max;
            self
        } else {
            if other.min < self.min {
                self.min = other.min;
            }
            if other.max > self.max {
                self.max = other.max;
            }
            self
        }
    }

    //cp intersect
    /// Consume the range, and find the intersection min and max of this with
    /// another, returning the new region
    pub fn intersect(mut self, other: &Range) -> Self {
        if other.is_none() {
            self
        } else if self.is_none() {
            self.min = other.min;
            self.max = other.max;
            self
        } else {
            if other.min > self.min {
                self.min = other.min;
            }
            if other.max < self.max {
                self.max = other.max;
            }
            self
        }
    }
}

//ip std::ops::Add<f64> for Range
impl std::ops::Add<f64> for Range {
    type Output = Self;
    fn add(self, scale: f64) -> Self {
        Self {
            min: self.min + scale,
            max: self.max + scale,
        }
    }
}

//ip std::ops::AddAssign<f64> for Range
impl std::ops::AddAssign<f64> for Range {
    fn add_assign(&mut self, delta: f64) {
        self.min += delta;
        self.max += delta;
    }
}

//ip std::ops::Sub<f64> for Range
impl std::ops::Sub<f64> for Range {
    type Output = Self;
    fn sub(self, scale: f64) -> Self {
        Self {
            min: self.min - scale,
            max: self.max - scale,
        }
    }
}

//ip std::ops::SubAssign<f64> for Range
impl std::ops::SubAssign<f64> for Range {
    fn sub_assign(&mut self, delta: f64) {
        self.min -= delta;
        self.max -= delta;
    }
}

//ip std::ops::Mul<f64> for Range
impl std::ops::Mul<f64> for Range {
    type Output = Self;
    fn mul(self, scale: f64) -> Self {
        if scale < 0. {
            Self {
                min: self.max * scale,
                max: self.min * scale,
            }
        } else {
            Self {
                min: self.min * scale,
                max: self.max * scale,
            }
        }
    }
}
//ip std::ops::MulAssign<f64> for Range
impl std::ops::MulAssign<f64> for Range {
    fn mul_assign(&mut self, scale: f64) {
        self.max *= scale;
        self.min *= scale;
    }
}

//ip std::ops::Div<f64> for Range
impl std::ops::Div<f64> for Range {
    type Output = Self;
    fn div(self, scale: f64) -> Self {
        if scale < 0. {
            Self {
                min: self.max / scale,
                max: self.min / scale,
            }
        } else {
            Self {
                min: self.min / scale,
                max: self.max / scale,
            }
        }
    }
}
//ip std::ops::DivAssign<f64> for Range
impl std::ops::DivAssign<f64> for Range {
    fn div_assign(&mut self, scale: f64) {
        self.max /= scale;
        self.min /= scale;
    }
}

//ip std::ops::Index<usize> for Range
impl std::ops::Index<usize> for Range {
    type Output = f64;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < 2);
        if index == 0 {
            &self.min
        } else {
            &self.max
        }
    }
}

//a Tests
//mt Test for Range
#[cfg(test)]
mod test_range {
    use super::*;
    pub fn rng_eq(rng: &Range, min: f64, max: f64) {
        assert!(
            (rng.min - min).abs() < 1E-8,
            "mismatch in x {:?} {} {}",
            rng,
            min,
            max
        );
        assert!(
            (rng.max - max).abs() < 1E-8,
            "mismatch in x {:?} {} {}",
            rng,
            min,
            max
        );
    }
    #[test]
    fn test_simple() {
        assert!(Range::none().is_none());
        rng_eq(&Range::new(1., 2.), 1., 2.);
        assert!(Range::new(0.1, 0.).is_none());
        assert!(!Range::new(0., 0.1).is_none());
        rng_eq(&(Range::new(1., 2.) * 3.), 3., 6.);
        rng_eq(&(Range::new(3., 6.) / 3.), 1., 2.);

        assert_eq!(Range::none().size(), 0.);
        assert_eq!(Range::new(1., 0.).size(), 0.);
        assert_eq!(Range::new(0., 1.).size(), 1.);
        assert_eq!(Range::new(2., 0.).size(), 0.);
        assert_eq!(Range::new(0., 2.).size(), 2.);
    }
    #[test]
    fn test_union() {
        rng_eq(&Range::new(0., 4.).union(&Range::new(0., 4.)), 0., 4.);
        rng_eq(&Range::new(0., 4.).union(&Range::new(0., 5.)), 0., 5.);
        rng_eq(&Range::new(0., 4.).union(&Range::new(2., 5.)), 0., 5.);
        rng_eq(&Range::new(0., 4.).union(&Range::new(2., 3.)), 0., 4.);
        rng_eq(&Range::new(0., 4.).union(&Range::new(-1., 3.)), -1., 4.);
        rng_eq(&Range::new(0., 4.).union(&Range::new(-1., 5.)), -1., 5.);
    }
    #[test]
    fn test_intersect() {
        rng_eq(&Range::new(0., 4.).intersect(&Range::new(0., 4.)), 0., 4.);
        rng_eq(&Range::new(0., 4.).intersect(&Range::new(0., 5.)), 0., 4.);
        rng_eq(&Range::new(0., 4.).intersect(&Range::new(2., 5.)), 2., 4.);
        rng_eq(&Range::new(0., 4.).intersect(&Range::new(2., 3.)), 2., 3.);
        rng_eq(&Range::new(0., 4.).intersect(&Range::new(-1., 3.)), 0., 3.);
        rng_eq(&Range::new(0., 4.).intersect(&Range::new(-1., 5.)), 0., 4.);
    }
}
