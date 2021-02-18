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
use crate::error_correction::ErrorCorrectionLevel;
use crate::qr_version::Version;

pub struct NumericDataEncoder {
    // TODO: Combine Version and ErrorCorrectionLevel
    pub(crate) version: Version,
    pub(crate) error_correction: ErrorCorrectionLevel,
}

impl NumericDataEncoder {
    //TODO: Spec contains a formula for calculating the length of the output before encoding it.

    fn encode_mode_indicator(&self, buffer: &mut Buffer) {
        buffer.append_bits(&[false, false, false, true])
    }

    fn encode_character_count_indicator(&self, count: u32, buffer: &mut Buffer) {
        let bit_len = self
            .version
            .character_count_indicator_bit_length(EncodingMode::Numeric);
        buffer.append_number(count, bit_len);
    }

    fn encode_data(&self, data: &str, buffer: &mut Buffer) {
        let mut i = 0;
        loop {
            let chars_left = data.len() - i;
            if chars_left >= 3 {
                let number = data[i..i + 3].parse::<u32>().unwrap();
                buffer.append_number(number, 10);
                i += 3;
            } else if chars_left == 2 {
                let number = data[i..i + 2].parse::<u32>().unwrap();
                buffer.append_number(number, 7);
                break;
            } else if chars_left == 1 {
                let number = data[i..i + 1].parse::<u32>().unwrap();
                buffer.append_number(number, 3);
                break;
            } else {
                break;
            }
        }
    }

    fn encode_terminator(&self, buffer: &mut Buffer) {
        let max_data_bit_len = self.version.data_codeword_bit_len(self.error_correction);

        let buffer_bit_len = buffer.bit_len();
        if max_data_bit_len - buffer_bit_len < 4 {
            buffer.append_number(0, max_data_bit_len - buffer_bit_len)
        } else {
            let alignment = 8 - ((buffer_bit_len + 4) % 8);
            buffer.append_number(0, 4 + alignment)
        }
    }

    fn encode_padding(&self, buffer: &mut Buffer) {
        let max_data_bit_len = self.version.data_codeword_bit_len(self.error_correction);
        loop {
            let bit_len_diff = max_data_bit_len - buffer.bit_len();
            if bit_len_diff == 0 {
                break;
            } else if bit_len_diff >= 16 {
                buffer.append_number(0b1110_1100_0001_0001, 16);
            } else if bit_len_diff == 8 {
                buffer.append_number(0b1110_1100, 8);
            } else {
                unreachable!()
            }
        }
    }

    pub fn encode(&self, data: &str) -> Buffer {
        let mut buffer = Buffer::new();
        self.encode_mode_indicator(&mut buffer);
        self.encode_character_count_indicator(data.len() as u32, &mut buffer);
        self.encode_data(data, &mut buffer);
        self.encode_terminator(&mut buffer);
        self.encode_padding(&mut buffer);
        buffer
    }
}

pub struct AlphanumericDataEncoder {
    // TODO: Combine Version and ErrorCorrectionLevel
    pub(crate) version: Version,
    pub(crate) error_correction: ErrorCorrectionLevel,
}

impl AlphanumericDataEncoder {
    //TODO: Spec contains a formula for calculating the length of the output before encoding it.

    fn convert_alphanumeric(c: char) -> u32 {
        match c {
            '0' => 0,
            '1' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            'A' => 10,
            'B' => 11,
            'C' => 12,
            'D' => 13,
            'E' => 14,
            'F' => 15,
            'G' => 16,
            'H' => 17,
            'I' => 18,
            'J' => 19,
            'K' => 20,
            'L' => 21,
            'M' => 22,
            'N' => 23,
            'O' => 24,
            'P' => 25,
            'Q' => 26,
            'R' => 27,
            'S' => 28,
            'T' => 29,
            'U' => 30,
            'V' => 31,
            'W' => 32,
            'X' => 33,
            'Y' => 34,
            'Z' => 35,
            ' ' => 36,
            '$' => 37,
            '%' => 38,
            '*' => 39,
            '+' => 40,
            '-' => 41,
            '.' => 42,
            '/' => 43,
            ':' => 44,
            _ => panic!(),
        }
    }

    fn encode_mode_indicator(&self, buffer: &mut Buffer) {
        buffer.append_bits(&[false, false, true, false])
    }

