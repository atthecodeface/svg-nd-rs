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
    fn new(name: &str, ns: Option<&str>) -> Self {
        let name = name.into();
        let ns = ns.map(|ns| ns.into());
        Self { name, ns }
    }
}

//a SvgElementType
pub trait SvgElementType: std::fmt::Debug {
    /// Get the SVG element name (e.g. 'path')
    fn ns_name(&self) -> NamespaceName;

    /// Finalize
    fn finalize(&mut self, _svg_cfg: &SvgConfig, _contents: &[SvgElement], _characters: &str) {}

    /// Get the bbox of the element (not its explicit contents) post-finalize
    ///
    /// The bbox will be extended by that of its contents
    #[must_use]
    fn bbox(&self) -> BBox {
        BBox::none()
    }

    /// Push the attributes when ready for rendering as SVG (post-finalize)
    fn push_attributes(&self, attrs: &mut Vec<(NamespaceName, String)>) {}
}

//a SvgElementTypes
//tp SvgSvg
#[derive(Debug)]
pub struct SvgSvg();

//ip SvgSvg
impl SvgSvg {
    pub fn new() -> SvgElement {
        let g = Self();
        SvgElement::new("svg", g)
    }
}

//ip SvgElementType for SvgSvg
impl SvgElementType for SvgSvg {
    fn ns_name(&self) -> NamespaceName {
        NamespaceName::local("svg")
    }
}

//tp SvgGroup
#[derive(Debug)]
pub struct SvgGroup();

//ip SvgGroup
impl SvgGroup {
    pub fn new() -> SvgElement {
        let g = Self();
        SvgElement::new("g", g)
    }
}

//ip SvgElementType for SvgGroup
impl SvgElementType for SvgGroup {
    fn ns_name(&self) -> NamespaceName {
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
    pub fn new_path(bp: BezierPath, closed: bool) -> SvgElement {
        let p = Self { path: bp, closed };
        SvgElement::new("path", p)
    }

    //fp new_box
    pub fn new_box(bbox: BBox) -> SvgElement {
        let (c, w, h) = bbox.get_cwh();
        let rect = Polygon::new_rect(w, h) + c;
        Self::new_polygon(rect, true)
    }

    //fp new_polygon
    pub fn new_polygon(p: Polygon, closed: bool) -> SvgElement {
        Self::new_path(p.as_paths(), closed)
    }
}

//ip SvgElementType for SvgPath
impl SvgElementType for SvgPath {
    fn ns_name(&self) -> NamespaceName {
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
    fn push_attributes(&self, attrs: &mut Vec<(NamespaceName, String)>) {
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
    pub fn new(bbox: BBox, spacings: (f64, f64)) -> SvgElement {
        let p = Self { spacings, bbox };
        SvgElement::new("grid", p)
    }
}

//ip SvgElementType for SvgGrid
impl SvgElementType for SvgGrid {
    fn ns_name(&self) -> NamespaceName {
        NamespaceName::local("path")
    }
    fn bbox(&self) -> BBox {
        self.bbox
    }
    /// Push the attributes when ready for rendering as SVG (post-finalize)
    fn push_attributes(&self, attrs: &mut Vec<(NamespaceName, String)>) {
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
pub struct SvgElement {
    ele_type: Box<dyn SvgElementType>,
    pub name: String,
    pub prefix: Option<String>,
    attributes: Vec<(NamespaceName, String)>,
    transform: Transform,
    contents: Vec<SvgElement>,
    characters: String,
    bbox: BBox,
}

//ip IndentedDisplay for SvgElement
impl<'a> IndentedDisplay<'a, IndentOpt> for SvgElement {
    fn indent(&self, f: &mut Indenter<'a, IndentOpt>) -> Result<(), std::fmt::Error> {
        use std::fmt::Write;
        if let Some(prefix) = &self.prefix {
            write!(f, "{}::{}", prefix, self.name)?;
        } else {
            write!(f, "{}", self.name)?;
        }
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
impl SvgElement {
    //fp new
    /// Create a new SvgElement from something that only contains static references
    pub fn new<I: Into<String>, E: SvgElementType + 'static>(name: I, ele_type: E) -> Self {
        let ele_type = Box::new(ele_type);
        Self {
            ele_type,
            name: name.into(),
            prefix: None,
            attributes: Vec::new(),
            transform: Transform::default(),
            contents: Vec::new(),
            characters: String::new(),
            bbox: BBox::default(),
        }
    }

    //ap name
    pub fn name(&self) -> &str {
        &self.name
    }

    //cp set_prefix
    pub fn set_prefix<I: Into<String>>(mut self, prefix: Option<I>) {
        self.prefix = prefix.map(|s| s.into())
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
    pub fn contents(&self) -> &[SvgElement] {
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
    pub fn add_attribute(&mut self, name: &str, prefix: Option<&str>, value: &str) {
        let ns_name = NamespaceName::new(name, prefix);
        self.attributes
            .push((name.into(), prefix.map(|s| s.into()), value.into()));
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
    pub fn add_size(&mut self, name: &str, value: f64) {
        self.add_attribute(name, None, &format!("{:.4}", value));
    }

    //fp add_color
    pub fn add_color<'a, T>(&mut self, attr_name: &str, color: T)
    where
        (T, &'a ColorDatabase<'a>): Into<Color>,
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
    pub fn push_content(&mut self, e: SvgElement) {
        self.contents.push(e);
    }

    //mp finalize
    pub fn finalize(&mut self, svg_cfg: &SvgConfig) -> Vec<SvgElement> {
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
    pub fn new_grid(bbox: BBox, spacing: f64, line_width: f64, color: &str) -> Option<Self> {
        let mut grid = SvgGrid::new(bbox, (spacing, spacing));
        grid.add_attribute("fill", None, "None");
        grid.add_attribute("stroke", None, color);
        grid.add_attribute("stroke-width", None, &format!("{}", line_width));
        Some(grid)
    }

    //fp display
    pub fn display(&self, indent: usize) {
        let indent_str = &INDENT_STRING[0..indent];
        println!("{}{}", indent_str, self.name);
        for (n, _p, v) in &self.attributes {
            println!("{}      {}={}", indent_str, n, v);
        }
        for e in &self.contents {
            e.display(indent + 2);
        }
    }

    //zz All done
}
