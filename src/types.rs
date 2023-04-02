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

@file    types.rs
@brief   Base types for the SVG library
 */

//a Imports
use indent_display::IndentedOptions;
pub struct IndentOpt();
impl<'a> IndentedOptions<'a> for IndentOpt {}

/// The [Point] type is a 2D point of f64's
pub type Point = geo_nd::FArray<f64, 2>;

/// The [Bezier] type is a Bezier curve of [Point]s
pub type Bezier = bezier_nd::Bezier<f64, Point, 2>;

mod bbox;
mod error;
mod range;
mod transform;
use crate::Attributes;
pub use bbox::BBox;
pub use error::Error;
pub use range::Range;
pub use transform::Transform;

#[derive(Debug, Default)]
pub struct Attr {
    bbox: BBox,
    transform: Transform,
}
impl Attributes for Attr {
    fn bbox(&self) -> BBox {
        self.bbox
    }
    fn set_bbox(&mut self, bbox: BBox) {
        self.bbox = bbox;
    }
    fn transform(&self) -> Transform {
        self.transform
    }
    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }
}
