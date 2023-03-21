pub mod font;
pub mod outliner;
pub mod rasterizer;

use crate::rusttype::font::RusttypeFont;
use crate::shape::Rect;
use glam::{vec2, Vec2};

use core::fmt;
pub use owned_ttf_parser::OutlineBuilder;

/// Linear interpolation between points.
#[inline]
pub(crate) fn lerp(t: f32, p0: Vec2, p1: Vec2) -> Vec2 {
    vec2(p0.x + t * (p1.x - p0.x), p0.y + t * (p1.y - p0.y))
}

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct GlyphId(pub u16);

impl From<owned_ttf_parser::GlyphId> for GlyphId {
    fn from(id: owned_ttf_parser::GlyphId) -> Self {
        Self(id.0)
    }
}
impl From<GlyphId> for owned_ttf_parser::GlyphId {
    fn from(id: GlyphId) -> Self {
        Self(id.0)
    }
}

/// A single glyph of a font.
///
/// A `Glyph` does not have an inherent scale or position associated with it. To
/// augment a glyph with a size, give it a scale using `scaled`. You can then
/// position it using `positioned`.
#[derive(Clone)]
pub struct Glyph<'font> {
    pub(crate) font: RusttypeFont<'font>,
    pub(crate) id: GlyphId,
}

impl<'font> Glyph<'font> {
    /// The font to which this glyph belongs.
    pub fn font(&self) -> &RusttypeFont<'font> {
        &self.font
    }

    /// The glyph identifier for this glyph.
    pub fn id(&self) -> GlyphId {
        self.id
    }

    /// Augments this glyph with scaling information, making methods that depend
    /// on the scale of the glyph available.
    pub fn scaled(self, scale: Scale) -> ScaledGlyph<'font> {
        let scale_y = self.font.scale_for_pixel_height(scale.y);
        let scale_x = scale_y * scale.x / scale.y;
        ScaledGlyph {
            g: self,
            api_scale: scale,
            scale: vec2(scale_x, scale_y),
        }
    }
}

impl fmt::Debug for Glyph<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Glyph").field("id", &self.id().0).finish()
    }
}

/// The "horizontal metrics" of a glyph. This is useful for calculating the
/// horizontal offset of a glyph from the previous one in a string when laying a
/// string out horizontally.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct HMetrics {
    /// The horizontal offset that the origin of the next glyph should be from
    /// the origin of this glyph.
    pub advance_width: f32,
    /// The horizontal offset between the origin of this glyph and the leftmost
    /// edge/point of the glyph.
    pub left_side_bearing: f32,
}

/// The "vertical metrics" of a font at a particular scale. This is useful for
/// calculating the amount of vertical space to give a line of text, and for
/// computing the vertical offset between successive lines.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct VMetrics {
    /// The highest point that any glyph in the font extends to above the
    /// baseline. Typically positive.
    pub ascent: f32,
    /// The lowest point that any glyph in the font extends to below the
    /// baseline. Typically negative.
    pub descent: f32,
    /// The gap to leave between the descent of one line and the ascent of the
    /// next. This is of course only a guideline given by the font's designers.
    pub line_gap: f32,
}

impl core::ops::Mul<f32> for VMetrics {
    type Output = VMetrics;

    fn mul(self, rhs: f32) -> Self {
        Self {
            ascent: self.ascent * rhs,
            descent: self.descent * rhs,
            line_gap: self.line_gap * rhs,
        }
    }
}

/// A glyph augmented with scaling information. You can query such a glyph for
/// information that depends on the scale of the glyph.
#[derive(Clone)]
pub struct ScaledGlyph<'font> {
    g: Glyph<'font>,
    api_scale: Scale,
    scale: Vec2,
}

impl<'font> ScaledGlyph<'font> {
    /// The glyph identifier for this glyph.
    pub fn id(&self) -> GlyphId {
        self.g.id()
    }

