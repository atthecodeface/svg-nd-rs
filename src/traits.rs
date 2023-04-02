use crate::{Attributes, BBox, LayoutElement, Svg, SvgElement, Transform};

pub trait SvgAttributes: Attributes {
    fn add_attributes(&self, svg: &mut Svg, element: &mut SvgElement);
}

pub trait CreateSvg<A>: LayoutElement<A> + std::fmt::Debug
where
    A: SvgAttributes,
{
    /// Create SVG
    fn create_svg(&self, svg: &mut Svg);
}
