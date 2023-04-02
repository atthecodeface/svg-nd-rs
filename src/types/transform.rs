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

@file    transform.rs
@brief   Transformation class
 */

//a Imports
use geo_nd::Vector;

use crate::{Error, Point};

//a Transform type
//tp Transform
/// A Transfom is a transformation applied to something - for example,
/// applied to content to present it in its parent coordinates.
///
/// The transformation is translate(rotate(scale(pt)))
///
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    /// Translation - applied last
    translation: Point,
    /// Rotation around the origin in *degrees*
    rotation: f64,
    /// Scale factor
    scale: f64,
}

//ip Default for Transform
impl std::default::Default for Transform {
    fn default() -> Self {
        Self {
            translation: Point::default(),
            rotation: 0.,
            scale: 1.,
        }
    }
}

//ip Transform
impl Transform {
    //ap translation
    #[inline]
    pub fn translation(&self) -> Point {
        self.translation
    }

    //ap rotation
    #[inline]
    pub fn rotation(&self) -> f64 {
        self.rotation
    }

    //ap scale
    #[inline]
    pub fn scale(&self) -> f64 {
        self.scale
    }

    //cp of_trs
    /// Create a transform from a translation, rotation and scale
    #[inline]
    #[must_use]
    pub fn of_trs(translation: Point, rotation: f64, scale: f64) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    //fp of_rotation
    /// Create a transform from a rotation
    #[inline]
    #[must_use]
    pub fn of_rotation(rotation: f64) -> Self {
        Self::of_trs(Point::zero(), rotation, 1.)
    }

    //cp of_translation
    /// Create a transform from a translation
    #[inline]
    #[must_use]
    pub fn of_translation(translation: Point) -> Self {
        Self::of_trs(translation, 0., 1.)
    }

    //fp of_matrix
    /// Set to be whatever a 3x3 matrix indicates
    ///
    /// Note that the matrix must be cs -ss dx; ss cs dy; 0 0 1
    ///
    /// Hence the top left (scaled rotation) is (cs -ss) (ss cs)
    ///
    /// Hence the top row multiplied together added to the bottom row
    /// multiplied together should be 0
    ///
    /// Also the determinant of this is cs*cs + ss*ss = scale^2 * (cos^2+sin^2)
    ///
    /// Hence the determinant must be >0 and its square root is the scale
    ///
    pub fn of_matrix(matrix: &[f64]) -> Result<Self, Error> {
        if matrix.len() != 9 {
            Err(Error::InvalidTransformationMatrix {
                reason: "matrix was not 3-by-3".into(),
            })?
        }
        if !(matrix[8] == 1. && matrix[7] == 0. && matrix[6] == 0.) {
            Err(Error::InvalidTransformationMatrix {
                reason: "bottom row must be 0, 0, 1".into(),
            })?
        }
        let dx = matrix[2];
        let dy = matrix[5];
        let skew = matrix[0] * matrix[1] + matrix[4] * matrix[3];
        if skew.abs() > 1.0E-6 {
            Err(Error::InvalidTransformationMatrix {
                reason: "rotation portion (top left 4 values) represent a skew not a rotation"
                    .into(),
            })?
        }
        let sc2 = matrix[0] * matrix[4] - matrix[1] * matrix[3];
        if sc2 < -1.0E-9 {
            Err(Error::InvalidTransformationMatrix {
                reason: "determinant (scale squared) is negative".into(),
            })?
        }
        let sc = {
            if sc2 < 0. {
                0.
            } else {
                sc2.sqrt()
            }
        };
        let angle = matrix[3].atan2(matrix[4]).to_degrees();

        Ok(Self::of_trs([dx, dy].into(), angle, sc))
    }

    //mp is_identity
    /// Return true if this is an identity transform
    pub fn is_identity(&self) -> bool {
        self.rotation == 0. && self.scale == 1. && self.translation.is_zero()
    }

