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

use std::convert::TryInto;

use crate::numeric::{PrimitiveZero, RoundFloat};
use glam::{IVec2, UVec2, Vec2};

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_arithmetic() {
        assert_eq!(IVec2::new(15, 20), IVec2::new(10, 4) + IVec2::new(5, 16));

        assert_eq!(IVec2::new(5, -12), IVec2::new(10, 4) - IVec2::new(5, 16));

        assert_eq!(IVec2::new(-5, 10), IVec2::new(3, 10) - IVec2::new(8, 0));

        assert_eq!(IVec2::new(-5, 17), IVec2::new(-5, 10) + IVec2::new(0, 7));
    }

    #[test]
    fn test_add_assign() {
        let mut left = IVec2::new(1, 2);
        let right = IVec2::new(3, 4);
        left += right;
        assert_eq!(left, IVec2::new(4, 6));
    }

    #[test]
    fn test_sub_assign() {
        let mut left = IVec2::new(9, 8);
        let right = IVec2::new(1, 2);
        left -= right;
        assert_eq!(left, IVec2::new(8, 6));
    }

    #[test]
    fn test_mul_assign() {
        let mut left = IVec2::new(2, 3);
        left *= 5;
        assert_eq!(left, IVec2::new(10, 15));
    }

    #[test]
    fn test_div_assign() {
        let mut left = IVec2::new(12, 8);
        left /= 2;
        assert_eq!(left, IVec2::new(6, 4));
    }
}
