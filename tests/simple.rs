//a Imports
use svg_nd::{BezierPath, Svg, SvgConfig, SvgElement, SvgPath, Transform};

#[test]
fn test_0() {
    let svg_config = SvgConfig::default()
        .set_show_grid(true)
        .set_content_rectangles(0.6, "AliceBlue");

    let mut svg = Svg::new(svg_config);

    let b = BezierPath::of_ellipse([0., 0.].into(), 10., 1.40, 65.0);
    let mut e = SvgPath::new_path(b, true);
    e.add_color("fill", "red");
    svg.stack_push(e);
    svg.contents_take_stack();

    let b = BezierPath::of_ellipse([0., 0.].into(), 10., 1.40, 35.0);
    let mut e = SvgPath::new_path(b, true);
    e.add_color("fill", (0., 1.0, 0.));
    e.apply_transform(&(Transform::of_rotation(30.0) + [20., 20.].into()));
    svg.stack_push(e);
    svg.contents_take_stack();

    let mut stdout = std::io::stdout();
    use indent_display::IndentedDisplay;
    let mut ind = indent_display::Indenter::new(&mut stdout, "  ", &svg_nd::IndentOpt());
    svg.finalize();
    svg.indent(&mut ind);
    svg.generate_diagram();
    for e in svg.iter_events() {
        println!("{}", e.as_xml());
    }
    assert!(false);
}
