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

@file    svg.rs
@brief   Generate SVG output
 */

//a Imports
use indent_display::{IndentedDisplay, Indenter};

use crate::IndentOpt;
use crate::{
    BBox, Color, ColorDatabase, ElementIter, SvgColorDatabase, SvgElement, SvgError, SvgSvg,
};

//a SvgVersion
//tp SvgVersion
/// Version of SVG to output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvgVersion {
    Version1_0,
    Version1_1,
    Version2_0,
}

//ip From<&str> for SvgVersion
impl From<&str> for SvgVersion {
    fn from(x: &str) -> Self {
        match x {
            "1.0" => Self::Version1_0,
            "1.1" => Self::Version1_1,
            "2.0" => Self::Version2_0,
            _ => {
                panic!("Version supported are 1.0, 1.1 and 2.0")
            }
        }
    }
}

//ip From<SvgVersion> for &str
impl From<SvgVersion> for &str {
    fn from(x: SvgVersion) -> Self {
        match x {
            SvgVersion::Version1_0 => "1.0",
            SvgVersion::Version1_1 => "1.1",
            SvgVersion::Version2_0 => "2.0",
        }
    }
}

//a SvgConfig
//tp SvgConfig
/// Configuration of SVG output
#[derive(Debug, Clone, Default)]
pub struct SvgConfig {
    /// if asserted then show grid at the toplevel layout
    pub show_grid: bool,
    /// if asserted then show layout of grids
    pub show_layout: bool,
    /// if asserted then show content rectangles as translucent green rectangles
    pub show_content_rectangles: Option<(f64, Color)>,
}

//ip SvgConfig
impl SvgConfig {
    //bp set_show_grid
    pub fn set_show_grid(mut self, show_grid: bool) -> Self {
        self.show_grid = show_grid;
        self
    }
    //bp set_show_layout
    pub fn set_show_layout(mut self, show_layout: bool) -> Self {
        self.show_layout = show_layout;
        self
    }
    //bp set_content_rectangles
    pub fn set_content_rectangles<'a, T>(mut self, width: f64, color: T) -> Self
    where
        (T, &'a ColorDatabase<'a>): Into<Color>,
    {
        let color = (color, &SvgColorDatabase).into();
        self.show_content_rectangles = Some((width, color));
        self
    }
    //bp clear_content_rectangles
    pub fn clear_content_rectangles(mut self) -> Self {
        self.show_content_rectangles = None;
        self
    }
}

//a Svg
//tp Svg
/// This structure is used to create SVG renderings of a `Diagram` It
/// should be constructed, and mutably borrowed by a diagram when it's
/// `generate_svg` method is invoked.
///
/// This method requires the `GenerateSvg` to be brought in to scope.
pub struct Svg<'a> {
    /// version of SVG - 10, 11 or 20
    pub version: SvgVersion,
    /// Configuration
    pub config: SvgConfig,
    /// Bounding box
    bbox: BBox,
    /// Contents of the SVG
    contents: Vec<SvgElement<'a>>,
    /// Definitions in the SVG
    definitions: Vec<SvgElement<'a>>,
    /// Stack of elements being created
    stack: Vec<SvgElement<'a>>,
}

//ip IndentedDisplay for Svg
impl<'a, 'i> IndentedDisplay<'i, IndentOpt> for Svg<'a> {
    fn indent(&self, f: &mut Indenter<'i, IndentOpt>) -> Result<(), std::fmt::Error> {
        "Svg".indent(f)?;
        {
            let mut sub = f.push("...");
            for c in self.contents.iter() {
                c.indent(&mut sub)?;
            }
        }
        Ok(())
    }
}

//ip Svg
impl<'a> Svg<'a> {
    //fp new
    /// Create a new `Svg` instance, to render a `Diagram` into
    pub fn new(config: SvgConfig) -> Self {
        Self {
            version: "2.0".into(),
            config,
            bbox: BBox::none(),
            contents: vec![],
            definitions: vec![],
            stack: vec![],
        }
    }

