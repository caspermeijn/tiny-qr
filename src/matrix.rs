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

use std::fmt::{Debug, Display, Formatter, Write};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Point {
        Point { x, y }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    fn inverse(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Module {
    /// Part of the encoded region and filled with a specific color
    Filled(Color),
    /// Part of the encoded region, but not yet filled with a color
    Empty,
    /// Part of the finder pattern and filled with a specific color
    Static(Color),
    /// Part of the QR code structure that is not yet filled with a color
    Reserved,
}

impl From<Module> for Color {
    fn from(module: Module) -> Self {
        match module {
            Module::Filled(color) => color,
            Module::Empty => Color::White,
            Module::Static(color) => color,
            Module::Reserved => Color::White,
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Matrix {
    data: [[Module; 21]; 21],
}

impl Default for Matrix {
    fn default() -> Self {
        Self::new()
    }
}

impl Matrix {
    pub fn new() -> Matrix {
        Matrix {
            data: [[Module::Empty; 21]; 21],
        }
    }

    pub fn size(&self) -> Point {
        Point::new(self.data.len(), self.data[0].len())
    }

    pub fn fill_whole(&mut self, data: Module) {
        self.data.iter_mut().for_each(|row| {
            row.iter_mut().for_each(|module| {
                *module = data;
            })
        });
    }

    pub fn fill_module(&mut self, pos: Point, data: Module) {
        self.data[pos.x][pos.y] = data;
    }

    pub fn fill_line(&mut self, pos1: Point, pos2: Point, data: Module) {
        if pos1.x == pos2.x {
            let x = pos1.x;
            assert!(pos1.y < pos2.y);
            for y in pos1.y..=pos2.y {
                self.fill_module(Point::new(x, y), data);
            }
        } else if pos1.y == pos2.y {
            let y = pos1.y;
            assert!(pos1.x < pos2.x);
            for x in pos1.x..=pos2.x {
                self.fill_module(Point::new(x, y), data);
            }
        } else {
            panic!()
        }
    }

    pub fn fill_finder_pattern(&mut self, pos: Point) {
        let black = Module::Static(Color::Black);
        let white = Module::Static(Color::White);

        self.fill_line(pos, Point::new(pos.x, pos.y + 6), black);
        self.fill_line(
            Point::new(pos.x, pos.y + 6),
            Point::new(pos.x + 5, pos.y + 6),
            black,
        );
        self.fill_line(
            Point::new(pos.x + 6, pos.y + 1),
            Point::new(pos.x + 6, pos.y + 6),
            black,
        );
        self.fill_line(
            Point::new(pos.x + 1, pos.y),
            Point::new(pos.x + 6, pos.y),
            black,
        );

        self.fill_line(
            Point::new(pos.x + 1, pos.y + 1),
            Point::new(pos.x + 1, pos.y + 4),
            white,
        );
        self.fill_line(
            Point::new(pos.x + 1, pos.y + 5),
            Point::new(pos.x + 4, pos.y + 5),
            white,
        );
        self.fill_line(
            Point::new(pos.x + 5, pos.y + 2),
            Point::new(pos.x + 5, pos.y + 5),
            white,
        );
        self.fill_line(
            Point::new(pos.x + 2, pos.y + 1),
            Point::new(pos.x + 5, pos.y + 1),
            white,
        );

        self.fill_module(Point::new(pos.x + 2, pos.y + 2), black);
        self.fill_module(Point::new(pos.x + 2, pos.y + 3), black);
        self.fill_module(Point::new(pos.x + 2, pos.y + 4), black);
        self.fill_module(Point::new(pos.x + 3, pos.y + 2), black);
        self.fill_module(Point::new(pos.x + 3, pos.y + 3), black);
        self.fill_module(Point::new(pos.x + 3, pos.y + 4), black);
        self.fill_module(Point::new(pos.x + 4, pos.y + 2), black);
        self.fill_module(Point::new(pos.x + 4, pos.y + 3), black);
        self.fill_module(Point::new(pos.x + 4, pos.y + 4), black);
    }

    pub fn fill_finder_patterns(&mut self) {
        let white = Module::Static(Color::White);
        let size = self.size();

        // Left-top
        self.fill_finder_pattern(Point::new(0, 0));
        self.fill_line(Point::new(0, 7), Point::new(7, 7), white);
        self.fill_line(Point::new(7, 0), Point::new(7, 6), white);

        // Left-bottom
        self.fill_finder_pattern(Point::new(size.x - 7, 0));
        self.fill_line(Point::new(size.x - 8, 0), Point::new(size.x - 8, 7), white);
        self.fill_line(Point::new(size.x - 8, 7), Point::new(size.x - 1, 7), white);

        // Right-top
        self.fill_finder_pattern(Point::new(0, size.y - 7));
        self.fill_line(Point::new(7, size.y - 8), Point::new(7, size.y - 1), white);
        self.fill_line(Point::new(0, size.y - 8), Point::new(7, size.y - 8), white);
    }

    pub fn fill_reserved(&mut self) {
        let reserved = Module::Reserved;
        let size = self.size();

        // Left-top
        self.fill_line(Point::new(0, 8), Point::new(5, 8), reserved);
        self.fill_line(Point::new(8, 0), Point::new(8, 5), reserved);
        self.fill_module(Point::new(7, 8), reserved);
        self.fill_module(Point::new(8, 8), reserved);
        self.fill_module(Point::new(8, 7), reserved);

        // Left-bottom
        self.fill_line(
            Point::new(size.x - 8, 8),
            Point::new(size.x - 1, 8),
            reserved,
        );
        // Right-top
        self.fill_line(
            Point::new(8, size.y - 8),
            Point::new(8, size.y - 1),
            reserved,
        );
    }

    pub fn fill_timing_pattern(&mut self) {
        fn color(i: usize) -> Module {
            if i % 2 == 0 {
                Module::Static(Color::Black)
            } else {
                Module::Static(Color::White)
            }
        }

        let size = self.size();

        let x = 6;
        for y in 8..size.y - 8 {
            self.fill_module(Point::new(x, y), color(y));
        }

        let y = 6;
        for x in 8..size.x - 8 {
            self.fill_module(Point::new(x, y), color(x));
        }
    }

    pub fn place_data(&mut self, data: &[u8]) {
        let data_iter = BitIterator::new(data);
        let pos_iter = PositionIterator::new(self.size());

        for bit in data_iter {
            for pos in pos_iter {
                if self.data[pos.x][pos.y] == Module::Empty {
                    self.data[pos.x][pos.y] = if bit {
                        Module::Filled(Color::Black)
                    } else {
                        Module::Filled(Color::White)
                    };
                    break;
                }
            }
        }
    }

    pub fn place_format(&mut self, data: u16) {
        let pos_iter = FormatPositionIterator::new(self.size());
        for (index, pos_list) in pos_iter.enumerate() {
            let mask = 1 << index;
            let color = if data & mask != 0 {
                Color::Black
            } else {
                Color::White
            };
            for pos in &pos_list {
                self.fill_module(*pos, Module::Static(color));
            }
        }
        self.fill_module(
            Point::new(self.size().y - 8, 8),
            Module::Static(Color::Black),
        );
    }

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
        masked.data.iter_mut().enumerate().for_each(|(x, row)| {
            row.iter_mut().enumerate().for_each(|(y, module)| {
                if let Module::Filled(color) = module {
                    if condition(x, y) {
                        *module = Module::Filled(color.inverse())
                    }
                }
            })
        });
        masked
    }
}

impl Debug for Matrix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.data.iter().try_for_each(|row| {
            row.iter().try_for_each(|&module| match module {
                Module::Filled(color) => match color {
                    Color::White => f.write_char('_'),
                    Color::Black => f.write_char('\u{2588}'),
                },
                Module::Empty => f.write_char('\u{FFFD}'),
                Module::Static(color) => match color {
                    Color::White => f.write_char('\u{2591}'),
                    Color::Black => f.write_char('\u{2593}'),
                },
                Module::Reserved => f.write_char('\u{2592}'),
            })?;
            f.write_char('\n')
        })
    }
}

impl Display for Matrix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.data.chunks(2).try_for_each(|rows| {
            if rows.len() == 2 {
                rows[0]
                    .iter()
                    .zip(rows[1].iter())
                    .try_for_each(|(&up, &down)| {
                        f.write_char(match (up.into(), down.into()) {
                            (Color::Black, Color::Black) => '\u{2588}',
                            (Color::Black, Color::White) => '\u{2580}',
                            (Color::White, Color::Black) => '\u{2584}',
                            (Color::White, Color::White) => ' ',
                        })
                    })
            } else {
                rows[0].iter().try_for_each(|&up| {
                    f.write_char(match up.into() {
                        Color::Black => '\u{2580}',
                        Color::White => ' ',
                    })
                })
            }?;
            f.write_char('\n')
        })
    }
}

