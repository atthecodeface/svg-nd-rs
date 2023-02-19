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
use indent_display::{IndentedDisplay, Indenter};

use crate::IndentOpt;
use crate::{BBox, BezierPath, Color, ColorDatabase, Point, Polygon, Transform};
use crate::{SvgColorDatabase, SvgConfig};

//a Useful stuff
fn pt_as_str(pt: &Point) -> String {
    format!("{:.4},{:.4}", pt[0], pt[1])
}
const INDENT_STRING: &str = "                                                            ";

//a SvgElement
//tp SvgElement
#[derive(Debug)]
pub struct SvgElement {
    pub name: String,
    pub prefix: Option<String>,
    attributes: Vec<(String, Option<String>, String)>,
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
    pub fn new<I: Into<String>>(name: I) -> Self {
        Self {
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
    pub fn attributes(&self) -> &[(String, Option<String>, String)] {
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

    //fp add_bezier_path
    pub fn add_bezier_path(&mut self, bp: &BezierPath, closed: bool) {
        let mut r = String::new();
        r.push_str(&format!("M {}", pt_as_str(&bp.get_pt(0))));
        for b in bp.iter_beziers() {
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
        if closed {
            r.push_str(" z");
        }
        self.add_attribute("d", None, &r);
        let mut bbox = self.bbox;
        for b in bp.iter_beziers() {
            for p in b.as_points(0.1) {
                bbox = bbox.include(p);
            }
        }
        self.bbox = bbox;
    }

    //fp add_polygon_path
    pub fn add_polygon_path(&mut self, p: &Polygon, closed: bool) {
        self.add_bezier_path(&p.as_paths(), closed);
    }

    //fp push_content
    pub fn push_content(&mut self, e: SvgElement) {
        self.contents.push(e);
    }

    //mp finalize
    pub fn finalize(&mut self, svg_cfg: &SvgConfig) -> Vec<SvgElement> {
        let transform = self.transform.as_svg_attribute_string();
        if !transform.is_empty() {
            self.add_attribute("transform", None, &transform);
        }

        let mut child_extra = vec![];
        for c in self.contents.iter_mut() {
            child_extra.append(&mut c.finalize(svg_cfg));
            self.bbox = self.bbox.union(c.bbox());
        }
        // Children are finalized now
        for c in child_extra {
            self.contents.push(c);
        }
        let mut extra = vec![];
        if let Some((width, color)) = &svg_cfg.show_content_rectangles {
            let mut e = Self::new_box(self.bbox, *width, color);
            e.add_attribute("transform", None, &transform);
            extra.push(e);
        }
        self.bbox = self.bbox.transform(&self.transform);
        extra
    }

    //cp new_box
    /// Create a box for a BBox
    pub fn new_box(bbox: BBox, line_width: f64, color: &Color) -> Self {
        let r = format!(
            "M {:.4},{:.4} v {:.4} h {:.4} v {:.4} z",
            bbox.x[0],
            bbox.y[0],
            bbox.y[1] - bbox.y[0],
            bbox.x[1] - bbox.x[0],
            bbox.y[0] - bbox.y[1]
        );
        let mut e = SvgElement::new("path");
        e.add_color("fill", "none");
        e.add_color("stroke", color);
        e.add_attribute("stroke-width", None, &format!("{:.4}", line_width));
        e.add_attribute("d", None, &r);
        e
    }

    //cp new_grid
    /// Create a grid element with given region, spacing, line
    /// width and color
    pub fn new_grid(bbox: BBox, spacing: f64, line_width: f64, color: &str) -> Option<Self> {
        let xmin = ((bbox.x[0] / spacing) + 0.).floor() as isize;
        let xmax = ((bbox.x[1] / spacing) + 1.).floor() as isize;
        let xlen = xmax - xmin;

        let ymin = ((bbox.y[0] / spacing) + 0.).floor() as isize;
        let ymax = ((bbox.y[1] / spacing) + 1.).floor() as isize;
        let ylen = ymax - ymin;

        if xlen < 2 || ylen < 2 {
            return None;
        }

        let mut r = String::new();
        for i in xmin..xmax + 1 {
            let coord = (i as f64) * spacing;
            r.push_str(&format!(
                "M {},{} v {} ",
                coord,
                bbox.y[0],
                bbox.y[1] - bbox.y[0]
            ));
        }

        for i in ymin..ymax + 1 {
            let coord = (i as f64) * spacing;
            r.push_str(&format!(
                "M {},{} h {} ",
                bbox.x[0],
                coord,
                bbox.x[1] - bbox.x[0]
            ));
        }

        let mut grid = SvgElement::new("path");
        grid.add_attribute("fill", None, "None");
        grid.add_attribute("stroke", None, color);
        grid.add_attribute("stroke-width", None, &format!("{}", line_width));
        grid.add_attribute("d", None, &r);
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
