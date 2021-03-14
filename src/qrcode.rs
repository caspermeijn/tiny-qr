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

use crate::array_2d::Array2D;
use crate::encoding::encode_text;
use crate::error_correction::{add_error_correction, ErrorCorrectionLevel};
use crate::mask::ScoreMasked;
use crate::matrix::{Color, Matrix};
use crate::qr_version::Version;
use core::fmt::{Debug, Display, Formatter, Write};

const MAX_VERSION: usize = 4;

pub struct QrCodeBuilder<'a> {
    max_version: Option<u8>,
    min_error_correction_level: ErrorCorrectionLevel,
    mask_reference: Option<u8>,
    text: Option<&'a str>,
}

impl<'a> QrCodeBuilder<'a>
where
    [u8; MAX_VERSION * 4 + 17]: Sized,
{
    pub fn new() -> Self {
        Self {
            max_version: None,
            min_error_correction_level: ErrorCorrectionLevel::Medium,
            mask_reference: None,
            text: None,
        }
    }

    pub fn with_max_version(mut self, max_version: u8) -> Self {
        self.max_version = Some(max_version);
        self
    }

    pub fn with_min_error_correction_level(
        mut self,
        min_error_correction_level: ErrorCorrectionLevel,
    ) -> Self {
        self.min_error_correction_level = min_error_correction_level;
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
        let max_version = Version {
            version: self.max_version.unwrap_or(MAX_VERSION as u8),
        };
        assert!(max_version.version <= MAX_VERSION as u8);

        let encoded_data = encode_text(max_version, self.min_error_correction_level, self.text.unwrap()).unwrap();

        let error_corrected_data = add_error_correction(encoded_data);

        let matrix = Matrix::from_data(error_corrected_data);

        let masked = if let Some(mask_reference) = self.mask_reference {
            matrix.mask(mask_reference)
        } else {
            matrix.best_mask()
        };

        QrCode::from(masked)
    }
}

pub struct QrCode<const MAX_VERSION: usize>
where
    [u8; MAX_VERSION * 4 + 17]: Sized,
{
    data: Array2D<Color, { MAX_VERSION * 4 + 17 }>,
}

impl<const MAX_VERSION: usize> QrCode<MAX_VERSION>
where
    [u8; MAX_VERSION * 4 + 17]: Sized,
{
    pub fn from(scored: ScoreMasked<{ MAX_VERSION * 4 + 17 }>) -> Self {
        let data = scored.masked.matrix.data;
        let size = data.size();

        let mut out = Array2D::new();
        out.set_size(data.size());
        for x in 0..size.x {
            for y in 0..size.y {
                let pos = (x, y).into();
                out[pos] = data[pos].into();
            }
        }

        Self { data: out }
    }
}

impl<const MAX_VERSION: usize> Debug for QrCode<MAX_VERSION>
where
    [u8; MAX_VERSION * 4 + 17]: Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.data.rows().try_for_each(|mut row| {
            row.try_for_each(|color| {
                f.write_char(match color {
                    Color::Black => '\u{2588}',
                    Color::White => '_',
                })
            })?;
            f.write_char('\n')
        })
    }
}

