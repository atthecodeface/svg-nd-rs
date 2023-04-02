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

@file    elements.rs
@brief   Elements of a diagram - shapes, text, groups, definitions, and a diagram container
 */

//a Imports
mod traits;
pub use traits::{Attributes, Config, LayoutElement, PreLayoutElement};
mod group;
mod path;
pub use group::Group;
pub use path::Path;

use crate::BBox;

#[derive(Debug)]
pub enum Element<A: Attributes> {
    Group(Group<A>),
    Path(Path<A>),
}
impl<A> Element<A>
where
    A: Attributes,
{
    pub fn new_group(group: Group<A>) -> Self {
        Self::Group(group)
    }
    pub fn new_path(path: Path<A>) -> Self {
        Self::Path(path)
    }
}
impl<A> PreLayoutElement<A> for Element<A>
where
    A: Attributes,
{
    type LayoutElement = Element<A>;

    fn attr_mut(&mut self) -> &mut A {
        use Element::*;
        match self {
            Group(x) => x.attr_mut(),
            Path(x) => x.attr_mut(),
        }
    }
    fn attr(&self) -> &A {
        use Element::*;
        match self {
            Group(x) => PreLayoutElement::attr(x),
            Path(x) => PreLayoutElement::attr(x),
        }
    }

    /// Finalize
    fn layout(self, cfg: &dyn Config, within: &BBox) -> Self {
        use Element::*;
        match self {
            Group(x) => Group(x.layout(cfg, within)),
            Path(x) => Path(x.layout(cfg, within)),
        }
    }
}

impl<A> LayoutElement<A> for Element<A>
where
    A: Attributes,
{
    fn attr(&self) -> &A {
        use Element::*;
        match self {
            Group(x) => LayoutElement::attr(x),
            Path(x) => LayoutElement::attr(x),
        }
    }
    /// Finalize
    fn finalize(&mut self, cfg: &dyn Config) {
        use Element::*;
        match self {
            Group(x) => x.finalize(cfg),
            Path(x) => x.finalize(cfg),
        }
    }
}

/*
impl<A> CreateSvg<A> for Element<A>
where
    A: SvgAttributes,
{
    /// Create SVG
    fn create_svg(&self, svg: &mut Svg) {
        use Element::*;
        match self {
            Group(x) => x.create_svg(svg),
            Path(x) => x.create_svg(svg),
        }
    }
}
 */
