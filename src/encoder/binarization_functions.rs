//! Binarization encoding functions.

use crate::common::data_structures::MbType;
use crate::common::data_structures::SliceHeader;
use crate::common::data_structures::SubMbType;
use crate::common::helper::is_slice_type;
use crate::encoder::expgolomb;
use std::cmp;

/// Encode a num as unary
pub fn generate_unary_value(num: u32) -> Vec<u8> {
    let mut res: Vec<u8> = vec![1; num as usize];

    res.push(0);
    res
}

/// Encode a num as unary up to a max
///
/// Any number less than c_max is treated as a unary value, while
/// anything greater than or equal is set to all 1s (i.e. not ending with a 0)
pub fn generate_truncated_unary_value(num: u32, c_max: u32) -> Vec<u8> {
    let res: Vec<u8> = if num >= c_max {
        vec![1; c_max as usize]
    } else {
        generate_unary_value(num)
    };

    res
}

/// Encode a value as binary that gets either padded up to `c_max` or truncated to `c_max`
pub fn generate_fixed_length_value(num: u32, c_max: usize) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();

    let mut bin_string: String = format!("{:b}", num);

    if bin_string.len() < c_max {
        while res.len() < c_max - bin_string.len() {
            res.push(0);
        }
    }

    // truncate it if it's too long
    if bin_string.len() > c_max {
        bin_string = bin_string[0..c_max].to_string();
    }

    for b in bin_string.chars() {
        if b == '0' {
            res.push(0);
        } else {
            res.push(1);
        }
    }

    if res.len() != c_max {
        panic!(
            "generate_fixed_length_value - error with {} and {}",
            num, c_max
        );
    }

    res
}

