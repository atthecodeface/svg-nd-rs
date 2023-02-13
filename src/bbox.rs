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

@file    bbox.rs
@brief   Part of SVG library
 */
//a Imports
use geo_nd::vector;

use crate::{Point, Range};

//a BBox
//tp BBox
#[derive(Debug, Clone, Copy, Default, PartialEq)]
/// [BBox] describes a region bounded by (x0,y0) and (x1,y1) It
/// requires x0 <= x1 and y0 <= y1, and if either are equal then the
/// region is deemed to be *none*
pub struct BBox {
    /// X range
    x: Range,
    /// Y range
    y: Range,
}

//ti Display for BBox
impl std::fmt::Display for BBox {
    //mp fmt - format for a human
    /// Display the BBox
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[({},{}):({},{})]",
            self.x[0], self.y[0], self.x[1], self.y[1]
        )
    }

    //zz All done
}

//ti BBox
impl BBox {
    //mp none
    /// Create a none bbox - where both ranges are none
    pub fn none() -> Self {
        Self {
            x: Range::none(),
            y: Range::none(),
        }
    }

    //mp is_none
    /// Return `true` if the rectangle describes a 'none' region
    pub fn is_none(&self) -> bool {
        self.x.is_none() || self.y.is_none()
    }

    //fp new
    /// Make a rectangle using the coordinates supplied, ensuring that
    /// the rectangle is correctly defined
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        let x = if x0 < x1 {
            Range::new(x0, x1)
        } else {
            Range::new(x1, x0)
        };
        let y = if y0 < y1 {
            Range::new(y0, y1)
        } else {
            Range::new(y1, y0)
        };
        Self { x, y }
    }

    //cp of_ranges
    /// Create a [BBox] from two pRange]
    pub fn of_ranges(x: Range, y: Range) -> Self {
        Self { x, y }
    }

    //cp of_points
    /// Make a new rectangle that is the bounding box of a vec of points
    pub fn of_points(pts: &Vec<Point>) -> Self {
        let mut s = Self::none();
        for p in pts.iter() {
            s.x = s.x.include(p[0]);
            s.y = s.y.include(p[1]);
        }
        s
    }

    //fp of_cwh
    /// Generate a rectangle from a centre `Point` and a width/height.
    pub fn of_cwh(centre: Point, width: f64, height: f64) -> Self {
        Self::new(
            centre[0] - width / 2.,
            centre[1] - height / 2.,
            centre[0] + width / 2.,
            centre[1] + height / 2.,
        )
    }

    //mp pt_within
    /// Consume a point, and return a new point that whose X
    /// coordinate indicate the fraction of this rectangles' width the
    /// original point is along its width, and similarly for the Y
    /// coordinate
    ///
    pub fn pt_within(&self, mut pt: Point) -> Point {
        if self.is_none() {
            pt
        } else {
            let (rx, ry) = (pt[0] - self.x[0], pt[1] - self.y[0]);
            [rx / self.x.size(), ry / self.y.size()].into()
        }
    }

    //mp add_as_points
    /// Create a vector of four points that are the
    /// anticlockwise-ordered corners of the rectangle starting at the
    /// minumum (x,y)
    pub fn add_as_points(&self, close: bool, mut v: Vec<Point>) -> Vec<Point> {
        v.push([self.x[0], self.y[0]].into());
        v.push([self.x[1], self.y[0]].into());
        v.push([self.x[1], self.y[1]].into());
        v.push([self.x[0], self.y[1]].into());
        if close {
            v.push([self.x[0], self.y[0]].into());
        }
        v
    }

    //mp get_wh
    /// Return a point consisting of the width and height of the rectangle
    pub fn get_wh(&self) -> (f64, f64) {
        (self.x.size(), self.y.size())
    }

    //mp center
    /// Return a point indicating the centre of the rectangle
    pub fn center(&self) -> Point {
        [self.x.center(), self.y.center()].into()
    }

    //mp xrange
    /// Return the xrange of the bbox
    ///
    pub fn xrange(&self) -> Range {
        self.x
    }

    //mp yrange
    /// Return the yrange of the bbox
    ///
    pub fn yrange(&self) -> Range {
        self.y
    }

    //mp width
    /// Return the width of the rectangle (`x1` - `x0`)
    pub fn width(&self) -> f64 {
        self.x.size()
    }

    //mp height
    /// Return the height of the rectangle (`y1` - `y0`)
    ///
    pub fn height(&self) -> f64 {
        self.y.size()
    }

    //mp get_cwh
    /// Get the centre, width and height of the rectangle
    pub fn get_cwh(&self) -> (Point, f64, f64) {
        (self.center(), self.width(), self.height())
    }

    //cp enlarge
    /// Consume the rectangle and return a new rectangle enlarge by a
    /// fixed value
    ///
    pub fn enlarge(mut self, value: f64) -> Self {
        self.x.enlarge(value);
        self.y.enlarge(value);
        self
    }

    //cp reduce
    /// Shrink the rectangle, keeping the same center, by a fixed value
    ///
    pub fn reduce(mut self, value: f64) -> Self {
        self.x.enlarge(value);
        self.y.enlarge(value);
        self
    }

    //mp expand
    /// exand in-place by expansion scaled by 'scale'
    ///
    /// Was Float4 x0, x1, y0,, x1 now [x0, y0, x1, y1]
    pub fn expand(mut self, other: &[f64; 4], scale: f64) -> Self {
        self.x = Range::new(self.x[0] - scale * other[0], self.x[1] + scale * other[2]);
        self.y = Range::new(self.y[0] - scale * other[1], self.y[1] + scale * other[3]);
        self
    }

    //mp shrink
    /// shrink in-place by expansion scaled by 'scale'
    pub fn shrink(self, other: &[f64; 4], scale: f64) -> Self {
        self.expand(other, -scale)
    }

    //mp union
    /// union this with another; if either is_zero then just the other
    pub fn union(mut self, other: Self) -> Self {
        if other.is_none() {
            self
        } else if self.is_none() {
            other
        } else {
            self.x = self.x.union(&other.x);
            self.y = self.y.union(&other.y);
            self
        }
    }

    //mp intersect
    /// intersect this with another; if either is_zero then this will be zero
    pub fn intersect(mut self, other: Self) -> Self {
        if other.is_none() {
            other
        } else if self.is_none() {
            self
        } else {
            self.x = self.x.intersect(&other.x);
            self.y = self.y.intersect(&other.y);
            self
        }
    }

    //mp new_rotated_around
    /// Rotate the rectangle around a point by an angle,
    /// generating a new rectangle that is the bounding box of that rotated rectangle
    pub fn new_rotated_around(&self, pt: &Point, degrees: f64) -> Self {
        let radians = degrees.to_radians();
        let p0 = vector::rotate_around([self.x[0], self.y[0]], pt.as_ref(), radians, 0, 1);
        let p1 = vector::rotate_around([self.x[1], self.y[0]], pt.as_ref(), radians, 0, 1);
        let p2 = vector::rotate_around([self.x[0], self.y[1]], pt.as_ref(), radians, 0, 1);
        let p3 = vector::rotate_around([self.x[1], self.y[1]], pt.as_ref(), radians, 0, 1);
        let mut x = Range::none();
        let mut y = Range::none();
        x = x
            .include(p0[0])
            .include(p1[0])
            .include(p2[0])
            .include(p3[0]);
        y = y
            .include(p0[1])
            .include(p1[1])
            .include(p2[1])
            .include(p3[1]);
        Self { x, y }
    }

    //zz All done
}

