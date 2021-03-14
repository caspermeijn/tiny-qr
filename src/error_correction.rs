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

use crate::blocks::BlockLengthIterator;
use crate::buffer::Buffer;
use crate::encoding::EncodedData;
use crate::qr_version::Version;

/// Qr codes use Reed–Solomon error correction
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub enum ErrorCorrectionLevel {
    /// Allows recovery of 7% of missing data
    Low,
    /// Allows recovery of 15% of missing data
    Medium,
    /// Allows recovery of 25% of missing data
    Quartile,
    /// Allows recovery of 30% of missing data
    High,
}

impl ErrorCorrectionLevel {
    pub(crate) fn increment(self) -> Option<Self> {
        match self {
            ErrorCorrectionLevel::Low => Some(ErrorCorrectionLevel::Medium),
            ErrorCorrectionLevel::Medium => Some(ErrorCorrectionLevel::Quartile),
            ErrorCorrectionLevel::Quartile => Some(ErrorCorrectionLevel::High),
            ErrorCorrectionLevel::High => None,
        }
    }
}

pub struct ErrorCorrectedData {
    pub(crate) version: Version,
    pub(crate) error_correction: ErrorCorrectionLevel,
    pub(crate) buffer: Buffer,
}

pub fn add_error_correction(data: EncodedData) -> ErrorCorrectedData {
    let mut buffer = data.buffer;

    let blocks = BlockLengthIterator::new(data.version, data.error_correction);
    for block in blocks {
        let encoder = reed_solomon::Encoder::new(block.ecc_len);
        let ecc_buffer =
            encoder.encode(&buffer.data()[block.data_pos..block.data_pos + block.data_len]);
        buffer.append_bytes(ecc_buffer.ecc());
    }

    ErrorCorrectedData {
        version: data.version,
        error_correction: data.error_correction,
        buffer,
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::Buffer;
    use crate::encoding::EncodedData;
    use crate::error_correction::{add_error_correction, ErrorCorrectionLevel};
    use crate::qr_version::Version;

    #[test]
    fn error_correction_encoding() {
        // Version 1-M with text "01234567"
        let mut buffer = Buffer::new();
        buffer.append_bytes(&[
            0b00010000, 0b00100000, 0b00001100, 0b01010110, 0b01100001, 0b10000000, 0b11101100,
            0b00010001, 0b11101100, 0b00010001, 0b11101100, 0b00010001, 0b11101100, 0b00010001,
            0b11101100, 0b00010001,
        ]);

        let data = EncodedData {
            version: Version { version: 1 },
            error_correction: ErrorCorrectionLevel::Medium,
            buffer,
        };

        let error_corrected_data = add_error_correction(data);
        assert_eq!(
            error_corrected_data.buffer.data(),
            [
                0b00010000, 0b00100000, 0b00001100, 0b01010110, 0b01100001, 0b10000000, 0b11101100,
                0b00010001, 0b11101100, 0b00010001, 0b11101100, 0b00010001, 0b11101100, 0b00010001,
                0b11101100, 0b00010001, 0b10100101, 0b00100100, 0b11010100, 0b11000001, 0b11101101,
                0b00110110, 0b11000111, 0b10000111, 0b00101100, 0b01010101
            ]
        )
    }

    #[test]
    fn error_correction_encoding_5q() {
        // Version 1-M with text "01234567"
        let mut buffer = Buffer::new();
        buffer.append_bytes(&[
            67, 85, 70, 134, 87, 38, 85, 194, 119, 50, 6, 18, 6, 103, 38, 246, 246, 66, 7, 118,
            134, 242, 7, 38, 86, 22, 198, 199, 146, 6, 182, 230, 247, 119, 50, 7, 118, 134, 87, 38,
            82, 6, 134, 151, 50, 7, 70, 247, 118, 86, 194, 6, 151, 50, 16, 236, 17, 236, 17, 236,
            17, 236,
        ]);

        let data = EncodedData {
            version: Version { version: 5 },
            error_correction: ErrorCorrectionLevel::Quartile,
            buffer,
        };

        let error_corrected_data = add_error_correction(data);
        assert_eq!(
            error_corrected_data.buffer.data(),
            [
                67, 85, 70, 134, 87, 38, 85, 194, 119, 50, 6, 18, 6, 103, 38, 246, 246, 66, 7, 118,
                134, 242, 7, 38, 86, 22, 198, 199, 146, 6, 182, 230, 247, 119, 50, 7, 118, 134, 87,
                38, 82, 6, 134, 151, 50, 7, 70, 247, 118, 86, 194, 6, 151, 50, 16, 236, 17, 236,
                17, 236, 17, 236, 213, 199, 11, 45, 115, 247, 241, 223, 229, 248, 154, 117, 154,
                111, 86, 161, 111, 39, 87, 204, 96, 60, 202, 182, 124, 157, 200, 134, 27, 129, 209,
                17, 163, 163, 120, 133, 148, 116, 177, 212, 76, 133, 75, 242, 238, 76, 195, 230,
                189, 10, 108, 240, 192, 141, 235, 159, 5, 173, 24, 147, 59, 33, 106, 40, 255, 172,
                82, 2, 131, 32, 178, 236,
            ]
        )
    }
}
