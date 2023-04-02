//tp GlyphMetrics
#[derive(Debug, Default, Clone, Copy)]
pub struct GlyphMetrics {
    /// Width of the glyph
    width: f32,
    /// Ascent of the glyph - amount above the baseline (+ve for most glyphs)
    ascent: f32,
    /// Descent of the glyph - amount below the baseline (+ve for most glyphs)
    descent: f32,
    /// Left-side bearing - amount to left of glyph ink to left of glyph BBox
    lsb: f32,
    /// Righ-side bearing - amount to right of glyph ink to right of glyph BBox
    rsb: f32,
    /// Slant tan to italicize the glyph
    italic: f32,
    /// Some option bits
    options: usize,
}

//ip GlyphMetrics
impl GlyphMetrics {}

//ip std::ops::Add for GlyphMetrics
impl std::ops::Add<GlyphMetrics> for GlyphMetrics {
    type Output = Self;
    #[inline]
    fn add(mut self, other: GlyphMetrics) -> Self {
        self.width += other.width;
        self.ascent = self.ascent.max(other.ascent);
        self.descent = self.descent.max(other.descent);
        self.rsb = other.rsb;
        self
    }
}

//tp CharIndices
/// This structure provides simple metrics for a font or a region of
/// characters in a font. It is based on the TeX Font Metrics.
#[derive(Debug, Clone, Copy)]
pub struct CharIndices(u32);
impl std::fmt::Display for CharIndices {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

//ip CharIndices
// hhea has ascent, descent, line gap (gap between above line's descender and below line's ascender)
//    line height is ascent - descent (as that is negative) + line gap/
//    For consistency across platforms linegap should be 0 (you can always add it to descender?)
//
// hmtx has lsb (left side bearing) and rsb (right side bearing) and width
impl CharIndices {
    //fi of_indices
    fn of_indices(
        width: usize,
        height: usize,
        depth: usize,
        italic: usize,
        options: usize,
    ) -> Self {
        assert!(width < 256);
        assert!(height < 16);
        assert!(depth < 16);
        assert!(italic < 64);
        assert!(options < 1024);
        let v = (options << 22) | (italic << 16) | (depth << 12) | (height << 8) | (width << 0);
        Self(v as u32)
    }

