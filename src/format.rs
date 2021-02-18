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

use crate::error_correction::ErrorCorrectionLevel;

pub struct FormatEncoder {
    // TODO: Combine Version and ErrorCorrectionLevel
    pub(crate) error_correction_level: ErrorCorrectionLevel,
    pub(crate) mask_reference: u8,
}

impl FormatEncoder {
    fn masked_sequence(data_bits: u8) -> u16 {
        match data_bits {
            0 => 0x5412,
            1 => 0x5125,
            2 => 0x5e7c,
            3 => 0x5b4b,
            4 => 0x45f9,
            5 => 0x40ce,
            6 => 0x4f97,
            7 => 0x4aa0,
            8 => 0x77c4,
            9 => 0x72f3,
            10 => 0x7daa,
            11 => 0x789d,
            12 => 0x662f,
            13 => 0x6318,
            14 => 0x6c41,
            15 => 0x6976,
            16 => 0x1689,
            17 => 0x13be,
            18 => 0x1ce7,
            19 => 0x19d0,
            20 => 0x0762,
            21 => 0x0255,
            22 => 0x0d0c,
            23 => 0x083b,
            24 => 0x355f,
            25 => 0x3068,
            26 => 0x3f31,
            27 => 0x3a06,
            28 => 0x24b4,
            29 => 0x2183,
            30 => 0x2eda,
            31 => 0x2bed,
            _ => panic!(),
        }
    }

    pub fn encode(&self) -> u16 {
        let error_correction_level = match self.error_correction_level {
            ErrorCorrectionLevel::Low => 0b01,
            ErrorCorrectionLevel::Medium => 0b00,
            ErrorCorrectionLevel::Quartile => 0b11,
            ErrorCorrectionLevel::High => 0b10,
        };
        let data = (error_correction_level << 3) + self.mask_reference;
        Self::masked_sequence(data)
    }
}