//ip std::ops::Add<Point> for BBox
impl std::ops::Add<Point> for BBox {
    type Output = Self;
    fn add(mut self, dxy: Point) -> Self {
        self.x += dxy[0];
        self.y += dxy[1];
        self
    }
}
//ip std::ops::AddAssign<Point> for BBox
impl std::ops::AddAssign<Point> for BBox {
    fn add_assign(&mut self, dxy: Point) {
        self.x += dxy[0];
        self.y += dxy[1];
    }
}

//ip std::ops::Sub<Point> for BBox
impl std::ops::Sub<Point> for BBox {
    type Output = Self;
    fn sub(mut self, dxy: Point) -> Self {
        self.x -= dxy[0];
        self.y -= dxy[1];
        self
    }
}
//ip std::ops::SubAssign<Point> for BBox
impl std::ops::SubAssign<Point> for BBox {
    fn sub_assign(&mut self, dxy: Point) {
        self.x -= dxy[0];
        self.y -= dxy[1];
    }
}

//ip std::ops::Mul<f64> for BBox
impl std::ops::Mul<f64> for BBox {
    type Output = Self;
    fn mul(mut self, scale: f64) -> Self {
        self.x *= scale;
        self.y *= scale;
        self
    }
}

//ip std::ops::MulAssign<f64> for BBox
impl std::ops::MulAssign<f64> for BBox {
    fn mul_assign(&mut self, scale: f64) {
        self.x *= scale;
        self.y *= scale;
    }
}

//ip std::ops::Div<f64> for BBox
impl std::ops::Div<f64> for BBox {
    type Output = Self;
    fn div(mut self, scale: f64) -> Self {
        self.x /= scale;
        self.y /= scale;
        self
    }
}

//ip std::ops::DivAssign<f64> for BBox
impl std::ops::DivAssign<f64> for BBox {
    fn div_assign(&mut self, scale: f64) {
        self.x /= scale;
        self.y /= scale;
    }
}

