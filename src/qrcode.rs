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

pub struct QrCodeBuilder<'a> {
    version: Option<u8>,
    error_correction_level: ErrorCorrectionLevel,
    mask_reference: Option<u8>,
    text: Option<&'a str>,
}

impl<'a> QrCodeBuilder<'a> {
    pub fn new() -> Self {
        Self {
            version: None,
            error_correction_level: ErrorCorrectionLevel::Medium,
            mask_reference: None,
            text: None,
        }
    }

    pub fn with_version(mut self, version: u8) -> Self {
        self.version = Some(version);
        self
    }

    pub fn with_error_correction_level(
        mut self,
        error_correction_level: ErrorCorrectionLevel,
    ) -> Self {
        self.error_correction_level = error_correction_level;
        self
    }

    pub fn with_mask_reference(mut self, mask_reference: u8) -> Self {
        self.mask_reference = Some(mask_reference);
        self
    }

    pub fn with_text(mut self, text: &'a str) -> Self {
        self.text = Some(text);
        self
    }

    pub fn build(self) -> QrCode {
        let selected_version = self.version.unwrap_or(1);
        let data = self.text.unwrap();

        let encoding = EncodingMode::select_best_encoding(data);
        let mut buffer = match encoding {
            Some(EncodingMode::Numeric) => {
                let encoder = NumericDataEncoder {
                    version: Version { version: selected_version },
                    error_correction: self.error_correction_level,
                };
                encoder.encode(data)
            }
            Some(EncodingMode::Alphanumeric) => {
                let encoder = AlphanumericDataEncoder {
                    version: Version { version: selected_version },
                    error_correction: self.error_correction_level,
                };
                encoder.encode(data)
            }
            _ => {
                panic!("Sorry, this input is not yet supported");
            }
        };

        let mut matrix = Matrix::new();
        matrix.fill_symbol();


        let encoder = ErrorCorrectionEncoder {
            version: Version { version: selected_version },
            error_correction: self.error_correction_level,
        };

        encoder.encode(&mut buffer);

        matrix.place_data(buffer.data());

        let mut matrix = matrix.mask(self.mask_reference.unwrap());

        let format_encoder = FormatEncoder {
            error_correction_level: self.error_correction_level,
            mask_reference: self.mask_reference.unwrap(),
        };

        let format = format_encoder.encode();
        matrix.place_format(format);

        QrCode { matrix }
    }
}

pub struct QrCode {
    pub matrix: Matrix<21>,
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
    use crate::error_correction::ErrorCorrectionLevel;
    use crate::qrcode::QrCodeBuilder;

    #[test]
    fn numeric_version_1() {
        let qr_code = QrCodeBuilder::new()
            .with_text("01234567")
            .with_mask_reference(0b010)
            .build();

        assert_eq!(
            format!("{:?}", qr_code.matrix),
            "\
▓▓▓▓▓▓▓░░█_██░▓▓▓▓▓▓▓
▓░░░░░▓░░████░▓░░░░░▓
▓░▓▓▓░▓░▓____░▓░▓▓▓░▓
▓░▓▓▓░▓░▓█___░▓░▓▓▓░▓
▓░▓▓▓░▓░▓_███░▓░▓▓▓░▓
▓░░░░░▓░▓___█░▓░░░░░▓
▓▓▓▓▓▓▓░▓░▓░▓░▓▓▓▓▓▓▓
░░░░░░░░▓__██░░░░░░░░
▓░▓▓▓▓▓░░█__█░▓▓▓▓▓░░
___█_█░██_█_█__█_██__
__█___▓█_█_█_█__█████
____█_░__█_____████__
___███▓██__█_█__█____
░░░░░░░░▓_█████__██__
▓▓▓▓▓▓▓░░██_█_██_____
▓░░░░░▓░▓_█████___█_█
▓░▓▓▓░▓░▓___█__█_██__
▓░▓▓▓░▓░▓█__█__█_____
▓░▓▓▓░▓░▓_██_█__█_█__
▓░░░░░▓░░______██_██_
▓▓▓▓▓▓▓░▓███_█__█_█__
"
        );
    }

    #[test]
    fn alphanumeric_version_1() {
        let qr_code = QrCodeBuilder::new()
            .with_version(1)
            .with_error_correction_level(ErrorCorrectionLevel::Quartile)
            .with_mask_reference(0b110)
            .with_text("HELLO WORLD")
            .build();

        assert_eq!(
            format!("{:?}", qr_code.matrix),
            "\
▓▓▓▓▓▓▓░░__█_░▓▓▓▓▓▓▓
▓░░░░░▓░▓█__█░▓░░░░░▓
▓░▓▓▓░▓░░█_██░▓░▓▓▓░▓
▓░▓▓▓░▓░▓████░▓░▓▓▓░▓
▓░▓▓▓░▓░▓█_█_░▓░▓▓▓░▓
▓░░░░░▓░░█__█░▓░░░░░▓
▓▓▓▓▓▓▓░▓░▓░▓░▓▓▓▓▓▓▓
░░░░░░░░▓█_██░░░░░░░░
░▓░▓▓▓▓░▓█__█▓▓░▓▓░▓░
█_████░█____████_███_
__█_█_▓█___█__██_____
█_██_█░__█_██___██___
██_███▓████_███_█████
░░░░░░░░▓___█__█_█___
▓▓▓▓▓▓▓░░██__██__████
▓░░░░░▓░▓_█__█__█_███
▓░▓▓▓░▓░▓█_█__█___███
▓░▓▓▓░▓░▓_███___█_█__
▓░▓▓▓░▓░░█____█____██
▓░░░░░▓░▓██__███__██_
▓▓▓▓▓▓▓░░█_█_______█_
"
        );
    }
}
