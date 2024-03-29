// From matplotlib
struct Text {
    text: String,
    // Other attributes, possibily markup text
}
struct Label {
    text: Text,
    location: f32,
    // padding etc
}
struct Axis {
    label: Label,
    // major
    // minor
    // (parent axes?)
    /// Distance between the label and the ticks
    label_pad: f32,
}
struct Axes {
    data_bbox: BBbox,
    view_bbox: Bbox,
    xlabel: Label,
    ylabel: Label,
    title: Label,
    // Margin - added around the data bbox before it is placed in the view
}
 