    //mp to_matrix
    /// Returns a 3x3 matrix that can be applied to points (x,y,1) or vectors (dx,dy,0)
    pub fn to_matrix(&self) -> [f64; 9] {
        let mut result = [0.; 9];
        let sc = self.scale;
        let s = self.rotation.to_radians().sin();
        let c = self.rotation.to_radians().cos();
        let dx = self.translation[0];
        let dy = self.translation[1];
        // the result of three matrices
        // scale      sc  0  0;  0 sc  0;  0  0  1
        // rotate      c -s  0;  s  c  0;  0  0  1
        // translate   1  0 dx;  0  1 dy;  0  0  1
        // i.e.
        // R.S    =   c*sc -s*sc  0;  s*sc  c*sc  0;  0  0  1
        // T.R.S  =   c*sc -s*sc  dx;  s*sc  c*sc  dy;  0  0  1
        result[0] = sc * c;
        result[1] = -sc * s;
        result[2] = dx;
        result[3] = sc * s;
        result[4] = sc * c;
        result[5] = dy;
        result[8] = 1.;
        result
    }

    //mp apply
    /// Apply this transform to a point
    pub fn apply(&self, pt: Point) -> Point {
        let m = self.to_matrix();
        let x = pt[0];
        let y = pt[1];
        [m[0] * x + m[1] * y + m[2], m[3] * x + m[4] * y + m[5]].into()
    }

    //mp apply_to_transform
    /// Apply this transform to another transform, returning a new
    /// transform
    // The result will be a scaling of both multipled together, and a
    // rotation of both added together, plus a translation
    //
    // Note that matrix(other) = CS -SS DX; SS CS DY; 0 0 1
    // Note that matrix(self)  = cs -ss dx; ss cs dy; 0 0 1
    // Combine we get _ _ cs.DX-ss.DY+dx ; _ _ ss.DX+cs.DY+dy; 0 0 1
    // i.e. the resultant translation is:
    // self.rotate_scale(other.translate)+self.translate
    pub fn apply_to_transform(&self, other: &Self) -> Self {
        let mut dxy = other.translation;
        dxy.rotate_around(&Point::zero(), self.rotation, 0, 1);
        dxy = dxy * self.scale + self.translation;
        Self::of_trs(
            dxy,
            self.rotation + other.rotation,
            self.scale * other.scale,
        )
    }

    //fp as_svg_attribute_string
    pub fn as_svg_attribute_string(&self) -> String {
        let mut r = String::new();
        let dxy = self.translation;
        if dxy[0] != 0. || dxy[1] != 0. {
            r.push_str(&format!("translate({:.4} {:.4}) ", dxy[0], dxy[1]));
        }
        if self.rotation != 0. {
            r.push_str(&format!("rotate({:.4}) ", self.rotation));
            dbg!(&r);
        }
        if self.scale != 1. {
            r.push_str(&format!("scale({:.4}) ", self.scale));
        }
        r
    }

    //zz All done
}

//ip std::ops::Add<Point> for Transform
impl std::ops::Add<Point> for Transform {
    type Output = Self;
    #[inline]
    fn add(mut self, dxy: Point) -> Self {
        self.translation += dxy;
        self
    }
}

//ip std::ops::Sub<Point> for Transform
impl std::ops::Sub<Point> for Transform {
    type Output = Self;
    #[inline]
    fn sub(mut self, dxy: Point) -> Self {
        self.translation -= dxy;
        self
    }
}

//ip std::ops::Mul<f64> for Transform
impl std::ops::Mul<f64> for Transform {
    type Output = Self;
    #[inline]
    fn mul(mut self, scale: f64) -> Self {
        self.translation *= scale;
        self.scale *= scale;
        self
    }
}

//ip std::ops::Div<f64> for Transform
impl std::ops::Div<f64> for Transform {
    type Output = Self;
    #[inline]
    fn div(mut self, scale: f64) -> Self {
        self.translation /= scale;
        self.scale /= scale;
        self
    }
}

//ip std::fmt::Display for Transform
impl std::fmt::Display for Transform {
    //mp fmt - format a `Transform` for display
    /// Display the `Transform` in a human-readable form
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.translation.is_zero() && self.rotation == 0. && self.scale == 1. {
            write!(f, "<identity>")
        } else if self.rotation == 0. && self.scale == 1. {
            write!(
                f,
                "<+({:.4}, {:.4})>",
                self.translation[0], self.translation[1]
            )
        } else {
            if !self.translation.is_zero() {
                write!(
                    f,
                    "<+({:.4}, {:.4})>",
                    self.translation[0], self.translation[1]
                )?
            };
            if self.rotation != 0. {
                write!(f, "<rot({})>", self.rotation)?
            };
            if self.scale != 1. {
                write!(f, "<*{}>", self.scale)?
            };
            Ok(())
        }
    }
}

