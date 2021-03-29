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

use crate::matrix::Color;
use crate::qrcode::QrCode;

const BORDER_SIZE: usize = 4;

pub struct CoordinatedColor {
    pub x: usize,
    pub y: usize,
    pub color: Color,
}

pub struct DrawIterator<'a, const N: usize> {
    qrcode: &'a QrCode<N>,
    x: usize,
    y: usize,
}

impl<'a, const N: usize> DrawIterator<'a, N> {
    pub(crate) fn new(qrcode: &'a QrCode<N>) -> Self {
        DrawIterator { qrcode, x: 0, y: 0 }
    }

    pub fn height(&self) -> usize {
        let size = self.qrcode.data.size();
        8 + size.x
    }

    pub fn width(&self) -> usize {
        let size = self.qrcode.data.size();
        8 + size.y
    }

    fn is_current_pos_border(&self) -> bool {
        let data_size = self.qrcode.data.size();

        self.x < 4
            || self.y < 4
            || self.x >= data_size.x + BORDER_SIZE
            || self.y >= data_size.y + BORDER_SIZE
    }
}

impl<const N: usize> Iterator for DrawIterator<'_, N> {
    // we will be counting with usize
    type Item = CoordinatedColor;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        let result = if self.y >= self.height() {
            None
        } else if self.is_current_pos_border() {
            Some(CoordinatedColor {
                x: self.x,
                y: self.y,
                color: Color::White,
            })
        } else {
            let data_pos = (self.x - BORDER_SIZE, self.y - BORDER_SIZE).into();
            Some(CoordinatedColor {
                x: self.x,
                y: self.y,
                color: self.qrcode.data[data_pos],
            })
        };

        self.x += 1;
        if self.x >= self.width() {
            self.x = 0;
            self.y += 1;
        }

        result
    }
}
