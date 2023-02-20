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

@file    svg_element.rs
@brief   An element of an SVG diagram
 */

//a Imports
use std::borrow::Cow;

use indent_display::{IndentedDisplay, Indenter};

use crate::IndentOpt;
use crate::{BBox, BezierPath, Color, ColorDatabase, Point, Polygon, Transform};
use crate::{SvgColorDatabase, SvgConfig};

//a Useful stuff
fn pt_as_str(pt: &Point) -> String {
    format!("{:.4},{:.4}", pt[0], pt[1])
}
const INDENT_STRING: &str = "                                                            ";

//a NamespaceName
pub struct NamespaceName<'a> {
    name: Cow<'a, str>,
    ns: Option<Cow<'a, str>>,
}

//ip NamespaceName
impl<'a> NamespaceName<'a> {
    fn local<I: Into<Cow<'a, str>>>(name: I) -> Self {
        let name = name.into();
        Self { name, ns: None }
    }
    fn new(name: &'a str, ns: Option<&'a str>) -> Self {
        let name = name.into();
        let ns = ns.map(|ns| ns.into());
        Self { name, ns }
    }
}
impl<'a> std::fmt::Display for NamespaceName<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        if let Some(ns) = &self.ns {
            write!(fmt, "{}:{}", ns, self.name)
        } else {
            self.name.fmt(fmt)
        }
    }
}
impl<'a> std::fmt::Debug for NamespaceName<'a> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        std::fmt::Display::fmt(self, fmt)
    }
}

