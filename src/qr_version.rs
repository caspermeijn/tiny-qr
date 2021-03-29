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

use crate::encoding::EncodingMode;
use crate::error_correction::ErrorCorrectionLevel;

pub const fn version_to_size(version: u8) -> usize {
    version as usize * 4 + 17
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct Version {
    pub(crate) version: u8,
}

impl Version {
    pub fn decrement(self) -> Option<Self> {
        if self.version > 1 {
            Some(Self {
                version: self.version - 1,
            })
        } else {
            None
        }
    }

    pub const fn width(&self) -> usize {
        version_to_size(self.version)
    }

    pub fn character_count_indicator_bit_length(&self, encoding: EncodingMode) -> usize {
        match encoding {
            EncodingMode::Numeric => match self.version {
                0..=9 => 10,
                10..=26 => 12,
                27..=40 => 14,
                _ => panic!(),
            },
            EncodingMode::Alphanumeric => match self.version {
                0..=9 => 9,
                10..=26 => 11,
                27..=40 => 13,
                _ => panic!(),
            },
            EncodingMode::Byte => match self.version {
                0..=9 => 8,
                10..=26 => 16,
                27..=40 => 16,
                _ => panic!(),
            },
        }
    }

    pub fn total_codeword_count(&self) -> usize {
        match self.version {
            1 => 26,
            2 => 44,
            3 => 70,
            4 => 100,
            5 => 134,
            6 => 172,
            7 => 196,
            8 => 242,
            9 => 292,
            10 => 346,
            11 => 404,
            12 => 466,
            13 => 532,
            14 => 581,
            15 => 655,
            16 => 733,
            17 => 815,
            18 => 901,
            19 => 991,
            20 => 1085,
            21 => 1156,
            22 => 1258,
            23 => 1364,
            24 => 1474,
            25 => 1588,
            // TODO: Finish table 9 edition 2006
            _ => panic!(),
        }
    }

    pub fn data_codeword_count(&self, error_correction: ErrorCorrectionLevel) -> usize {
        self.total_codeword_count()
            - self
                .error_correction_codeword_blocks_count(error_correction)
                .0
    }

    pub fn data_codeword_bit_len(&self, error_correction: ErrorCorrectionLevel) -> usize {
        self.data_codeword_count(error_correction) * 8
    }

    pub fn error_correction_codeword_blocks_count(
        &self,
        error_correction: ErrorCorrectionLevel,
    ) -> (usize, usize) {
        use ErrorCorrectionLevel::{High as H, Low as L, Medium as M, Quartile as Q};
        match (self.version, error_correction) {
            (1, L) => (7, 1),
            (1, M) => (10, 1),
            (1, Q) => (13, 1),
            (1, H) => (17, 1),
            (2, L) => (10, 1),
            (2, M) => (16, 1),
            (2, Q) => (22, 1),
            (2, H) => (28, 1),
            (3, L) => (15, 1),
            (3, M) => (26, 1),
            (3, Q) => (36, 2),
            (3, H) => (44, 2),
            (4, L) => (20, 1),
            (4, M) => (36, 2),
            (4, Q) => (52, 2),
            (4, H) => (64, 4),
            (5, L) => (26, 1),
            (5, M) => (48, 2),
            (5, Q) => (72, 4),
            (5, H) => (88, 4),
            // TODO: Finish table 9 edition 2006
            _ => panic!(),
        }
    }
}
