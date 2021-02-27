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

use core::ops::{Index, IndexMut};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Coordinate {
    pub x: usize,
    pub y: usize,
}

impl Coordinate {
    pub fn new(x: usize, y: usize) -> Coordinate {
        Coordinate { x, y }
    }
}

impl From<(usize, usize)> for Coordinate {
    fn from(pos: (usize, usize)) -> Self {
        let (x, y) = pos;
        Coordinate::new(x, y)
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Array2D<T, const N: usize> {
    data: [[T; N]; N],
    size: Coordinate,
}

impl<T, const N: usize> Index<Coordinate> for Array2D<T, N> {
    type Output = T;

    fn index(&self, index: Coordinate) -> &Self::Output {
        &self.data[index.x][index.y]
    }
}

impl<T, const N: usize> IndexMut<Coordinate> for Array2D<T, N> {
    fn index_mut(&mut self, index: Coordinate) -> &mut Self::Output {
        &mut self.data[index.x][index.y]
    }
}

impl<T, const N: usize> Array2D<T, N>
where
    T: Default + Copy,
{
    pub fn new() -> Self {
        Self {
            data: [[T::default(); N]; N],
            size: Coordinate::new(N, N),
        }
    }
}

impl<T, const N: usize> Array2D<T, N> {
    pub fn capacity() -> Coordinate {
        Coordinate::new(N, N)
    }

    pub fn size(&self) -> Coordinate {
        self.size
    }

    pub fn rows(&self) -> Rows<'_, T, N> {
        Rows { data: &self, x: 0 }
    }

    pub fn columns(&self) -> Columns<'_, T, N> {
        Columns { data: &self, y: 0 }
    }

    pub fn set_size(&mut self, size: Coordinate) {
        self.size = size;
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Rows<'a, T, const N: usize> {
    data: &'a Array2D<T, N>,
    x: usize,
}

impl<'a, T, const N: usize> Iterator for Rows<'a, T, N> {
    // we will be counting with usize
    type Item = Row<'a, T, N>;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        let size = self.data.size();
        if self.x < size.x {
            let result = Row {
                data: self.data,
                x: self.x,
                y: 0,
            };
            self.x += 1;
            Some(result)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Row<'a, T, const N: usize> {
    data: &'a Array2D<T, N>,
    x: usize,
    y: usize,
}

impl<'a, T, const N: usize> Iterator for Row<'a, T, N> {
    // we will be counting with usize
    type Item = &'a T;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        let size = self.data.size();
        if self.y < size.y {
            let result = &self.data[(self.x, self.y).into()];
            self.y += 1;
            Some(result)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Columns<'a, T, const N: usize> {
    data: &'a Array2D<T, N>,
    y: usize,
}

impl<'a, T, const N: usize> Iterator for Columns<'a, T, N> {
    // we will be counting with usize
    type Item = Column<'a, T, N>;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        let size = self.data.size();
        if self.y < size.y {
            let result = Column {
                data: self.data,
                x: 0,
                y: self.y,
            };
            self.y += 1;
            Some(result)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Column<'a, T, const N: usize> {
    data: &'a Array2D<T, N>,
    x: usize,
    y: usize,
}

impl<'a, T, const N: usize> Iterator for Column<'a, T, N> {
    // we will be counting with usize
    type Item = &'a T;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        let size = self.data.size();
        if self.x < size.x {
            let result = &self.data[(self.x, self.y).into()];
            self.x += 1;
            Some(result)
        } else {
            None
        }
    }
}

