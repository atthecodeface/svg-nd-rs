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

@file    bezier_path.rs
@brief   Part of geometry library
 */

//a Imports
use geo_nd::Vector;

use crate::{Bezier, Point};

//a BezierPath
//tp BezierPath
/// A path is a set of Beziers that form a chain (i.e. the end of one is the start of the next)
#[derive(Debug, Clone, Default)]
pub struct BezierPath {
    elements: Vec<Bezier>,
}

//ip BezierPath
impl BezierPath {
    //fp of_ellipse
    /// Create a set of paths that make an ellipse
    pub fn of_ellipse(origin: Point, radius: f64, eccentricity: f64, degrees: f64) -> Self {
        let ra = (90.0f64).to_radians();
        let rd = degrees.to_radians();
        let x = Point::from_array([eccentricity, 0.]);
        let y = Point::from_array([0., 1.]);
        let mut v = Vec::new();
        v.push(Bezier::arc(ra, radius, &origin, &x, &y, rd));
        v.push(Bezier::arc(ra, radius, &origin, &y, &(-x), rd));
        v.push(Bezier::arc(ra, radius, &origin, &(-x), &(-y), rd));
        v.push(Bezier::arc(ra, radius, &origin, &(-y), &x, rd));
        Self { elements: v }
    }

    //fp of_points
    /// Generate a set of Beziers that join the corners
    pub fn of_points(corners: &[Point], rounding: f64) -> Self {
        let mut bp = Self::default();
        let n = corners.len();
        for i in 0..n {
            let i_0 = (i) % n;
            let i_1 = (i + 1) % n;
            bp.add_bezier(Bezier::line(&corners[i_0], &corners[i_1]));
        }
        bp.round(rounding, true);
        bp
    }