//a SvgElementType
pub trait SvgElementType<'a>: std::fmt::Debug {
    /// Get the SVG element name (e.g. 'path')
    fn ns_name(&self) -> NamespaceName<'a>;

    /// Finalize
    fn finalize(&mut self, _svg_cfg: &SvgConfig, _contents: &[SvgElement<'a>], _characters: &str) {}

    /// Get the bbox of the element (not its explicit contents) post-finalize
    ///
    /// The bbox will be extended by that of its contents
    #[must_use]
    fn bbox(&self) -> BBox {
        BBox::none()
    }

    /// Push the attributes when ready for rendering as SVG (post-finalize)
    fn push_attributes(&self, attrs: &mut Vec<(NamespaceName<'a>, String)>) {}
}

//a SvgElementTypes
//tp SvgSvg
#[derive(Debug)]
pub struct SvgSvg();

//ip SvgSvg
impl SvgSvg {
    pub fn new<'a>() -> SvgElement<'a> {
        let g = Self();
        SvgElement::new(g)
    }
}

//ip SvgElementType for SvgSvg
impl<'a> SvgElementType<'a> for SvgSvg {
    fn ns_name(&self) -> NamespaceName<'a> {
        NamespaceName::local("svg")
    }
}

//tp SvgGroup
#[derive(Debug)]
pub struct SvgGroup();

//ip SvgGroup
impl SvgGroup {
    pub fn new<'a>() -> SvgElement<'a> {
        let g = Self();
        SvgElement::new(g)
    }
}

//ip SvgElementType for SvgGroup
impl<'a> SvgElementType<'a> for SvgGroup {
    fn ns_name(&self) -> NamespaceName<'a> {
        NamespaceName::local("g")
    }
}

//tp SvgPath
#[derive(Debug)]
pub struct SvgPath {
    path: BezierPath,
    closed: bool,
}

//ip SvgPath
impl SvgPath {
    //fp new_path
    pub fn new_path<'a>(bp: BezierPath, closed: bool) -> SvgElement<'a> {
        let p = Self { path: bp, closed };
        SvgElement::new(p)
    }

    //fp new_box
    pub fn new_box<'a>(bbox: BBox) -> SvgElement<'a> {
        let (c, w, h) = bbox.get_cwh();
        let rect = Polygon::new_rect(w, h) + c;
        Self::new_polygon(rect, true)
    }

    //fp new_polygon
    pub fn new_polygon<'a>(p: Polygon, closed: bool) -> SvgElement<'a> {
        Self::new_path(p.as_paths(), closed)
    }
}

//ip SvgElementType for SvgPath
impl<'a> SvgElementType<'a> for SvgPath {
    fn ns_name(&self) -> NamespaceName<'a> {
        NamespaceName::local("path")
    }
    fn bbox(&self) -> BBox {
        let mut bbox = BBox::none();
        for b in self.path.iter_beziers() {
            for p in b.as_points(0.1) {
                bbox = bbox.include(p);
            }
        }
        bbox
    }
    /// Push the attributes when ready for rendering as SVG (post-finalize)
    fn push_attributes(&self, attrs: &mut Vec<(NamespaceName<'a>, String)>) {
        let mut r = String::new();
        r.push_str(&format!("M {}", pt_as_str(&self.path.get_pt(0))));
        for b in self.path.iter_beziers() {
            if b.degree() == 1 {
                r.push_str(&format!(" L {}", pt_as_str(b.borrow_pt(1))))
            } else if b.degree() == 2 {
                r.push_str(&format!(
                    " Q {} {}",
                    pt_as_str(b.borrow_pt(2)),
                    pt_as_str(b.borrow_pt(1))
                ));
            } else {
                r.push_str(&format!(
                    " C {} {} {}",
                    pt_as_str(b.borrow_pt(2)),
                    pt_as_str(b.borrow_pt(3)),
                    pt_as_str(b.borrow_pt(1))
                ));
            }
        }
        if self.closed {
            r.push_str(" z");
        }
        attrs.push((NamespaceName::local("d"), r));
    }
}

//tp SvgGrid
/// An [SvgGrid] is generally an artefact; it is created to show the
/// grid of some SVG contents.
///
/// Possibly it optionally should be a kind of group which then adds a
/// grid path as an artefact on finalization.
#[derive(Debug)]
pub struct SvgGrid {
    /// X and Y spacing of grid lines in mm
    ///
    /// grid lines are placed at integer multiples of the spacing
    spacings: (f64, f64),
    /// Bounding box of grid
    bbox: BBox,
}

//ip SvgGrid
impl SvgGrid {
    //fp new
    pub fn new<'a>(bbox: BBox, spacings: (f64, f64)) -> SvgElement<'a> {
        let p = Self { spacings, bbox };
        SvgElement::new(p)
    }
}

//ip SvgElementType for SvgGrid
impl<'a> SvgElementType<'a> for SvgGrid {
    fn ns_name(&self) -> NamespaceName<'a> {
        NamespaceName::local("path")
    }
    fn bbox(&self) -> BBox {
        self.bbox
    }
    /// Push the attributes when ready for rendering as SVG (post-finalize)
    fn push_attributes(&self, attrs: &mut Vec<(NamespaceName<'a>, String)>) {
        let xmin = ((self.bbox.x[0] / self.spacings.0) + 0.).floor() as isize;
        let xmax = ((self.bbox.x[1] / self.spacings.0) + 1.).floor() as isize;
        let xlen = xmax - xmin;

        let ymin = ((self.bbox.y[0] / self.spacings.1) + 0.).floor() as isize;
        let ymax = ((self.bbox.y[1] / self.spacings.1) + 1.).floor() as isize;
        let ylen = ymax - ymin;

        let mut r = String::new();
        if xlen >= 2 {
            for i in xmin..=xmax {
                let coord = (i as f64) * self.spacings.0;
                r.push_str(&format!(
                    "M {},{} v {} ",
                    coord,
                    self.bbox.y[0],
                    self.bbox.y[1] - self.bbox.y[0]
                ));
            }
        }
        if ylen >= 2 {
            for i in ymin..ymax + 1 {
                let coord = (i as f64) * self.spacings.1;
                r.push_str(&format!(
                    "M {},{} h {} ",
                    self.bbox.x[0],
                    coord,
                    self.bbox.x[1] - self.bbox.x[0]
                ));
            }
        }

        attrs.push((NamespaceName::local("d"), r));
    }
}

//a SvgElement
//tp SvgElement
#[derive(Debug)]
pub struct SvgElement<'a> {
    ele_type: Box<dyn SvgElementType<'a> + 'a>,
    attributes: Vec<(NamespaceName<'a>, String)>,
    transform: Transform,
    contents: Vec<SvgElement<'a>>,
    characters: String,
    bbox: BBox,
}

//ip IndentedDisplay for SvgElement
impl<'a, 'i> IndentedDisplay<'i, IndentOpt> for SvgElement<'a> {
    fn indent(&self, f: &mut Indenter<'i, IndentOpt>) -> Result<(), std::fmt::Error> {
        use std::fmt::Write;
        write!(f, "{}", self.ele_type.ns_name())?;
        if !self.transform.is_identity() {
            write!(f, " {}", self.transform)?;
        }
        writeln!(f)?;
        {
            let mut sub = f.push("...");
            for c in self.contents.iter() {
                c.indent(&mut sub)?;
            }
        }
        Ok(())
    }
}

//ip SvgElement
impl<'a> SvgElement<'a> {
    //fp new
    /// Create a new SvgElement from something that only contains static references
    pub fn new<E: SvgElementType<'a> + 'a>(ele_type: E) -> Self {
        let ele_type = Box::new(ele_type);
        Self {
            ele_type,
            attributes: Vec::new(),
            transform: Transform::default(),
            contents: Vec::new(),
            characters: String::new(),
            bbox: BBox::default(),
        }
    }

    //ap ns_name
    pub fn ns_name(&self) -> NamespaceName<'a> {
        self.ele_type.ns_name()
    }

    //ap attributes
    pub fn attributes(&self) -> &[(NamespaceName, String)] {
        &self.attributes
    }

    //ap characters
    pub fn characters(&self) -> &str {
        &self.characters
    }

    //ap contents
    pub fn contents(&self) -> &[Self] {
        &self.contents
    }

    //ap transform
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    //ap bbox
    pub fn bbox(&self) -> BBox {
        self.bbox
    }

    //fp add_attribute
    pub fn add_attribute(&mut self, name: &'a str, prefix: Option<&'a str>, value: &str) {
        let ns_name = NamespaceName::new(name, prefix);
        self.attributes.push((ns_name, value.into()));
    }

    //fp push_string
    pub fn push_string(&mut self, s: &str) {
        self.characters.push_str(s);
    }

    //fp apply_transform
    pub fn apply_transform(&mut self, transform: &Transform) {
        self.transform = transform.apply_to_transform(&self.transform);
    }

    //fp transform_inner
    pub fn transform_inner(&mut self, transform: &Transform) {
        self.transform = self.transform.apply_to_transform(transform);
    }

    //fp add_size
    pub fn add_size(&mut self, name: &'a str, value: f64) {
        self.add_attribute(name, None, &format!("{:.4}", value));
    }

    //fp add_color
    pub fn add_color<'c, T>(&mut self, attr_name: &'a str, color: T)
    where
        (T, &'c ColorDatabase<'c>): Into<Color>,
    {
        let color: Color = (color, &SvgColorDatabase).into();
        let color = color.as_str();
        self.add_attribute(attr_name, None, &color);
    }

    //fp add_markers
    pub fn add_markers(&mut self, markers: &(Option<String>, Option<String>, Option<String>)) {
        if let Some(ref s) = markers.0 {
            self.add_attribute("marker-start", None, &format!("url(#{})", s));
        }
        if let Some(ref s) = markers.1 {
            self.add_attribute("marker-mid", None, &format!("url(#{})", s));
        }
        if let Some(ref s) = markers.2 {
            self.add_attribute("marker-end", None, &format!("url(#{})", s));
        }
    }

    //fp push_content
    pub fn push_content(&mut self, e: Self) {
        self.contents.push(e);
    }

    //mp finalize
    pub fn finalize(&mut self, svg_cfg: &SvgConfig) -> Vec<Self> {
        let transform = self.transform.as_svg_attribute_string();

        let mut bbox = BBox::none();
        let mut child_extra = vec![];
        for c in self.contents.iter_mut() {
            child_extra.append(&mut c.finalize(svg_cfg));
            bbox = bbox.union(c.bbox());
        }

        // Children are finalized now
        self.ele_type
            .finalize(svg_cfg, &self.contents, &self.characters);
        self.bbox = bbox.union(self.ele_type.bbox());

        for c in child_extra {
            self.contents.push(c);
        }

        let mut extra = vec![];
        if let Some((width, color)) = &svg_cfg.show_content_rectangles {
            let mut e = Self::new_box(self.bbox, *width, color);
            if !transform.is_empty() {
                e.add_attribute("transform", None, &transform);
            }
            e.ele_type.push_attributes(&mut e.attributes);
            extra.push(e);
        }
        self.bbox = self.bbox.transform(&self.transform);
        if !transform.is_empty() {
            self.add_attribute("transform", None, &transform);
        }
        self.ele_type.push_attributes(&mut self.attributes);
        extra
    }

    //cp new_box
    /// Create a box for a BBox
    pub fn new_box(bbox: BBox, line_width: f64, color: &Color) -> Self {
        let mut e = SvgPath::new_box(bbox);
        e.add_color("fill", "none");
        e.add_color("stroke", color);
        e.add_attribute("stroke-width", None, &format!("{:.4}", line_width));
        e
    }

    //cp new_grid
    /// Create a grid element with given region, spacing, line
    /// width and color
    pub fn new_grid(bbox: BBox, spacing: f64, line_width: f64, color: &str) -> Self {
        let mut grid = SvgGrid::new(bbox, (spacing, spacing));
        grid.add_attribute("fill", None, "None");
        grid.add_attribute("stroke", None, color);
        grid.add_attribute("stroke-width", None, &format!("{}", line_width));
        grid
    }

    //fp display
    pub fn display(&self, indent: usize) {
        let indent_str = &INDENT_STRING[0..indent];
        println!("{}{}", indent_str, self.ele_type.ns_name());
        for (n, v) in &self.attributes {
            println!("{}      {}={}", indent_str, n, v);
        }
        for e in &self.contents {
            e.display(indent + 2);
        }
    }

    //zz All done
}
