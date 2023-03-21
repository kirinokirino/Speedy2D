pub mod font;
pub mod outliner;
pub mod rasterizer;

use crate::rusttype::font::RusttypeFont;
use glam::{vec2, Vec2};
use glam_rect::Rect;

use core::fmt;
pub use owned_ttf_parser::{GlyphId, OutlineBuilder};

/// Linear interpolation between points.
#[inline]
pub(crate) fn lerp(t: f32, p0: Vec2, p1: Vec2) -> Vec2 {
    vec2(p0.x + t * (p1.x - p0.x), p0.y + t * (p1.y - p0.y))
}

/// Defines the size of a rendered face of a font, in pixels, horizontally and
/// vertically. A vertical scale of `y` pixels means that the distance between
/// the ascent and descent lines (see `VMetrics`) of the face will be `y`
/// pixels. If `x` and `y` are equal the scaling is uniform. Non-uniform scaling
/// by a factor *f* in the horizontal direction is achieved by setting `x` equal
/// to *f* times `y`.
pub type Scale = Vec2;

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

        let offset = vec2(bb.top_left.x, bb.top_left.y);

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
        if p_diff.x.fract().abs() <= core::f32::EPSILON
            && p_diff.y.fract().abs() <= core::f32::EPSILON
        {
            if let Some(bb) = self.bb.as_mut() {
                let rounded_diff = vec2(p_diff.x.round(), p_diff.y.round());
                bb.top_left += rounded_diff;
                bb.bottom_right += rounded_diff;
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
impl IntoGlyphId for GlyphId {
    #[inline]
    fn into_glyph_id(self, _font: &RusttypeFont<'_>) -> GlyphId {
        self
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