#[derive(Copy, Clone)]
struct FormatPositionIterator {
    size: Point,
    index: usize,
}

impl FormatPositionIterator {
    fn new(size: Point) -> FormatPositionIterator {
        FormatPositionIterator { size, index: 0 }
    }
}

impl Iterator for FormatPositionIterator {
    type Item = [Point; 2];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index <= 14 {
            // Left-top
            let pos1 = match self.index {
                0..=5 => Point::new(self.index, 8),
                6..=7 => Point::new(self.index + 1, 8),
                8 => Point::new(8, 14 - self.index + 1),
                9..=14 => Point::new(8, 14 - self.index),
                _ => panic!(),
            };

            // Right-top and Left-bottom
            let pos2 = match self.index {
                0..=7 => Point::new(8, self.size.y - 1 - self.index),
                8..=14 => Point::new(self.size.x - 1 - 14 + self.index, 8),
                _ => panic!(),
            };
            self.index += 1;
            Some([pos1, pos2])
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
struct PositionIterator {
    size: Point,
    current_pos: Point,
    next_pos: Option<Point>,
    upwards: bool,
}

impl PositionIterator {
    fn new(size: Point) -> PositionIterator {
        PositionIterator {
            size,
            current_pos: Point::new(size.x - 1, size.y - 1),
            next_pos: None,
            upwards: true,
        }
    }
}

impl Iterator for PositionIterator {
    // we will be counting with usize
    type Item = Point;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_pos.is_some() {
            self.next_pos.take()
        } else {
            let current_pos = self.current_pos;
            self.next_pos = Some(Point::new(current_pos.x, current_pos.y - 1));
            if self.upwards {
                if self.current_pos.x == 0 {
                    self.upwards = false;
                    self.current_pos.y -= 2;
                    if self.current_pos.y == 6 {
                        self.current_pos.y -= 1;
                    }
                } else {
                    self.current_pos.x -= 1;
                }
            } else {
                if self.current_pos.x == self.size.x - 1 {
                    self.upwards = true;
                    self.current_pos.y -= 2;
                    if self.current_pos.y == 6 {
                        self.current_pos.y -= 1;
                    }
                } else {
                    self.current_pos.x += 1;
                }
            }
            Some(current_pos)
        }
    }
}

#[derive(Copy, Clone)]
struct BitIterator<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: usize,
}

