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

use bmp::{px, Image, Pixel};
use std::env;
use tiny_qr::{Color, ErrorCorrectionLevel, QrCode};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let data = args.join(" ");

    let qr_code = QrCode::generator()
        .with_specific_error_correction_level(ErrorCorrectionLevel::Low)
        .with_text(data.as_str())
        .build();

    let iter = qr_code.draw_iter();

    let mut img = Image::new(iter.width() as u32, iter.height() as u32);
    for module in iter {
        img.set_pixel(
            module.x as u32,
            module.y as u32,
            match module.color {
                Color::White => px!(255, 255, 255),
                Color::Black => px!(0, 0, 0),
            },
        )
    }

    let filename = "img.bmp";
    let result = img.save(filename);
    if let Err(err) = result {
        eprintln!("Unable to write to file: {}", err);
        std::process::exit(1);
    }
    println!("Generated QR code for {} to {}", data, filename);
}
