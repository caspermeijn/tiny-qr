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

fn calculate_encoded_data_bit_length(
    data_len: usize,
    version: Option<Version>,
    mode: EncodingMode,
) -> usize {
    let version = version.unwrap_or(Version { version: 40 });
    let mode_bits = 4;
    let char_count_len = version.character_count_indicator_bit_length(mode);

    match mode {
        EncodingMode::Numeric => {
            mode_bits
                + char_count_len
                + 10 * (data_len / 3)
                + match data_len % 3 {
                    0 => 0,
                    1 => 4,
                    2 => 7,
                    _ => panic!(),
                }
        }
        EncodingMode::Alphanumeric => {
            mode_bits + char_count_len + 11 * (data_len / 2) + 6 * (data_len % 2)
        }
        EncodingMode::Iso8859_1 => mode_bits + char_count_len + 8 * data_len,
    }
}

pub fn encode_text(
    max_version: Version,
    min_error_correction: ErrorCorrectionLevel,
    text: &str,
) -> Result<EncodedData, ()> {
    let encoding = EncodingMode::select_best_encoding(text)?;

    let bit_len = calculate_encoded_data_bit_length(text.len(), Some(max_version), encoding);

    if max_version.data_codeword_bit_len(min_error_correction) < bit_len {
        return Err(());
    }

    let mut selected_error_correction = min_error_correction;
    while let Some(increased_error_correction) = selected_error_correction.increment() {
        if max_version.data_codeword_bit_len(increased_error_correction) >= bit_len {
            selected_error_correction = increased_error_correction;
        } else {
            break;
        }
    }

    let mut selected_version = max_version;
    while let Some(decreased_version) = selected_version.decrement() {
        if decreased_version.data_codeword_bit_len(selected_error_correction) >= bit_len {
            selected_version = decreased_version;
        } else {
            break;
        }
    }

    let buffer = match encoding {
        EncodingMode::Numeric => {
            let encoder = NumericDataEncoder {
                version: selected_version,
                error_correction: selected_error_correction,
            };
            encoder.encode(text)
        }
        EncodingMode::Alphanumeric => {
            let encoder = AlphanumericDataEncoder {
                version: selected_version,
                error_correction: selected_error_correction,
            };
            encoder.encode(text)
        }
        EncodingMode::Iso8859_1 => {
            let encoder = Iso8859_1DataEncoder {
                version: selected_version,
                error_correction: selected_error_correction,
            };
            encoder.encode(text)
        }
    };
    Ok(EncodedData {
        version: selected_version,
        error_correction: selected_error_correction,
        buffer,
    })
}

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

pub struct Iso8859_1DataEncoder {
    // TODO: Combine Version and ErrorCorrectionLevel
    pub(crate) version: Version,
    pub(crate) error_correction: ErrorCorrectionLevel,
}

impl Iso8859_1DataEncoder {
    //TODO: Spec contains a formula for calculating the length of the output before encoding it.

    fn convert_iso8859_1(c: char) -> u32 {
        assert!(c as u32 <= 0xFF);
        c as u32
    }

    fn encode_mode_indicator(&self, buffer: &mut Buffer) {
        buffer.append_bits(&[false, true, false, false])
    }

    fn encode_character_count_indicator(&self, count: u32, buffer: &mut Buffer) {
        let bit_len = self
            .version
            .character_count_indicator_bit_length(EncodingMode::Iso8859_1);
        buffer.append_number(count, bit_len);
    }

    fn encode_data(&self, data: &str, buffer: &mut Buffer) {
        let mut chars = data.chars();
        for char1 in chars {
            let char1 = Self::convert_iso8859_1(char1);
            buffer.append_number(char1, 8);
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
    Iso8859_1,
}

fn is_char_numeric(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_char_alphanumeric(c: char) -> bool {
    matches!(c, '0'..='9' | 'A'..='Z' | ' ' | '$' | '%' | '*' | '+' | '-' | '.' | '/' | ':')
}

fn is_char_iso_8859_1(c: char) -> bool {
    c as u32 <= 0xff
}

impl EncodingMode {
    pub fn select_best_encoding(data: &str) -> Result<EncodingMode, ()> {
        if data.chars().all(is_char_numeric) {
            Ok(EncodingMode::Numeric)
        } else if data.chars().all(is_char_alphanumeric) {
            Ok(EncodingMode::Alphanumeric)
        } else if data.chars().all(is_char_iso_8859_1) {
            Ok(EncodingMode::Iso8859_1)
        } else {
            Err(())
        }
    }
}

pub struct EncodedData {
    pub(crate) version: Version,
    pub(crate) error_correction: ErrorCorrectionLevel,
    pub(crate) buffer: Buffer,
}

#[cfg(test)]
mod tests {
    use crate::encoding::{
        AlphanumericDataEncoder, EncodingMode, Iso8859_1DataEncoder, NumericDataEncoder,
    };
    use crate::error_correction::ErrorCorrectionLevel;
    use crate::qr_version::Version;

    #[test]
    fn numeric() {
        let data = "01234567";

        let best_encoding = EncodingMode::select_best_encoding(data);
        assert_eq!(best_encoding, Ok(EncodingMode::Numeric));

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
        assert_eq!(best_encoding, Ok(EncodingMode::Alphanumeric));

        let buffer = encoder.encode(data);
        assert_eq!(
            buffer.data(),
            [
                0b00100000, 0b01011011, 0b00001011, 0b01111000, 0b11010001, 0b01110010, 0b11011100,
                0b01001101, 0b01000011, 0b01000000, 0b11101100, 0b00010001, 0b11101100
            ]
        )
    }

    #[test]
    fn iso8859_1() {
        let data = "[H@llo wórld]";
        let encoder = Iso8859_1DataEncoder {
            version: Version { version: 2 },
            error_correction: ErrorCorrectionLevel::Quartile,
        };

        let best_encoding = EncodingMode::select_best_encoding(data);
        assert_eq!(best_encoding, Ok(EncodingMode::Iso8859_1));

        let buffer = encoder.encode(data);
        assert_eq!(
            buffer.data(),
            [
                64, 229, 180, 132, 6, 198, 198, 242, 7, 127, 55, 38, 198, 69, 208, 0, 236, 17, 236,
                17, 236, 17
            ]
        )
    }
}
