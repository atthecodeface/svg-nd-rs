//a Imports
use crate::{Attributes, BBox, Config, Element, LayoutElement, PreLayoutElement};

//a Group
//tp Group
#[derive(Debug)]
pub struct Group<A: Attributes> {
    attr: A,
    contents: Vec<Element<A>>,
}

//ip Group
impl<A> Group<A>
where
    A: Attributes,
{
    //fp new
    pub fn new<I>(contents: I) -> Self
    where
        I: Iterator<Item = Element<A>>,
    {
        let contents: Vec<Element<A>> = contents.collect();
        let mut attr = A::default();
        let mut bbox = BBox::none();
        for c in contents.iter() {
            bbox = bbox.union(PreLayoutElement::attr(c).bbox());
        }
        attr.set_bbox(bbox);
        Self { attr, contents }
    }

    //zz All done
}

//ip PreLayoutElement for Group
impl<A> PreLayoutElement<A> for Group<A>
where
    A: Attributes,
{
    type LayoutElement = Group<A>;
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

//ip LayoutElement for Group
impl<A> LayoutElement<A> for Group<A>
where
    A: Attributes,
{
    fn attr(&self) -> &A {
        &self.attr
    }
    fn finalize(&mut self, _cfg: &dyn Config) {}
}
