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

use glam::Vec2;

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
