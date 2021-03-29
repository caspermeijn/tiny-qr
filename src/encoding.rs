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

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum VersionRestriction {
    MaxVersion(Version),
    SpecificVersion(Version),
}

impl VersionRestriction {
    fn to_version(self) -> Version {
        match self {
            VersionRestriction::MaxVersion(version) => version,
            VersionRestriction::SpecificVersion(version) => version,
        }
    }
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum ErrorCorrectionRestriction {
    MinErrorCorrection(ErrorCorrectionLevel),
    SpecificErrorCorrection(ErrorCorrectionLevel),
}

impl ErrorCorrectionRestriction {
    fn to_error_correction(self) -> ErrorCorrectionLevel {
        match self {
            ErrorCorrectionRestriction::MinErrorCorrection(error_correction) => error_correction,
            ErrorCorrectionRestriction::SpecificErrorCorrection(error_correction) => {
                error_correction
            }
        }
    }
}

fn calculate_encoded_data_bit_length(
    data_len: usize,
    version: Version,
    character_set: CharacterSet,
) -> usize {
    let mode_bits = 4;
    let char_count_len =
        version.character_count_indicator_bit_length(character_set.to_encoding_mode());

    match character_set {
        CharacterSet::Numeric => {
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
        CharacterSet::Alphanumeric => {
            mode_bits + char_count_len + 11 * (data_len / 2) + 6 * (data_len % 2)
        }
        CharacterSet::Iso8859_1 => mode_bits + char_count_len + 8 * data_len,
        CharacterSet::Unicode => 4 + 8 + mode_bits + char_count_len + 8 * data_len,
    }
}

pub fn encode_text(
    version_restriction: VersionRestriction,
    error_correction_restriction: ErrorCorrectionRestriction,
    text: &str,
) -> Result<EncodedData, ()> {
    // Find the character set to encode in
    let character_set = detect_character_set(text);

    // Check whether the data could fit with the provided restrictions
    let max_version = version_restriction.to_version();
    let min_error_correction = error_correction_restriction.to_error_correction();
    let bit_len = calculate_encoded_data_bit_length(text.len(), max_version, character_set);
    if max_version.data_codeword_bit_len(min_error_correction) < bit_len {
        return Err(());
    }

    // Try to increase the error correction while the data still fits and it is allowed by the restriction
    let selected_error_correction = match error_correction_restriction {
        ErrorCorrectionRestriction::MinErrorCorrection(min_error_correction) => {
            let mut selected_error_correction = min_error_correction;
            while let Some(increased_error_correction) = selected_error_correction.increment() {
                if max_version.data_codeword_bit_len(increased_error_correction) >= bit_len {
                    selected_error_correction = increased_error_correction;
                } else {
                    break;
                }
            }
            selected_error_correction
        }
        ErrorCorrectionRestriction::SpecificErrorCorrection(error_correction) => error_correction,
    };

    // Try to decrease the version while the data still fits and it is allowed by the restriction
    let selected_version = match version_restriction {
        VersionRestriction::MaxVersion(max_version) => {
            let mut selected_version = max_version;
            while let Some(decreased_version) = selected_version.decrement() {
                if decreased_version.data_codeword_bit_len(selected_error_correction) >= bit_len {
                    selected_version = decreased_version;
                } else {
                    break;
                }
            }
            selected_version
        }
        VersionRestriction::SpecificVersion(version) => version,
    };

    // Encode the data
    let buffer = match character_set {
        CharacterSet::Numeric => {
            let encoder = NumericDataEncoder {
                version: selected_version,
                error_correction: selected_error_correction,
            };
            encoder.encode(text)
        }
        CharacterSet::Alphanumeric => {
            let encoder = AlphanumericDataEncoder {
                version: selected_version,
                error_correction: selected_error_correction,
            };
            encoder.encode(text)
        }
        CharacterSet::Iso8859_1 => {
            let encoder = Iso8859_1DataEncoder {
                version: selected_version,
                error_correction: selected_error_correction,
            };
            encoder.encode(text)
        }
        CharacterSet::Unicode => {
            let encoder = UnicodeDataEncoder {
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
            .character_count_indicator_bit_length(EncodingMode::Byte);
        buffer.append_number(count, bit_len);
    }

    fn encode_data(&self, data: &str, buffer: &mut Buffer) {
        for char1 in data.chars() {
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

pub struct UnicodeDataEncoder {
    // TODO: Combine Version and ErrorCorrectionLevel
    pub(crate) version: Version,
    pub(crate) error_correction: ErrorCorrectionLevel,
}

impl UnicodeDataEncoder {
    //TODO: Spec contains a formula for calculating the length of the output before encoding it.

    fn encode_mode_indicator(&self, buffer: &mut Buffer) {
        // ECI indicator for UTF-8
        buffer.append_bits(&[false, true, true, true]);
        buffer.append_byte(26);
        // Byte mode indicator
        buffer.append_bits(&[false, true, false, false])
    }

    fn encode_character_count_indicator(&self, count: u32, buffer: &mut Buffer) {
        let bit_len = self
            .version
            .character_count_indicator_bit_length(EncodingMode::Byte);
        buffer.append_number(count, bit_len);
    }

    fn encode_data(&self, data: &str, buffer: &mut Buffer) {
        for byte1 in data.bytes() {
            buffer.append_byte(byte1);
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
}

#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum CharacterSet {
    Numeric,
    Alphanumeric,
    Iso8859_1,
    Unicode,
}

impl CharacterSet {
    fn to_encoding_mode(self) -> EncodingMode {
        match self {
            CharacterSet::Numeric => EncodingMode::Numeric,
            CharacterSet::Alphanumeric => EncodingMode::Alphanumeric,
            CharacterSet::Iso8859_1 => EncodingMode::Byte,
            CharacterSet::Unicode => EncodingMode::Byte,
        }
    }
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

fn detect_character_set(data: &str) -> CharacterSet {
    if data.chars().all(is_char_numeric) {
        CharacterSet::Numeric
    } else if data.chars().all(is_char_alphanumeric) {
        CharacterSet::Alphanumeric
    } else if data.chars().all(is_char_iso_8859_1) {
        CharacterSet::Iso8859_1
    } else {
        CharacterSet::Unicode
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
        detect_character_set, AlphanumericDataEncoder, CharacterSet, Iso8859_1DataEncoder,
        NumericDataEncoder, UnicodeDataEncoder,
    };
    use crate::error_correction::ErrorCorrectionLevel;
    use crate::qr_version::Version;

    #[test]
    fn numeric() {
        let data = "01234567";

        let character_set = detect_character_set(data);
        assert_eq!(character_set, CharacterSet::Numeric);

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

        let character_set = detect_character_set(data);
        assert_eq!(character_set, CharacterSet::Alphanumeric);

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

        let character_set = detect_character_set(data);
        assert_eq!(character_set, CharacterSet::Iso8859_1);

        let buffer = encoder.encode(data);
        assert_eq!(
            buffer.data(),
            [
                64, 229, 180, 132, 6, 198, 198, 242, 7, 127, 55, 38, 198, 69, 208, 0, 236, 17, 236,
                17, 236, 17
            ]
        )
    }

    #[test]
    fn unicode() {
        let data = "I 💓 you";

        let character_set = detect_character_set(data);
        assert_eq!(character_set, CharacterSet::Unicode);

        let encoder = UnicodeDataEncoder {
            version: Version { version: 1 },
            error_correction: ErrorCorrectionLevel::Quartile,
        };
        let buffer = encoder.encode(data);
        assert_eq!(
            buffer.data(),
            [
                0b0111_0001,
                0b1010_0100,
                10,
                73,
                32,
                240,
                159,
                146,
                147,
                32,
                'y' as u8,
                'o' as u8,
                'u' as u8,
            ]
        )
    }
}
