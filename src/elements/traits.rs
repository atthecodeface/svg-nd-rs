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

@file    traits.rs
@brief   Trairs for elements
 */

//a Imports
use crate::{BBox, Transform};

//a Traits
//tt Config
pub trait Config {}

//tt Attributes
/// A trait that can be supported in common for an element to access
/// and support modification of the important attributes as regarded
/// by a diagram
pub trait Attributes: std::default::Default + Sized + std::fmt::Debug {
    fn bbox(&self) -> BBox;
    fn set_bbox(&mut self, bbox: BBox);
    fn transform(&self) -> Transform;
    fn set_transform(&mut self, transform: Transform);
}

//tt PreLayoutElement
/// A trait required by an element pre-layout, which allows access to
/// its attributes *for modification*, and provides a methods to layout the element with
/// a configuration within a bounding box
///
/// This produces a resultant laid-out element type
///
/// Change to produce a Box<dyn<LayoutElement> + 'static>
pub trait PreLayoutElement<A: Attributes> {
    type LayoutElement: Sized + std::fmt::Debug;

    fn attr_mut(&mut self) -> &mut A;
    fn attr(&self) -> &A;

    /// Finalize
    fn layout(self, _cfg: &dyn Config, within: &BBox) -> Self::LayoutElement;
}

//tt LayoutElement
/// A trait required by an element after layout; the attributes at this point are fixed, as is the bounding box for the element.
///
/// The element itself may have finalization work? Why?
///
/// This produces a resultant laid-out element type
pub trait LayoutElement<A: Attributes>: Sized + std::fmt::Debug {
    fn attr(&self) -> &A;
    /// Finalize
    fn finalize(&mut self, _cfg: &dyn Config);
}
