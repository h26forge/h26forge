//! RNG management to sample from a seed or read from a file.
//!
//! FILM - Fuzzing Integration Layer for Mutation

use crate::common::helper::bitstream_to_bytestream;
use crate::encoder::binarization_functions::generate_fixed_length_value;
use rand::prelude::*;
use rand_pcg::Lcg128Xsl64;
use rand_pcg::Pcg64;
use std::fs::File;
use std::io::prelude::*;

/// Maintains the randomly sampled values
pub struct FilmStream {
    pub contents: Vec<u8>,     // Bytestream
    pub read_byte_offset: u32, // Current byte we're on
    pub read_bit_offset: u8,   // Current bit we're on
    pub write_bit_offset: u8,  // Offset for when writing bits
}

impl FilmStream {
    fn new() -> FilmStream {
        FilmStream {
            contents: Vec::new(),
            read_byte_offset: 0,
            read_bit_offset: 0,
            write_bit_offset: 0,
        }
    }

    fn append_bytes(&mut self, byte_stream: &[u8]) {
        // For random bytes we'll make it easy and just align
        self.write_bit_offset = 0;
        self.contents.extend(byte_stream);
    }

    fn append_bits(&mut self, bit_stream: &[u8]) {
        let mut bit_stream_copy = bit_stream.to_vec();

        if bit_stream.len() < 8 {
            if self.write_bit_offset == 0 {
                let single_byte = bitstream_to_bytestream(bit_stream_copy.clone(), 0);
                if !single_byte.is_empty() {
                    self.contents.push(single_byte[0]);
                    self.write_bit_offset += bit_stream_copy.len() as u8;
                }
            } else {
                let last_idx = self.contents.len() - 1;
                let mut last_byte = self.contents[last_idx];
                let mut overflowed = false;
                for b in bit_stream_copy {
                    last_byte |= b << (7 - self.write_bit_offset);
                    self.write_bit_offset += 1;
                    if self.write_bit_offset % 8 == 0 {
                        self.write_bit_offset = 0;
                        self.contents[last_idx] = last_byte; // overwrite the last byte
                                                             //self.contents.push(0); // this is the new byte
                        last_byte = 0;
                        overflowed = true;
                    }
                }
                if self.write_bit_offset > 0 {
                    if overflowed {
                        self.contents.push(last_byte)
                    } else {
                        self.contents[last_idx] = last_byte;
                    }
                }
            }
        } else {
            if self.write_bit_offset != 0 {
                let last_idx = self.contents.len() - 1;
                let mut last_byte = self.contents[last_idx];
                while self.write_bit_offset % 8 != 0 {
                    last_byte |= bit_stream_copy.remove(0) << (7 - self.write_bit_offset);
                    self.write_bit_offset += 1;
                }
                self.contents[last_idx] = last_byte;
            }
            let byte_stream = bitstream_to_bytestream(bit_stream_copy.clone(), 0);

            self.contents.extend(byte_stream);
            self.write_bit_offset = (bit_stream_copy.len() as u8) % 8;
        }
    }

    fn read_bytes(&mut self, size: usize) -> Option<Vec<u8>> {
        // Align byte reading values
        self.read_bit_offset = 0;
        self.read_byte_offset += 1;

        // do not read past what is available
        if self.read_byte_offset >= self.contents.len() as u32 {
            return None;
        }

        // if not enough bytes then return
        if self.read_byte_offset + size as u32 >= self.contents.len() as u32 {
            return None;
        }

        let start = self.read_byte_offset as usize;
        let end = start + size;
        let res = (&self.contents[start..end]).to_vec();

        self.read_byte_offset += size as u32;
        Some(res)
    }

    fn read_bits(&mut self, size: u32) -> Option<u32> {
        let mut res = 0;

        // do not read past what is available
        if self.read_byte_offset >= self.contents.len() as u32 {
            return None;
        }

        for i in (0..size).rev() {
            let intermediate: u32 = (((self.contents[self.read_byte_offset as usize]
                & (1 << (7 - self.read_bit_offset))) as u32)
                >> (7 - self.read_bit_offset))
                << i;

            res |= intermediate;

            self.read_bit_offset += 1;
            if self.read_bit_offset >= 8 {
                self.read_bit_offset = 0;
                self.read_byte_offset += 1;

                if self.read_byte_offset as usize >= self.contents.len() {
                    return Some(res);
                }
            }
        }

        Some(res)
    }
}