//a Test
#[cfg(test)]
mod tests_polygon {
    use super::*;
    pub fn range_eq(pt: &Range, x: f64, y: f64) {
        assert!(
            (pt[0] - x).abs() < 1E-8,
            "mismatch in x {:?} {} {}",
            pt,
            x,
            y
        );
        assert!(
            (pt[1] - y).abs() < 1E-8,
            "mismatch in y {:?} {} {}",
            pt,
            x,
            y
        );
    }
    pub fn pt_eq(pt: &Point, x: f64, y: f64) {
        assert!(
            (pt[0] - x).abs() < 1E-8,
            "mismatch in x {:?} {} {}",
            pt,
            x,
            y
        );
        assert!(
            (pt[1] - y).abs() < 1E-8,
            "mismatch in y {:?} {} {}",
            pt,
            x,
            y
        );
    }
    pub fn pair_eq(pt: &(f64, f64), x: f64, y: f64) {
        assert!(
            (pt.0 - x).abs() < 1E-8,
            "mismatch in x {:?} {} {}",
            pt,
            x,
            y
        );
        assert!(
            (pt.1 - y).abs() < 1E-8,
            "mismatch in y {:?} {} {}",
            pt,
            x,
            y
        );
    }
    #[test]
    fn test_zero() {
        let x = BBox::none();
        assert!(x.is_none());
        dbg!("center {:?}", x.get_cwh());
        assert_eq!(x.width(), 0.);
        assert_eq!(x.height(), 0.);
    }
    #[test]
    fn test_0() {
        let x = BBox::new(-3., 1., 5., 7.);
        pt_eq(&x.center(), 1., 4.);
        assert_eq!(x.width(), 8.);
        assert_eq!(x.height(), 6.);
        pair_eq(&x.get_wh(), 8., 6.);
        range_eq(&x.xrange(), -3., 5.);
        range_eq(&x.yrange(), 1., 7.);
        pt_eq(&x.get_cwh().0, 1., 4.);
        assert_eq!(x.get_cwh().1, 8.);
        assert_eq!(x.get_cwh().2, 6.);
    }
    #[test]
    fn test_ops_0() {
        let x = BBox::new(2., 1., 5., 7.);
        let y = BBox::new(4., 0., 6., 3.);
        let z = BBox::new(5., 1., 7., 4.);
        let x_and_y = x.clone().intersect(y);
        let x_or_y = x.clone().union(y);
        let x_and_z = x.clone().intersect(z);
        let x_or_z = x.clone().union(z);
        println!("x_and_y:{}", x_and_y);
        println!("x_or_y:{}", x_or_y);
        println!("x_and_z:{}", x_and_z);
        println!("x_or_z:{}", x_or_z);
        dbg!("One");
        range_eq(&x_and_y.xrange(), 4., 5.);
        dbg!("Two");
        range_eq(&x_and_y.yrange(), 1., 3.);
        dbg!("Three");
        range_eq(&x_or_y.xrange(), 2., 6.);
        dbg!("Four");
        range_eq(&x_or_y.yrange(), 0., 7.);
        dbg!("Five");

        assert!(!x_and_z.is_none()); // was none originally
        dbg!(x_and_z.xrange());
        dbg!(x_and_z.yrange());
        dbg!(x_or_z.xrange());
        dbg!(x_or_z.yrange());
        range_eq(&x_and_z.xrange(), 5., 5.);
        range_eq(&x_and_z.yrange(), 1., 4.);
        range_eq(&x_or_z.xrange(), 2., 7.);
        range_eq(&x_or_z.yrange(), 1., 7.);
    }
    #[test]
    fn test_ops_1() {
        let x = BBox::new(2., 1., 5., 7.);
        let y = [0.1, 0.2, 0.3, 0.5];
        let x_p_y = x.clone().expand(&y, 1.);
        let x_p_2y = x.clone().expand(&y, 2.);
        println!("x_p_y:{}", x_p_y);
        println!("x_p_2y:{}", x_p_2y);
        range_eq(&x_p_y.xrange(), 1.9, 5.3);
        range_eq(&x_p_y.yrange(), 0.8, 7.5);
        range_eq(&x_p_2y.xrange(), 1.8, 5.6);
        range_eq(&x_p_2y.yrange(), 0.6, 8.);
    }
    #[test]
    fn test_ops_2() {
        let x = BBox::new(2., 1., 5., 7.);
        let y = [0.1, 0.2, 0.3, 0.5];
        let x_m_y = x.clone().shrink(&y, 1.);
        let x_m_2y = x.clone().shrink(&y, 2.);
        println!("x_m_y:{}", x_m_y);
        println!("x_m_2y:{}", x_m_2y);
        range_eq(&x_m_y.xrange(), 2.1, 4.7);
        range_eq(&x_m_y.yrange(), 1.2, 6.5);
        range_eq(&x_m_2y.xrange(), 2.2, 4.4);
        range_eq(&x_m_2y.yrange(), 1.4, 6.);
    }
}