    /// The font to which this glyph belongs.
    #[inline]
    pub fn font(&self) -> &RusttypeFont<'font> {
        self.g.font()
    }

    /// A reference to this glyph without the scaling
    pub fn into_unscaled(self) -> Glyph<'font> {
        self.g
    }

    /// Removes the scaling from this glyph
    pub fn unscaled(&self) -> &Glyph<'font> {
        &self.g
    }

    /// Builds the outline of the glyph with the builder specified. Returns
    /// `false` when the outline is either malformed or empty.
    pub fn build_outline(&self, builder: &mut impl OutlineBuilder) -> bool {
        let mut outliner = outliner::OutlineScaler::new(builder, vec2(self.scale.x, -self.scale.y));

        self.font()
            .inner()
            .outline_glyph(self.id().into(), &mut outliner)
            .is_some()
    }

    /// Augments this glyph with positioning information, making methods that
    /// depend on the position of the glyph available.
    pub fn positioned(self, p: Vec2) -> PositionedGlyph<'font> {
        let bb = self.pixel_bounds_at(p);
        PositionedGlyph {
            sg: self,
            position: p,
            bb,
        }
    }

    pub fn scale(&self) -> Scale {
        self.api_scale
    }

    /// Retrieves the "horizontal metrics" of this glyph. See `HMetrics` for
    /// more detail.
    pub fn h_metrics(&self) -> HMetrics {
        let inner = self.font().inner();
        let id = self.id().into();

        let advance = inner.glyph_hor_advance(id).unwrap();
        let left_side_bearing = inner.glyph_hor_side_bearing(id).unwrap();

        HMetrics {
            advance_width: advance as f32 * self.scale.x,
            left_side_bearing: left_side_bearing as f32 * self.scale.x,
        }
    }

    /// The bounding box of the shape of this glyph, not to be confused with
    /// `pixel_bounding_box`, the conservative pixel-boundary bounding box. The
    /// coordinates are relative to the glyph's origin.
    pub fn exact_bounding_box(&self) -> Option<Rect> {
        let owned_ttf_parser::Rect {
            x_min,
            y_min,
            x_max,
            y_max,
        } = self.font().inner().glyph_bounding_box(self.id().into())?;

        Some(Rect {
            top_left: vec2(x_min as f32 * self.scale.x, -y_max as f32 * self.scale.y),
            bottom_right: vec2(x_max as f32 * self.scale.x, -y_min as f32 * self.scale.y),
        })
    }

    fn glyph_bitmap_box_subpixel(
        &self,
        font: &RusttypeFont<'font>,
        shift_x: f32,
        shift_y: f32,
    ) -> Option<Rect> {
        let owned_ttf_parser::Rect {
            x_min,
            y_min,
            x_max,
            y_max,
        } = font.inner().glyph_bounding_box(self.id().into())?;

        Some(Rect {
            top_left: vec2(
                (x_min as f32 * self.scale.x + shift_x).floor(),
                (-y_max as f32 * self.scale.y + shift_y).floor(),
            ),
            bottom_right: vec2(
                (x_max as f32 * self.scale.x + shift_x).ceil(),
                (-y_min as f32 * self.scale.y + shift_y).ceil(),
            ),
        })
    }

    #[inline]
    fn pixel_bounds_at(&self, p: Vec2) -> Option<Rect> {
        // Use subpixel fraction in floor/ceil rounding to eliminate rounding error
        // from identical subpixel positions
        let (x_trunc, x_fract) = (p.x.trunc(), p.x.fract());
        let (y_trunc, y_fract) = (p.y.trunc(), p.y.fract());

        let Rect {
            top_left,
            bottom_right,
        } = self.glyph_bitmap_box_subpixel(self.font(), x_fract, y_fract)?;
        Some(Rect {
            top_left: vec2(x_trunc + top_left.x, y_trunc + top_left.y),
            bottom_right: vec2(x_trunc + bottom_right.x, y_trunc + bottom_right.y),
        })
    }
}

