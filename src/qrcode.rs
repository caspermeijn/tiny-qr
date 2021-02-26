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

use crate::blocks::BlockIterator;
use crate::encoding::{AlphanumericDataEncoder, EncodingMode, NumericDataEncoder};
use crate::error_correction::{ErrorCorrectionEncoder, ErrorCorrectionLevel};
use crate::format::FormatEncoder;
use crate::matrix::Matrix;
use crate::qr_version::Version;

const MAX_VERSION: usize = 4;

pub struct QrCodeBuilder<'a> {
    version: Option<u8>,
    error_correction_level: ErrorCorrectionLevel,
    mask_reference: Option<u8>,
    text: Option<&'a str>,
}

impl<'a> QrCodeBuilder<'a>
where
    [u8; MAX_VERSION * 4 + 17]: Sized,
{
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

    pub fn build(self) -> QrCode<MAX_VERSION> {
        let selected_version_number = self.version.unwrap_or(MAX_VERSION as u8);
        assert!(selected_version_number <= MAX_VERSION as u8);
        let selected_version = Version {
            version: selected_version_number,
        };
        let selected_mask_reference = self.mask_reference.unwrap_or(0);
        let data = self.text.unwrap();

        let encoding = EncodingMode::select_best_encoding(data);
        let mut buffer = match encoding {
            Some(EncodingMode::Numeric) => {
                let encoder = NumericDataEncoder {
                    version: selected_version,
                    error_correction: self.error_correction_level,
                };
                encoder.encode(data)
            }
            Some(EncodingMode::Alphanumeric) => {
                let encoder = AlphanumericDataEncoder {
                    version: selected_version,
                    error_correction: self.error_correction_level,
                };
                encoder.encode(data)
            }
            _ => {
                panic!("Sorry, this input is not yet supported");
            }
        };

        let mut matrix = Matrix::new();
        matrix.set_version(selected_version);
        matrix.fill_symbol();

        let encoder = ErrorCorrectionEncoder {
            version: selected_version,
            error_correction: self.error_correction_level,
        };

        encoder.encode(&mut buffer);

        let data = BlockIterator::new(buffer.data(), selected_version, self.error_correction_level);

        matrix.place_data(data);

        let mut matrix = matrix.mask(selected_mask_reference);

        let format_encoder = FormatEncoder {
            error_correction_level: self.error_correction_level,
            mask_reference: self.mask_reference.unwrap(),
        };

        let format = format_encoder.encode();
        matrix.place_format(format);

        QrCode { matrix }
    }
}

pub struct QrCode<const MAX_VERSION: usize>
where
    [u8; MAX_VERSION * 4 + 17]: Sized,
{
    pub matrix: Matrix<{ MAX_VERSION * 4 + 17 }>,
}

#[cfg(test)]
mod tests {
    use crate::error_correction::ErrorCorrectionLevel;
    use crate::qrcode::QrCodeBuilder;

