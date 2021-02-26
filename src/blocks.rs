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

use crate::error_correction::ErrorCorrectionLevel;
use crate::qr_version::Version;
use core::iter::Chain;

pub struct BlockIterator<'a> {
    iter: Chain<BlockDataIterator<'a>, BlockEccIterator<'a>>,
}

impl BlockIterator<'_> {
    pub fn new(
        data: &[u8],
        version: Version,
        error_correction: ErrorCorrectionLevel,
    ) -> BlockIterator {
        let data_iter = BlockDataIterator::new(data, version, error_correction);
        let ecc_iter = BlockEccIterator::new(data, version, error_correction);
        BlockIterator {
            iter: data_iter.chain(ecc_iter),
        }
    }
}

impl<'a> Iterator for BlockIterator<'a> {
    type Item = &'a u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Copy, Clone)]
pub struct BlockDataIterator<'a> {
    data: &'a [u8],
    blocks: BlockLengthIterator,
    data_offset: usize,
}

impl<'a> BlockDataIterator<'a> {
    pub fn new(data: &'a [u8], version: Version, error_correction: ErrorCorrectionLevel) -> Self {
        let data_len = version.data_codeword_count(error_correction);
        Self {
            data: &data[0..data_len],
            blocks: BlockLengthIterator::new(version, error_correction),
            data_offset: 0,
        }
    }

    pub fn next_block_length(&mut self) -> Option<BlockLength> {
        if let Some(block) = self.blocks.next() {
            if self.data_offset < block.data_len {
                Some(block)
            } else {
                if block.block_number < block.block_count - 1 {
                    self.next_block_length()
                } else {
                    None
                }
            }
        } else {
            self.data_offset += 1;
            self.blocks.reset();
            self.next_block_length()
        }
    }
}

impl<'a> Iterator for BlockDataIterator<'a> {
    type Item = &'a u8;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(block) = self.next_block_length() {
            Some(&self.data[block.data_pos + self.data_offset])
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
pub struct BlockEccIterator<'a> {
    data: &'a [u8],
    blocks: BlockLengthIterator,
    ecc_offset: usize,
}

impl<'a> BlockEccIterator<'a> {
    pub fn new(data: &'a [u8], version: Version, error_correction: ErrorCorrectionLevel) -> Self {
        Self {
            data: &data,
            blocks: BlockLengthIterator::new(version, error_correction),
            ecc_offset: 0,
        }
    }
}

impl<'a> Iterator for BlockEccIterator<'a> {
    type Item = &'a u8;

