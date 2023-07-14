//! Binarization decoding functions.
//!
//! Entropy encoding function definitions
//! - ae(v): context adaptive arithmetic entropy-coded syntax element
//! - b(8):  byte having any pattern of bit string (8 bits)
//! - ce(v): context adaptive variable-length entropy-coded
//!          syntax element with the left bit first
//! - f(n):  fixed-pattern bit string using n bits written
//!          (from left to right) with the left bit first
//! - i(n):  signed integer using n bits
//! - me(v): mapped Exp-Golomb-coded syntax element with the left bit first
//! - se(v): signed integer Exp-Golomb-coded syntax element with the left bit first
//! - te(v): truncated Exp-Golomb-coded syntax element with the left bit first
//! - u(n):  unsigned integer using n bits. If "v" is used, then the number of bits varies
//!          depending on the value of other syntax elements
//! - ue(v): unsigned integer Exp-Golomb-coded syntax element with the left bit first

use crate::common::helper::is_slice_type;
use crate::decoder::expgolomb::exp_golomb_decode_no_stream;

/// Reads a unary value with 0 as the ending character
pub fn read_unary_value(bs: &[u8]) -> Option<i32> {
    if bs.is_empty() {
        panic!("read_unary_value - empty bitstream value!");
    }

    let mut cur_bit = bs[0];

    let mut cur_val: i32 = 0;

    while cur_bit != 0 {
        cur_val += 1;
        if cur_val >= bs.len() as i32 {
            return None;
        }
        cur_bit = bs[cur_val as usize];
    }

    Some(cur_val)
}

/// Reads a unary value up to max_val
pub fn read_truncated_unary_value(max_val: u32, bs: &[u8]) -> Option<i32> {
    if bs.is_empty() {
        panic!("read_truncated_unary_value - empty bitstream value!");
    }

    let mut cur_bit = bs[0];
    let mut cur_val: i32 = 0;

    while cur_bit != 0 {
        cur_val += 1;
        if cur_val < max_val as i32 {
            if cur_val >= bs.len() as i32 {
                return None;
            }
            cur_bit = bs[cur_val as usize];
        } else {
            cur_val = 2i32.pow(max_val) - 1; // return the string of all 1s of length max_val
            break;
        }
    }

    Some(cur_val)
}

/// Read concatenated unary/k-th order Exp-Golomb value
pub fn read_uegk(signed_val_flag: bool, ucoff: u32, k: u8, bs: &[u8]) -> Option<i32> {
    // UEGk bin string is a concatenation of a prefix bit string and suffix bit string
    let prefix: i32;
    match read_truncated_unary_value(ucoff, bs) {
        Some(x) => prefix = x as i32,
        _ => return None,
    }
    if (!signed_val_flag && prefix != 2i32.pow(ucoff) - 1) || (signed_val_flag && prefix == 0) {
        // only prefix bit required
        Some(prefix)
    } else {
        let res = exp_golomb_decode_no_stream(&bs[ucoff as usize..], signed_val_flag, k, true);
        res.map(|x| ucoff as i32 + x)
    }
}

