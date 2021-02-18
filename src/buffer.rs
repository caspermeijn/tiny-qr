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

pub struct Buffer {
    data: [u8; 1024],
    bit_len: usize,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    /// Creates a new empty buffer
    ///
    /// # Example
    ///```
    ///     use tiny_qr::buffer::Buffer;
    ///        let mut buffer = Buffer::new();
    ///         assert_eq!(buffer.data(), []);
    ///         buffer.append_bytes(&[1,2,3]);
    ///         assert_eq!(buffer.data(), [1,2,3]);
    ///```
    pub fn new() -> Buffer {
        Buffer {
            data: [0; 1024],
            bit_len: 0,
        }
    }

    /// Return the amount of bytes and bits written to the buffer
    ///
    /// # Example
    ///```
    ///      use tiny_qr::buffer::Buffer;
    ///        let mut buffer = Buffer::new();
    ///              buffer.append_bytes(&[1,2,3]);
    ///         buffer.append_bits(&[true, false, true, false]);
    ///         assert_eq!(buffer.byte_bit_len(), (3, 4));
    ///```
    pub fn byte_bit_len(&self) -> (usize, usize) {
        (self.bit_len / 8, self.bit_len % 8)
    }

    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    /// Adds a single byte to the buffer
    ///
    /// # Example
    ///```
    ///      use tiny_qr::buffer::Buffer;
    ///        let mut buffer = Buffer::new();
    ///         buffer.append_bit(true);
    ///          buffer.append_bit(false);
    ///          buffer.append_bit(true);
    ///          buffer.append_bit(false);
    ///         assert_eq!(buffer.data(), [0b1010_0000]);
    ///```
    pub fn append_bit(&mut self, bit: bool) {
        let (byte_len, bit_len) = self.byte_bit_len();
        if bit {
            let mask = 1 << (7 - bit_len);
            self.data[byte_len] |= mask;
        }
        self.bit_len += 1;
    }

    /// Adds a single byte to the buffer
    ///
    /// # Example
    ///```
    ///      use tiny_qr::buffer::Buffer;
    ///        let mut buffer = Buffer::new();
    ///         buffer.append_bits(&[true, false, true, false]);
    ///         assert_eq!(buffer.data(), [0b1010_0000]);
    ///```
    pub fn append_bits(&mut self, bits: &[bool]) {
        for bit in bits {
            self.append_bit(*bit)
        }
    }

    /// Adds a single byte to the buffer
    ///
    /// # Example
    ///```
    ///      use tiny_qr::buffer::Buffer;
    ///        let mut buffer = Buffer::new();
    ///         buffer.append_byte(1);
    ///         buffer.append_byte(2);
    ///         buffer.append_byte(3);
    ///         assert_eq!(buffer.data(), [1,2,3]);
    ///```
    pub fn append_byte(&mut self, byte: u8) {
        let (byte_len, bit_len) = self.byte_bit_len();
        if bit_len == 0 {
            self.data[byte_len] = byte;
            self.bit_len += 8;
        } else {
            for index in (0..8).rev() {
                let mask = 1 << index;
                let bit = byte & mask != 0;
                self.append_bit(bit);
            }
        }
    }

    /// Adds multiple bytes to the buffer
    ///
    /// # Example
    ///```
    ///      use tiny_qr::buffer::Buffer;
    ///        let mut buffer = Buffer::new();
    ///         buffer.append_bytes(&[1,2,3]);
    ///         assert_eq!(buffer.data(), [1,2,3]);
    ///```
    pub fn append_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.append_byte(*byte)
        }
    }

    /// Adds a number of a specific length to the buffer
    ///
    /// # Example
    ///```
    ///      use tiny_qr::buffer::Buffer;
    ///        let mut buffer = Buffer::new();
    ///         buffer.append_number(3, 4);
    /// buffer.append_number(0b111100, 6);
    /// buffer.append_number(2, 2);
    ///         assert_eq!(buffer.data(), [0b0011_1111, 0b0010_0000]);
    ///```
    pub fn append_number(&mut self, number: u32, bit_len: usize) {
        for index in (0..bit_len).rev() {
            let mask = 1 << index;
            self.append_bit(number & mask != 0)
        }
    }

    /// Returns a slice of all written data.
    ///
    /// # Example
    ///```
    ///      use tiny_qr::buffer::Buffer;
    ///        let mut buffer = Buffer::new();
    ///         buffer.append_bytes(&[1,2,3]);
    ///         assert_eq!(buffer.data(), [1,2,3]);
    ///```
    pub fn data(&self) -> &[u8] {
        let (byte_len, bit_len) = self.byte_bit_len();
        if bit_len == 0 {
            &self.data[0..byte_len]
        } else {
            &self.data[0..byte_len + 1]
        }
    }
}