    fn next(&mut self) -> Option<Self::Item> {
        let block = self
            .blocks
            .next()
            .or_else(|| {
                self.ecc_offset += 1;
                self.blocks.reset();
                self.blocks.next()
            })
            .unwrap();

        let position = block.ecc_pos + self.ecc_offset;

        if self.ecc_offset < block.ecc_len && position < self.data.len() {
            Some(&self.data[block.ecc_pos + self.ecc_offset])
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct BlockLength {
    pub block_number: usize,
    pub block_count: usize,
    pub data_pos: usize,
    pub data_len: usize,
    pub ecc_pos: usize,
    pub ecc_len: usize,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct BlockLengthIterator {
    //TODO: Combine version and error correction
    version: Version,
    error_correction: ErrorCorrectionLevel,
    last: Option<BlockLength>,
}

impl BlockLengthIterator {
    pub fn new(version: Version, error_correction: ErrorCorrectionLevel) -> BlockLengthIterator {
        BlockLengthIterator {
            version,
            error_correction,
            last: None,
        }
    }

    pub fn reset(&mut self) {
        self.last = None;
    }
}

impl Iterator for BlockLengthIterator {
    type Item = BlockLength;

    fn next(&mut self) -> Option<Self::Item> {
        let data_len = self.version.data_codeword_count(self.error_correction);
        if self.last.is_none() {
            let (ecc_len, blocks) = self
                .version
                .error_correction_codeword_blocks_count(self.error_correction);
            assert_eq!(ecc_len % blocks, 0);

            self.last = Some(BlockLength {
                block_number: 0,
                block_count: blocks,
                data_pos: 0,
                data_len: data_len / blocks,
                ecc_pos: data_len,
                ecc_len: ecc_len / blocks,
            });
            self.last
        } else {
            let next = self.last.as_mut().unwrap();
            next.block_number += 1;
            if next.block_number < next.block_count {
                next.data_pos += next.data_len;
                next.ecc_pos += next.ecc_len;
                // If data_len % blocks != 0, then the first blocks are smaller
                if next.block_number == data_len % next.block_count {
                    next.data_len += 1
                }
                self.last
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::blocks::{BlockIterator, BlockLength, BlockLengthIterator};
    use crate::error_correction::ErrorCorrectionLevel;
    use crate::qr_version::Version;

    #[test]
    fn block_len_iter_5q() {
        let mut iter =
            BlockLengthIterator::new(Version { version: 5 }, ErrorCorrectionLevel::Quartile);
        assert_eq!(
            iter.next(),
            Some(BlockLength {
                block_number: 0,
                block_count: 4,
                data_pos: 0,
                data_len: 15,
                ecc_pos: 62,
                ecc_len: 18
            })
        );
        assert_eq!(
            iter.next(),
            Some(BlockLength {
                block_number: 1,
                block_count: 4,
                data_pos: 15,
                data_len: 15,
                ecc_pos: 80,
                ecc_len: 18
            })
        );
        assert_eq!(
            iter.next(),
            Some(BlockLength {
                block_number: 2,
                block_count: 4,
                data_pos: 30,
                data_len: 16,
                ecc_pos: 98,
                ecc_len: 18
            })
        );
        assert_eq!(
            iter.next(),
            Some(BlockLength {
                block_number: 3,
                block_count: 4,
                data_pos: 46,
                data_len: 16,
                ecc_pos: 116,
                ecc_len: 18
            })
        );
        assert_eq!(iter.next(), None);
        iter.reset();
        assert_eq!(
            iter.next(),
            Some(BlockLength {
                block_number: 0,
                block_count: 4,
                data_pos: 0,
                data_len: 15,
                ecc_pos: 62,
                ecc_len: 18
            })
        );
    }

    #[test]
    fn block_iter_5q() {
        let data = [
            67, 85, 70, 134, 87, 38, 85, 194, 119, 50, 6, 18, 6, 103, 38, 246, 246, 66, 7, 118,
            134, 242, 7, 38, 86, 22, 198, 199, 146, 6, 182, 230, 247, 119, 50, 7, 118, 134, 87, 38,
            82, 6, 134, 151, 50, 7, 70, 247, 118, 86, 194, 6, 151, 50, 16, 236, 17, 236, 17, 236,
            17, 236, 213, 199, 11, 45, 115, 247, 241, 223, 229, 248, 154, 117, 154, 111, 86, 161,
            111, 39, 87, 204, 96, 60, 202, 182, 124, 157, 200, 134, 27, 129, 209, 17, 163, 163,
            120, 133, 148, 116, 177, 212, 76, 133, 75, 242, 238, 76, 195, 230, 189, 10, 108, 240,
            192, 141, 235, 159, 5, 173, 24, 147, 59, 33, 106, 40, 255, 172, 82, 2, 131, 32, 178,
            236,
        ];

        let iter = BlockIterator::new(
            &data,
            Version { version: 5 },
            ErrorCorrectionLevel::Quartile,
        );

        assert!(iter.eq([
            67, 246, 182, 70, 85, 246, 230, 247, 70, 66, 247, 118, 134, 7, 119, 86, 87, 118, 50,
            194, 38, 134, 7, 6, 85, 242, 118, 151, 194, 7, 134, 50, 119, 38, 87, 16, 50, 86, 38,
            236, 6, 22, 82, 17, 18, 198, 6, 236, 6, 199, 134, 17, 103, 146, 151, 236, 38, 6, 50,
            17, 7, 236, 213, 87, 148, 235, 199, 204, 116, 159, 11, 96, 177, 5, 45, 60, 212, 173,
            115, 202, 76, 24, 247, 182, 133, 147, 241, 124, 75, 59, 223, 157, 242, 33, 229, 200,
            238, 106, 248, 134, 76, 40, 154, 27, 195, 255, 117, 129, 230, 172, 154, 209, 189, 82,
            111, 17, 10, 2, 86, 163, 108, 131, 161, 163, 240, 32, 111, 120, 192, 178, 39, 133, 141,
            236,
        ]
        .iter()));
    }
}
