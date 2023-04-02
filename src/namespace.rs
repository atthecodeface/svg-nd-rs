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

@file    namespace.rs
@brief   An XML namespace thing
 */

//a Imports
use std::borrow::Cow;

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
