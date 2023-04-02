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

@file    svg_element_iter.rs
@brief   Iterator over the SVG element
 */

//a Imports
use crate::SvgElement;

//a XmlEvent
//tp XmlEvent
#[derive(Debug)]
pub enum XmlEvent<'a, 'x> {
    StartDocument,
    EndDocument,
    StartElement(&'x SvgElement<'a>),
    EndElement(&'x SvgElement<'a>),
    Characters(&'x SvgElement<'a>),
}

//ip XmlEvent
impl<'a, 'x> XmlEvent<'a, 'x> {
    pub fn as_xml(&self) -> String {
        use XmlEvent::*;
        match self {
            StartDocument => r#"<?xml version="1.0" encoding="utf8"?>"#.into(),
            EndDocument => "".into(),
            StartElement(e) => {
                let mut r = format!("<{}", e.ns_name());
                for (n, v) in e.attributes() {
                    r.push_str(&format!(r#" {}="{}""#, n, v));
                }
                r.push('>');
                r
            }
            Characters(e) => e.characters().into(),
            EndElement(e) => {
                format!("</{}>", e.ns_name())
            }
        }
    }
}

//a SvgElement iterator
//ti IterState
#[derive(Debug)]
enum IterState {
    PreDocument,
    PreElement,
    PreString,
    PreContent,
    PostContent,
    FindNextElement,
    DocumentEnd,
    Completed,
}

//tp ElementIter
/// An iterator structure to permit iteration over an Svg object's elements
pub struct ElementIter<'a, 'i> {
    state: IterState,
    elements: Vec<(&'i SvgElement<'a>, usize)>,
}

//ip ElementIter
impl<'a, 'i> ElementIter<'a, 'i> {
    //fp new
    /// Create a new Svg element iterator
    pub fn new(e: &'i SvgElement<'a>) -> Self {
        let elements = vec![(e, 0)];
        Self {
            state: IterState::PreDocument,
            elements,
        }
    }
}

//ip Iterator for ElementIter
impl<'a, 'i> Iterator for ElementIter<'a, 'i> {
    type Item = XmlEvent<'a, 'i>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            IterState::PreDocument => {
                self.state = IterState::PreElement;
                Some(XmlEvent::StartDocument)
            }
            IterState::PreElement => {
                let (ele, n) = self.elements.pop().unwrap();
                self.state = IterState::PreString;
                self.elements.push((ele, n));
                Some(XmlEvent::StartElement(ele))
            }
            IterState::PreString => {
                let (ele, n) = self.elements.pop().unwrap();
                self.state = IterState::PreContent;
                if ele.characters() != "" {
                    self.elements.push((ele, n));
                    Some(XmlEvent::Characters(ele))
                } else {
                    self.elements.push((ele, n));
                    self.next()
                }
            }
            IterState::PreContent => {
                let (ele, n) = self.elements.pop().unwrap();
                if n < ele.contents().len() {
                    let next_ele = &ele.contents()[n];
                    self.elements.push((ele, n));
                    self.elements.push((next_ele, 0));
                    self.state = IterState::PreElement;
                } else {
                    self.state = IterState::PostContent;
                    self.elements.push((ele, n));
                }
                self.next()
            }
            IterState::PostContent => {
                let (ele, n) = self.elements.pop().unwrap();
                self.state = IterState::FindNextElement;
                self.elements.push((ele, n));
                Some(XmlEvent::EndElement(ele))
            }
            IterState::FindNextElement => {
                if self.elements.len() > 1 {
                    let (_ele, _n) = self.elements.pop().unwrap();
                    let (ele, n) = self.elements.pop().unwrap();
                    if n + 1 < ele.contents().len() {
                        let next_ele = &ele.contents()[n + 1];
                        self.elements.push((ele, n + 1));
                        self.elements.push((next_ele, 0));
                        self.state = IterState::PreElement;
                    } else {
                        self.elements.push((ele, n + 1));
                        self.state = IterState::PostContent;
                    }
                } else {
                    self.state = IterState::DocumentEnd;
                }
                self.next()
            }
            IterState::DocumentEnd => {
                self.state = IterState::Completed;
                Some(XmlEvent::EndDocument)
            }
            IterState::Completed => None,
        }
    }
}
