/*
 *  Copyright 2021 QuantumBadger
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

use glam::{IVec2, UVec2, Vec2};

/// A struct representing an axis-aligned rectangle. Two points are stored: the
/// top left vertex, and the bottom right vertex.
#[derive(Debug, PartialEq, Clone)]
#[repr(C)]
pub struct Rect {
    pub top_left: Vec2,
    pub bottom_right: Vec2,
}

impl Rect {
    /// Constructs a new `Rect`. The top left vertex must be above and to
    /// the left of the bottom right vertex.
    #[inline]
    pub const fn new(top_left: Vec2, bottom_right: Vec2) -> Self {
        Self {
            top_left,
            bottom_right,
        }
    }

    #[inline]
    pub const fn corners(self) -> [Vec2; 4] {
        let top_right = Vec2::new(self.bottom_right.x, self.top_left.y);
        let bottom_left = Vec2::new(self.top_left.x, self.bottom_right.y);
        [self.top_left, top_right, self.bottom_right, bottom_left]
    }

    /// Constructs a new `Rect`. The top left vertex must be above and to
    /// the left of the bottom right vertex.
    #[inline]
    pub fn from_tuples(top_left: (f32, f32), bottom_right: (f32, f32)) -> Self {
        Self {
            top_left: Vec2::new(top_left.0, top_left.1),
            bottom_right: Vec2::new(bottom_right.0, bottom_right.1),
        }
    }

    /// Returns the width of the rectangle.
    #[inline]
    pub fn width(&self) -> f32 {
        self.bottom_right.x - self.top_left.x
    }

    /// Returns the height of the rectangle.
    #[inline]
    pub fn height(&self) -> f32 {
        self.bottom_right.y - self.top_left.y
    }

    /// Returns a `Vector2` containing the width and height of the rectangle.
    #[inline]
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }
    /// Returns true if the specified point is inside this rectangle. This is
    /// inclusive of the top and left coordinates, and exclusive of the bottom
    /// and right coordinates.
    #[inline]
    #[must_use]
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.top_left.x
            && point.y >= self.top_left.y
            && point.x < self.bottom_right.x
            && point.y < self.bottom_right.y
    }
    /// Finds the intersection of two rectangles -- in other words, the area
    /// that is common to both of them.
    ///
    /// If there is no common area between the two rectangles, then this
    /// function will return `None`.
    #[inline]
    #[must_use]
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let result = Self {
            top_left: Vec2::new(
                self.top_left.x.max(other.top_left.x),
                self.top_left.y.max(other.top_left.y),
            ),
            bottom_right: Vec2::new(
                self.bottom_right.x.min(other.bottom_right.x),
                self.bottom_right.y.min(other.bottom_right.y),
            ),
        };

        if result.is_positive_area() {
            Some(result)
        } else {
            None
        }
    }
    /// A constant representing a rectangle with position (0, 0) and zero area.
    /// Each component is set to zero.
    pub const ZERO: Rect = Rect::new(Vec2::ZERO, Vec2::ZERO);
    /// Returns `true` if the rectangle has zero area.
    #[inline]
    pub fn is_zero_area(&self) -> bool {
        self.top_left.x == self.bottom_right.x || self.top_left.y == self.bottom_right.y
    }
    /// Returns `true` if the rectangle has an area greater than zero.
    #[inline]
    pub fn is_positive_area(&self) -> bool {
        self.top_left.x < self.bottom_right.x && self.top_left.y < self.bottom_right.y
    }
    /// Returns a new rectangle, whose vertices are offset relative to the
    /// current rectangle by the specified amount. This is equivalent to
    /// adding the specified vector to each vertex.
    #[inline]
    pub fn with_offset(&self, offset: impl Into<Vec2>) -> Self {
        let offset = offset.into();
        Self::new(self.top_left + offset, self.bottom_right + offset)
    }
    /// Returns a new rectangle, whose vertices are negatively offset relative
    /// to the current rectangle by the specified amount. This is equivalent
    /// to subtracting the specified vector to each vertex.
    #[inline]
    pub fn with_negative_offset(&self, offset: impl Into<Vec2>) -> Self {
        let offset = offset.into();
        Self::new(self.top_left - offset, self.bottom_right - offset)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub struct URect {
    pub top_left: UVec2,
    pub bottom_right: UVec2,
}

impl URect {
    /// Constructs a new `Rect`. The top left vertex must be above and to
    /// the left of the bottom right vertex.
    #[inline]
    pub const fn new(top_left: UVec2, bottom_right: UVec2) -> Self {
        Self {
            top_left,
            bottom_right,
        }
    }

    /// Constructs a new `Rect`. The top left vertex must be above and to
    /// the left of the bottom right vertex.
    #[inline]
    pub fn from_tuples(top_left: (u32, u32), bottom_right: (u32, u32)) -> Self {
        Self {
            top_left: UVec2::new(top_left.0, top_left.1),
            bottom_right: UVec2::new(bottom_right.0, bottom_right.1),
        }
    }

    /// Returns the width of the rectangle.
    #[inline]
    pub fn width(&self) -> u32 {
        self.bottom_right.x - self.top_left.x
    }

    /// Returns the height of the rectangle.
    #[inline]
    pub fn height(&self) -> u32 {
        self.bottom_right.y - self.top_left.y
    }

    /// Returns a `Vector2` containing the width and height of the rectangle.
    #[inline]
    pub fn size(&self) -> UVec2 {
        UVec2::new(self.width(), self.height())
    }
    /// Returns true if the specified point is inside this rectangle. This is
    /// inclusive of the top and left coordinates, and exclusive of the bottom
    /// and right coordinates.
    #[inline]
    #[must_use]
    pub fn contains(&self, point: UVec2) -> bool {
        point.x >= self.top_left.x
            && point.y >= self.top_left.y
            && point.x < self.bottom_right.x
            && point.y < self.bottom_right.y
    }
    /// Finds the intersection of two rectangles -- in other words, the area
    /// that is common to both of them.
    ///
    /// If there is no common area between the two rectangles, then this
    /// function will return `None`.
    #[inline]
    #[must_use]
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let result = Self {
            top_left: UVec2::new(
                self.top_left.x.max(other.top_left.x),
                self.top_left.y.max(other.top_left.y),
            ),
            bottom_right: UVec2::new(
                self.bottom_right.x.min(other.bottom_right.x),
                self.bottom_right.y.min(other.bottom_right.y),
            ),
        };

        if result.is_positive_area() {
            Some(result)
        } else {
            None
        }
    }
    /// A constant representing a rectangle with position (0, 0) and zero area.
    /// Each component is set to zero.
    pub const ZERO: URect = URect::new(UVec2::ZERO, UVec2::ZERO);
    /// Returns `true` if the rectangle has zero area.
    #[inline]
    pub fn is_zero_area(&self) -> bool {
        self.top_left.x == self.bottom_right.x || self.top_left.y == self.bottom_right.y
    }
    /// Returns `true` if the rectangle has an area greater than zero.
    #[inline]
    pub fn is_positive_area(&self) -> bool {
        self.top_left.x < self.bottom_right.x && self.top_left.y < self.bottom_right.y
    }
    /// Returns a new rectangle, whose vertices are offset relative to the
    /// current rectangle by the specified amount. This is equivalent to
    /// adding the specified vector to each vertex.
    #[inline]
    pub fn with_offset(&self, offset: impl Into<UVec2>) -> Self {
        let offset = offset.into();
        Self::new(self.top_left + offset, self.bottom_right + offset)
    }
    /// Returns a new rectangle, whose vertices are negatively offset relative
    /// to the current rectangle by the specified amount. This is equivalent
    /// to subtracting the specified vector to each vertex.
    #[inline]
    pub fn with_negative_offset(&self, offset: impl Into<UVec2>) -> Self {
        let offset = offset.into();
        Self::new(self.top_left - offset, self.bottom_right - offset)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(C)]
pub struct IRect {
    pub top_left: IVec2,
    pub bottom_right: IVec2,
}

impl IRect {
    /// Constructs a new `Rect`. The top left vertex must be above and to
    /// the left of the bottom right vertex.
    #[inline]
    pub const fn new(top_left: IVec2, bottom_right: IVec2) -> Self {
        Self {
            top_left,
            bottom_right,
        }
    }

    /// Constructs a new `Rect`. The top left vertex must be above and to
    /// the left of the bottom right vertex.
    #[inline]
    pub fn from_tuples(top_left: (i32, i32), bottom_right: (i32, i32)) -> Self {
        Self {
            top_left: IVec2::new(top_left.0, top_left.1),
            bottom_right: IVec2::new(bottom_right.0, bottom_right.1),
        }
    }

    /// Returns the width of the rectangle.
    #[inline]
    pub fn width(&self) -> i32 {
        self.bottom_right.x - self.top_left.x
    }

    /// Returns the height of the rectangle.
    #[inline]
    pub fn height(&self) -> i32 {
        self.bottom_right.y - self.top_left.y
    }

    /// Returns a `Vector2` containing the width and height of the rectangle.
    #[inline]
    pub fn size(&self) -> IVec2 {
        IVec2::new(self.width(), self.height())
    }
    /// Returns true if the specified point is inside this rectangle. This is
    /// inclusive of the top and left coordinates, and exclusive of the bottom
    /// and right coordinates.
    #[inline]
    #[must_use]
    pub fn contains(&self, point: IVec2) -> bool {
        point.x >= self.top_left.x
            && point.y >= self.top_left.y
            && point.x < self.bottom_right.x
            && point.y < self.bottom_right.y
    }
    /// Finds the intersection of two rectangles -- in other words, the area
    /// that is common to both of them.
    ///
    /// If there is no common area between the two rectangles, then this
    /// function will return `None`.
    #[inline]
    #[must_use]
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let result = Self {
            top_left: IVec2::new(
                self.top_left.x.max(other.top_left.x),
                self.top_left.y.max(other.top_left.y),
            ),
            bottom_right: IVec2::new(
                self.bottom_right.x.min(other.bottom_right.x),
                self.bottom_right.y.min(other.bottom_right.y),
            ),
        };

        if result.is_positive_area() {
            Some(result)
        } else {
            None
        }
    }
    /// A constant representing a rectangle with position (0, 0) and zero area.
    /// Each component is set to zero.
    pub const ZERO: URect = URect::new(UVec2::ZERO, UVec2::ZERO);
    /// Returns `true` if the rectangle has zero area.
    #[inline]
    pub fn is_zero_area(&self) -> bool {
        self.top_left.x == self.bottom_right.x || self.top_left.y == self.bottom_right.y
    }
    /// Returns `true` if the rectangle has an area greater than zero.
    #[inline]
    pub fn is_positive_area(&self) -> bool {
        self.top_left.x < self.bottom_right.x && self.top_left.y < self.bottom_right.y
    }
    /// Returns a new rectangle, whose vertices are offset relative to the
    /// current rectangle by the specified amount. This is equivalent to
    /// adding the specified vector to each vertex.
    #[inline]
    pub fn with_offset(&self, offset: impl Into<IVec2>) -> Self {
        let offset = offset.into();
        Self::new(self.top_left + offset, self.bottom_right + offset)
    }
    /// Returns a new rectangle, whose vertices are negatively offset relative
    /// to the current rectangle by the specified amount. This is equivalent
    /// to subtracting the specified vector to each vertex.
    #[inline]
    pub fn with_negative_offset(&self, offset: impl Into<IVec2>) -> Self {
        let offset = offset.into();
        Self::new(self.top_left - offset, self.bottom_right - offset)
    }
}

/// A struct representing a polygon.
#[derive(Debug, Clone)]
pub struct Polygon {
    pub(crate) triangles: Vec<[Vec2; 3]>,
}

impl Polygon {
    /// Generate a new polygon given points that describe it's outline.
    ///
    /// The points must be in either clockwise or couter-clockwise order.
    pub fn new<Point: Into<Vec2> + Copy>(vertices: &[Point]) -> Self {
        // We have to flatten the vertices in order for
        // [earcutr](https://github.com/frewsxcv/earcutr/) to accept it.
        // In the future, we can add a triangulation algorithm directly into Speed2D if
        // performance is an issue, but for now, this is simpler and easier
        let mut flattened = Vec::with_capacity(vertices.len() * 2);

        for vertex in vertices {
            let vertex: Vec2 = (*vertex).into();

            flattened.push(vertex.x);
            flattened.push(vertex.y);
        }

        let mut triangulation = earcutr::earcut(&flattened, &Vec::new(), 2).unwrap();
        let mut triangles = Vec::with_capacity(triangulation.len() / 3);

        while !triangulation.is_empty() {
            triangles.push([
                vertices[triangulation.pop().unwrap()].into(),
                vertices[triangulation.pop().unwrap()].into(),
                vertices[triangulation.pop().unwrap()].into(),
            ])
        }

        Polygon { triangles }
    }
}

#[cfg(test)]
mod test {
    use crate::shape::Rect;

    #[test]
    pub fn test_intersect_1() {
        let r1 = Rect::from_tuples((100.0, 100.0), (200.0, 200.0));
        let r2 = Rect::from_tuples((100.0, 300.0), (200.0, 400.0));
        let r3 = Rect::from_tuples((125.0, 50.0), (175.0, 500.0));

        assert_eq!(None, r1.intersect(&r2));

        assert_eq!(
            Some(Rect::from_tuples((125.0, 100.0), (175.0, 200.0))),
            r1.intersect(&r3)
        );

        assert_eq!(
            Some(Rect::from_tuples((125.0, 300.0), (175.0, 400.0))),
            r2.intersect(&r3)
        );

        assert_eq!(Some(r1.clone()), r1.intersect(&r1));
        assert_eq!(Some(r2.clone()), r2.intersect(&r2));
        assert_eq!(Some(r3.clone()), r3.intersect(&r3));
    }

    #[test]
    pub fn test_intersect_2() {
        let r1 = Rect::from_tuples((100.0, 100.0), (200.0, 200.0));
        let r2 = Rect::from_tuples((100.0, 200.0), (200.0, 300.0));

        assert_eq!(None, r1.intersect(&r2));
    }
}
