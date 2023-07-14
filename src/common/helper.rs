//! Helper functions for working with byte streams and printing.

use log::debug;
use std::collections::VecDeque;
use std::fmt::Debug;

/// Maintains the encoded bytestream and the byte offset
#[derive(Debug, Clone)]
pub struct ByteStream {
    pub bytestream: VecDeque<u8>,
    pub byte_offset: u8,
}

impl ByteStream {
    pub fn new(bytestream: Vec<u8>) -> ByteStream {
        ByteStream {
            bytestream: bytestream.into(),
            byte_offset: 0,
        }
    }
    #[allow(dead_code)]
    pub fn print(&self) {
        println!(
            "Byte_offset: {}\nBytestream: {:x?}",
            self.byte_offset, &self.bytestream
        );
    }

    #[allow(dead_code)]
    pub fn debug_print(&self) {
        debug!(
            "Byte_offset: {}\nBytestream: {:x?}",
            self.byte_offset, &self.bytestream
        );
    }

    /// implements more_rbsp_data() check - see section 7.2 and Annex B
    pub fn more_data(&self) -> bool {
        // definitely true if more than one byte in the stream
        if self.bytestream.len() > 1 {
            return true;
        } else if self.bytestream.is_empty() || self.byte_offset == 7 {
            // definitely false if bytestream is empty or only 1 byte and byte_offset is maximum possible value
            return false;
        }
        // only 1 byte, need to check byte_offset value is 1 and the rest 0

        // get the rest of the data, and see if it's greater than 2^byte_offset
        let mut new_bs = ByteStream {
            byte_offset: self.byte_offset,
            bytestream: self.bytestream.clone(),
        };
        let intermediate = new_bs.read_bits(7 - new_bs.byte_offset + 1);
        if intermediate == 0 || intermediate == (1 << (7 - self.byte_offset)) {
            // if we get zero or equal to the max power of two, then no more data
            return false;
        }

        true
    }

    pub fn read_bits(&mut self, bits_to_read: u8) -> u32 {
        let mut result: u32 = 0;

        // TODO: should this be replaced with an Option?
        if bits_to_read == 0 {
            return result;
        } else if self.bytestream.len() == 0 {
            panic!("read_bits - Bytestream is empty!");
        } else if (bits_to_read as usize) > (self.bytestream.len() * 8 - self.byte_offset as usize)
        {
            panic!("read_bits - Trying to read outside bounds!");
        }

        for i in (0..bits_to_read).rev() {
            let intermediate: u32 = (((self.bytestream[0] & (1 << (7 - self.byte_offset))) as u32)
                >> (7 - self.byte_offset))
                << i;

            result |= intermediate;

            self.byte_offset += 1;
            if self.byte_offset >= 8 {
                self.byte_offset = 0;
                self.bytestream.pop_front();
            }
        }

        result
    }

    pub fn read_bits64(&mut self, bits_to_read: u8) -> u64 {
        let mut result: u64 = 0;

        // TODO: should this be replaced with an Option?
        if bits_to_read == 0 {
            return result;
        } else if self.bytestream.len() == 0 {
            panic!("read_bits - Bytestream is empty!");
        } else if (bits_to_read as usize) > (self.bytestream.len() * 8 - self.byte_offset as usize)
        {
            panic!("read_bits - Trying to read outside bounds!");
        }

        for i in (0..bits_to_read).rev() {
            let intermediate: u64 = (((self.bytestream[0] & (1 << (7 - self.byte_offset))) as u64)
                >> (7 - self.byte_offset))
                << i;

            result |= intermediate;

            self.byte_offset += 1;
            if self.byte_offset >= 8 {
                self.byte_offset = 0;
                self.bytestream.pop_front();
            }
        }

        result
    }
}

/// Defined in equation 5-8 of the H.264 spec
///
/// An implicit in-range check, and if not in range then return the
/// minimum or maximum value possible.
pub fn clip3(x: i32, y: i32, z: i32) -> i32 {
    assert!(x < y);

    if z < x {
        x
    } else if z > y {
        y
    } else {
        z
    }
}

/// Implements equation 5-11 of the H.264 spec
pub fn inverse_raster_scan(a: usize, b: usize, c: usize, d: usize, e: usize) -> usize {
    if e == 0 {
        (a % (d / b)) * b
    } else {
        (a / (d / b)) * c
    }
}