    //fp width_index
    pub fn width_index(&self) -> usize {
        ((self.0 >> 0) & 0xff) as usize
    }
    //fp height_index
    pub fn height_index(&self) -> usize {
        ((self.0 >> 8) & 0xf) as usize
    }
    //fp depth_index
    pub fn depth_index(&self) -> usize {
        ((self.0 >> 12) & 0xf) as usize
    }
    //fp italic_index
    pub fn italic_index(&self) -> usize {
        ((self.0 >> 16) & 0x3f) as usize
    }
    //fp options
    pub fn options(&self) -> usize {
        ((self.0 >> 22) & 0x3ff) as usize
    }
    //fp width
    pub fn width<V: Value>(&self, metrics: &Metrics) -> f32 {
        metrics.get_width(self.width_index())
    }
    pub fn height<V: Value>(&self, metrics: &Metrics) -> f32 {
        metrics.get_height(self.height_index())
    }
    pub fn depth<V: Value>(&self, metrics: &Metrics) -> f32 {
        metrics.get_depth(self.depth_index())
    }
    pub fn italic<V: Value>(&self, metrics: &Metrics) -> f32 {
        metrics.get_italic(self.italic_index())
    }
}

//tp Parameter
/// Font metric parameters
#[derive(Debug, Copy, Clone)]
pub enum Parameter {
    /// Size of a space in the font (standard gap between words)
    Space(V),
    /// Size of an 'em' in the font (length of an em-dash, not necessarily the width of 'M')
    Em(V),
    /// Space after a period at the end of a sentence
    PunctSpace(V),
    // x height
    // cap height
    // ascent
    // descent
}

//tp CharRangeMetrics
#[derive(Debug)]
pub struct CharRangeMetrics {
    /// First Unicode Scalar Value represented by these metrics (inclusive)
    first_char: char,
    /// Last Unicode Scalar Value represented by these metrics (inclusive)
    last_char: char,
    /// Widths - at most 256 long, with zeroth element of 0
    /// then Heights - at most 16 long, with zeroth element of 0
    /// then Depths - at most 16 long, with zeroth element of 0
    /// then Italics - at most 64 long, with zeroth element of 0
    data: Vec<f32>,
    /// Number of widths in the data
    num_widths: usize,
    /// Number of heights in the data
    num_heights: usize,
    /// Number of depths in the data
    num_depths: usize,
    /// Number of italics in the data
    num_italics: usize,
    /// Character metrics - as indices in to the above vectors, for characters from first_char to last_char
    char_metrics: Vec<CharIndices>,
    /// parameters, sorted by the parameter order for faster indexing
    parameters: Vec<Parameter>,
    /// Exceptions to the metrics provided here - allowing for more than 16 heights, 256 widths, etc.
    exceptions: Vec<Metrics>,
}
//ip CharRangeMetrics
impl CharRangeMetrics {
    pub fn new_monospace(width: f32, height: f32, depth: f32, italic: f32) -> Self {
        let first_char = '\0';
        let last_char = '\0';
        let data = vec![width, height, depth, italic];
        let num_widths = 1;
        let num_heights = 1;
        let num_depths = 1;
        let num_italics = 1;
        let char_metrics = vec![CharIndices::of_indices(1, 1, 1, 1, 0)];
        let parameters = Vec::new();
        let exceptions = Vec::new();
        Self {
            first_char,
            last_char,
            data,
            num_widths,
            num_heights,
            num_depths,
            num_italics,
            char_metrics,
            parameters,
            exceptions,
        }
    }
    #[inline]
    pub fn get_width(&self, index: usize) -> f32 {
        assert!(index <= self.num_widths);
        if index == 0 {
            0.0
        } else {
            self.data[index - 1]
        }
    }
    #[inline]
    pub fn get_height(&self, index: usize) -> f32 {
        assert!(index <= self.num_heights);
        if index == 0 {
            0.0
        } else {
            self.data[self.num_widths + index - 1]
        }
    }
    #[inline]
    pub fn get_depth(&self, index: usize) -> f32 {
        assert!(index <= self.num_depths);
        if index == 0 {
            0.0
        } else {
            self.data[self.num_widths + self.num_heights + index - 1]
        }
    }
    #[inline]
    pub fn get_italic(&self, index: usize) -> f32 {
        assert!(index <= self.num_italics);
        if index == 0 {
            0.0
        } else {
            self.data[self.num_widths + self.num_heights + self.num_depths + index - 1]
        }
    }
    pub fn get_glyph_metrics(&self, index: usize) -> GlyphMetrics {
        let ci = self.char_metrics[index];
        let width = ci.width(self);
        let height = ci.height(self);
        let depth = ci.depth(self);
        let italic = ci.italic(self);
        let options = ci.options();
        GlyphMetrics {
            width,
            ascent: height,
            descent: depth,
            lsb: 0.,
            rsb: 0.,
            italic,
            options,
        }
    }
    /// Get the [CharRangeMetrics] for a Unicode codepoint and the
    /// index into it for it
    pub fn metrics_of_char(&self, c: char) -> Option<(&Self, usize)> {
        if c < self.first_char {
            None
        } else if c > self.last_char {
            None
        } else {
            for e in &self.exceptions {
                if let Some(m) = e.metrics_of_char(c) {
                    return Some(m);
                }
            }
            Some((&self, ((c as u32) - (self.first_char as u32)) as usize))
        }
    }
    /// Get the glyph metrics for a Unicode codepoint
    pub fn glyph_metrics(&self, c: char) -> Option<GlyphMetrics> {
        self.metrics_of_char(c).map(|(m, i)| m.get_glyph_metrics(i))
    }
}

//tp Font
/// This structure provides simple metric storage for
#[derive(Debug)]
pub struct Font {
    metrics: CharRangeMetrics,
}

impl Font {
    pub fn default() -> Self {
        Self {
            metrics: CharRangeMetrics::new_monospace(0.5, 1.1, 0.3, 0.),
        }
    }
}

impl FontMetrics for Font {
    fn get_metrics(&self, text: &str, style: &FontStyle) -> TextMetrics {
        let mut gm = GlyphMetrics::zero();
        for c in text.chars() {
            // if a space, add metrics.space?
            gm = gm.add(&self.metrics.glyph_metrics(c));
        }
        let size = style.size * 25.4 / 72.0;
        let width = gm.width * size;
        let ascender = gm.height * size;
        let descender = gm.depth * size;
        TextMetrics {
            width,
            descender,
            ascender,
        }
    }
}