    #[test]
    fn numeric_version_1() {
        let qr_code = QrCodeBuilder::new()
            .with_text("01234567")
            .with_version(1)
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

    #[test]
    fn alphanumeric_version_2() {
        let qr_code = QrCodeBuilder::new()
            .with_version(2)
            .with_error_correction_level(ErrorCorrectionLevel::Quartile)
            .with_mask_reference(0b110)
            .with_text("HTTPS://CASPERMEIJN.NL")
            .build();

        assert_eq!(
            format!("{:?}", qr_code.matrix),
            "\
▓▓▓▓▓▓▓░░__█████_░▓▓▓▓▓▓▓
▓░░░░░▓░▓_█_████_░▓░░░░░▓
▓░▓▓▓░▓░░█___██__░▓░▓▓▓░▓
▓░▓▓▓░▓░▓█__█_███░▓░▓▓▓░▓
▓░▓▓▓░▓░▓_██_█_█_░▓░▓▓▓░▓
▓░░░░░▓░░_█_███_█░▓░░░░░▓
▓▓▓▓▓▓▓░▓░▓░▓░▓░▓░▓▓▓▓▓▓▓
░░░░░░░░▓_█___█__░░░░░░░░
░▓░▓▓▓▓░▓████████▓▓░▓▓░▓░
█_█_█_░█_████_█_█_█████__
███_█_▓_██_██_██__█_____█
█_____░███_██_█_█_█_████_
██_█__▓█_█_████_█__█_███_
�___██░████_██___█_█_█___
��____▓██_█__███_███_█___
��___█░██______██___█_███
��██__▓_█_██_██_▓▓▓▓▓_███
░░░░░░░░▓___█___▓░░░▓█___
▓▓▓▓▓▓▓░░█__██__▓░▓░▓_███
▓░░░░░▓░▓_█____█▓░░░▓███_
▓░▓▓▓░▓░▓██__███▓▓▓▓▓_█__
▓░▓▓▓░▓░▓_____█_█_█___█_█
▓░▓▓▓░▓░░█_█___██____████
▓░░░░░▓░▓_███_█████_█_██_
▓▓▓▓▓▓▓░░██████_____██_██
"
        );
    }

    #[test]
    fn alphanumeric_version_4() {
        let qr_code = QrCodeBuilder::new()
            .with_version(4)
            .with_error_correction_level(ErrorCorrectionLevel::High)
            .with_mask_reference(0b110)
            .with_text("HTTPS://GITHUB.COM/CASPERMEIJN/TINY-QR")
            .build();

        assert_eq!(
            format!("{:?}", qr_code.matrix),
            "\
▓▓▓▓▓▓▓░░__██__█_█_█_██__░▓▓▓▓▓▓▓
▓░░░░░▓░░█_███_█_____████░▓░░░░░▓
▓░▓▓▓░▓░▓__█______██___██░▓░▓▓▓░▓
▓░▓▓▓░▓░▓___████__█████_█░▓░▓▓▓░▓
▓░▓▓▓░▓░░_█____████_█__█_░▓░▓▓▓░▓
▓░░░░░▓░░__██_█__█__█___█░▓░░░░░▓
▓▓▓▓▓▓▓░▓░▓░▓░▓░▓░▓░▓░▓░▓░▓▓▓▓▓▓▓
░░░░░░░░░__███__█__█___█_░░░░░░░░
░░░▓▓░▓▓░████_█_██_███_█_░░░░▓▓░░
_█____░_█____█________█_█_█████_█
_███_█▓█___█__███__███_████___███
_█_█_█░__███___█████___█___██_██_
███_██▓_█_______████████____█_█_█
██___█░__█__█_█____█__██_█____█__
_██_██▓█__█_████_█_██_█_███_█_██_
█___██░███_█__█████__███_█__█_██_
__█___▓_____██_███_____█_█__███_█
███__█░_█_████_█_████_██_█___██__
____█_▓███__█_████__█_█_____█_█_█
_███_█░_█_███_█__███_██__█_██____
__█_█_▓_█_███_████_█_███__███__██
�_█_██░___████_████_█___█_███████
��_█_█▓███__█_█_█████_██_██_█__█_
��█_██░____██___█__█___██_████___
��████▓██___█__██_██__██▓▓▓▓▓____
░░░░░░░░▓__██_██_███_█_█▓░░░▓_█_█
▓▓▓▓▓▓▓░▓_█_██_███_██__█▓░▓░▓██__
▓░░░░░▓░░█__█__███___██_▓░░░▓_███
▓░▓▓▓░▓░▓█_█_█____█__███▓▓▓▓▓████
▓░▓▓▓░▓░▓█__██_█__██████_______█_
▓░▓▓▓░▓░░████████__████_█___██_██
▓░░░░░▓░░__██_█___█__███____██_█_
▓▓▓▓▓▓▓░░█__██_█_█__███_█_██__██_
"
        );
    }
}