/// Output to STDOUT a string and its value at a particular width
pub fn formatted_print<T: Debug>(out: &str, val: T, width: usize) {
    println!("\t{out:<width$} ({:?})", val, out = out, width = width);
}

/// Output to decoding debug file a string and its value at a particular width
pub fn decoder_formatted_print<T: Debug>(out: &str, val: T, width: usize) {
    debug!(target: "decode",
        "\t{out:<width$} ({:?})",
        val,
        out = out,
        width = width
    );
}

/// Output to encoding debug file a string and its value at a particular width
pub fn encoder_formatted_print<T: Debug>(out: &str, val: T, width: usize) {
    debug!(target: "encode",
        "\t{out:<width$} ({:?})",
        val,
        out = out,
        width = width
    );
}

/// Returns whether a Slice letter matches the slice number
/// # Arguments
///
/// * `slice_type_num` - the decoded slice number
/// * `slice_letter` - the slice type we're interested in matching
pub fn is_slice_type(slice_type_num: u8, slice_letter: &str) -> bool {
    let sl = slice_letter.to_uppercase();
    match sl.as_str() {
        "P" => slice_type_num % 5 == 0,  // 0 or 5,
        "B" => slice_type_num % 5 == 1,  // 1 or 6,
        "I" => slice_type_num % 5 == 2,  // 2 or 7
        "SP" => slice_type_num % 5 == 3, // 3 or 8
        "SI" => slice_type_num % 5 == 4, // 4 or 9
        _ => panic!("Nonexistent slice_letter passed: {:?}", slice_letter),
    }
}

/// Convert a bitstream to a bytestream and pad the bitstream with a chosen value
pub fn bitstream_to_bytestream<BS: AsRef<[u8]>>(bitstream: BS, padding_bit: u8) -> Vec<u8> {
    let bitstream = bitstream.as_ref();
    let mut res: Vec<u8> = Vec::new();

    let mut cur_byte: u8 = 0;
    for (i, bit) in bitstream.iter().enumerate() {
        let working_i = i % 8;
        if working_i == 0 && i != 0 {
            res.push(cur_byte);
            cur_byte = 0;
        }
        cur_byte |= bit << (7 - working_i);
    }

    if bitstream.len() % 8 != 0 {
        // need to add padding bit if 1
        if padding_bit == 1 {
            cur_byte |= 2u8.pow(8 - (bitstream.len() % 8) as u32) - 1;
        }
    }

    // if we get an empty input we'll provide an empty output to avoid confusion
    if !bitstream.is_empty() {
        res.push(cur_byte);
    }

    res
}

#[cfg(test)]
mod test {
    use super::*;
    use std::iter::FromIterator;

    #[test]
    fn test_more_data_simple() {
        let test_cases = vec![
            (
                ByteStream {
                    byte_offset: 4,
                    bytestream: vec![0xc8].into(),
                },
                false,
            ),
            (
                ByteStream {
                    byte_offset: 0,
                    bytestream: vec![].into(),
                },
                false,
            ),
            (
                ByteStream {
                    byte_offset: 7,
                    bytestream: vec![0xc8].into(),
                },
                false,
            ),
            (
                ByteStream {
                    byte_offset: 6,
                    bytestream: vec![0xc8].into(),
                },
                false,
            ),
            (
                ByteStream {
                    byte_offset: 0,
                    bytestream: vec![0xc8].into(),
                },
                true,
            ),
            (
                ByteStream {
                    byte_offset: 3,
                    bytestream: vec![0xc8].into(),
                },
                true,
            ),
            (
                ByteStream {
                    byte_offset: 4,
                    bytestream: vec![0xc9].into(),
                },
                true,
            ),
            (
                ByteStream {
                    byte_offset: 3,
                    bytestream: vec![0xc8, 0xc4].into(),
                },
                true,
            ),
            (
                ByteStream {
                    byte_offset: 7,
                    bytestream: vec![0xc8, 0xc8].into(),
                },
                true,
            ),
        ];

        for t in test_cases {
            println!("Expected: {} and got {} for test: ", t.1, t.0.more_data());
            t.0.print();
            assert_eq!(t.0.more_data(), t.1);
        }
    }

