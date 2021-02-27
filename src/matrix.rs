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

use crate::array_2d::{Array2D, Coordinate};
use crate::blocks::BlockIterator;
use crate::error_correction::ErrorCorrectedData;
use crate::qr_version::Version;
use core::fmt::{Debug, Display, Formatter, Write};
use core::iter::Peekable;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub(crate) fn inverse(self) -> Self {
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

impl Default for Module {
    fn default() -> Self {
        Module::Empty
    }
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
pub struct Matrix<const N: usize> {
    pub(crate) data: Array2D<Module, N>,
}

impl<const N: usize> Default for Matrix<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Matrix<N> {
    pub fn new() -> Self {
        Self {
            data: Array2D::new(),
        }
    }

    pub(crate) fn fill_whole(&mut self, data: Module) {
        let size = self.data.size();
        for x in 0..size.x {
            for y in 0..size.y {
                self.data[(x, y).into()] = data;
            }
        }
    }

    fn fill_module(&mut self, pos: Coordinate, data: Module) {
        self.data[pos] = data;
    }

    fn fill_line(&mut self, pos1: Coordinate, pos2: Coordinate, data: Module) {
        if pos1.x == pos2.x {
            let x = pos1.x;
            assert!(pos1.y < pos2.y);
            for y in pos1.y..=pos2.y {
                self.fill_module(Coordinate::new(x, y), data);
            }
        } else if pos1.y == pos2.y {
            let y = pos1.y;
            assert!(pos1.x < pos2.x);
            for x in pos1.x..=pos2.x {
                self.fill_module(Coordinate::new(x, y), data);
            }
        } else {
            panic!()
        }
    }

    fn fill_finder_pattern(&mut self, pos: Coordinate) {
        let black = Module::Static(Color::Black);
        let white = Module::Static(Color::White);

        self.fill_line(pos, Coordinate::new(pos.x, pos.y + 6), black);
        self.fill_line(
            Coordinate::new(pos.x, pos.y + 6),
            Coordinate::new(pos.x + 5, pos.y + 6),
            black,
        );
        self.fill_line(
            Coordinate::new(pos.x + 6, pos.y + 1),
            Coordinate::new(pos.x + 6, pos.y + 6),
            black,
        );
        self.fill_line(
            Coordinate::new(pos.x + 1, pos.y),
            Coordinate::new(pos.x + 6, pos.y),
            black,
        );

        self.fill_line(
            Coordinate::new(pos.x + 1, pos.y + 1),
            Coordinate::new(pos.x + 1, pos.y + 4),
            white,
        );
        self.fill_line(
            Coordinate::new(pos.x + 1, pos.y + 5),
            Coordinate::new(pos.x + 4, pos.y + 5),
            white,
        );
        self.fill_line(
            Coordinate::new(pos.x + 5, pos.y + 2),
            Coordinate::new(pos.x + 5, pos.y + 5),
            white,
        );
        self.fill_line(
            Coordinate::new(pos.x + 2, pos.y + 1),
            Coordinate::new(pos.x + 5, pos.y + 1),
            white,
        );

        self.fill_module(Coordinate::new(pos.x + 2, pos.y + 2), black);
        self.fill_module(Coordinate::new(pos.x + 2, pos.y + 3), black);
        self.fill_module(Coordinate::new(pos.x + 2, pos.y + 4), black);
        self.fill_module(Coordinate::new(pos.x + 3, pos.y + 2), black);
        self.fill_module(Coordinate::new(pos.x + 3, pos.y + 3), black);
        self.fill_module(Coordinate::new(pos.x + 3, pos.y + 4), black);
        self.fill_module(Coordinate::new(pos.x + 4, pos.y + 2), black);
        self.fill_module(Coordinate::new(pos.x + 4, pos.y + 3), black);
        self.fill_module(Coordinate::new(pos.x + 4, pos.y + 4), black);
    }

    fn fill_finder_patterns(&mut self) {
        let white = Module::Static(Color::White);
        let size = self.data.size();

        // Left-top
        self.fill_finder_pattern(Coordinate::new(0, 0));
        self.fill_line(Coordinate::new(0, 7), Coordinate::new(7, 7), white);
        self.fill_line(Coordinate::new(7, 0), Coordinate::new(7, 6), white);

        // Left-bottom
        self.fill_finder_pattern(Coordinate::new(size.x - 7, 0));
        self.fill_line(
            Coordinate::new(size.x - 8, 0),
            Coordinate::new(size.y - 8, 7),
            white,
        );
        self.fill_line(
            Coordinate::new(size.x - 8, 7),
            Coordinate::new(size.y - 1, 7),
            white,
        );

        // Right-top
        self.fill_finder_pattern(Coordinate::new(0, size.y - 7));
        self.fill_line(
            Coordinate::new(7, size.y - 8),
            Coordinate::new(7, size.y - 1),
            white,
        );
        self.fill_line(
            Coordinate::new(0, size.y - 8),
            Coordinate::new(7, size.y - 8),
            white,
        );
    }

    fn fill_alignment_pattern(&mut self, center_pos: Coordinate) {
        let black = Module::Static(Color::Black);
        let white = Module::Static(Color::White);

        self.fill_module(center_pos, black);

        self.fill_module(Coordinate::new(center_pos.x - 1, center_pos.y - 1), white);
        self.fill_module(Coordinate::new(center_pos.x - 1, center_pos.y), white);
        self.fill_module(Coordinate::new(center_pos.x - 1, center_pos.y + 1), white);
        self.fill_module(Coordinate::new(center_pos.x + 1, center_pos.y - 1), white);
        self.fill_module(Coordinate::new(center_pos.x + 1, center_pos.y), white);
        self.fill_module(Coordinate::new(center_pos.x + 1, center_pos.y + 1), white);
        self.fill_module(Coordinate::new(center_pos.x, center_pos.y - 1), white);
        self.fill_module(Coordinate::new(center_pos.x, center_pos.y + 1), white);

        self.fill_line(
            Coordinate::new(center_pos.x - 2, center_pos.y - 2),
            Coordinate::new(center_pos.x - 2, center_pos.y + 1),
            black,
        );
        self.fill_line(
            Coordinate::new(center_pos.x - 2, center_pos.y + 2),
            Coordinate::new(center_pos.x + 1, center_pos.y + 2),
            black,
        );
        self.fill_line(
            Coordinate::new(center_pos.x + 2, center_pos.y - 1),
            Coordinate::new(center_pos.x + 2, center_pos.y + 2),
            black,
        );
        self.fill_line(
            Coordinate::new(center_pos.x - 1, center_pos.y - 2),
            Coordinate::new(center_pos.x + 2, center_pos.y - 2),
            black,
        );
    }

    fn fill_alignment_patterns(&mut self) {
        let size = self.data.size();

        if size.x > 21 {
            self.fill_alignment_pattern(Coordinate::new(size.x - 7, size.y - 7));
        }
    }

    fn fill_reserved(&mut self) {
        let reserved = Module::Reserved;
        let size = self.data.size();

        // Left-top
        self.fill_line(Coordinate::new(0, 8), Coordinate::new(5, 8), reserved);
        self.fill_line(Coordinate::new(8, 0), Coordinate::new(8, 5), reserved);
        self.fill_module(Coordinate::new(7, 8), reserved);
        self.fill_module(Coordinate::new(8, 8), reserved);
        self.fill_module(Coordinate::new(8, 7), reserved);

        // Left-bottom
        self.fill_line(
            Coordinate::new(size.x - 8, 8),
            Coordinate::new(size.x - 1, 8),
            reserved,
        );
        // Right-top
        self.fill_line(
            Coordinate::new(8, size.y - 8),
            Coordinate::new(8, size.y - 1),
            reserved,
        );
    }

    fn fill_timing_pattern(&mut self) {
        fn color(i: usize) -> Module {
            if i % 2 == 0 {
                Module::Static(Color::Black)
            } else {
                Module::Static(Color::White)
            }
        }

        let size = self.data.size();

        let x = 6;
        for y in 8..size.y - 8 {
            self.fill_module(Coordinate::new(x, y), color(y));
        }

        let y = 6;
        for x in 8..size.x - 8 {
            self.fill_module(Coordinate::new(x, y), color(x));
        }
    }

    pub fn place_data(&mut self, error_corrected_data: ErrorCorrectedData) {
        self.set_version(error_corrected_data.version);
        self.fill_symbol();

        let data = BlockIterator::new(&error_corrected_data);

        let data_iter = BitIterator::new(data);
        let pos_iter = PositionIterator::new(self.data.size());

        for bit in data_iter {
            for pos in pos_iter {
                if self.data[pos] == Module::Empty {
                    self.data[pos] = if bit {
                        Module::Filled(Color::Black)
                    } else {
                        Module::Filled(Color::White)
                    };
                    break;
                }
            }
        }
    }

    fn fill_symbol(&mut self) {
        self.fill_finder_patterns();
        self.fill_reserved();
        self.fill_timing_pattern();
        self.fill_alignment_patterns();
    }

    pub fn place_format(&mut self, data: u16) {
        let pos_iter = FormatPositionIterator::new(self.data.size());
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
            Coordinate::new(self.data.size().y - 8, 8),
            Module::Static(Color::Black),
        );
    }

    fn set_version(&mut self, version: Version) {
        assert!(version.width() <= N);
        self.data
            .set_size((version.width(), version.width()).into());
    }
}

impl<const N: usize> Debug for Matrix<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.data.rows().try_for_each(|mut row| {
            row.try_for_each(|module| match module {
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

impl<const N: usize> Display for Matrix<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let iter1 = self.data.rows().step_by(2);
        let iter2 = self.data.rows().skip(1).step_by(2);
        iter1.zip(iter2).try_for_each(|rows| {
            rows.0.zip(rows.1).try_for_each(|(&up, &down)| {
                f.write_char(match (up.into(), down.into()) {
                    (Color::Black, Color::Black) => '\u{2588}',
                    (Color::Black, Color::White) => '\u{2580}',
                    (Color::White, Color::Black) => '\u{2584}',
                    (Color::White, Color::White) => ' ',
                })
            })?;
            f.write_char('\n')
        })?;

        let mut last_row = self.data.rows().last().unwrap();
        last_row.try_for_each(|&up| {
            f.write_char(match up.into() {
                Color::Black => '\u{2580}',
                Color::White => ' ',
            })
        })?;
        f.write_char('\n')
    }
}

#[derive(Copy, Clone)]
struct FormatPositionIterator {
    size: Coordinate,
    index: usize,
}

impl FormatPositionIterator {
    fn new(size: Coordinate) -> FormatPositionIterator {
        FormatPositionIterator { size, index: 0 }
    }
}

impl Iterator for FormatPositionIterator {
    type Item = [Coordinate; 2];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index <= 14 {
            // Left-top
            let pos1 = match self.index {
                0..=5 => Coordinate::new(self.index, 8),
                6..=7 => Coordinate::new(self.index + 1, 8),
                8 => Coordinate::new(8, 14 - self.index + 1),
                9..=14 => Coordinate::new(8, 14 - self.index),
                _ => panic!(),
            };

            // Right-top and Left-bottom
            let pos2 = match self.index {
                0..=7 => Coordinate::new(8, self.size.y - 1 - self.index),
                8..=14 => Coordinate::new(self.size.x - 1 - 14 + self.index, 8),
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
    size: Coordinate,
    current_pos: Coordinate,
    next_pos: Option<Coordinate>,
    upwards: bool,
}

impl PositionIterator {
    fn new(size: Coordinate) -> PositionIterator {
        PositionIterator {
            size,
            current_pos: Coordinate::new(size.x - 1, size.y - 1),
            next_pos: None,
            upwards: true,
        }
    }
}

impl Iterator for PositionIterator {
    // we will be counting with usize
    type Item = Coordinate;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_pos.is_some() {
            self.next_pos.take()
        } else {
            let current_pos = self.current_pos;
            self.next_pos = Some(Coordinate::new(current_pos.x, current_pos.y - 1));
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

struct BitIterator<'a, T>
where
    T: Iterator<Item = &'a u8>,
{
    data_iter: Peekable<T>,
    bit_pos: usize,
}

impl<'a, T> BitIterator<'a, T>
where
    T: Iterator<Item = &'a u8>,
{
    fn new(data_iter: T) -> Self {
        BitIterator {
            data_iter: data_iter.peekable(),
            bit_pos: 7,
        }
    }
}

impl<'a, T> Iterator for BitIterator<'a, T>
where
    T: Iterator<Item = &'a u8>,
{
    // we will be counting with usize
    type Item = bool;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(&&byte) = self.data_iter.peek() {
            let mask = 1 << self.bit_pos;
            let result = byte & mask != 0;

            if self.bit_pos == 0 {
                self.data_iter.next();
                self.bit_pos = 7;
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
    use crate::buffer::Buffer;
    use crate::error_correction::{ErrorCorrectedData, ErrorCorrectionLevel};
    use crate::matrix::Matrix;
    use crate::qr_version::Version;
    use alloc::format;

    #[test]
    fn finder_pattern_version_1() {
        let mut matrix = Matrix::<21>::new();
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
    fn reserved_version_1() {
        let mut matrix = Matrix::<21>::new();
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
        let mut matrix = Matrix::<21>::new();
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
    fn symbol_version_2() {
        let mut matrix = Matrix::<25>::new();
        matrix.fill_symbol();

        assert_eq!(
            format!("{:?}", matrix),
            "\
▓▓▓▓▓▓▓░▒��������░▓▓▓▓▓▓▓
▓░░░░░▓░▒��������░▓░░░░░▓
▓░▓▓▓░▓░▒��������░▓░▓▓▓░▓
▓░▓▓▓░▓░▒��������░▓░▓▓▓░▓
▓░▓▓▓░▓░▒��������░▓░▓▓▓░▓
▓░░░░░▓░▒��������░▓░░░░░▓
▓▓▓▓▓▓▓░▓░▓░▓░▓░▓░▓▓▓▓▓▓▓
░░░░░░░░▒��������░░░░░░░░
▒▒▒▒▒▒▓▒▒��������▒▒▒▒▒▒▒▒
������░������������������
������▓������������������
������░������������������
������▓������������������
������░������������������
������▓������������������
������░������������������
������▓���������▓▓▓▓▓����
░░░░░░░░▒�������▓░░░▓����
▓▓▓▓▓▓▓░▒�������▓░▓░▓����
▓░░░░░▓░▒�������▓░░░▓����
▓░▓▓▓░▓░▒�������▓▓▓▓▓����
▓░▓▓▓░▓░▒����������������
▓░▓▓▓░▓░▒����������������
▓░░░░░▓░▒����������������
▓▓▓▓▓▓▓░▒����������������
"
        );
    }

    #[test]
    fn placement() {
        let mut matrix = Matrix::<21>::new();

        let mut buffer = Buffer::new();
        buffer.append_bytes(&[
            0b00010000, 0b00100000, 0b00001100, 0b01010110, 0b01100001, 0b10000000, 0b11101100,
            0b00010001, 0b11101100, 0b00010001, 0b11101100, 0b00010001, 0b11101100, 0b00010001,
            0b11101100, 0b00010001, 0b10100101, 0b00100100, 0b11010100, 0b11000001, 0b11101101,
            0b00110110, 0b11000111, 0b10000111, 0b00101100, 0b01010101,
        ]);
        let data = ErrorCorrectedData {
            version: Version { version: 1 },
            error_correction: ErrorCorrectionLevel::Quartile,
            buffer,
        };

        matrix.place_data(data);

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
    fn format() {
        let mut matrix = Matrix::<21>::new();
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

    #[test]
    fn large_matrix_small_pattern() {
        let mut matrix = Matrix::<100>::new();
        matrix.set_version(Version { version: 1 });
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
}
