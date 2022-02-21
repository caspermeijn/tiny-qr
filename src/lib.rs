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

#![no_std]

//TODO: Remove alloc requirement
extern crate alloc;

mod array_2d;
mod blocks;
pub mod buffer;
mod draw_iterator;
mod encoding;
mod error_correction;
mod format;
mod mask;
mod matrix;
mod qr_version;
mod qrcode;

pub use error_correction::ErrorCorrectionLevel;
pub use matrix::Color;
pub use qrcode::QrCode;
pub use qrcode::QrCodeGenerator;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
