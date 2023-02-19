//a Imports
use xml::common::XmlVersion as XmlWriteVersion;
use xml::namespace::Namespace as XmlWriteNamespace;
use xml::write::events::XmlEvent as XmlWriteEvent;

//a Event
pub struct NamespaceName<'a> {
    name: &'a str,
    prefix: Option<&'a str>,
}

impl<'a> From<(XmlEvent<'a>, Cow<'a, XmlWriteNamespace>)> for xml_write::XmlEvent<'a> {
    fn from(xml_event_ns: (XmlEvent<'a>, XmlWriteNamespace)) -> Option<xml_write::XmlEvent<'a>> {
        let (xml_event, namespace) = xml_event_ns;
        match xml_event {
            XmlEvent::StartDocument => Some(XmlWriteEvent::StartDocument {
                version: XmlWriteVersion::Version11,
                encoding: Some("utf8"),
                standalone: None,
            }),
            XmlEvent::EndDocument => None,
            XmlEvent::StartElement { name, attributes } => Some(XmlWriteEvent::StartElement {
                name: name.into_xml_name(),
                attributes: attributes.into_xml_attributes(),
                namespace: namespace,
            }),
            XmlEvent::EndElement => Some(XmlWriteEvent::EndElement { None }),
            XmlEvent::Characters(s) => Some(XmlWriteEvent::Characters(s)),
        }
    }
}
// NamespaceName
//
// into_xml_name( &'a self) -> xml_writer::Name<'a>
// into_xml_namespace( &'a self) -> Cow<'a, xml_writer::Namespace>
// into_xml_attributes( &'a Vec<NamespaceName<'a>, &'a str> ) -> Cow<'a, [xml_writer::Attribute<'a>]>>
//
// pub struct Name<'a> {
//     pub local_name: &'a str,
//     pub namespace: Option<&'a str>, = None
//     pub prefix: Option<&'a str>,
// }

// pub struct Namespace(pub BTreeMap<String, String>);
// pub struct Attribute<'a> {
//     pub name: Name<'a>,
//     pub value: &'a str,
// }
//
// StartElement : Contents of the namespace mapping at this point of the document.
