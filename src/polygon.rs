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

@file    polygon.rs
@brief   Regular polygon for the SVG library
 */

//a Imports
use geo_nd::Vector;

use crate::{BezierPath, Point};
// use super::Rectangle;

//tp Polygon
/// A polygon here defines an n-gon, from which one can generate a bezier path
///
/// It may have rounded corners
///
/// Nominally it is a regular n-gon, but it may have an eccentricity
///
#[derive(Debug, Clone, Copy)]
pub struct Polygon {
    center: Point,
    vertices: usize,
    size: f64,          // height
    stellate_size: f64, // if not 0., then double the points and make a star
    eccentricity: f64,  // width/height; i.e. width = size*eccentricity
    rotation: f64,      // rotation in degrees (after eccentricity)
    rounding: f64,      // 0 for no rounding of corners
}

//ip std::fmt::Display for Polygon
impl std::fmt::Display for Polygon {
    //mp fmt - format a `Polygon` for display
    /// Display the `Polygon` in a human-readable form
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.vertices == 0 {
            if self.eccentricity == 1. {
                write!(f, "Circle<{:?}, r={}>", self.center, self.size)
            } else {
                write!(
                    f,
                    "Ellipse<{:?}, a={}, b={}, rot={}>",
                    self.center,
                    self.size * self.eccentricity,
                    self.size,
                    self.rotation
                )
            }
        } else {
            write!(
                f,
                "Poly<{:?}, n={}, s={}, e={}, rot={}, stel={}, rnd={}>",
                self.center,
                self.vertices,
                self.size,
                self.eccentricity,
                self.rotation,
                self.stellate_size,
                self.rounding
            )
        }
    }
}

//ip Default
impl std::default::Default for Polygon {
    fn default() -> Self {
        Self {
            center: Point::default(),
            vertices: 0,
            rotation: 0.,
            rounding: 0.,
            size: 0.,
            eccentricity: 1.,
            stellate_size: 0.,
        }
    }
}

//ti Polygon
impl Polygon {
    //fp new
    /// Create a new Polygon with a certain number of vertices and a
    /// stellation size The polygon has a default of rotation of 0,
    /// size of 0, and rounding of 0, with no eccentricity
    pub fn new(vertices: usize, stellate_size: f64) -> Self {
        Self::default()
            .set_vertices(vertices)
            .set_stellate_size(stellate_size)
    }

    //fp new_rect
    /// Create a new rectangle of a certain width and height, with no
    /// rotation nor rounding
    pub fn new_rect(w: f64, h: f64) -> Self {
        Self::default()
            .set_vertices(4)
            .set_size(h / (2.0_f64.sqrt()), w / h)
    }

    //fp new_polygon
    /// Create a new polygon with a certain number of vertices and size, with a given rotation and rounding
    pub fn new_polygon(vertices: usize, size: f64, rotation: f64, rounding: f64) -> Self {
        Self::default()
            .set_vertices(vertices)
            .set_size(size, 1.0)
            .set_rotation(rotation)
            .set_rounding(rounding)
    }

    //fp new_star
    /// Create a new star with a certain number of points
    pub fn new_star(vertices: usize, size: f64, in_out: f64, rotation: f64, rounding: f64) -> Self {
        Self::default()
            .set_vertices(vertices)
            .set_size(size, 1.0)
            .set_rotation(rotation)
            .set_rounding(rounding)
            .set_stellate_size(size * in_out)
    }

    //fp new_circle
    /// Create a new circle of a certain radius
    pub fn new_circle(r: f64) -> Self {
        Self::default().set_vertices(0).set_size(r, 1.0)
    }

    //fp new_ellipse
    /// Create a new ellipse with two radii at a particular rotation
    pub fn new_ellipse(rx: f64, ry: f64, rotation: f64) -> Self {
        Self::default()
            .set_vertices(0)
            .set_rotation(rotation)
            .set_size(ry, rx / ry)
    }

    //bp set_vertices
    /// Set the number of vertices (note that 0 makes it circular)
    pub fn set_vertices(mut self, vertices: usize) -> Self {
        self.vertices = vertices;
        self
    }

    //bp set_size
    /// Set the size and eccentricity (effectively width and height)
    pub fn set_size(mut self, size: f64, eccentricity: f64) -> Self {
        self.size = size;
        self.eccentricity = eccentricity;
        self
    }

    //bp set_rounding
    /// Set the rounding of the corners
    pub fn set_rounding(mut self, rounding: f64) -> Self {
        self.rounding = rounding;
        self
    }

    //bp set_rotation
    /// Set the rotation
    pub fn set_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    //bp set_stellate_size
    /// Set the stellation of a polygon (not useful for circles...!)
    pub fn set_stellate_size(mut self, stellate_size: f64) -> Self {
        self.stellate_size = stellate_size;
        self
    }

    //mp as_paths
    /// Append the polygon as a set of Beziers
    pub fn as_paths(&self) -> BezierPath {
        match self.vertices {
            0 => BezierPath::of_ellipse(Point::zero(), self.size, self.eccentricity, self.rotation),
            1 => BezierPath::default(),
            _ => {
                let corners = self.get_points();
                BezierPath::of_points(&corners, self.rounding)
            }
        }
    }