/// Maintains the randomness source state
pub struct FilmState {
    pub use_film_file: bool,
    pub seed: u64,
    pub rng: Lcg128Xsl64,
    pub film_file_contents: FilmStream,
}

impl FilmState {
    pub fn setup_film_from_seed(seed: u64) -> FilmState {
        let rng: Lcg128Xsl64 = Pcg64::seed_from_u64(seed);

        FilmState {
            use_film_file: false,
            seed,
            rng,
            film_file_contents: FilmStream::new(),
        }
    }

    pub fn setup_film() -> FilmState {
        let mut rng_seed = rand::thread_rng();
        let seed = rng_seed.gen::<u64>();

        FilmState::setup_film_from_seed(seed)
    }

    pub fn setup_film_from_file(filename: &str) -> FilmState {
        let mut film_file = match File::open(filename) {
            Err(_) => panic!("couldn't open {}", filename),
            Ok(file) => file,
        };

        let mut film_file_contents = Vec::new();

        film_file
            .read_to_end(&mut film_file_contents)
            .expect("Unable to read data");

        // set this up as backup
        let mut rng_seed = rand::thread_rng();
        let seed = rng_seed.gen::<u64>();
        let rng: Lcg128Xsl64 = Pcg64::seed_from_u64(seed);

        FilmState {
            use_film_file: true,
            seed,
            rng,
            film_file_contents: FilmStream {
                contents: film_file_contents,
                read_byte_offset: 0,
                read_bit_offset: 0,
                write_bit_offset: 0,
            },
        }
    }

    pub fn setup_film_from_file_and_seed(filename: &str, seed: u64) -> FilmState {
        let mut film_file = match File::open(filename) {
            Err(_) => panic!("couldn't open {}", filename),
            Ok(file) => file,
        };

        let mut film_file_contents = Vec::new();

        film_file
            .read_to_end(&mut film_file_contents)
            .expect("Unable to read data");

        let rng: Lcg128Xsl64 = Pcg64::seed_from_u64(seed);

        FilmState {
            use_film_file: true,
            seed,
            rng,
            film_file_contents: FilmStream {
                contents: film_file_contents,
                read_byte_offset: 0,
                read_bit_offset: 0,
                write_bit_offset: 0,
            },
        }
    }

    /// Returns a u32 value from [min, max], inclusive
    pub fn read_film_u32(&mut self, min: u32, max: u32) -> u32 {
        let mut bit_size = ((max as f64) + 1f64).log2().ceil() as usize;

        if bit_size == 0 {
            bit_size += 1;
        }

        if self.use_film_file {
            let val = self.film_file_contents.read_bits(bit_size as u32);
            match val {
                Some(x) => {
                    // do not let the value go past the max.
                    // cast to u64 to avoid overflow because max may be u32::max
                    return (x as u64 % (max as u64 + 1u64)) as u32;
                }
                _ => {
                    println!(
                        "[WARNING] Issue reading film file; falling back to RNG with seed {}",
                        self.seed
                    );
                    self.use_film_file = false;
                }
            };
        }

        // to make the min and max inclusive, we add 1 here to the max;
        // when std::u32::MAX is used, we'll get an overflow so we transform it
        // to u64 just for the sampling
        let val = self.rng.gen_range(min as u64..=max as u64) as u32;

        let binarized = generate_fixed_length_value(val, bit_size);

        self.film_file_contents.append_bits(&binarized);

        val
    }

    /// Returns an i32 value from [min, max], inclusive
    pub fn read_film_i32(&mut self, min: i32, max: i32) -> i32 {
        let mut bit_size = ((max as f64) - (min as f64)).log2().ceil() as usize;

        if bit_size == 0 {
            bit_size += 1;
            println!("read_film_i32 - bit_size is 0 -- setting to 1");
        }

        if self.use_film_file {
            let sign_bit_wrap = self.film_file_contents.read_bits(1);

            match sign_bit_wrap {
                Some(x) => {
                    let sign_bit = 1 - 2 * x as i32;

                    let val = self.film_file_contents.read_bits(bit_size as u32);

                    match val {
                        Some(y) => {
                            return sign_bit * ( (y % (i32::MAX as u32)) as i32);
                        }
                        _ => {
                            println!("[WARNING] Issue reading film file; falling back to RNG with seed {}", self.seed);
                            self.use_film_file = false;
                        }
                    };
                }
                _ => {
                    println!(
                        "[WARNING] Issue reading film file; falling back to RNG with seed {}",
                        self.seed
                    );
                    self.use_film_file = false;
                }
            };
        }
        // min to max inclusive
        let val = self.rng.gen_range(min as i64..=max as i64) as i32;

        let mut binarized = vec![];
        // we store the sign bit as the first bit
        let sign_bit = match val < 0 {
            true => 1,
            false => 0,
        };

        binarized.push(sign_bit);
        binarized.append(&mut generate_fixed_length_value(val.abs() as u32, bit_size));

        self.film_file_contents.append_bits(&binarized);

        val
    }

