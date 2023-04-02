use crate::{Attributes, BBox, BezierPath, Config, LayoutElement, Polygon, PreLayoutElement};
//tp Path
#[derive(Debug)]
pub struct Path<A: Attributes> {
    attr: A,
    path: BezierPath,
    closed: bool,
}

//ip Path
impl<A> Path<A>
where
    A: Attributes,
{
    //fp new_path
    pub fn new_path(path: BezierPath, closed: bool) -> Self {
        let attr = A::default();
        Self { attr, path, closed }
    }

    //fp new_polygon
    pub fn new_polygon(p: Polygon, closed: bool) -> Self {
        Self::new_path(p.as_paths(), closed)
    }

    //fp new_box
    pub fn new_box(bbox: BBox) -> Self {
        let (c, w, h) = bbox.get_cwh();
        let rect = Polygon::new_rect(w, h) + c;
        Self::new_polygon(rect, true)
    }
}

//ip PreLayoutElement for Path
impl<A> PreLayoutElement<A> for Path<A>
where
    A: Attributes,
{
    type LayoutElement = Path<A>;
    fn attr_mut(&mut self) -> &mut A {
        &mut self.attr
    }
    fn attr(&self) -> &A {
        &self.attr
    }
    fn layout(self, _cfg: &dyn Config, _within: &BBox) -> Self::LayoutElement {
        self
    }
}

//ip LayoutElement for Path
impl<A> LayoutElement<A> for Path<A>
where
    A: Attributes,
{
    fn attr(&self) -> &A {
        &self.attr
    }
    fn finalize(&mut self, _cfg: &dyn Config) {}
}
