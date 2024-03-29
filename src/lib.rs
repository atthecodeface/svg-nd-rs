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
mod types;
pub use types::{Attr, BBox, Bezier, Error, IndentOpt, Point, Range, Transform};

mod colors;
pub use colors::{Color, ColorDatabase};

mod shapes;
pub use shapes::{BezierPath, Polygon};

mod traits;

mod namespace;
pub use namespace::NamespaceName;

pub use traits::{CreateSvg, SvgAttributes};
mod elements;
pub use elements::{Attributes, Config, LayoutElement, PreLayoutElement};
pub use elements::{Element, Group, Path};

pub struct Svg {}
pub struct SvgElement {}