impl fmt::Debug for ScaledGlyph<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScaledGlyph")
            .field("id", &self.id().0)
            .field("scale", &self.api_scale)
            .finish()
    }
}

/// A glyph augmented with positioning and scaling information. You can query
/// such a glyph for information that depends on the scale and position of the
/// glyph.
#[derive(Clone)]
pub struct PositionedGlyph<'font> {
    sg: ScaledGlyph<'font>,
    position: Vec2,
    bb: Option<Rect>,
}

impl<'font> PositionedGlyph<'font> {
    /// The glyph identifier for this glyph.
    pub fn id(&self) -> GlyphId {
        self.sg.id()
    }

    /// The font to which this glyph belongs.
    #[inline]
    pub fn font(&self) -> &RusttypeFont<'font> {
        self.sg.font()
    }

    /// A reference to this glyph without positioning
    pub fn unpositioned(&self) -> &ScaledGlyph<'font> {
        &self.sg
    }

    /// Removes the positioning from this glyph
    pub fn into_unpositioned(self) -> ScaledGlyph<'font> {
        self.sg
    }

    /// The conservative pixel-boundary bounding box for this glyph. This is the
    /// smallest rectangle aligned to pixel boundaries that encloses the shape
    /// of this glyph at this position. Note that the origin of the glyph, at
    /// pixel-space coordinates (0, 0), is at the top left of the bounding box.
    pub fn pixel_bounding_box(&self) -> Option<Rect> {
        self.bb.clone()
    }

    pub fn scale(&self) -> Scale {
        self.sg.api_scale
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    /// Builds the outline of the glyph with the builder specified. Returns
    /// `false` when the outline is either malformed or empty.
    pub fn build_outline(&self, builder: &mut impl OutlineBuilder) -> bool {
        let bb = if let Some(bb) = self.bb.as_ref() {
            bb
        } else {
            return false;
        };

        let offset = vec2(bb.top_left.x as f32, bb.top_left.y as f32);

        let mut outliner = outliner::OutlineTranslator::new(builder, self.position - offset);

        self.sg.build_outline(&mut outliner)
    }

    /// Rasterises this glyph. For each pixel in the rect given by
    /// `pixel_bounding_box()`, `o` is called:
    ///
    /// ```ignore
    /// o(x, y, v)
    /// ```
    ///
    /// where `x` and `y` are the coordinates of the pixel relative to the `min`
    /// coordinates of the bounding box, and `v` is the analytically calculated
    /// coverage of the pixel by the shape of the glyph. Calls to `o` proceed in
    /// horizontal scanline order, similar to this pseudo-code:
    ///
    /// ```ignore
    /// let bb = glyph.pixel_bounding_box();
    /// for y in 0..bb.height() {
    ///     for x in 0..bb.width() {
    ///         o(x, y, calc_coverage(&glyph, x, y));
    ///     }
    /// }
    /// ```
    pub fn draw<O: FnMut(u32, u32, f32)>(&self, o: O) {
        let bb = if let Some(bb) = self.bb.as_ref() {
            bb
        } else {
            return;
        };

        let width = (bb.bottom_right.x - bb.top_left.x) as u32;
        let height = (bb.bottom_right.y - bb.top_left.y) as u32;

        let mut outliner = outliner::OutlineRasterizer::new(width as _, height as _);

        self.build_outline(&mut outliner);

        outliner.rasterizer.for_each_pixel_2d(o);
    }

    /// Resets positioning information and recalculates the pixel bounding box
    pub fn set_position(&mut self, p: Vec2) {
        let p_diff = p - self.position;
        if p_diff.x.fract().is_near_zero() && p_diff.y.fract().is_near_zero() {
            if let Some(bb) = self.bb.as_mut() {
                let rounded_diff = vec2(p_diff.x.round(), p_diff.y.round());
                bb.top_left = bb.top_left + rounded_diff;
                bb.bottom_right = bb.bottom_right + rounded_diff;
            }
        } else {
            self.bb = self.sg.pixel_bounds_at(p);
        }
        self.position = p;
    }
}

