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

use crate::buffer::Buffer;
use crate::qr_version::Version;

/// Qr codes use Reed–Solomon error correction
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
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

pub struct ErrorCorrectionEncoder {
    // TODO: Combine Version and ErrorCorrectionLevel
    pub(crate) version: Version,
    pub(crate) error_correction: ErrorCorrectionLevel,
}

impl ErrorCorrectionEncoder {
    pub fn encode(&self, buffer: &mut Buffer) {
        let ecc_len = self
            .version
            .error_correction_codeword_count(self.error_correction);
        let encoder = reed_solomon::Encoder::new(ecc_len);
        let ecc_buffer = encoder.encode(buffer.data());
        buffer.append_bytes(ecc_buffer.ecc());
    }
}

#[cfg(test)]
mod tests {
    use crate::buffer::Buffer;
    use crate::error_correction::{ErrorCorrectionEncoder, ErrorCorrectionLevel};
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

        let encoder = ErrorCorrectionEncoder {
            version: Version { version: 1 },
            error_correction: ErrorCorrectionLevel::Medium,
        };

        encoder.encode(&mut buffer);
        assert_eq!(
            buffer.data(),
            [
                0b00010000, 0b00100000, 0b00001100, 0b01010110, 0b01100001, 0b10000000, 0b11101100,
                0b00010001, 0b11101100, 0b00010001, 0b11101100, 0b00010001, 0b11101100, 0b00010001,
                0b11101100, 0b00010001, 0b10100101, 0b00100100, 0b11010100, 0b11000001, 0b11101101,
                0b00110110, 0b11000111, 0b10000111, 0b00101100, 0b01010101
            ]
        )
    }
}
