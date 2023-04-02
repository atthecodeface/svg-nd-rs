//a Imports
use svg_nd::{Attr, Element, Group, Path, Polygon};

#[test]
fn test_me() {
    let x: Element<Attr> = Element::new_path(Path::new_polygon(
        Polygon::new_star(5, 10.0, 0.7, 0., 1.),
        true,
    ));
}