/// Specified in clause 9.3.2.5
pub fn generate_mb_type_value(mt: MbType, sh: &SliceHeader) -> (Vec<u8>, bool, usize) {
    let mut i_mb_type_in_non_i_slice: bool = false;
    let mut res: Vec<u8> = Vec::new();

    if is_slice_type(sh.slice_type, "SI") {
        if mt == MbType::SI {
            res.push(0);
        } else {
            res.push(1);
            i_mb_type_in_non_i_slice = true;
        }
    } else if is_slice_type(sh.slice_type, "P") || is_slice_type(sh.slice_type, "SP") {
        if mt == MbType::PL016x16 {
            res.push(0);
            res.push(0);
            res.push(0);
        } else if mt == MbType::PL0L016x8 {
            res.push(0);
            res.push(1);
            res.push(1);
        } else if mt == MbType::PL0L08x16 {
            res.push(0);
            res.push(1);
            res.push(0);
        } else if mt == MbType::P8x8 {
            res.push(0);
            res.push(0);
            res.push(1);
        } else if mt == MbType::P8x8ref0 {
            // Seems to only exist in CAVLC encoded
            panic!("generate_mb_type_value - not allowed MbType - P8x8ref0");
        } else {
            res.push(1);
            i_mb_type_in_non_i_slice = true;
        }
    } else if is_slice_type(sh.slice_type, "B") {
        if mt == MbType::BDirect16x16 {
            res.push(0);
        } else if mt == MbType::BL016x16 {
            res.push(1);
            res.push(0);
            res.push(0);
        } else if mt == MbType::BL116x16 {
            res.push(1);
            res.push(0);
            res.push(1);
        } else if mt == MbType::BBi16x16 {
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(0);
            res.push(0);
            res.push(0);
        } else if mt == MbType::BL0L016x8 {
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(0);
            res.push(0);
            res.push(1);
        } else if mt == MbType::BL0L08x16 {
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(0);
            res.push(1);
            res.push(0);
        } else if mt == MbType::BL1L116x8 {
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(0);
            res.push(1);
            res.push(1);
        } else if mt == MbType::BL1L18x16 {
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(1);
            res.push(0);
            res.push(0);
        } else if mt == MbType::BL0L116x8 {
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(1);
            res.push(0);
            res.push(1);
        } else if mt == MbType::BL0L18x16 {
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(1);
            res.push(1);
            res.push(0);
        } else if mt == MbType::BL1L016x8 {
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(1);
            res.push(1);
            res.push(1);
        } else if mt == MbType::BL1L08x16 {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
        } else if mt == MbType::BL0Bi16x8 {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(0);
            res.push(0);
            res.push(0);
        } else if mt == MbType::BL0Bi8x16 {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(0);
            res.push(0);
            res.push(1);
        } else if mt == MbType::BL1Bi16x8 {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(0);
            res.push(1);
            res.push(0);
        } else if mt == MbType::BL1Bi8x16 {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(0);
            res.push(1);
            res.push(1);
        } else if mt == MbType::BBiL016x8 {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(1);
            res.push(0);
            res.push(0);
        } else if mt == MbType::BBiL08x16 {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(1);
            res.push(0);
            res.push(1);
        } else if mt == MbType::BBiL116x8 {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(1);
            res.push(1);
            res.push(0);
        } else if mt == MbType::BBiL18x16 {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(1);
            res.push(1);
            res.push(1);
        } else if mt == MbType::BBiBi16x8 {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(0);
            res.push(0);
        } else if mt == MbType::BBiBi8x16 {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(0);
            res.push(1);
        } else if mt == MbType::B8x8 {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(1);
        } else {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(1);
            i_mb_type_in_non_i_slice = true;
        }
    }

    let prefix_len: usize = if i_mb_type_in_non_i_slice {
        res.len()
    } else {
        0
    };

    if is_slice_type(sh.slice_type, "I") || i_mb_type_in_non_i_slice {
        match mt {
            MbType::INxN => {
                res.push(0);
            }
            MbType::I16x16_0_0_0 => {
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(0);
                res.push(0);
                res.push(0);
            }
            MbType::I16x16_1_0_0 => {
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(0);
                res.push(0);
                res.push(1);
            }
            MbType::I16x16_2_0_0 => {
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(0);
                res.push(1);
                res.push(0);
            }
            MbType::I16x16_3_0_0 => {
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(0);
                res.push(1);
                res.push(1);
            }
            MbType::I16x16_0_1_0 => {
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(0);
            }
            MbType::I16x16_1_1_0 => {
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(1);
            }
            MbType::I16x16_2_1_0 => {
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(0);
            }
            MbType::I16x16_3_1_0 => {
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(1);
            }
            MbType::I16x16_0_2_0 => {
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(1);
                res.push(1);
                res.push(0);
                res.push(0);
            }
            MbType::I16x16_1_2_0 => {
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(1);
                res.push(1);
                res.push(0);
                res.push(1);
            }
            MbType::I16x16_2_2_0 => {
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(1);
                res.push(1);
                res.push(1);
                res.push(0);
            }
            MbType::I16x16_3_2_0 => {
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(1);
                res.push(1);
                res.push(1);
                res.push(1);
            }
            MbType::I16x16_0_0_1 => {
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(0);
            }
            MbType::I16x16_1_0_1 => {
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(1);
            }
            MbType::I16x16_2_0_1 => {
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(0);
            }
            MbType::I16x16_3_0_1 => {
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(1);
            }
            MbType::I16x16_0_1_1 => {
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(0);
            }
            MbType::I16x16_1_1_1 => {
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(1);
                res.push(0);
                res.push(0);
                res.push(1);
            }
            MbType::I16x16_2_1_1 => {
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(0);
            }
            MbType::I16x16_3_1_1 => {
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(1);
            }
            MbType::I16x16_0_2_1 => {
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(1);
                res.push(1);
                res.push(0);
                res.push(0);
            }
            MbType::I16x16_1_2_1 => {
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(1);
                res.push(1);
                res.push(0);
                res.push(1);
            }
            MbType::I16x16_2_2_1 => {
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(1);
                res.push(1);
                res.push(1);
                res.push(0);
            }
            MbType::I16x16_3_2_1 => {
                res.push(1);
                res.push(0);
                res.push(1);
                res.push(1);
                res.push(1);
                res.push(1);
                res.push(1);
            }
            MbType::IPCM => {
                res.push(1);
                res.push(1);
            }
            _ => {}
        }
    }

    (res, i_mb_type_in_non_i_slice, prefix_len)
}

/// Specified in clause 9.3.2.5
pub fn generate_sub_mb_type_value(smt: SubMbType) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();

    // Table 9-38 - Binarization for sub-macroblock types in P, SP, and B slices
    match smt {
        SubMbType::NA => {
            panic!("generate_sub_mb_type_value - trying to encode an uninitialized submacroblock");
        }
        // P, SP slices
        SubMbType::PL08x8 => {
            res.push(1);
        }
        SubMbType::PL08x4 => {
            res.push(0);
            res.push(0);
        }
        SubMbType::PL04x8 => {
            res.push(0);
            res.push(1);
            res.push(1);
        }
        SubMbType::PL04x4 => {
            res.push(0);
            res.push(1);
            res.push(0);
        }
        // B slices
        SubMbType::BDirect8x8 => {
            res.push(0);
        }
        SubMbType::BL08x8 => {
            res.push(1);
            res.push(0);
            res.push(0);
        }
        SubMbType::BL18x8 => {
            res.push(1);
            res.push(0);
            res.push(1);
        }
        SubMbType::BBi8x8 => {
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(0);
            res.push(0);
        }
        SubMbType::BL08x4 => {
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(0);
            res.push(1);
        }
        SubMbType::BL04x8 => {
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(1);
            res.push(0);
        }
        SubMbType::BL18x4 => {
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(1);
            res.push(1);
        }
        SubMbType::BL14x8 => {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(0);
            res.push(0);
        }
        SubMbType::BBi8x4 => {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(0);
            res.push(1);
        }
        SubMbType::BBi4x8 => {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(1);
            res.push(0);
        }
        SubMbType::BL04x4 => {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
            res.push(1);
            res.push(1);
        }
        SubMbType::BL14x4 => {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(0);
        }
        SubMbType::BBi4x4 => {
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(1);
            res.push(1);
        }
    }

    res
}