/// Specified in section 9.3.2.5
pub fn read_mb_types(bs: &[u8], slice_type: u8) -> Option<i32> {
    if bs.is_empty() {
        panic!("read_mb_types - empty bitstream value!");
    }

    // Table 9-37
    if is_slice_type(slice_type, "B") {
        let b0 = bs[0];
        if b0 == 0 {
            return Some(0);
        } else {
            if bs.len() < 2 {
                return None;
            }
            let b1 = bs[1];
            if b1 == 0 {
                if bs.len() < 3 {
                    return None;
                }

                let b2 = bs[2];
                if b2 == 0 {
                    return Some(1);
                } else {
                    return Some(2);
                }
            } else {
                // 11
                if bs.len() < 3 {
                    return None;
                }
                let b2 = bs[2];

                if b2 == 0 {
                    // 110
                    if bs.len() < 6 {
                        return None;
                    }

                    let b35 = (bs[3] << 2) | (bs[4] << 1) | bs[5];
                    match b35 {
                        0 => return Some(3),
                        1 => return Some(4),
                        2 => return Some(5),
                        3 => return Some(6),
                        4 => return Some(7),
                        5 => return Some(8),
                        6 => return Some(9),
                        7 => return Some(10),
                        _ => panic!("read_mb_types - incorrect pattern for B slice {:?}", bs),
                    }
                } else {
                    // 111
                    if bs.len() < 6 {
                        return None;
                    }

                    let b35 = (bs[3] << 2) | (bs[4] << 1) | bs[5];
                    match b35 {
                        6 => {
                            // 111110
                            return Some(11);
                        }
                        0 => {
                            // 111000
                            if bs.len() < 7 {
                                return None;
                            }
                            let b6 = bs[6];
                            if b6 == 0 {
                                return Some(12);
                            } else {
                                return Some(13);
                            }
                        }
                        1 => {
                            // 111001
                            if bs.len() < 7 {
                                return None;
                            }
                            let b6 = bs[6];
                            if b6 == 0 {
                                return Some(14);
                            } else {
                                return Some(15);
                            }
                        }
                        2 => {
                            // 111010
                            if bs.len() < 7 {
                                return None;
                            }
                            let b6 = bs[6];
                            if b6 == 0 {
                                return Some(16);
                            } else {
                                return Some(17);
                            }
                        }
                        3 => {
                            // 111011
                            if bs.len() < 7 {
                                return None;
                            }
                            let b6 = bs[6];
                            if b6 == 0 {
                                return Some(18);
                            } else {
                                return Some(19);
                            }
                        }
                        4 => {
                            // 111100
                            if bs.len() < 7 {
                                return None;
                            }
                            let b6 = bs[6];
                            if b6 == 0 {
                                return Some(20);
                            } else {
                                return Some(21);
                            }
                        }
                        7 => {
                            return Some(22);
                        }
                        5 => {
                            return Some(100); // this signals that we need to decode the prefix for intra mode
                        }
                        _ => panic!("read_mb_types - incorrect pattern for B slice {:?}", bs),
                    }
                }
            }
        }
    }

    if is_slice_type(slice_type, "P") || is_slice_type(slice_type, "SP") {
        if bs.len() < 2 {
            return None;
        } else {
            let b01 = (bs[0] << 1) | bs[1];
            match b01 {
                0 => return Some(0),
                1 => return Some(3),
                2 => return Some(2),
                3 => return Some(1),
                _ => panic!("read_mb_types - incorrect pattern for P slice {:?}", bs),
            }
        }
    }

    // specified by table 9-36
    if is_slice_type(slice_type, "I") {
        let b0 = bs[0];
        if b0 == 0 {
            return Some(0); // (INxN)
        } else {
            if bs.len() < 2 {
                return None;
            }
            let b1 = bs[1];
            if b1 == 1 {
                return Some(25); // (IPCM)
            } else {
                if bs.len() < 6 {
                    return None;
                }
                // 10
                let b25 = bs[2] << 3 | bs[3] << 2 | bs[4] << 1 | bs[5];
                match b25 {
                    0 => return Some(1),
                    1 => return Some(2),
                    2 => return Some(3),
                    3 => return Some(4),
                    4 => {
                        if bs.len() < 7 {
                            return None;
                        }
                        let b6 = bs[6];
                        if b6 == 0 {
                            return Some(5);
                        } else {
                            return Some(6);
                        }
                    }
                    5 => {
                        if bs.len() < 7 {
                            return None;
                        }
                        let b6 = bs[6];
                        if b6 == 0 {
                            return Some(7);
                        } else {
                            return Some(8);
                        }
                    }
                    6 => {
                        if bs.len() < 7 {
                            return None;
                        }
                        let b6 = bs[6];
                        if b6 == 0 {
                            return Some(9);
                        } else {
                            return Some(10);
                        }
                    }
                    7 => {
                        if bs.len() < 7 {
                            return None;
                        }
                        let b6 = bs[6];
                        if b6 == 0 {
                            return Some(11);
                        } else {
                            return Some(12);
                        }
                    }
                    8 => return Some(13),
                    9 => return Some(14),
                    10 => return Some(15),
                    11 => return Some(16),
                    12 => {
                        if bs.len() < 7 {
                            return None;
                        }
                        let b6 = bs[6];
                        if b6 == 0 {
                            return Some(17);
                        } else {
                            return Some(18);
                        }
                    }
                    13 => {
                        if bs.len() < 7 {
                            return None;
                        }
                        let b6 = bs[6];
                        if b6 == 0 {
                            return Some(19);
                        } else {
                            return Some(20);
                        }
                    }
                    14 => {
                        if bs.len() < 7 {
                            return None;
                        }
                        let b6 = bs[6];
                        if b6 == 0 {
                            return Some(21);
                        } else {
                            return Some(22);
                        }
                    }
                    15 => {
                        if bs.len() < 7 {
                            return None;
                        }
                        let b6 = bs[6];
                        if b6 == 0 {
                            return Some(23);
                        } else {
                            return Some(24);
                        }
                    }
                    _ => panic!("read_mb_types - failure with read_bits, got {}", b25),
                };
            }
        }
    }
    Some(100) // 100 is not any MB type
}