impl fmt::Debug for PositionedGlyph<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PositionedGlyph")
            .field("id", &self.id().0)
            .field("scale", &self.scale())
            .field("position", &self.position)
            .finish()
    }
}

/// Defines the size of a rendered face of a font, in pixels, horizontally and
/// vertically. A vertical scale of `y` pixels means that the distance between
/// the ascent and descent lines (see `VMetrics`) of the face will be `y`
/// pixels. If `x` and `y` are equal the scaling is uniform. Non-uniform scaling
/// by a factor *f* in the horizontal direction is achieved by setting `x` equal
/// to *f* times `y`.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Scale {
    /// Horizontal scale, in pixels.
    pub x: f32,
    /// Vertical scale, in pixels.
    pub y: f32,
}

impl Scale {
    /// Uniform scaling, equivalent to `Scale { x: s, y: s }`.
    #[inline]
    pub fn uniform(s: f32) -> Scale {
        Scale { x: s, y: s }
    }
}
/// A trait for types that can be converted into a `GlyphId`, in the context of
/// a specific font.
///
/// Many `rusttype` functions that operate on characters accept values of any
/// type that implements `IntoGlyphId`. Such types include `char`, `Codepoint`,
/// and obviously `GlyphId` itself.
pub trait IntoGlyphId {
    /// Convert `self` into a `GlyphId`, consulting the index map of `font` if
    /// necessary.
    fn into_glyph_id(self, font: &RusttypeFont<'_>) -> GlyphId;
}
impl IntoGlyphId for char {
    #[inline]
    fn into_glyph_id(self, font: &RusttypeFont<'_>) -> GlyphId {
        font.inner()
            .glyph_index(self)
            .unwrap_or(owned_ttf_parser::GlyphId(0))
            .into()
    }
}
impl<G: Into<GlyphId>> IntoGlyphId for G {
    #[inline]
    fn into_glyph_id(self, _font: &RusttypeFont<'_>) -> GlyphId {
        self.into()
    }
}

#[derive(Clone)]
pub struct GlyphIter<'a, 'font, I: Iterator>
where
    I::Item: IntoGlyphId,
{
    pub(crate) font: &'a RusttypeFont<'font>,
    pub(crate) itr: I,
}

impl<'a, 'font, I> Iterator for GlyphIter<'a, 'font, I>
where
    I: Iterator,
    I::Item: IntoGlyphId,
{
    type Item = Glyph<'font>;

    fn next(&mut self) -> Option<Glyph<'font>> {
        self.itr.next().map(|c| self.font.glyph(c))
    }
}

#[derive(Clone)]
pub struct LayoutIter<'a, 'font, 's> {
    pub(crate) font: &'a RusttypeFont<'font>,
    pub(crate) chars: core::str::Chars<'s>,
    pub(crate) caret: f32,
    pub(crate) scale: Scale,
    pub(crate) start: Vec2,
    pub(crate) last_glyph: Option<GlyphId>,
}

impl<'a, 'font, 's> Iterator for LayoutIter<'a, 'font, 's> {
    type Item = PositionedGlyph<'font>;

    fn next(&mut self) -> Option<PositionedGlyph<'font>> {
        self.chars.next().map(|c| {
            let g = self.font.glyph(c).scaled(self.scale);
            if let Some(last) = self.last_glyph {
                self.caret += self.font.pair_kerning(self.scale, last, g.id());
            }
            let g = g.positioned(vec2(self.start.x + self.caret, self.start.y));
            self.caret += g.sg.h_metrics().advance_width;
            self.last_glyph = Some(g.id());
            g
        })
    }
}