    //mp get_bbox
    /// Get the bounding box for the polygon (it may be pessimistic)
    /*
    pub fn get_bbox(&self) -> Rectangle {
        match self.vertices {
            0 => Rectangle::new(
                -self.size * self.eccentricity,
                -self.size,
                self.size * self.eccentricity,
                self.size,
            ),
            1 => Rectangle::new(
                self.center[0],
                self.center[1],
                self.center[0],
                self.center[1],
            ),
            _ => Rectangle::bbox_of_points(&self.get_points()),
        }
    }*/
    //mp get_points
    /// Get the points that make up the corners of the polygon, in
    /// anticlockwise order
    fn get_points(&self) -> Vec<Point> {
        assert!(self.vertices > 1);
        let origin = Point::zero();
        let mut corners = Vec::new();
        let delta_angle = (360.0f64).to_radians() / (self.vertices as f64);
        for i in 0..self.vertices {
            let mut p = Point::from_array([self.size, 0.]).rotate_around(
                &origin,
                delta_angle * (0.5 - (i as f64)),
                0,
                1,
            );
            p[0] *= self.eccentricity;
            p = p.rotate_around(&origin, self.rotation.to_radians(), 0, 1) + self.center;
            corners.push(p);

            if self.stellate_size != 0. {
                let mut p = Point::from_array([self.stellate_size, 0.]).rotate_around(
                    &origin,
                    delta_angle * (0.0 - (i as f64)),
                    0,
                    1,
                );
                p[0] *= self.eccentricity;
                p = p.rotate_around(&origin, self.rotation.to_radians(), 0, 1) + self.center;
                corners.push(p);
            }
        }
        // println!("{:?} {}",corners, self.stellate_size);
        corners
    }

    //zz All done
}

//ip std::ops::Add<Point> for Polygon
impl std::ops::Add<Point> for Polygon {
    type Output = Self;
    #[inline]
    fn add(mut self, dxy: Point) -> Self {
        self.center += dxy;
        self
    }
}

//ip std::ops::Sub<Point> for Polygon
impl std::ops::Sub<Point> for Polygon {
    type Output = Self;
    #[inline]
    fn sub(mut self, dxy: Point) -> Self {
        self.center += dxy;
        self
    }
}

//ip std::ops::Mul<f64> for Polygon
impl std::ops::Mul<f64> for Polygon {
    type Output = Self;
    #[inline]
    fn mul(mut self, scale: f64) -> Self {
        self.size *= scale;
        self
    }
}

//ip std::ops::Div<f64> for Polygon
impl std::ops::Div<f64> for Polygon {
    type Output = Self;
    #[inline]
    fn div(mut self, scale: f64) -> Self {
        self.size /= scale;
        self
    }
}
//a Test
#[cfg(test)]
mod tests_polygon {
    use super::super::Bezier;
    use super::*;
    pub fn pt_eq(pt: &Point, x: f64, y: f64) {
        assert!(
            (pt[0] - x).abs() < 1E-7,
            "mismatch in x {:?} {} {}",
            pt,
            x,
            y
        );
        assert!(
            (pt[1] - y).abs() < 1E-7,
            "mismatch in y {:?} {} {}",
            pt,
            x,
            y
        );
    }
    pub fn bezier_eq(bez: &Bezier, v: Vec<(f64, f64)>) {
        if bez.is_cubic() {
            pt_eq(bez.borrow_pt(0), v[0].0, v[0].1);
            pt_eq(bez.borrow_pt(2), v[1].0, v[1].1);
            pt_eq(bez.borrow_pt(3), v[2].0, v[2].1);
            pt_eq(bez.borrow_pt(1), v[3].0, v[3].1);
        } else if bez.is_quadratic() {
            pt_eq(bez.borrow_pt(0), v[0].0, v[0].1);
            pt_eq(bez.borrow_pt(2), v[1].0, v[1].1);
            pt_eq(bez.borrow_pt(1), v[2].0, v[2].1);
        } else {
            pt_eq(bez.borrow_pt(0), v[0].0, v[0].1);
            pt_eq(bez.borrow_pt(1), v[1].0, v[1].1);
        }
    }
    #[test]
    fn test_circle() {
        let x = Polygon::new_circle(1.0);
        let mut v = Vec::new();
        for b in x.as_paths().iter_beziers() {
            v.push(b.clone());
        }
        let magic = 0.5571469;
        bezier_eq(&v[0], vec![(1., 0.), (1., magic), (magic, 1.), (0., 1.)]);
        bezier_eq(&v[1], vec![(0., 1.), (-magic, 1.), (-1., magic), (-1., 0.)]);
        bezier_eq(
            &v[2],
            vec![(-1., 0.), (-1., -magic), (-magic, -1.), (0., -1.)],
        );
        bezier_eq(&v[3], vec![(0., -1.), (magic, -1.), (1., -magic), (1., 0.)]);
    }
}