impl<const MAX_VERSION: usize> Display for QrCode<MAX_VERSION>
where
    [u8; MAX_VERSION * 4 + 17]: Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let iter1 = self.data.rows().step_by(2);
        let iter2 = self.data.rows().skip(1).step_by(2);
        iter1.zip(iter2).try_for_each(|rows| {
            rows.0.zip(rows.1).try_for_each(|(&up, &down)| {
                f.write_char(match (up, down) {
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
            f.write_char(match up {
                Color::Black => '\u{2580}',
                Color::White => ' ',
            })
        })?;
        f.write_char('\n')
    }
}

#[cfg(test)]
mod tests {
    use crate::error_correction::ErrorCorrectionLevel;
    use crate::qrcode::QrCodeBuilder;
    use alloc::format;

    #[test]
    fn numeric_version_1_auto_select_high() {
        let qr_code = QrCodeBuilder::new()
            .with_text("01234567")
            .with_max_version(1)
            .with_min_error_correction_level(ErrorCorrectionLevel::Medium)
            .with_mask_reference(0b010)
            .build();

        assert_eq!(
            format!("{:?}", qr_code),
            "\
███████_████__███████
█_____█_█████_█_____█
█_███_█_█_█___█_███_█
█_███_█_______█_███_█
█_███_█__███__█_███_█
█_____█_██__█_█_____█
███████_█_█_█_███████
________█_██_________
__███_█_█__█_███__███
_█_█___█__█_█__█_██__
__█__███_████___█████
_█___█__██_████████__
_█__███_███___█_█____
________██____█__██__
███████__█__██_█_____
█_____█__██_███___█_█
█_███_█_████_█_█_██__
█_███_█_█_█_█__█_____
█_███_█_█_█_____█_█__
█_____█_____█__██_██_
███████__█_█__█_█_█__
"
        );
    }

    #[test]
    fn numeric_auto_select_1_h() {
        let qr_code = QrCodeBuilder::new()
            .with_text("01234567")
            .build();

        assert_eq!(
            format!("{:?}", qr_code),
            "\
███████__█____███████
█_____█___███_█_____█
█_███_█_█_____█_███_█
█_███_█_███___█_███_█
█_███_█__██___█_███_█
█_____█___███_█_____█
███████_█_█_█_███████
_________███_________
___██_██__██_____██__
_██_█__███__█_█_███_█
______█████_█_█_█_██_
_█_██___█_█_███___█__
__█___██_█_█_█___█_██
________█____█_█_████
███████_███_█__██__█_
█_____█_____██_██_█__
█_███_█_███__███__█_█
█_███_█_██_██___██___
█_███_█____█_██__████
█_____█__█__███_█_█_█
███████__███_██___██_
"
        );
    }

    #[test]
    fn alphanumeric_version_1() {
        let qr_code = QrCodeBuilder::new()
            .with_max_version(1)
            .with_min_error_correction_level(ErrorCorrectionLevel::Quartile)
            .with_mask_reference(0b110)
            .with_text("HELLO WORLD")
            .build();

        assert_eq!(
            format!("{:?}", qr_code),
            "\
███████____█__███████
█_____█_██__█_█_____█
█_███_█__█_██_█_███_█
█_███_█_█████_█_███_█
█_███_█_██_█__█_███_█
█_____█__█__█_█_____█
███████_█_█_█_███████
________██_██________
_█_████_██__███_██_█_
█_████_█____████_███_
__█_█_██___█__██_____
█_██_█___█_██___██___
██_████████_███_█████
________█___█__█_█___
███████__██__██__████
█_____█_█_█__█__█_███
█_███_█_██_█__█___███
█_███_█_█_███___█_█__
█_███_█__█____█____██
█_____█_███__███__██_
███████__█_█_______█_
"
        );
    }

    #[test]
    fn alphanumeric_version_2() {
        let qr_code = QrCodeBuilder::new()
            .with_max_version(2)
            .with_min_error_correction_level(ErrorCorrectionLevel::Quartile)
            .with_mask_reference(0b110)
            .with_text("HTTPS://CASPERMEIJN.NL")
            .build();

        assert_eq!(
            format!("{:?}", qr_code),
            "\
███████____█████__███████
█_____█_█_█_████__█_____█
█_███_█__█___██___█_███_█
█_███_█_██__█_███_█_███_█
█_███_█_█_██_█_█__█_███_█
█_____█___█_███_█_█_____█
███████_█_█_█_█_█_███████
________█_█___█__________
_█_████_███████████_██_█_
█_█_█__█_████_█_█_█████__
███_█_█_██_██_██__█_____█
█______███_██_█_█_█_████_
██_█__██_█_████_█__█_███_
____██_████_██___█_█_█___
______███_█__███_███_█___
_____█_██______██___█_███
__██__█_█_██_██_█████_███
________█___█___█___██___
███████__█__██__█_█_█_███
█_____█_█_█____██___████_
█_███_█_███__████████_█__
█_███_█_█_____█_█_█___█_█
█_███_█__█_█___██____████
█_____█_█_███_█████_█_██_
███████__██████_____██_██
"
        );
    }

    #[test]
    fn alphanumeric_version_4() {
        let qr_code = QrCodeBuilder::new()
            .with_max_version(4)
            .with_min_error_correction_level(ErrorCorrectionLevel::High)
            .with_mask_reference(0b110)
            .with_text("HTTPS://GITHUB.COM/CASPERMEIJN/TINY-QR")
            .build();

        assert_eq!(
            format!("{:?}", qr_code),
            "\
███████____██__█_█_█_██___███████
█_____█__█_███_█_____████_█_____█
█_███_█_█__█______██___██_█_███_█
█_███_█_█___████__█████_█_█_███_█
█_███_█___█____████_█__█__█_███_█
█_____█____██_█__█__█___█_█_____█
███████_█_█_█_█_█_█_█_█_█_███████
___________███__█__█___█_________
___██_██_████_█_██_███_█_____██__
_█______█____█________█_█_█████_█
_███_███___█__███__███_████___███
_█_█_█___███___█████___█___██_██_
███_███_█_______████████____█_█_█
██___█___█__█_█____█__██_█____█__
_██_████__█_████_█_██_█_███_█_██_
█___██_███_█__█████__███_█__█_██_
__█___█_____██_███_____█_█__███_█
███__█__█_████_█_████_██_█___██__
____█_████__█_████__█_█_____█_█_█
_███_█__█_███_█__███_██__█_██____
__█_█_█_█_███_████_█_███__███__██
__█_██____████_████_█___█_███████
___█_█████__█_█_█████_██_██_█__█_
__█_██_____██___█__█___██_████___
__███████___█__██_██__███████____
________█__██_██_███_█_██___█_█_█
███████_█_█_██_███_██__██_█_███__
█_____█__█__█__███___██_█___█_███
█_███_█_██_█_█____█__████████████
█_███_█_██__██_█__██████_______█_
█_███_█__████████__████_█___██_██
█_____█____██_█___█__███____██_█_
███████__█__██_█_█__███_█_██__██_
"
        );
    }
}