impl<'a> BitIterator<'a> {
    fn new(data: &'a [u8]) -> BitIterator {
        BitIterator {
            data,
            byte_pos: 0,
            bit_pos: 7,
        }
    }
}

impl<'a> Iterator for BitIterator<'a> {
    // we will be counting with usize
    type Item = bool;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        if self.byte_pos < self.data.len() {
            let byte = self.data[self.byte_pos];
            let mask = 1 << self.bit_pos;
            let result = byte & mask != 0;

            if self.bit_pos == 0 {
                self.bit_pos = 7;
                self.byte_pos += 1;
            } else {
                self.bit_pos -= 1;
            }

            Some(result)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::matrix::{Color, Matrix, Module};

    #[test]
    fn finder_pattern() {
        let mut matrix = Matrix::new();
        matrix.fill_finder_patterns();

        assert_eq!(
            format!("{:?}", matrix),
            "\
▓▓▓▓▓▓▓░�����░▓▓▓▓▓▓▓
▓░░░░░▓░�����░▓░░░░░▓
▓░▓▓▓░▓░�����░▓░▓▓▓░▓
▓░▓▓▓░▓░�����░▓░▓▓▓░▓
▓░▓▓▓░▓░�����░▓░▓▓▓░▓
▓░░░░░▓░�����░▓░░░░░▓
▓▓▓▓▓▓▓░�����░▓▓▓▓▓▓▓
░░░░░░░░�����░░░░░░░░
���������������������
���������������������
���������������������
���������������������
���������������������
░░░░░░░░�������������
▓▓▓▓▓▓▓░�������������
▓░░░░░▓░�������������
▓░▓▓▓░▓░�������������
▓░▓▓▓░▓░�������������
▓░▓▓▓░▓░�������������
▓░░░░░▓░�������������
▓▓▓▓▓▓▓░�������������
"
        );
    }

    #[test]
    fn reserved() {
        let mut matrix = Matrix::new();
        matrix.fill_reserved();

        assert_eq!(
            format!("{:?}", matrix),
            "\
��������▒������������
��������▒������������
��������▒������������
��������▒������������
��������▒������������
��������▒������������
���������������������
��������▒������������
▒▒▒▒▒▒�▒▒����▒▒▒▒▒▒▒▒
���������������������
���������������������
���������������������
���������������������
��������▒������������
��������▒������������
��������▒������������
��������▒������������
��������▒������������
��������▒������������
��������▒������������
��������▒������������
"
        );
    }

    #[test]
    fn timing_pattern() {
        let mut matrix = Matrix::new();
        matrix.fill_timing_pattern();

        assert_eq!(
            format!("{:?}", matrix),
            "\
���������������������
���������������������
���������������������
���������������������
���������������������
���������������������
��������▓░▓░▓��������
���������������������
������▓��������������
������░��������������
������▓��������������
������░��������������
������▓��������������
���������������������
���������������������
���������������������
���������������������
���������������������
���������������������
���������������������
���������������������
"
        );
    }

    #[test]
    fn placement() {
        let mut matrix = Matrix::new();
        matrix.fill_finder_patterns();
        matrix.fill_reserved();
        matrix.fill_timing_pattern();

        matrix.place_data(&[
            0b00010000, 0b00100000, 0b00001100, 0b01010110, 0b01100001, 0b10000000, 0b11101100,
            0b00010001, 0b11101100, 0b00010001, 0b11101100, 0b00010001, 0b11101100, 0b00010001,
            0b11101100, 0b00010001, 0b10100101, 0b00100100, 0b11010100, 0b11000001, 0b11101101,
            0b00110110, 0b11000111, 0b10000111, 0b00101100, 0b01010101,
        ]);

        assert_eq!(
            format!("{:?}", matrix),
            "\
▓▓▓▓▓▓▓░▒__█_░▓▓▓▓▓▓▓
▓░░░░░▓░▒_██_░▓░░░░░▓
▓░▓▓▓░▓░▒█__█░▓░▓▓▓░▓
▓░▓▓▓░▓░▒___█░▓░▓▓▓░▓
▓░▓▓▓░▓░▒███_░▓░▓▓▓░▓
▓░░░░░▓░▒█___░▓░░░░░▓
▓▓▓▓▓▓▓░▓░▓░▓░▓▓▓▓▓▓▓
░░░░░░░░▒█_█_░░░░░░░░
▒▒▒▒▒▒▓▒▒____▒▒▒▒▒▒▒▒
█____█░████______█___
█_██__▓█___███_███_██
█__██_░_____█___██___
█___██▓███_███_██_█__
░░░░░░░░▒███_███_█___
▓▓▓▓▓▓▓░▒_█___█___█__
▓░░░░░▓░▒███_███____█
▓░▓▓▓░▓░▒█_______█___
▓░▓▓▓░▓░▒_________█__
▓░▓▓▓░▓░▒█████_██____
▓░░░░░▓░▒█__█___█__█_
▓▓▓▓▓▓▓░▒_████_██____
"
        );
    }

    #[test]
    fn mask_pattern0() {
        let mut matrix = Matrix::new();
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
        let mut matrix = Matrix::new();
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
        let mut matrix = Matrix::new();
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
        let mut matrix = Matrix::new();
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
        let mut matrix = Matrix::new();
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
        let mut matrix = Matrix::new();
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
        let mut matrix = Matrix::new();
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
        let mut matrix = Matrix::new();
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
        let mut matrix = Matrix::new();
        matrix.fill_finder_patterns();
        matrix.fill_reserved();
        matrix.fill_timing_pattern();

        matrix.place_data(&[
            0b00010000, 0b00100000, 0b00001100, 0b01010110, 0b01100001, 0b10000000, 0b11101100,
            0b00010001, 0b11101100, 0b00010001, 0b11101100, 0b00010001, 0b11101100, 0b00010001,
            0b11101100, 0b00010001, 0b10100101, 0b00100100, 0b11010100, 0b11000001, 0b11101101,
            0b00110110, 0b11000111, 0b10000111, 0b00101100, 0b01010101,
        ]);

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
    fn format() {
        let mut matrix = Matrix::new();
        matrix.fill_reserved();
        matrix.place_format(0b100000011001110);

        assert_eq!(
            format!("{:?}", matrix),
            "\
��������░������������
��������▓������������
��������▓������������
��������▓������������
��������░������������
��������░������������
���������������������
��������▓������������
▓░░░░░�░▓����▓▓░░▓▓▓░
���������������������
���������������������
���������������������
���������������������
��������▓������������
��������░������������
��������░������������
��������░������������
��������░������������
��������░������������
��������░������������
��������▓������������
"
        );
    }
}