/// Specified in section 9.3.2.5 Table 9-38
pub fn read_sub_mb_types(bs: &[u8], slice_type: u8) -> Option<i32> {
    if bs.is_empty() {
        panic!("read_sub_mb_types - empty bitstream value!");
    }

    if is_slice_type(slice_type, "P") || is_slice_type(slice_type, "SP") {
        let b0 = bs[0];
        if b0 == 1 {
            Some(0) // PL08x8
        } else {
            if bs.len() < 2 {
                return None;
            }
            let b1 = bs[1];
            if b1 == 0 {
                Some(1) // PL08x4
            } else {
                if bs.len() < 3 {
                    return None;
                }
                let b2 = bs[2];
                if b2 == 1 {
                    Some(2) // PL04x8
                } else {
                    Some(3) // PL04x4
                }
            }
        }
    } else if is_slice_type(slice_type, "B") {
        let b0 = bs[0];
        if b0 == 0 {
            Some(0) // BDirect8x8
        } else {
            if bs.len() < 3 {
                return None;
            }
            let b12 = (bs[1] << 1) | bs[2];
            if b12 == 0 {
                Some(1) // BL08x8
            } else if b12 == 1 {
                Some(2) // BL18x8
            } else {
                if bs.len() < 5 {
                    return None;
                }
                if b12 == 2 {
                    let b34 = (bs[3] << 1) | bs[4];

                    if b34 == 0 {
                        Some(3) // BBi8x8
                    } else if b34 == 1 {
                        Some(4) // BL08x4
                    } else if b34 == 2 {
                        Some(5) // BL04x8
                    } else {
                        // b34 == 3
                        Some(6) // BL18x4
                    }
                } else {
                    // b12 == 3
                    let b3 = bs[3];
                    if b3 == 0 {
                        if bs.len() < 6 {
                            return None;
                        }

                        let b45 = (bs[4] << 1) | bs[5];
                        if b45 == 0 {
                            Some(7) // BL14x8
                        } else if b45 == 1 {
                            Some(8) // BBi8x4
                        } else if b45 == 2 {
                            Some(9) // BBi4x8
                        } else {
                            Some(10) // BL04x4
                        }
                    } else {
                        let b4 = bs[4];
                        if b4 == 0 {
                            Some(11) // BL14x4
                        } else {
                            Some(12) // BBi4x4
                        }
                    }
                }
            }
        }
    } else {
        panic!(
            "read_sub_mb_types - Incorrect slice_type provided: {:?}",
            slice_type
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_unary_value_simple() {
        let test_cases = [
            (0i32, vec![0u8]),
            (1, vec![1u8, 0]),
            (2, vec![1u8, 1, 0]),
            (8, vec![1u8, 1, 1, 1, 1, 1, 1, 1, 0]),
            (9, vec![1u8, 1, 1, 1, 1, 1, 1, 1, 1, 0]),
        ];

        for t in test_cases.iter() {
            let res = read_unary_value(&t.1);
            println!("Expected: {}; Got: {:?}", t.0, res);
            match res {
                Some(x) => assert!(t.0 == x),
                _ => panic!("Error in parsing"),
            }
        }
    }

    #[test]
    fn test_read_truncated_unary_value_simple() {
        let max_val = 3;
        let test_cases = [
            (0i32, vec![0u8]),
            (1, vec![1u8, 0]),
            (2, vec![1u8, 1, 0]),
            (7, vec![1u8, 1, 1, 0]),
            (7, vec![1u8, 1, 1, 1, 0]),
        ];

        for t in test_cases.iter() {
            let res = read_truncated_unary_value(max_val, &t.1);
            println!("Expected: {}; Got: {:?}", t.0, res);
            match res {
                Some(x) => assert!(t.0 == x),
                _ => panic!("Error in parsing"),
            }
        }
    }
}