/// Generate concatenated unary/k-th order Exp-Golomb value
pub fn generate_uegk(num: i32, ucoff: u32, k: usize, _signed_val_flag: bool) -> (Vec<u8>, bool) {
    let mut res: Vec<u8> = Vec::new();
    let suffix_exist: bool;

    if num.abs() >= ucoff as i32 {
        res = vec![1; ucoff as usize];

        res.append(&mut expgolomb::exp_golomb_encode_one(
            num.abs() - (ucoff as i32),
            false,
            k,
            true,
        ));
        suffix_exist = true;
    } else {
        res.append(&mut generate_truncated_unary_value(num.abs() as u32, ucoff));
        suffix_exist = false;
    }

    (res, suffix_exist)
}

/// Specified in clause 9.3.2.6
// TODO: pass in VP or just ChromaArrayType for CAVLC 422/444
pub fn generate_coded_block_pattern_value(num: u32) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();

    // consists of a prefix part and (when present) a suffix part
    // prefix is FL binarization of CodedBlockPatternLuma with cMax = 15
    // when ChromaArrayType is not equal to 0 or 3, the suffix is present and
    // consists of the TU binarization of CodedBlockPatternChroma with cMax = 2

    // first the Luma component
    res.append(&mut generate_fixed_length_value(num & 0xf, 4)); // first 4 bits are luma

    // reverse it because the bits are read from lsb to msb
    res.reverse();

    // TODO: adjust for other ChromaArrayType for CAVLC 422/444
    res.append(&mut generate_truncated_unary_value((num & 0x30) >> 4, 2)); // next 2 are chroma

    res
}

/// Specified in clause 9.3.2.7
pub fn generate_mb_qp_delta_value(num: i32) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();

    // binarization of mb_qp_delta is derived by the U binarization of the mapped value of the
    // syntax element mb_qp_delta where the assignment between the signed
    // value of mb_qp_delta and its mapped value is given as specified
    // in Table 9-3

    let working_num: i32 = if num <= 0 { -2 * num } else { 2 * num - 1 };

    res.append(&mut generate_unary_value(working_num as u32));

    res
}

/// Specified in clause 9.3.2.3
///
/// Binarization is given by UEG0 with signed_val_flag = 0 && u_coff = 14
pub fn generate_coeff_abs_level_minus1_value(num: u32) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();

    let u_coff: u32 = 14;
    // this is a concatenation of a prefix bit string and a suffix bit string

    if num >= u_coff {
        res = vec![1; u_coff as usize];
        res.append(&mut expgolomb::exp_golomb_encode_one(
            (num - u_coff) as i32,
            false,
            0,
            true,
        ));
    } else {
        res.append(&mut generate_truncated_unary_value(num, u_coff));
    }

    res
}

