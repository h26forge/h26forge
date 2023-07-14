//! Exp-golomb entropy decoding.

use crate::common::helper::ByteStream;

/// Exp-Golomb decode -- returns an option
pub fn exp_golomb_decode_one(mut bs: &mut ByteStream, signed: bool, k: u8) -> Option<i32> {
    if bs.byte_offset > 7 {
        return None;
    }

    let mut zerocount: u8 = 0;

    let mut foundstart = false;

    while !foundstart {
        let start_offset = bs.byte_offset;
        for i in (0..(8 - start_offset)).rev() {
            if ((bs.bytestream[0] >> i) & 1u8) == 0u8 {
                zerocount += 1;
            } else {
                foundstart = true;
                break;
            }
            bs.byte_offset += 1;
        }
        if !foundstart {
            // if we get here, we're going off to the next byte
            bs.bytestream.pop_front();
            bs.byte_offset = 0; // we only care about start_offset at the first byte
        }
    }

    // If it's 0s all the way, then we have an error
    if !foundstart {
        return None;
    }

    // add 1 to zerocount and that's how many bits we have to read
    zerocount += 1;

    let mut res: i32 = bs.read_bits(zerocount) as i32; // we got the top most bit now

    // res has the raw number - we need to determine signed or not

    res -= 1;

    if signed {
        if res % 2 == 0 {
            res /= -2;
        } else {
            res = (res + 1) / 2;
        }
    } else {
        //kth order whenever it's unsigned
        res = (res << k) + (bs.read_bits(k) as i32);
    }

    Some(res)
}

/// Exp-Golomb decode -- calls `exp_golomb_decode_one`
pub fn exp_golomb_decode_one_wrapper(bs: &mut ByteStream, signed: bool, k: u8) -> i32 {
    let r = exp_golomb_decode_one(bs, signed, k);
    match r {
        Some(x) => x,
        _ => panic!("Error with exp_golomb decoding"),
    }
}