    //mp round
    /// Run through the path; for every adjacent pair of Beziers that
    /// are *line*s add an intermediate Bezier that is a rounded
    /// corner of the correct radius.
    ///
    /// If the path is closed, thenn treat the first Bezier as
    /// adjacent to the last Bezier
    pub fn round(&mut self, rounding: f64, closed: bool) {
        let mut n = self.elements.len();
        if n < 2 || rounding == 0. {
            return;
        }
        let mut i = n - 1;
        if !closed {
            i -= 1;
        }
        loop {
            let i_1 = (i + 1) % n;
            if self.elements[i].is_line() && self.elements[i_1].is_line() {
                let corner = self.elements[i].borrow_pt(1); // same as i_1.borrow_pt(0);
                let v0 = self.elements[i].tangent_at(1.);
                let v1 = -self.elements[i_1].tangent_at(0.);
                let bezier = Bezier::of_round_corner(&corner, &v0, &v1, rounding);
                let np00 = self.elements[i].borrow_pt(0).clone();
                let np01 = bezier.borrow_pt(0).clone();
                let np10 = bezier.borrow_pt(1).clone();
                let np11 = self.elements[i_1].borrow_pt(1).clone();
                self.elements[i] = Bezier::line(&np00, &np01);
                self.elements[i_1] = Bezier::line(&np10, &np11);
                self.elements.insert(i + 1, bezier);
                n += 1; // Not really required but it keeps n == self.elements.len()
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
    }

    //mp get_pt
    /// Get the start or the end point
    pub fn get_pt(&self, index: usize) -> Point {
        let n = self.elements.len();
        if n == 0 {
            Point::zero()
        } else if index == 0 {
            self.elements[0].borrow_pt(0).clone()
        } else {
            self.elements[n - 1].borrow_pt(1).clone()
        }
    }

    //mp add_bezier
    /// Add a Bezier at the end of the path
    pub fn add_bezier(&mut self, b: Bezier) {
        self.elements.push(b);
    }

    //mp apply_relief
    /// Attempt to remove `distance` from the start or end of the path
    /// but leave rest of path the same
    pub fn apply_relief(&mut self, index: usize, straightness: f64, distance: f64) {
        if self.elements.is_empty() {
            return;
        }
        if index == 0 {
            let b = self.elements[0];
            let l = b.length(straightness);
            // println!("Applying relief to start of bezier path {} straightness {} distance {} length {}",b, straightness, distance, l);
            if distance > l {
                self.elements.remove(0);
                self.apply_relief(index, straightness, distance - l)
            } else {
                let (t, _in_bezier) = b.t_of_distance(straightness, distance);
                if t == 0. {
                    ()
                } else if t == 1. {
                    self.elements.remove(0);
                    ()
                } else {
                    self.elements[0] = b.bezier_between(t, 1.);
                    ()
                }
            }
        } else {
            let n = self.elements.len();
            let b = self.elements[n - 1];
            let l = b.length(straightness);
            // println!("Applying relief to end of bezier path {} straightness {} distance {} length {}",b, straightness, distance, l);
            if distance > l {
                self.elements.pop();
                self.apply_relief(index, straightness, distance - l)
            } else {
                let (t, _in_bezier) = b.t_of_distance(straightness, l - distance);
                if t == 0. {
                    self.elements.pop();
                    ()
                } else if t == 1. {
                    ()
                } else {
                    self.elements[n - 1] = b.bezier_between(0., t);
                    ()
                }
            }
        }
    }

    //mp iter_beziers
    /// Iterate through all the Beziers
    pub fn iter_beziers(&self) -> impl Iterator<Item = &Bezier> {
        self.elements.iter()
    }
}

//ip std::ops::Index<Idx>
impl<Idx> std::ops::Index<Idx> for BezierPath
where
    Idx: std::slice::SliceIndex<[Bezier]>,
{
    type Output = <Idx as std::slice::SliceIndex<[Bezier]>>::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.elements[index]
    }
}

//a Test
#[cfg(test)]
mod test_path {
    use super::*;
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
            "mismatch in x {:?} {} {}",
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
    fn test_round_open() {
        let p0 = Point::zero();
        let p1 = Point::from_array([1., 0.]);
        let p2 = Point::from_array([1., 1.]);
        let p3 = Point::from_array([0., 1.]);
        let mut bp = BezierPath::default();
        bp.add_bezier(Bezier::line(&p0, &p1));
        bp.add_bezier(Bezier::line(&p1, &p2));
        bp.add_bezier(Bezier::line(&p2, &p3));
        bp.add_bezier(Bezier::line(&p3, &p0));

        // After open rounding of 0.1 the straight lines are 0.1-0.9 on each side except first and last
        // and 7 elements with round corners except at origin
        bp.round(0.1, false);
        for b in bp.iter_beziers() {
            println!("Bezier {}", b);
        }
        bezier_eq(&bp.elements[0], vec![(0., 0.), (0.9, 0.0)]);
        bezier_eq(&bp.elements[2], vec![(1., 0.1), (1., 0.9)]);
        bezier_eq(&bp.elements[4], vec![(0.9, 1.0), (0.1, 1.)]);
        bezier_eq(&bp.elements[6], vec![(0., 0.9), (0., 0.)]);
        assert_eq!(bp.elements.len(), 7, "Path should be 7 elements");
    }
    #[test]
    fn test_round_closed() {
        let p0 = Point::zero();
        let p1 = Point::from_array([1., 0.]);
        let p2 = Point::from_array([1., 1.]);
        let p3 = Point::from_array([0., 1.]);
        let mut bp = BezierPath::default();
        bp.add_bezier(Bezier::line(&p0, &p1));
        bp.add_bezier(Bezier::line(&p1, &p2));
        bp.add_bezier(Bezier::line(&p2, &p3));
        bp.add_bezier(Bezier::line(&p3, &p0));

        // After closed rounding of 0.1 the straight lines are 0.1-0.9 on each side
        // and 8 elements
        bp.round(0.1, true);
        for b in bp.iter_beziers() {
            println!("Bezier {}", b);
        }
        bezier_eq(&bp.elements[0], vec![(0.1, 0.), (0.9, 0.0)]);
        bezier_eq(&bp.elements[2], vec![(1., 0.1), (1., 0.9)]);
        bezier_eq(&bp.elements[4], vec![(0.9, 1.0), (0.1, 1.)]);
        bezier_eq(&bp.elements[6], vec![(0., 0.9), (0., 0.1)]);
        assert_eq!(bp.elements.len(), 8, "Path should be 8 elements");
    }
}