//mt Test for Transform
#[cfg(test)]
mod tests {
    use super::*;
    fn approx_eq(a: f64, b: f64) -> bool {
        let diff = a - b;
        diff > -1.0E-6 && diff < 1.0E-6
    }
    fn check_transform(t: &Transform, dx: f64, dy: f64, r: f64, sc: f64) {
        assert!(
            approx_eq(t.translation[0], dx),
            "Transform {} dx of {} {} {} {}",
            t,
            dx,
            dy,
            r,
            sc
        );
        assert!(
            approx_eq(t.translation[1], dy),
            "Transform {} dy of {} {} {} {}",
            t,
            dx,
            dy,
            r,
            sc
        );
        assert!(
            approx_eq(t.rotation, r),
            "Transform {} r of {} {} {} {}",
            t,
            dx,
            dy,
            r,
            sc
        );
        assert!(
            approx_eq(t.scale, sc),
            "Transform {} sc of {} {} {} {}",
            t,
            dx,
            dy,
            r,
            sc
        );
    }
    fn check_matrix(m: &[f64], e: &[f64]) {
        let okay = m
            .iter()
            .zip(e.iter())
            .fold(true, |acc, (m, e)| (acc && approx_eq(*m, *e)));
        assert!(okay, "Matrix {:?} expected {:?}", m, e);
    }
    #[test]
    fn test_0() {
        check_transform(&Transform::default(), 0., 0., 0., 1.);
        check_transform(&Transform::of_translation(Point::zero()), 0., 0., 0., 1.);
        check_transform(
            &Transform::of_translation(Point::from_array([1., 2.])),
            1.,
            2.,
            0.,
            1.,
        );
        check_transform(
            &Transform::of_trs(Point::from_array([3., -2.]), 7., 6.),
            3.,
            -2.,
            7.,
            6.,
        );
    }
    #[test]
    fn test_1() {
        let m = Transform::of_trs(Point::zero(), 0., 1.).to_matrix();
        assert_eq!(m, [1., 0., 0., 0., 1., 0., 0., 0., 1.]);
        let m = Transform::of_trs(Point::zero(), 0., 7.).to_matrix();
        assert_eq!(m, [7., 0., 0., 0., 7., 0., 0., 0., 1.]);
        let m = Transform::of_trs(Point::from_array([4., 5.]), 0., 7.).to_matrix();
        check_matrix(&m, &[7., 0., 4., 0., 7., 5., 0., 0., 1.]);
        let m = Transform::of_trs(Point::from_array([4., 5.]), 90., 7.).to_matrix();
        check_matrix(&m, &[0., -7., 4., 7., 0., 5., 0., 0., 1.]);
        let m = Transform::of_trs(Point::from_array([4., 5.]), 180., 7.).to_matrix();
        check_matrix(&m, &[-7., 0., 4., 0., -7., 5., 0., 0., 1.]);
        let m = Transform::of_trs(Point::from_array([4., 5.]), 270., 7.).to_matrix();
        check_matrix(&m, &[0., 7., 4., -7., 0., 5., 0., 0., 1.]);
    }
    #[test]
    fn test_2() {
        // Note matrix of 0. always produces a transform of 0.0., 0., 0.
        for (x, y) in vec![
            (0., 0.),
            (0., 1.),
            (1., 0.),
            (1., 1.),
            (-1., 0.),
            (-1., -1.),
        ] {
            for r in vec![0., 45., 90., 135.] {
                for s in vec![1., 5., 0.1] {
                    // cannot use 0.
                    let t = Transform::of_trs(Point::from_array([x, y]), r, s);
                    let m = t.to_matrix();
                    let t1 = Transform::of_matrix(&m).unwrap();
                    check_transform(&t1, x, y, r, s);
                }
            }
        }
    }
}
