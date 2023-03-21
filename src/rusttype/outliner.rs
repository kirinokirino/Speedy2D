use glam::{vec2, Vec2};
use owned_ttf_parser::OutlineBuilder;

use crate::rusttype::rasterizer::Rasterizer;

pub(crate) struct OutlineScaler<'b, T: ?Sized> {
    inner: &'b mut T,
    scale: Vec2,
}

impl<'b, T: ?Sized> OutlineScaler<'b, T> {
    pub(crate) fn new(inner: &'b mut T, scale: Vec2) -> Self {
        Self { inner, scale }
    }
}

impl<T: OutlineBuilder + ?Sized> OutlineBuilder for OutlineScaler<'_, T> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.inner.move_to(x * self.scale.x, y * self.scale.y)
    }

    fn line_to(&mut self, x1: f32, y1: f32) {
        self.inner.line_to(x1 * self.scale.x, y1 * self.scale.y)
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.inner.quad_to(
            x1 * self.scale.x,
            y1 * self.scale.y,
            x2 * self.scale.x,
            y2 * self.scale.y,
        )
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        self.inner.curve_to(
            x1 * self.scale.x,
            y1 * self.scale.y,
            x2 * self.scale.x,
            y2 * self.scale.y,
            x3 * self.scale.x,
            y3 * self.scale.y,
        )
    }

    fn close(&mut self) {
        self.inner.close()
    }
}

pub(crate) struct OutlineTranslator<'b, T: ?Sized> {
    inner: &'b mut T,
    translation: Vec2,
}

impl<'b, T: ?Sized> OutlineTranslator<'b, T> {
    pub(crate) fn new(inner: &'b mut T, translation: Vec2) -> Self {
        Self { inner, translation }
    }
}

impl<T: OutlineBuilder + ?Sized> OutlineBuilder for OutlineTranslator<'_, T> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.inner
            .move_to(x + self.translation.x, y + self.translation.y)
    }

    fn line_to(&mut self, x1: f32, y1: f32) {
        self.inner
            .line_to(x1 + self.translation.x, y1 + self.translation.y)
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.inner.quad_to(
            x1 + self.translation.x,
            y1 + self.translation.y,
            x2 + self.translation.x,
            y2 + self.translation.y,
        )
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        self.inner.curve_to(
            x1 + self.translation.x,
            y1 + self.translation.y,
            x2 + self.translation.x,
            y2 + self.translation.y,
            x3 + self.translation.x,
            y3 + self.translation.y,
        )
    }

    fn close(&mut self) {
        self.inner.close()
    }
}

pub(crate) struct OutlineRasterizer {
    pub(crate) rasterizer: Rasterizer,
    last: Vec2,
    last_move: Option<Vec2>,
}

impl OutlineRasterizer {
    pub(crate) fn new(width: usize, height: usize) -> Self {
        Self {
            rasterizer: Rasterizer::new(width, height),
            last: vec2(0.0, 0.0),
            last_move: None,
        }
    }
}

impl OutlineBuilder for OutlineRasterizer {
    fn move_to(&mut self, x: f32, y: f32) {
        self.last = Vec2 { x, y };
        self.last_move = Some(self.last);
    }

    fn line_to(&mut self, x1: f32, y1: f32) {
        let p1 = Vec2 { x: x1, y: y1 };

        self.rasterizer.draw_line(self.last, p1);
        self.last = p1;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let p1 = Vec2 { x: x1, y: y1 };
        let p2 = Vec2 { x: x2, y: y2 };

        self.rasterizer.draw_quad(self.last, p1, p2);
        self.last = p2;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        let p1 = Vec2 { x: x1, y: y1 };
        let p2 = Vec2 { x: x2, y: y2 };
        let p3 = Vec2 { x: x3, y: y3 };

        self.rasterizer.draw_cubic(self.last, p1, p2, p3);
        self.last = p3;
    }

    fn close(&mut self) {
        if let Some(m) = self.last_move {
            self.rasterizer.draw_line(self.last, m);
        }
    }
}
