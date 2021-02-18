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

use crate::encoding::{AlphanumericDataEncoder, EncodingMode, NumericDataEncoder};
use crate::error_correction::{ErrorCorrectionEncoder, ErrorCorrectionLevel};
use crate::format::FormatEncoder;
use crate::matrix::Matrix;
use crate::qr_version::Version;

pub struct QrCode {
    pub matrix: Matrix,
}

impl QrCode {
    pub fn from_str(data: &str) -> QrCode {
        let encoding = EncodingMode::select_best_encoding(data);
        let mut buffer = match encoding {
            Some(EncodingMode::Numeric) => {
                let encoder = NumericDataEncoder {
                    version: Version { version: 1 },
                    error_correction: ErrorCorrectionLevel::Medium,
                };
                encoder.encode(data)
            }
            Some(EncodingMode::Alphanumeric) => {
                let encoder = AlphanumericDataEncoder {
                    version: Version { version: 1 },
                    error_correction: ErrorCorrectionLevel::Medium,
                };
                encoder.encode(data)
            }
            _ => {
                panic!("Sorry, this input is not yet supported");
            }
        };

        let mut matrix = Matrix::new();
        matrix.fill_finder_patterns();
        matrix.fill_reserved();
        matrix.fill_timing_pattern();

        let encoder = ErrorCorrectionEncoder {
            version: Version { version: 1 },
            error_correction: ErrorCorrectionLevel::Medium,
        };

        encoder.encode(&mut buffer);

        matrix.place_data(buffer.data());

        let mut matrix = matrix.mask(0b010);

        let format_encoder = FormatEncoder {
            error_correction_level: ErrorCorrectionLevel::Medium,
            mask_reference: 0b010,
        };

        let format = format_encoder.encode();
        matrix.place_format(format);

        QrCode { matrix }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn creator() {
        assert_eq!(2 + 2, 4);
    }
}