    /// Sample from a range and return true if equal to or passed threshold
    pub fn read_film_bool(&mut self, min: u32, max: u32, threshold: u32) -> bool {
        let val = self.read_film_u32(min, max + 1);
        val >= threshold
    }

    /// Sample a random sequence of bytes of length `length`
    pub fn read_film_bytes(&mut self, length: u32) -> Vec<u8> {
        if self.use_film_file {
            let res = self.film_file_contents.read_bytes(length as usize);
            match res {
                Some(v) => v,
                None => {
                    self.use_film_file = false;
                    let random_bytes: Vec<u8> =
                        (0..length).map(|_| self.rng.next_u32() as u8).collect();

                    self.film_file_contents.append_bytes(&random_bytes);

                    random_bytes
                }
            }
        } else {
            let random_bytes: Vec<u8> = (0..length).map(|_| self.rng.next_u32() as u8).collect();

            self.film_file_contents.append_bytes(&random_bytes);

            random_bytes
        }
    }

    /// Save the collected randomness to a file
    pub fn save_film(&self, filename_prepend: &str) {
        let output_filename = format!("{}.film_file.seed_{}.bin", filename_prepend, self.seed);

        let mut f = match File::create(output_filename.as_str()) {
            Err(_) => panic!("couldn't open {}", output_filename.as_str()),
            Ok(file) => file,
        };

        match f.write_all(self.film_file_contents.contents.as_slice()) {
            Err(_) => panic!("couldn't write to file {}", output_filename.as_str()),
            Ok(()) => (),
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_filmstream_append_bits() {
        // test cases: (contents, write_bit_offset, bit_stream_to_add)
        let test_cases = vec![
            (vec![128u8], 1, vec![1]),
            (vec![128u8], 4, vec![0, 1, 1, 1]),
            (vec![1u8], 0, vec![0, 1, 1, 1, 0, 1]),
            (vec![196u8], 0, vec![0, 1, 1, 1, 0, 1]),
            (vec![64u8], 2, vec![0, 1, 1, 1, 0, 1, 1]),
            (
                vec![128u8],
                0,
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            ),
            (
                vec![128u8],
                0,
                vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            ),
            (vec![1u8, 2, 0], 6, vec![0, 1, 1, 1, 0, 1]),
        ];

        // expected outputs: (appended_byte_stream, new_write_offset)
        let results = vec![
            (vec![192u8], 2),
            (vec![135], 0),
            (vec![1, 116], 6),
            (vec![196, 116], 6),
            (vec![93, 128], 1),
            (vec![128, 255, 255], 0),
            (vec![128, 255, 255, 128], 1),
            (vec![1u8, 2, 1, 208], 4),
        ];

        for i in 0..test_cases.len() {
            let mut bs = FilmStream {
                contents: test_cases[i].0.clone(),
                read_byte_offset: 0,
                read_bit_offset: 0,
                write_bit_offset: test_cases[i].1,
            };
            bs.append_bits(&test_cases[i].2);

            assert!(results[i].0 == bs.contents); // check the results
            assert!(results[i].1 == bs.write_bit_offset);
        }
    }

    #[test]
    fn test_filmstream_read_bits() {
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
        let expected_results = vec![
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
            let mut bs = FilmStream {
                contents: test_cases[i].1.clone(),
                read_byte_offset: 0,
                read_bit_offset: test_cases[i].2,
                write_bit_offset: 0,
            };

            let result = bs.read_bits(test_cases[i].0);
            let r = match result {
                Some(x) => x,
                _ => panic!(
                    "Error with test: test_case - {:?} ; result - {:?}; expected result - {:?}",
                    test_cases[i], result, expected_results[i]
                ),
            };

            assert!(expected_results[i].0 == r); // check the results
            assert!(expected_results[i].1 == bs.read_byte_offset); // check the consumption
            assert!(expected_results[i].2 == bs.read_bit_offset); // check the consumption
        }
    }
}