    //cp set_version
    /// Used in a construction, to update the `Svg` instance to enable
    /// or disable the incorporation of a version in to the SVG output
    pub fn set_version<I: Into<SvgVersion>>(mut self, version: I) -> Self {
        self.version = version.into();
        self
    }

    //mp stack_push
    pub fn stack_push(&mut self, e: SvgElement<'a>) {
        self.stack.push(e);
    }

    //mp stack_pop
    pub fn stack_pop(&mut self) -> SvgElement<'a> {
        self.stack.pop().unwrap()
    }

    //mp stack_add_subelement
    pub fn stack_add_subelement(&mut self, e: SvgElement<'a>) {
        #![track_caller]
        assert!(
            !self.stack.is_empty(),
            "Stack cannot be empty when adding a subelement to the top entry of the stack"
        );
        let n = self.stack.len();
        self.stack[n - 1].push_content(e);
    }

    //mp stack_pop_to_child
    pub fn stack_pop_to_child(&mut self) {
        #![track_caller]
        assert!(self.stack.len() > 1, "Stack must have two elements to pop an element to be the child of the stack element above it");
        let e = self.stack.pop().unwrap();
        self.stack_add_subelement(e);
    }

    //mp contents_add_element
    pub fn contents_add_element(&mut self, e: SvgElement<'a>) {
        self.contents.push(e);
    }

    //mp contents_take_stack
    pub fn contents_take_stack(&mut self) {
        assert_eq!(
            self.stack.len(),
            1,
            "Stack must have just one element to be added to the contents"
        );
        self.contents.push(self.stack.pop().unwrap());
    }

    //mp definitions_add_element
    pub fn definitions_add_element(&mut self, e: SvgElement<'a>) {
        self.definitions.push(e);
    }

    //mp definitions_take_stack
    pub fn definitions_take_stack(&mut self) {
        assert_eq!(
            self.stack.len(),
            1,
            "Stack must have just one element to be added to the definitions"
        );
        self.definitions.push(self.stack.pop().unwrap());
    }

    //mp finalize
    pub fn finalize(&mut self) {
        assert!(
            self.stack.is_empty(),
            "The stack should be empty before finalizing"
        );

        let mut child_extra = vec![];
        let mut bbox = BBox::default();
        for c in self.contents.iter_mut() {
            child_extra.append(&mut c.finalize(&self.config));
            bbox = bbox.union(c.bbox());
        }
        self.bbox = bbox;
        // Children are finalized now
        for c in child_extra {
            self.contents.push(c);
        }
    }

    //mp generate_diagram
    pub fn generate_diagram(&mut self) -> Result<(), SvgError> {
        let (x, y, w, h) = self.bbox.get_bounds();
        let mut ele = SvgSvg::new();
        ele.add_attribute("svg", Some("xmlns"), "http://www.w3.org/2000/svg");
        ele.add_attribute("xmlns", None, "http://www.w3.org/2000/svg");
        ele.add_attribute("version", None, self.version.into());
        ele.add_attribute("width", None, &format!("{}mm", w));
        ele.add_attribute("height", None, &format!("{}mm", h));
        ele.add_attribute("viewBox", None, &format!("{} {} {} {}", x, y, w, h));
        self.stack_push(ele);

        // let ele = SvgElement::new("defs");
        // self.stack_push(ele);
        // for d in std::mem::take(&mut self.definitions) {
        // self.stack_add_subelement(d);
        //}
        // self.stack_pop_to_child();

        for d in std::mem::take(&mut self.contents) {
            self.stack_add_subelement(d);
        }

        if self.config.show_grid {
            let mut e = SvgElement::new_grid(self.bbox, 10., 0.1, "grey");
            let _ = e.finalize(&self.config);
            self.stack_add_subelement(e);
        }

        Ok(())
    }

    //mp iter_events
    /// Iterate over all the XML events the Svg would generate if it
    /// were an SVG file being read in by xml-rs
    ///
    /// This permits the SVG to be read by an XML reader, or written
    /// using xml-rs to convert reader XmlEvents to writer XmlEvents.
    pub fn iter_events<'i>(&'i self) -> ElementIter<'a, 'i> {
        ElementIter::new(&self.stack[0])
    }

    //zz All done
}