/// Exp-Golomb decode from a series of *bits*. Used in uegk() binarization.
///
/// This exp_golomb decode function takes in a series of bits and tries to do
/// an exponential decoding on it. If it's not successful it will return a None type.
///
/// bs:       bitstream to decode
/// signed:   whether our return value is signed or not
/// k:        the order of the exp-golomb
/// reversed: whether 0s and 1s are flipped - used in UEGk
///
pub fn exp_golomb_decode_no_stream(bs: &[u8], signed: bool, k: u8, reversed: bool) -> Option<i32> {
    let mut zerocount: u8 = 0; // also counts as the index to look for
    let mut foundstart = false;

    if reversed {
        for bit in bs.iter().copied() {
            if bit == 1u8 {
                zerocount += 1;
            } else {
                foundstart = true;
                break;
            }
        }
    } else {
        for bit in bs.iter().copied() {
            if bit == 0u8 {
                zerocount += 1;
            } else {
                foundstart = true;
                break;
            }
        }
    }

    // If it's 0s all the way, then we have an error
    if !foundstart {
        return None;
    }

    // add 1 to zerocount and that's how many bits we have to read
    zerocount += 1;
    if bs.len() < (2 * (zerocount as usize) - 1 + (k as usize)) {
        return None;
    }

    // if we're reversed, then the first bit we read will be 0. this guarantees the MSB is set
    let mut res: i32 = match reversed {
        true => 1,
        false => 0,
    };

    for i in zerocount - 1..2 * zerocount - 1 {
        res |= bs[i as usize] as i32;
        res <<= 1;
    }
    res >>= 1;

    res -= 1;

    // do kth order
    let d = (2 * zerocount - 1) as usize;
    for i in d..d + (k as usize) {
        res <<= 1;
        res |= bs[i] as i32;
    }

    // res has the raw number - we need to determine signed or not

    if signed {
        if res % 2 == 0 {
            res /= -2;
        } else {
            res = (res + 1) / 2;
        }
    }

    Some(res)
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::iter::FromIterator;

    use super::*;

    #[test]
    fn test_exp_golomb_decode_one_simple_unsigned() {
        // (integer, exp-golomb encoded)
        let test_cases = [(0i32, vec![128u8]), (1i32, vec![64u8]), (2i32, vec![96u8])];

        for r in test_cases.iter() {
            let mut bs = ByteStream {
                bytestream: VecDeque::from_iter(r.1.iter().copied()),
                byte_offset: 0,
            };
            println!("ByteStream before: {:?}", bs);
            let result = exp_golomb_decode_one(&mut bs, false, 0);
            println!("Expected: {}; Got: {:?}", r.0, result);
            println!("ByteStream after: {:?}", bs);

            match result {
                Some(x) => assert!(r.0 == x),
                _ => panic!("error in implementation"),
            }
        }
    }

    #[test]
    fn test_exp_golomb_decode_one_simple_unsigned_diff_start_offset() {
        // (integer, exp-golomb encoded, start_offset)
        let test_cases = [
            (0i32, vec![64u8], 1),
            (0i32, vec![32u8], 2),
            (0i32, vec![16u8], 3),
            (0i32, vec![8u8], 4),
            (0i32, vec![4u8], 5),
            (0i32, vec![2u8], 6),
            (0i32, vec![1u8], 7),
            (1i32, vec![32u8], 1),
            (2i32, vec![48u8], 1),
        ];

        for r in test_cases.iter() {
            let mut bs = ByteStream {
                bytestream: VecDeque::from_iter(r.1.iter().copied()),
                byte_offset: r.2,
            };
            println!("ByteStream before: {:?}", bs);
            let result = exp_golomb_decode_one(&mut bs, false, 0);
            println!("Expected: {}; Got: {:?}", r.0, result);
            println!("ByteStream after: {:?}", bs);
            match result {
                Some(x) => assert!(r.0 == x),
                _ => panic!("error in implementation"),
            }
        }
    }

    #[test]
    fn test_exp_golomb_decode_one_simple_single_stream() {
        // 10 0s
        let test_cases: [i32; 10] = [0; 10];
        let mut bs = ByteStream {
            bytestream: vec![255u8, 196u8].into(),
            byte_offset: 0,
        }; // 10 1s

        for r in test_cases.iter() {
            println!("ByteStream before: {:?}", bs);
            let result = exp_golomb_decode_one(&mut bs, false, 0);

            println!("Expected: {}; Got: {:?}", r, result);
            println!("ByteStream after: {:?}", bs);

            match result {
                Some(x) => assert!(r == &x),
                _ => panic!("error in implementation"),
            }
        }
    }

    #[test]
    fn test_exp_golomb_decode_one_complex_single_stream() {
        let test_cases: [i32; 10] = [0, 1, 22, 25, 84, 84, 129, 167, 255, 510];
        let mut bs = ByteStream {
            bytestream: vec![
                0xA0u8, 0xB8, 0x68, 0x0A, 0xA0, 0x55, 0x01, 0x04, 0x02, 0xA0, 0x02, 0x00, 0x01,
                0xFF,
            ]
            .into(),
            byte_offset: 0,
        };

        for r in test_cases.iter() {
            println!("ByteStream before: {:?}", bs);
            let result = exp_golomb_decode_one(&mut bs, false, 0);
            println!("Expected: {}; Got: {:?}", r, result);
            println!("ByteStream after: {:?}", bs);
            match result {
                Some(x) => assert!(r == &x),
                _ => panic!("error in implementation"),
            }
        }
    }

    #[test]
    fn test_exp_golomb_decode_one_complex_unsigned() {
        // (integer, exp-golomb encoded)
        let test_cases = [
            (22i32, vec![11u8, 128u8]),
            (25i32, vec![13u8, 0u8]),
            (84i32, vec![2u8, 168u8]),
            (84i32, vec![2u8, 169u8]), // last 3 digits should be ignored
            (129i32, vec![1u8, 4u8]),
            (167i32, vec![1u8, 80u8]),
            (255i32, vec![0u8, 128u8, 0u8]),
            (510i32, vec![0u8, 255u8, 128u8]),
        ];

        for r in test_cases.iter() {
            let mut bs = ByteStream {
                bytestream: VecDeque::from_iter(r.1.iter().copied()),
                byte_offset: 0,
            };
            println!("ByteStream before: {:?}", bs);
            let result = exp_golomb_decode_one(&mut bs, false, 0);
            println!("Expected: {}; Got: {:?}", r.0, result);
            println!("ByteStream after: {:?}", bs);

            match result {
                Some(x) => assert!(r.0 == x),
                _ => panic!("error in implementation"),
            }
        }
    }

    #[test]
    fn test_exp_golomb_decode_one_simple_signed() {
        // (integer, exp-golomb encoded)
        let test_cases = [
            (0i32, vec![128u8]),
            (1i32, vec![64u8]),
            (-1i32, vec![96u8]),
            (2i32, vec![32u8]),
            (-2i32, vec![40u8]),
            (3i32, vec![48u8]),
            (-3i32, vec![56u8]),
        ];

        for r in test_cases.iter() {
            let mut bs = ByteStream {
                bytestream: VecDeque::from_iter(r.1.iter().copied()),
                byte_offset: 0,
            };
            println!("ByteStream before: {:?}", bs);
            let result = exp_golomb_decode_one(&mut bs, true, 0);
            println!("Expected: {}; Got: {:?}", r.0, result);
            println!("ByteStream after: {:?}", bs);
            match result {
                Some(x) => assert!(r.0 == x),
                _ => panic!("error in implementation"),
            }
        }
    }

    #[test]
    fn test_exp_golomb_decode_one_simple_combined_signed() {
        // (integer, exp-golomb encoded, signed)
        let test_cases = [
            (0i32, vec![128u8], true),
            (1i32, vec![64u8], true),
            (-1i32, vec![96u8], true),
            (2i32, vec![96u8], false),
            (-2i32, vec![40u8], true),
            (3i32, vec![48u8], true),
            (-3i32, vec![56u8], true),
        ];

        for r in test_cases.iter() {
            let mut bs = ByteStream {
                bytestream: VecDeque::from_iter(r.1.iter().copied()),
                byte_offset: 0,
            };
            println!("ByteStream before: {:?}", bs);
            let result = exp_golomb_decode_one(&mut bs, r.2, 0);
            println!("Expected: {}; Got: {:?}", r.0, result);
            println!("ByteStream after: {:?}", bs);
            match result {
                Some(x) => assert!(r.0 == x),
                _ => panic!("error in implementation"),
            }
        }
    }

    #[test]
    fn test_exp_golomb_decode_one_complex_signed() {
        // (integer, exp-golomb encoded)
        let test_cases = [
            (0i32, vec![128u8]),
            (1i32, vec![64u8]),
            (-1i32, vec![96u8]),
            (2i32, vec![32u8]),
            (-2i32, vec![40u8]),
            (3i32, vec![48u8]),
            (-3i32, vec![56u8]),
        ];

        for r in test_cases.iter() {
            let mut bs = ByteStream {
                bytestream: VecDeque::from_iter(r.1.iter().copied()),
                byte_offset: 0,
            };
            println!("ByteStream before: {:?}", bs);
            let result = exp_golomb_decode_one(&mut bs, true, 0);
            println!("Expected: {}; Got: {:?}", r.0, result);
            println!("ByteStream after: {:?}", bs);

            match result {
                Some(x) => assert!(r.0 == x),
                _ => panic!("error in implementation"),
            }
        }
    }

    #[test]
    fn test_exp_golomb_decode_one_simple_variable_k() {
        // (integer, exp-golomb encoded, k)
        let test_cases = [
            (0i32, vec![128u8], 1),
            (0i32, vec![160u8], 1),
            (0i32, vec![128u8], 2),
            (0i32, vec![129u8], 2),
            (0i32, vec![128u8], 3),
            (0i32, vec![129u8], 4),
            (1i32, vec![196u8], 1),
            (1i32, vec![160u8], 2),
            (1i32, vec![144u8], 3),
            (2i32, vec![64u8], 1),
            (2i32, vec![196u8], 2),
            (2i32, vec![160u8], 3),
            (23i32, vec![124u8], 3),
            (11i32, vec![120u8], 2),
        ];

        for r in test_cases.iter() {
            let mut bs = ByteStream::new(r.1.clone());
            println!("ByteStream before: {:?}", bs);
            let result = exp_golomb_decode_one(&mut bs, false, r.2);
            println!("Expected: {}; Got: {:?}", r.0, result);
            println!("ByteStream after: {:?}", bs);

            match result {
                Some(x) => assert!(r.0 == x),
                _ => panic!("error in implementation"),
            }
        }
    }

    #[test]
    fn test_exp_golomb_decode_no_stream_simple() {
        // (integer, binary stream to decode, signed)
        let test_cases = [
            (0i32, vec![1u8], false),
            (1i32, vec![0u8, 1, 0], false),
            (2i32, vec![0u8, 1, 1], false),
            (3i32, vec![0u8, 0, 1, 0, 0], false),
            (4i32, vec![0u8, 0, 1, 0, 1], false),
            (5i32, vec![0u8, 0, 1, 1, 0], false),
            (0i32, vec![1u8], true),
            (1i32, vec![0u8, 1, 0], true),
            (-1i32, vec![0u8, 1, 1], true),
            (2i32, vec![0u8, 0, 1, 0, 0], true),
            (-2i32, vec![0u8, 0, 1, 0, 1], true),
            (3i32, vec![0u8, 0, 1, 1, 0], true),
        ];

        for t in test_cases.iter() {
            let result = exp_golomb_decode_no_stream(&t.1, t.2, 0, false);
            match result {
                Some(x) => {
                    println!("Expected: {}, got: {}", t.0, x);
                    assert!(x == t.0);
                }
                _ => panic!("error in implementation"),
            }
        }
    }

    #[test]
    fn test_exp_golomb_decode_no_stream_simple_k() {
        // (integer, binary stream to decode, k value)
        let test_cases = [
            (0i32, vec![1u8], 0),
            (1i32, vec![1u8, 1], 1),
            (2i32, vec![1u8, 1, 0], 2),
            (0i32, vec![1u8, 0, 0, 0], 3),
            (1i32, vec![1u8, 0, 0, 1], 3),
            (2i32, vec![1u8, 0, 1, 0], 3),
            (13i32, vec![0u8, 1, 0, 1, 0, 1], 3),
            (24i32, vec![0u8, 0, 1, 0, 0, 0, 0, 0], 3),
            (51i32, vec![0u8, 0, 1, 1, 1, 0, 1, 1], 3),
        ];

        for t in test_cases.iter() {
            let result = exp_golomb_decode_no_stream(&t.1, false, t.2, false);
            match result {
                Some(x) => {
                    println!("Expected: {}, got: {}", t.0, x);
                    assert!(x == t.0);
                }
                _ => panic!("error in implementation"),
            }
        }
    }
}
