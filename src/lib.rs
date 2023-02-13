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

@file    lib.rs
@brief   Generate SVG output
 */

//a Imports
//a Imports and exports
// mod point;
// mod range;
// mod path;
// mod rectangle;
// mod polygon;
mod transform;

/// The [Point] type is a 2D point of f64's
pub type Point = geo_nd::FArray<f64, 2>;

/// The [Bezier] type is a Bezier curve of [Point]s
pub type Bezier = bezier_nd::Bezier<f64, Point, 2>;

// pub use self::transform::Transform;
// pub use self::range::{Range};
// pub use self::path::{BezierPath};
// pub use self::rectangle::{Rectangle, Float4};
// pub use self::polygon::Polygon;

// mod generate_svg;
// mod svg;
// mod svg_element;
// mod svg_element_iter;
mod svg_error;

// pub use generate_svg::{GenerateSvg, GenerateSvgElement};
// pub use svg::Svg;
// pub use svg_element::SvgElement;
// pub use svg_element_iter::ElementIter;
pub use svg_error::SvgError;
