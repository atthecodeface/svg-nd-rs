//a Imports
use std::rc::Rc;

//tp Rgba
/// Stored as a u32 with (255-alpha) in top 8 bits, then R, then G, then B in bottom 8 bits
#[derive(Debug, Clone, Copy, Default)]
pub struct Rgba(u32);
impl From<u32> for Rgba {
    #[inline]
    fn from(rgb: u32) -> Self {
        Self(rgb)
    }
}
impl From<(u8, u8, u8)> for Rgba {
    #[inline]
    fn from(rgb: (u8, u8, u8)) -> Self {
        Self::from_tuple_rgb(rgb)
    }
}
impl From<(u8, u8, u8, u8)> for Rgba {
    #[inline]
    fn from(rgba: (u8, u8, u8, u8)) -> Self {
        Self::from_tuple_rgba(rgba)
    }
}
impl From<(f32, f32, f32)> for Rgba {
    #[inline]
    fn from((r, g, b): (f32, f32, f32)) -> Self {
        let r = (r * 255.9).floor() as u8;
        let g = (g * 255.9).floor() as u8;
        let b = (b * 255.9).floor() as u8;
        (r, g, b).into()
    }
}
impl From<(f32, f32, f32, f32)> for Rgba {
    #[inline]
    fn from((r, g, b, a): (f32, f32, f32, f32)) -> Self {
        let r = (r * 255.9).floor() as u8;
        let g = (g * 255.9).floor() as u8;
        let b = (b * 255.9).floor() as u8;
        let a = (a * 255.9).floor() as u8;
        (r, g, b, a).into()
    }
}
impl From<Rgba> for (u8, u8, u8, u8) {
    fn from(rgba: Rgba) -> (u8, u8, u8, u8) {
        rgba.as_tuple_rgba()
    }
}
impl From<Rgba> for String {
    fn from(rgba: Rgba) -> String {
        let (r, g, b, alpha) = rgba.as_tuple_rgba();
        if alpha == 255 {
            format!("#{:02x}{:02x}{:02x}", r, g, b)
        } else {
            format!("rgba({},{},{},{})", r, g, b, alpha)
        }
    }
}
impl Rgba {
    pub fn of_rgba(rgba: u32) -> Self {
        let r: Self = (rgba & 0xffffff).into();
        r.set_alpha((rgba >> 24) as u8)
    }
    fn from_tuple_rgb((r, g, b): (u8, u8, u8)) -> Self {
        let rgb = (b as u32) | ((g as u32) << 8) | ((r as u32) << 16);
        Self(rgb)
    }
    fn from_tuple_rgba((r, g, b, a): (u8, u8, u8, u8)) -> Self {
        let rgba = (b as u32) | ((g as u32) << 8) | ((r as u32) << 16) | (((255 - a) as u32) << 24);
        Self(rgba)
    }

    fn as_tuple_rgba(self) -> (u8, u8, u8, u8) {
        (
            ((self.0 >> 16) & 0xff) as u8,         // r
            ((self.0 >> 8) & 0xff) as u8,          // g
            (self.0 & 0xff) as u8,                 // b
            255 - (((self.0 >> 24) & 0xff) as u8), // alpha
        )
    }
    pub fn alpha(self) -> u8 {
        255 - (((self.0 >> 24) & 0xff) as u8)
    }
    pub fn set_alpha(mut self, alpha: u8) -> Self {
        self.0 = (self.0 & 0xffffff) | (((255 - alpha) as u32) << 24);
        self
    }
}
#[derive(Debug, Clone)]
pub struct Color {
    /// String representation (if transparency is 0)
    text: Rc<String>,
    /// RGBA
    rgba: Rgba,
}
impl Color {
    #[inline]
    #[must_use]
    fn new<I: Into<String>, J: Into<Rgba>>(text: I, rgba: J) -> Self {
        let text = text.into().into();
        let rgba = rgba.into();
        Self { text, rgba }
    }
    #[inline]
    #[must_use]
    pub fn set_alpha(mut self, alpha: u8) -> Self {
        self.rgba = self.rgba.set_alpha(alpha);
        self
    }
    #[inline]
    #[must_use]
    pub fn of_rgb<I: Into<Rgba>>(rgba: I) -> Self {
        let rgba = rgba.into();
        let text = Rc::new(rgba.into());
        Self { text, rgba }
    }
    pub fn name_is_none(name: &str) -> Option<Self> {
        match name {
            "None" | "none" | "NONE" => Some(Self::new("none", (0, 0, 0, 255))),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Rc<String> {
        if self.rgba.alpha() == 255 {
            self.text.clone()
        } else {
            Rc::new(self.rgba.into())
        }
    }
}
pub struct ColorDatabase<'a> {
    pub colors: &'a [(&'a str, u32)],
}
impl<'a> ColorDatabase<'a> {
    fn find_color_exact_index(&self, name: &str) -> Option<usize> {
        for (i, (n, _)) in self.colors.iter().enumerate() {
            if *n == name {
                return Some(i);
            }
        }
        None
    }
    fn canonicalize_name(name: &str) -> Option<String> {
        let mut r = String::new();
        for mut c in name.chars() {
            if c == '_' {
                continue;
            }
            c.make_ascii_lowercase();
            r.push(c);
        }
        Some(r)
    }
    fn find_color_index(&self, name: &str) -> Option<usize> {
        Self::canonicalize_name(name).and_then(|s| self.find_color_exact_index(&s))
    }
    pub fn find_color_name(&self, name: &str) -> Option<&str> {
        self.find_color_index(name).map(|i| self.colors[i].0)
    }
    pub fn find_color_rgb(&self, name: &str) -> Option<u32> {
        self.find_color_index(name).map(|i| self.colors[i].1)
    }
    pub fn find_color(&self, name: &str) -> Option<Color> {
        if let Some(color_none) = Color::name_is_none(name) {
            Some(color_none)
        } else {
            self.find_color_index(name)
                .map(|i| Color::new(self.colors[i].0, self.colors[i].1))
        }
    }
}
impl<'a> From<(&str, &'a ColorDatabase<'a>)> for Color {
    #[inline]
    fn from((s, db): (&str, &'a ColorDatabase<'a>)) -> Self {
        db.find_color(s)
            .unwrap_or_else(|| panic!("Color must be found in the database, but '{}' was not", s))
    }
}
impl<'a> From<(&Color, &'a ColorDatabase<'a>)> for Color {
    #[inline]
    fn from((c, _db): (&Color, &'a ColorDatabase<'a>)) -> Self {
        c.clone()
    }
}
impl<'a, I: Into<Rgba>> From<(I, &'a ColorDatabase<'a>)> for Color {
    #[inline]
    fn from((rgb, _db): (I, &'a ColorDatabase<'a>)) -> Self {
        Color::of_rgb(rgb.into())
    }
}