pub(crate) trait NearZero {
    /// Returns if this number is kinda pretty much zero.
    fn is_near_zero(&self) -> bool;
}
impl NearZero for f32 {
    #[inline]
    fn is_near_zero(&self) -> bool {
        self.abs() <= core::f32::EPSILON
    }
}

/*
use font::Font;
use rusttype::Scale;
use simple_pixels::{
    rgb::{RGB8, RGBA8},
    start, Config, Context, KeyCode, State,
};
fn main() {
    let (width, height) = (400, 400);
    let config = Config {
        window_title: "FLOATING".to_string(),
        window_width: width,
        window_height: height,
        fullscreen: false,
        icon: None,
    };

    let game = Game::new(width, height);
    start(config, game);
}

struct Game {
    mouse_pos: Vec2,
    width: u32,
    height: u32,
    fonts: Vec<Font<'static>>,
}

impl Game {
    pub fn new(width: u32, height: u32) -> Self {
        let mouse_pos = Vec2::new(0.0, 0.0);

        let center = Vec2::new((width / 2) as f32, (height / 2) as f32);

        // Load the font
        let mut fonts: Vec<Font<'static>> = Vec::new();

        fonts.push(
            Font::try_from_bytes(include_bytes!("../fonts/NotoSansCJK-Regular.ttc") as &[u8])
                .unwrap(),
        );
        fonts.push(
            Font::try_from_bytes(include_bytes!("../fonts/VictorMono-Italic.ttf") as &[u8])
                .unwrap(),
        );
        fonts.push(
            Font::try_from_bytes(include_bytes!("../fonts/VictorMono-Regular.ttf") as &[u8])
                .unwrap(),
        );

        Self {
            mouse_pos,
            width,
            height,
            fonts,
        }
    }
}

impl State for Game {
    fn update(&mut self, ctx: &mut Context) {
        if ctx.is_key_down(KeyCode::Escape) {
            ctx.quit();
        }

        let mouse = ctx.get_mouse_pos();
    }

    fn draw(&mut self, ctx: &mut Context) {
        ctx.clear();
        let center = Vec2::new((self.width / 2) as f32, (self.height / 2) as f32);

        // The font size to use
        let scale = Scale::uniform(32.0);

        // The text to render
        let text = "This is RustType!おはよう日本語";

        let v_metrics = self.fonts[0].v_metrics(scale);

        // layout the glyphs in a line with 20 pixels padding
        let glyphs: Vec<_> = self.fonts[0]
            .layout(text, scale, vec2(20.0, 20.0 + v_metrics.ascent))
            .collect();

        // work out the layout size
        let glyphs_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;
        let glyphs_width = {
            let min_x = glyphs
                .first()
                .map(|g| g.pixel_bounding_box().unwrap().min.x)
                .unwrap();
            let max_x = glyphs
                .last()
                .map(|g| g.pixel_bounding_box().unwrap().max.x)
                .unwrap();
            (max_x - min_x) as u32
        };

        // Loop through the glyphs in the text, positing each one on a line
        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                // Draw the glyph into the image per-pixel by using the draw closure
                glyph.draw(|x, y, alpha| {
                    ctx.draw_pixel(
                        // Offset the position by the glyph bounding box
                        x + bounding_box.min.x as u32,
                        y + bounding_box.min.y as u32,
                        // Turn the coverage into an alpha value
                        color_lerp(RGB8::new(150, 60, 255), RGB8::new(0, 0, 0), alpha),
                    )
                });
            }
        }
    }
}

pub fn color_lerp(a: RGB8, b: RGB8, t: f32) -> RGBA8 {
    RGBA8::new(
        (a.r as f32 * t + b.r as f32 * (1.0 - t)) as u8,
        (a.g as f32 * t + b.g as f32 * (1.0 - t)) as u8,
        (a.b as f32 * t + b.b as f32 * (1.0 - t)) as u8,
        0, //(t * 255.0) as u8,
    )
}

*/