    fn encode_character_count_indicator(&self, count: u32, buffer: &mut Buffer) {
        let bit_len = self
            .version
            .character_count_indicator_bit_length(EncodingMode::Alphanumeric);
        buffer.append_number(count, bit_len);
    }

    fn encode_data(&self, data: &str, buffer: &mut Buffer) {
        let mut chars = data.chars();
        while let Some(char1) = chars.next() {
            if let Some(char2) = chars.next() {
                let char1 = Self::convert_alphanumeric(char1);
                let char2 = Self::convert_alphanumeric(char2);
                buffer.append_number(45 * char1 + char2, 11)
            } else {
                let char1 = Self::convert_alphanumeric(char1);
                buffer.append_number(char1, 6);
            }
        }
    }

    fn encode_terminator(&self, buffer: &mut Buffer) {
        let max_data_bit_len = self.version.data_codeword_bit_len(self.error_correction);

        let buffer_bit_len = buffer.bit_len();
        if max_data_bit_len - buffer_bit_len < 4 {
            buffer.append_number(0, max_data_bit_len - buffer_bit_len)
        } else {
            let alignment = 8 - ((buffer_bit_len + 4) % 8);
            buffer.append_number(0, 4 + alignment)
        }
    }

    fn encode_padding(&self, buffer: &mut Buffer) {
        let max_data_bit_len = self.version.data_codeword_bit_len(self.error_correction);
        loop {
            let bit_len_diff = max_data_bit_len - buffer.bit_len();
            if bit_len_diff == 0 {
                break;
            } else if bit_len_diff >= 16 {
                buffer.append_number(0b1110_1100_0001_0001, 16);
            } else if bit_len_diff == 8 {
                buffer.append_number(0b1110_1100, 8);
            } else {
                unreachable!()
            }
        }
    }

    pub fn encode(&self, data: &str) -> Buffer {
        let mut buffer = Buffer::new();
        self.encode_mode_indicator(&mut buffer);
        self.encode_character_count_indicator(data.len() as u32, &mut buffer);
        self.encode_data(data, &mut buffer);
        self.encode_terminator(&mut buffer);
        self.encode_padding(&mut buffer);
        buffer
    }
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum EncodingMode {
    Numeric,
    Alphanumeric,
    Byte,
    Kanji,
}

impl EncodingMode {
    pub fn select_best_encoding(data: &str) -> Option<EncodingMode> {
        if data.chars().all(|char| char.is_ascii_digit()) {
            Some(EncodingMode::Numeric)
        } else if data.chars().all(|char| {
            matches!(char, '0'..='9' | 'A'..='Z' | ' ' | '$' | '%' |
            '*' |
            '+' |
            '-' |
            '.' |
            '/' |
            ':')
        }) {
            Some(EncodingMode::Alphanumeric)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::encoding::{AlphanumericDataEncoder, EncodingMode, NumericDataEncoder};
    use crate::error_correction::ErrorCorrectionLevel;
    use crate::qr_version::Version;

    #[test]
    fn numeric() {
        let data = "01234567";

        let best_encoding = EncodingMode::select_best_encoding(data);
        assert_eq!(best_encoding, Some(EncodingMode::Numeric));

        let encoder = NumericDataEncoder {
            version: Version { version: 1 },
            error_correction: ErrorCorrectionLevel::Medium,
        };

        let buffer = encoder.encode(data);
        assert_eq!(
            buffer.data(),
            [
                0b00010000, 0b00100000, 0b00001100, 0b01010110, 0b01100001, 0b10000000, 0b11101100,
                0b00010001, 0b11101100, 0b00010001, 0b11101100, 0b00010001, 0b11101100, 0b00010001,
                0b11101100, 0b00010001
            ]
        )
    }

    #[test]
    fn alphanumeric() {
        let data = "HELLO WORLD";
        let encoder = AlphanumericDataEncoder {
            version: Version { version: 1 },
            error_correction: ErrorCorrectionLevel::Quartile,
        };

        let best_encoding = EncodingMode::select_best_encoding(data);
        assert_eq!(best_encoding, Some(EncodingMode::Alphanumeric));

        let buffer = encoder.encode(data);
        assert_eq!(
            buffer.data(),
            [
                0b00100000, 0b01011011, 0b00001011, 0b01111000, 0b11010001, 0b01110010, 0b11011100,
                0b01001101, 0b01000011, 0b01000000, 0b11101100, 0b00010001, 0b11101100
            ]
        )
    }
}