    #[test]
    fn test_read_bits_simple() {
        // test cases: (bits_to_read, bytestream, byte_offset)
        let test_cases = vec![
            (0, vec![1u8], 0),
            (1, vec![128u8], 0),
            (1, vec![1u8], 0),
            (2, vec![196u8], 0),
            (1, vec![64u8], 1),
            (2, vec![128u8], 0),
            (2, vec![1u8], 6),
            (0, vec![128u8], 3),
            (1, vec![1u8], 7),
            (9, vec![128u8, 128u8], 0),
            (9, vec![128u8, 128u8], 1),
        ];
        // expected outputs: (decoded, bytes_consumed, new_byte_offset)
        let results = vec![
            (0, 0, 0),
            (1, 0, 1),
            (0, 0, 1),
            (3, 0, 2),
            (1, 0, 2),
            (2, 0, 2),
            (1, 1, 0),
            (0, 0, 3),
            (1, 1, 0),
            (257, 1, 1),
            (2, 1, 2),
        ];

        for i in 0..test_cases.len() {
            let mut bs = ByteStream {
                bytestream: VecDeque::from_iter(test_cases[i].1.iter().cloned()),
                byte_offset: test_cases[i].2,
            };

            let prev_len = bs.bytestream.len();

            let r = bs.read_bits(test_cases[i].0);

            assert!(results[i].0 == r); // check the results
            assert!(results[i].1 == prev_len - bs.bytestream.len()); // check the consumption
            assert!(results[i].2 == bs.byte_offset); // check the consumption
        }
    }

    #[test]
    fn test_read_bits_simple_stream() {
        let mut bs = ByteStream {
            bytestream: vec![0x9F, 0xFF, 0xFF].into(),
            byte_offset: 0,
        };
        let results = vec![0, 1, 0, 7, 15, 31, 63];

        for i in 0..results.len() {
            let bits_to_read = i as u8;
            let prev_len = bs.bytestream.len();
            let r = bs.read_bits(bits_to_read);

            println!(
                "test_stream: {:?}, bits_to_read: {}, bytes_consumed: {}, byte_offset : {}",
                bs.bytestream,
                bits_to_read,
                prev_len - bs.bytestream.len(),
                bs.byte_offset
            );
            println!("Expected: {}\nReturned: {}\n-------", results[i], r);
            assert!(results[i] == r);
        }

        assert!(bs.byte_offset == 5);
    }

    #[test]
    fn test_bitstream_to_bytestream() {
        // (bitstream, bytestream)
        let test_cases = [
            (vec![0u8], vec![0u8], 0),
            (vec![1u8], vec![128u8], 0),
            (vec![0u8], vec![127u8], 1),
            (vec![1u8], vec![255u8], 1),
            (vec![1u8, 1u8, 1u8], vec![255u8], 1),
            (vec![1u8, 0u8, 1u8, 0u8], vec![160u8], 0),
            (vec![0u8, 1u8, 0u8, 1u8, 0u8], vec![80u8], 0),
            (vec![0u8, 0u8, 0u8, 0u8, 1u8, 0u8, 1u8, 0u8], vec![10u8], 0),
            (vec![1u8, 0u8, 0u8, 0u8, 1u8, 0u8, 1u8, 0u8], vec![138u8], 0),
            (
                vec![
                    1u8, 0u8, 0u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 0u8, 0u8, 1u8, 0u8, 1u8,
                ],
                vec![138u8, 138u8],
                0,
            ),
            (
                vec![
                    1u8, 0u8, 0u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 0u8, 0u8, 1u8, 0u8, 1u8, 0u8,
                ],
                vec![138u8, 138u8],
                0,
            ),
            (
                vec![
                    1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 0u8, 0u8, 0u8, 1u8, 0u8, 1u8, 0u8,
                    0u8,
                ],
                vec![255u8, 138u8, 0u8],
                0,
            ),
        ];

        for t in test_cases.iter() {
            let res = bitstream_to_bytestream(t.0.clone(), t.2);
            println!("Got: {:?}\nExpected: {:?}", res, t.1.clone());
            assert_eq!(res, t.1);
        }
    }
}