/// Encode an unsigned binary value of a passed in length
pub fn generate_unsigned_binary(num: u32, len: usize) -> Vec<u8> {
    let mut num_bits: Vec<u8>;

    // The number of bits to encode is the min of the desired length and the actual number of bits
    let num_bit_size = cmp::min(len, (num as f64 + 1.0).log2().ceil() as usize);

    // If the requested length is too big, pad the output with 0s
    if len > num_bit_size {
        num_bits = vec![0; len-num_bit_size]
    } else {
        num_bits = Vec::new();
    }

    for i in (0..num_bit_size).rev() {
        num_bits.push(((num & (1 << i)) >> i) as u8);
    }
    num_bits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_unary_value() {
        // (input, result)
        let test_cases = [
            (0, vec![0u8]),
            (1, vec![1u8, 0u8]),
            (2, vec![1u8, 1u8, 0u8]),
            (3, vec![1u8, 1u8, 1u8, 0u8]),
            (4, vec![1u8, 1u8, 1u8, 1u8, 0u8]),
            (
                10,
                vec![1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 0u8],
            ),
        ];

        for t in test_cases.iter() {
            let r = generate_unary_value(t.0);
            assert_eq!(r, t.1);
        }
    }

    #[test]
    fn test_generate_truncated_unary_value() {
        // (input, result, c_max)
        let test_cases = [
            (0, vec![0u8], 3),
            (1, vec![1u8, 0u8], 3),
            (2, vec![1u8, 1u8, 0u8], 3),
            (3, vec![1u8, 1u8, 1u8], 3),
            (4, vec![1u8, 1u8, 1u8], 3),
            (
                10,
                vec![1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 0u8],
                11,
            ),
        ];

        for t in test_cases.iter() {
            let r = generate_truncated_unary_value(t.0, t.2);
            assert_eq!(r, t.1);
        }
    }

    #[test]
    fn test_generate_fixed_length_value() {
        // (input, result, c_max)
        let test_cases = [
            (0, vec![0u8], 1),
            (1, vec![1u8], 1),
            (0, vec![0u8, 0u8, 0u8], 3),
            (1, vec![0u8, 0u8, 1u8], 3),
            (2, vec![0u8, 1u8, 0u8], 3),
            (
                55,
                vec![0u8, 0u8, 0u8, 0u8, 1u8, 1u8, 0u8, 1u8, 1u8, 1u8],
                10,
            ),
        ];

        for t in test_cases.iter() {
            let r = generate_fixed_length_value(t.0, t.2);
            assert_eq!(r, t.1);
        }
    }

    #[test]
    fn test_generate_mb_type_value_i_slice() {
        let test_cases = [
            (MbType::INxN, vec![0u8]),
            (MbType::I16x16_0_0_0, vec![1u8, 0, 0, 0, 0, 0]),
            (MbType::I16x16_1_0_0, vec![1u8, 0, 0, 0, 0, 1]),
            (MbType::I16x16_1_1_1, vec![1u8, 0, 1, 1, 0, 0, 1]),
        ];

        let mut sh = SliceHeader::new();
        sh.slice_type = 2;

        for t in test_cases.iter() {
            let r = generate_mb_type_value(t.0, &sh);
            assert_eq!(r.0, t.1);
        }
    }

    #[test]
    fn test_generate_coded_block_pattern_value_chroma_array_type_1_or_2() {
        // top 4 bits is luma; bottom 2 bits are chroma
        let test_cases = [
            (0, vec![0u8, 0u8, 0u8, 0u8, 0u8]),
            (1, vec![1u8, 0u8, 0u8, 0u8, 0u8]),
            (16, vec![0u8, 0u8, 0u8, 0u8, 1u8, 0u8]),
            (17, vec![1u8, 0u8, 0u8, 0u8, 1u8, 0u8]),
            (28, vec![0u8, 0u8, 1u8, 1u8, 1u8, 0u8]),
            (51, vec![1u8, 1u8, 0u8, 0u8, 1u8, 1u8]),
        ];

        for t in test_cases.iter() {
            let r = generate_coded_block_pattern_value(t.0);
            assert_eq!(r, t.1);
        }
    }

    #[test]
    fn test_generate_mb_qp_delta_value() {
        let test_cases = [
            (0, vec![0u8]),
            (1, vec![1u8, 0u8]),
            (-1, vec![1u8, 1u8, 0u8]),
            (2, vec![1u8, 1u8, 1u8, 0u8]),
            (-2, vec![1u8, 1u8, 1u8, 1u8, 0u8]),
            (3, vec![1u8, 1u8, 1u8, 1u8, 1u8, 0u8]),
            (-3, vec![1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 0u8]),
            (4, vec![1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 0u8]),
        ];

        for t in test_cases.iter() {
            let r = generate_mb_qp_delta_value(t.0);
            assert_eq!(r, t.1);
        }
    }

    #[test]
    fn test_generate_coeff_abs_level_minus1_value() {
        let test_cases = [
            (0, vec![0u8]),
            (1, vec![1u8, 0u8]),
            (
                13,
                vec![
                    1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 0u8,
                ],
            ),
            (
                14,
                vec![
                    1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 0u8,
                ],
            ),
            (
                15,
                vec![
                    1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 1u8, 0u8,
                    0u8,
                ],
            ),
        ];

        for t in test_cases.iter() {
            let r = generate_coeff_abs_level_minus1_value(t.0);
            assert_eq!(r, t.1);
        }
    }

    #[test]
    fn test_generate_unsigned_binary() {
        // (number, length, number_in_binary)
        let test_cases = [
            (0, 1, vec![0u8]),
            (0, 2, vec![0, 0]),
            (0, 3, vec![0, 0, 0]),
            (0, 4, vec![0, 0, 0, 0]),
            (1, 4, vec![0, 0, 0, 1]),
            (
                23,
                32,
                vec![
                    0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 1, 0, 1, 1, 1,
                ],
            ),
            (16, 4, vec![0, 0, 0, 0]),
            (
                u32::MAX,
                32,
                vec![
                    1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1,
                ],
            ),
            (
                u32::MAX,
                64,
                vec![
                    0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0,
                    1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1,
                ],
            )
        ];

        for t in test_cases.iter() {
            let r = generate_unsigned_binary(t.0, t.1);
            assert_eq!(r, t.2);
        }
    }
}
