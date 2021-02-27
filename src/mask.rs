/* Copyright (C) 2021 Casper Meijn <casper@meijn.net>
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use crate::matrix::{Matrix, Module, Color};
use core::iter::Peekable;

impl<const N: usize> Matrix<N> {
    pub fn mask(&self, reference: u8) -> Self {
        let condition = match reference {
            0b000 => |x, y| (x + y) % 2 == 0,
            0b001 => |x, _y| x % 2 == 0,
            0b010 => |_x, y| y % 3 == 0,
            0b011 => |x, y| (x + y) % 3 == 0,
            0b100 => |x, y| ((x / 2) + (y / 3)) % 2 == 0,
            0b101 => |x, y| (x * y) % 2 + (x * y) % 3 == 0,
            0b110 => |x, y| ((x * y) % 2 + (x * y) % 3) % 2 == 0,
            0b111 => |x, y| ((x + y) % 2 + (x * y) % 3) % 2 == 0,
            _ => panic!(),
        };
        let mut masked = *self;
        let size = masked.data.size();
        for x in 0..size.x {
            for y in 0..size.y {
                let module = &mut masked.data[(x, y).into()];
                if let Module::Filled(color) = module {
                    if condition(x, y) {
                        *module = Module::Filled(color.inverse())
                    }
                }
            }
        }
        masked
    }

    fn score_adjacent_horizontal(&self) -> usize {
        self.data
            .rows()
            .map(|row| {
                AdjacentIterator::new(row)
                    .filter(|&i| i >= 5)
                    .map(|i| i - 2)
                    .sum::<usize>()
            })
            .sum()
    }

    fn score_adjacent_vertical(&self) -> usize {
        self.data
            .columns()
            .map(|row| {
                AdjacentIterator::new(row)
                    .filter(|&i| i >= 5)
                    .map(|i| i - 2)
                    .sum::<usize>()
            })
            .sum()
    }

    fn score_blocks(&self) -> usize {
        let size = self.data.size();
        (0..size.x - 1).map(|x| {
            (0..size.y - 1).map(|y| {
                let top_left: Color = self.data[(x,y).into()].into();
                let top_right: Color = self.data[(x,y + 1).into()].into();
                let bottom_left: Color = self.data[(x + 1,y).into()].into();
                let bottom_right: Color = self.data[(x + 1,y + 1).into()].into();
                if top_left == top_right &&
                    top_left == bottom_left &&
                    top_left == bottom_right {
                    3
                } else {
                    0
                }
            }).sum::<usize>()
        }).sum()
    }

    fn score_match_pattern<'a, T>(mut iter: T) -> usize
        where T: Iterator<Item = &'a Module>{
        let match_pattern1 = [Color::Black, Color::White, Color::Black, Color::Black, Color::Black, Color::White, Color::Black, Color::White, Color::White, Color::White, Color::White, ];
        let match_pattern2 = [ Color::White, Color::White, Color::White, Color::White, Color::Black, Color::White, Color::Black, Color::Black, Color::Black, Color::White, Color::Black,];
        let initial_pattern = |iter: &mut T| {
            let mut pattern = [Color::White; 11];
            for i in 0..11 {
                let module = *iter.next().unwrap();
                pattern[i] = module.into();
            }
            pattern
        };

        let shift_pattern = |mut pattern: [Color;11], iter: &mut T| -> Option<[Color;11]>{
            if let Some(&next) = iter.next() {
                for i in 0..10 {
                    pattern[i] = pattern[i+1];
                }
                pattern[10] = next.into();
                Some(pattern)
            } else {
                None
            }
        };


        let mut pattern = initial_pattern(&mut iter);
        let mut total = if pattern == match_pattern1 || pattern == match_pattern2 { 1 } else { 0 };
        while let Some(shifted_pattern) = shift_pattern(pattern, &mut iter) {
            pattern = shifted_pattern;
            if pattern == match_pattern1 || pattern == match_pattern2 {
                total += 1;
            }
        }
        total
    }


    fn score_pattern_horizontal(&self) -> usize {
        self.data.rows().map(|row| {
            Self::score_match_pattern(row)
        }).sum::<usize>() * 40
    }

    fn score_pattern_vertical(&self) -> usize {
        self.data.columns().map(|column| {
            Self::score_match_pattern(column)
        }).sum::<usize>() * 40
    }

    fn score_proportion(&self) -> usize {
        let black_count: usize = self.data.rows().map(|row| {
            row.filter(|&&module| {
                let color:Color = module.into();
                color == Color::Black
            }).count()
        }).sum();
        let size = self.data.size();
        let percentage = black_count * 100 / (size.x * size.y);
        let k = if percentage < 50 { 50 - percentage } else { percentage - 50};
        k / 5 * 10
    }

    fn score(&self) -> usize {
        self.score_adjacent_horizontal() +
        self.score_adjacent_vertical() +
        self.score_blocks() +
        self.score_pattern_horizontal() +
        self.score_pattern_vertical() +
        self.score_proportion()
    }
}

struct AdjacentIterator<'a, T>
where
    T: Iterator<Item = &'a Module>,
{
    data_iter: Peekable<T>,
}

impl<'a, T> AdjacentIterator<'a, T>
where
    T: Iterator<Item = &'a Module>,
{
    fn new(data_iter: T) -> Self {
        Self {
            data_iter: data_iter.peekable(),
        }
    }
}

impl<'a, T> Iterator for AdjacentIterator<'a, T>
where
    T: Iterator<Item = &'a Module>,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(&first) = self.data_iter.next() {
            let first: Color = first.into();
            let mut count = 1;
            while let Some(&&later) = self.data_iter.peek() {
                let later: Color = later.into();
                if first == later {
                    count += 1;
                    self.data_iter.next();
                } else {
                    break;
                }
            }
            Some(count)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::matrix::{Color, Matrix, Module};
    use alloc::format;
    use crate::qr_version::Version;

    #[test]
    fn mask_pattern0() {
        let mut matrix = Matrix::<21>::new();
        matrix.fill_whole(Module::Filled(Color::White));
        let masked = matrix.mask(0);

        assert_eq!(
            format!("{:?}", masked),
            "\
█_█_█_█_█_█_█_█_█_█_█
_█_█_█_█_█_█_█_█_█_█_
█_█_█_█_█_█_█_█_█_█_█
_█_█_█_█_█_█_█_█_█_█_
█_█_█_█_█_█_█_█_█_█_█
_█_█_█_█_█_█_█_█_█_█_
█_█_█_█_█_█_█_█_█_█_█
_█_█_█_█_█_█_█_█_█_█_
█_█_█_█_█_█_█_█_█_█_█
_█_█_█_█_█_█_█_█_█_█_
█_█_█_█_█_█_█_█_█_█_█
_█_█_█_█_█_█_█_█_█_█_
█_█_█_█_█_█_█_█_█_█_█
_█_█_█_█_█_█_█_█_█_█_
█_█_█_█_█_█_█_█_█_█_█
_█_█_█_█_█_█_█_█_█_█_
█_█_█_█_█_█_█_█_█_█_█
_█_█_█_█_█_█_█_█_█_█_
█_█_█_█_█_█_█_█_█_█_█
_█_█_█_█_█_█_█_█_█_█_
█_█_█_█_█_█_█_█_█_█_█
"
        );
    }

    #[test]
    fn mask_pattern1() {
        let mut matrix = Matrix::<21>::new();
        matrix.fill_whole(Module::Filled(Color::White));
        let masked = matrix.mask(1);

        assert_eq!(
            format!("{:?}", masked),
            "\
█████████████████████
_____________________
█████████████████████
_____________________
█████████████████████
_____________________
█████████████████████
_____________________
█████████████████████
_____________________
█████████████████████
_____________________
█████████████████████
_____________________
█████████████████████
_____________________
█████████████████████
_____________________
█████████████████████
_____________________
█████████████████████
"
        );
    }

    #[test]
    fn mask_pattern2() {
        let mut matrix = Matrix::<21>::new();
        matrix.fill_whole(Module::Filled(Color::White));
        let masked = matrix.mask(2);

        assert_eq!(
            format!("{:?}", masked),
            "\
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
█__█__█__█__█__█__█__
"
        );
    }

    #[test]
    fn mask_pattern3() {
        let mut matrix = Matrix::<21>::new();
        matrix.fill_whole(Module::Filled(Color::White));
        let masked = matrix.mask(3);

        assert_eq!(
            format!("{:?}", masked),
            "\
█__█__█__█__█__█__█__
__█__█__█__█__█__█__█
_█__█__█__█__█__█__█_
█__█__█__█__█__█__█__
__█__█__█__█__█__█__█
_█__█__█__█__█__█__█_
█__█__█__█__█__█__█__
__█__█__█__█__█__█__█
_█__█__█__█__█__█__█_
█__█__█__█__█__█__█__
__█__█__█__█__█__█__█
_█__█__█__█__█__█__█_
█__█__█__█__█__█__█__
__█__█__█__█__█__█__█
_█__█__█__█__█__█__█_
█__█__█__█__█__█__█__
__█__█__█__█__█__█__█
_█__█__█__█__█__█__█_
█__█__█__█__█__█__█__
__█__█__█__█__█__█__█
_█__█__█__█__█__█__█_
"
        );
    }

    #[test]
    fn mask_pattern4() {
        let mut matrix = Matrix::<21>::new();
        matrix.fill_whole(Module::Filled(Color::White));
        let masked = matrix.mask(4);

        assert_eq!(
            format!("{:?}", masked),
            "\
███___███___███___███
███___███___███___███
___███___███___███___
___███___███___███___
███___███___███___███
███___███___███___███
___███___███___███___
___███___███___███___
███___███___███___███
███___███___███___███
___███___███___███___
___███___███___███___
███___███___███___███
███___███___███___███
___███___███___███___
___███___███___███___
███___███___███___███
███___███___███___███
___███___███___███___
___███___███___███___
███___███___███___███
"
        );
    }

    #[test]
    fn mask_pattern5() {
        let mut matrix = Matrix::<21>::new();
        matrix.fill_whole(Module::Filled(Color::White));
        let masked = matrix.mask(5);

        assert_eq!(
            format!("{:?}", masked),
            "\
█████████████████████
█_____█_____█_____█__
█__█__█__█__█__█__█__
█_█_█_█_█_█_█_█_█_█_█
█__█__█__█__█__█__█__
█_____█_____█_____█__
█████████████████████
█_____█_____█_____█__
█__█__█__█__█__█__█__
█_█_█_█_█_█_█_█_█_█_█
█__█__█__█__█__█__█__
█_____█_____█_____█__
█████████████████████
█_____█_____█_____█__
█__█__█__█__█__█__█__
█_█_█_█_█_█_█_█_█_█_█
█__█__█__█__█__█__█__
█_____█_____█_____█__
█████████████████████
█_____█_____█_____█__
█__█__█__█__█__█__█__
"
        );
    }

    #[test]
    fn mask_pattern6() {
        let mut matrix = Matrix::<21>::new();
        matrix.fill_whole(Module::Filled(Color::White));
        let masked = matrix.mask(6);

        assert_eq!(
            format!("{:?}", masked),
            "\
█████████████████████
███___███___███___███
██_██_██_██_██_██_██_
█_█_█_█_█_█_█_█_█_█_█
█_██_██_██_██_██_██_█
█___███___███___███__
█████████████████████
███___███___███___███
██_██_██_██_██_██_██_
█_█_█_█_█_█_█_█_█_█_█
█_██_██_██_██_██_██_█
█___███___███___███__
█████████████████████
███___███___███___███
██_██_██_██_██_██_██_
█_█_█_█_█_█_█_█_█_█_█
█_██_██_██_██_██_██_█
█___███___███___███__
█████████████████████
███___███___███___███
██_██_██_██_██_██_██_
"
        );
    }

    #[test]
    fn mask_pattern7() {
        let mut matrix = Matrix::<21>::new();
        matrix.fill_whole(Module::Filled(Color::White));
        let masked = matrix.mask(7);

        assert_eq!(
            format!("{:?}", masked),
            "\
█_█_█_█_█_█_█_█_█_█_█
___███___███___███___
█___███___███___███__
_█_█_█_█_█_█_█_█_█_█_
███___███___███___███
_███___███___███___██
█_█_█_█_█_█_█_█_█_█_█
___███___███___███___
█___███___███___███__
_█_█_█_█_█_█_█_█_█_█_
███___███___███___███
_███___███___███___██
█_█_█_█_█_█_█_█_█_█_█
___███___███___███___
█___███___███___███__
_█_█_█_█_█_█_█_█_█_█_
███___███___███___███
_███___███___███___██
█_█_█_█_█_█_█_█_█_█_█
___███___███___███___
█___███___███___███__
"
        );
    }

    #[test]
    fn mask() {
        let mut matrix = Matrix::<21>::new();
        matrix.fill_finder_patterns();
        matrix.fill_reserved();
        matrix.fill_timing_pattern();

        matrix.place_data(
            [
                0b00010000, 0b00100000, 0b00001100, 0b01010110, 0b01100001, 0b10000000, 0b11101100,
                0b00010001, 0b11101100, 0b00010001, 0b11101100, 0b00010001, 0b11101100, 0b00010001,
                0b11101100, 0b00010001, 0b10100101, 0b00100100, 0b11010100, 0b11000001, 0b11101101,
                0b00110110, 0b11000111, 0b10000111, 0b00101100, 0b01010101,
            ]
            .iter(),
        );

        let masked = matrix.mask(0b010);

        assert_eq!(
            format!("{:?}", masked),
            "\
▓▓▓▓▓▓▓░▒█_██░▓▓▓▓▓▓▓
▓░░░░░▓░▒████░▓░░░░░▓
▓░▓▓▓░▓░▒____░▓░▓▓▓░▓
▓░▓▓▓░▓░▒█___░▓░▓▓▓░▓
▓░▓▓▓░▓░▒_███░▓░▓▓▓░▓
▓░░░░░▓░▒___█░▓░░░░░▓
▓▓▓▓▓▓▓░▓░▓░▓░▓▓▓▓▓▓▓
░░░░░░░░▒__██░░░░░░░░
▒▒▒▒▒▒▓▒▒█__█▒▒▒▒▒▒▒▒
___█_█░██_█_█__█_██__
__█___▓█_█_█_█__█████
____█_░__█_____████__
___███▓██__█_█__█____
░░░░░░░░▒_█████__██__
▓▓▓▓▓▓▓░▒██_█_██_____
▓░░░░░▓░▒_█████___█_█
▓░▓▓▓░▓░▒___█__█_██__
▓░▓▓▓░▓░▒█__█__█_____
▓░▓▓▓░▓░▒_██_█__█_█__
▓░░░░░▓░▒______██_██_
▓▓▓▓▓▓▓░▒███_█__█_█__
"
        );

        let twice_masked = masked.mask(0b010);

        assert_eq!(format!("{:?}", twice_masked), format!("{:?}", matrix),);
    }

    #[test]
    fn score() {
        let mut matrix = Matrix::<21>::new();
        matrix.set_version(Version{version: 1});
        matrix.fill_symbol();

        // "HELLO WORLD" with version 1-Q
        matrix.place_data(
            [32, 91, 11, 120, 209, 114, 220, 77, 67, 64, 236, 17, 236, 168, 72, 22, 82, 217, 54, 156, 0, 46, 15, 180, 122, 16]
            .iter(),
        );
        let masked = matrix.mask(0);

        let adjacent_horizontal = masked.score_adjacent_horizontal();
        assert_eq!(adjacent_horizontal, 101);

        let adjacent_vertical = masked.score_adjacent_vertical();
        assert_eq!(adjacent_vertical, 101);

        let blocks = masked.score_blocks();
        assert_eq!(blocks, 207);

        let pattern_horizontal = masked.score_pattern_horizontal();
        assert_eq!(pattern_horizontal, 200);

        let pattern_vertical = masked.score_pattern_vertical();
        assert_eq!(pattern_vertical, 120);

        let proportion = masked.score_proportion();
        assert_eq!(proportion, 10);

        let total = masked.score();
        assert_eq!(total, 739);

        let masked = matrix.mask(1);
        let total = masked.score();
        assert_eq!(total, 507);

        let masked = matrix.mask(2);
        let total = masked.score();
        assert_eq!(total, 638);

        let masked = matrix.mask(3);
        let total = masked.score();
        assert_eq!(total, 569);

        let masked = matrix.mask(4);
        let total = masked.score();
        assert_eq!(total, 763);

        let masked = matrix.mask(5);
        let total = masked.score();
        assert_eq!(total, 572);

        let masked = matrix.mask(6);
        let total = masked.score();
        assert_eq!(total, 440);

        let masked = matrix.mask(7);
        let total = masked.score();
        assert_eq!(total, 829);

        let masked = matrix.mask(7);
        let total = masked.score();
        assert_eq!(total, 829);
    }


}
