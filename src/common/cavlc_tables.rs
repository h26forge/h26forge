//! CAVLC constant values.

use std::collections::HashMap;

/// Table 9-4 (a) Assignment of codeNum to values of coded_block_pattern for macroblock prediction modes
pub const MAPPED_EXP_GOLOMB_CAT12: [[i32; 2]; 48] = [
    [47, 0],
    [31, 16],
    [15, 1],
    [0, 2],
    [23, 4],
    [27, 8],
    [29, 32],
    [30, 3],
    [7, 5],
    [11, 10],
    [13, 12],
    [14, 15],
    [39, 47],
    [43, 7],
    [45, 11],
    [46, 13],
    [16, 14],
    [3, 6],
    [5, 9],
    [10, 31],
    [12, 35],
    [19, 37],
    [21, 42],
    [26, 44],
    [28, 33],
    [35, 34],
    [37, 36],
    [42, 40],
    [44, 39],
    [1, 43],
    [2, 45],
    [4, 46],
    [8, 17],
    [17, 18],
    [18, 20],
    [20, 24],
    [24, 19],
    [6, 21],
    [9, 26],
    [22, 28],
    [25, 23],
    [32, 27],
    [33, 29],
    [34, 30],
    [36, 22],
    [40, 25],
    [38, 38],
    [41, 41],
];

/// Table 9-4 (a) Assignment of codeNum to values of coded_block_pattern for macroblock prediction modes
pub const ENCODE_MAPPED_EXP_GOLOMB_CAT12: [[i32; 2]; 48] = [
    [3, 0],
    [29, 2],
    [30, 3],
    [17, 7],
    [31, 4],
    [18, 8],
    [37, 17],
    [8, 13],
    [32, 5],
    [38, 18],
    [19, 9],
    [9, 14],
    [20, 10],
    [10, 15],
    [11, 16],
    [2, 11],
    [16, 1],
    [33, 32],
    [34, 33],
    [21, 36],
    [35, 34],
    [22, 37],
    [39, 44],
    [4, 40],
    [36, 35],
    [40, 45],
    [23, 38],
    [5, 41],
    [24, 39],
    [6, 42],
    [7, 43],
    [1, 19],
    [41, 6],
    [42, 24],
    [43, 25],
    [25, 20],
    [44, 26],
    [26, 21],
    [46, 46],
    [12, 28],
    [45, 27],
    [47, 47],
    [27, 22],
    [13, 29],
    [28, 23],
    [14, 30],
    [15, 31],
    [0, 12],
];

/// Table 9-4 (b) Assignment of codeNum to values of coded_block_pattern for macroblock prediction modes
pub const MAPPED_EXP_GOLOMB_CAT03: [[i32; 2]; 16] = [
    [15, 0],
    [0, 1],
    [7, 2],
    [11, 4],
    [13, 8],
    [14, 3],
    [3, 5],
    [5, 10],
    [10, 12],
    [12, 15],
    [1, 7],
    [2, 11],
    [4, 13],
    [8, 14],
    [6, 6],
    [9, 9],
];

/// Table 9-4 (b) Assignment of codeNum to values of coded_block_pattern for macroblock prediction modes
pub const ENCODE_MAPPED_EXP_GOLOMB_CAT03: [[i32; 2]; 16] = [
    [1, 0],
    [10, 1],
    [11, 2],
    [6, 5],
    [12, 3],
    [7, 6],
    [14, 14],
    [2, 10],
    [13, 4],
    [15, 15],
    [8, 7],
    [3, 11],
    [9, 8],
    [4, 12],
    [5, 13],
    [0, 9],
];

/// Implements Table 9-5 and returns the relevant look-up-table for decoding
///
/// The hashmap look up is the particular bitstream as a String which returns
/// the TrailingOnes value and the TotalCoeff
pub fn create_coeff_token_mappings(n_c: i8) -> HashMap<String, (usize, usize)> {
    let mut res = HashMap::new();

    // format is <Bytestream, (TrailingOnes, TotalCoeff)
    if (0..2).contains(&n_c) {
        res.insert("1".to_string(), (0, 0));
        res.insert("000101".to_string(), (0, 1));
        res.insert("01".to_string(), (1, 1));
        res.insert("00000111".to_string(), (0, 2));
        res.insert("000100".to_string(), (1, 2));
        res.insert("001".to_string(), (2, 2));
        res.insert("000000111".to_string(), (0, 3));
        res.insert("00000110".to_string(), (1, 3));
        res.insert("0000101".to_string(), (2, 3));
        res.insert("00011".to_string(), (3, 3));
        res.insert("0000000111".to_string(), (0, 4));
        res.insert("000000110".to_string(), (1, 4));
        res.insert("00000101".to_string(), (2, 4));
        res.insert("000011".to_string(), (3, 4));
        res.insert("00000000111".to_string(), (0, 5));
        res.insert("0000000110".to_string(), (1, 5));
        res.insert("000000101".to_string(), (2, 5));
        res.insert("0000100".to_string(), (3, 5));
        res.insert("0000000001111".to_string(), (0, 6));
        res.insert("00000000110".to_string(), (1, 6));
        res.insert("0000000101".to_string(), (2, 6));
        res.insert("00000100".to_string(), (3, 6));
        res.insert("0000000001011".to_string(), (0, 7));
        res.insert("0000000001110".to_string(), (1, 7));
        res.insert("00000000101".to_string(), (2, 7));
        res.insert("000000100".to_string(), (3, 7));
        res.insert("0000000001000".to_string(), (0, 8));
        res.insert("0000000001010".to_string(), (1, 8));
        res.insert("0000000001101".to_string(), (2, 8));
        res.insert("0000000100".to_string(), (3, 8));
        res.insert("00000000001111".to_string(), (0, 9));
        res.insert("00000000001110".to_string(), (1, 9));
        res.insert("0000000001001".to_string(), (2, 9));
        res.insert("00000000100".to_string(), (3, 9));
        res.insert("00000000001011".to_string(), (0, 10));
        res.insert("00000000001010".to_string(), (1, 10));
        res.insert("00000000001101".to_string(), (2, 10));
        res.insert("0000000001100".to_string(), (3, 10));
        res.insert("000000000001111".to_string(), (0, 11));
        res.insert("000000000001110".to_string(), (1, 11));
        res.insert("00000000001001".to_string(), (2, 11));
        res.insert("00000000001100".to_string(), (3, 11));
        res.insert("000000000001011".to_string(), (0, 12));
        res.insert("000000000001010".to_string(), (1, 12));
        res.insert("000000000001101".to_string(), (2, 12));
        res.insert("00000000001000".to_string(), (3, 12));
        res.insert("0000000000001111".to_string(), (0, 13));
        res.insert("000000000000001".to_string(), (1, 13));
        res.insert("000000000001001".to_string(), (2, 13));
        res.insert("000000000001100".to_string(), (3, 13));
        res.insert("0000000000001011".to_string(), (0, 14));
        res.insert("0000000000001110".to_string(), (1, 14));
        res.insert("0000000000001101".to_string(), (2, 14));
        res.insert("000000000001000".to_string(), (3, 14));
        res.insert("0000000000000111".to_string(), (0, 15));
        res.insert("0000000000001010".to_string(), (1, 15));
        res.insert("0000000000001001".to_string(), (2, 15));
        res.insert("0000000000001100".to_string(), (3, 15));
        res.insert("0000000000000100".to_string(), (0, 16));
        res.insert("0000000000000110".to_string(), (1, 16));
        res.insert("0000000000000101".to_string(), (2, 16));
        res.insert("0000000000001000".to_string(), (3, 16));
    } else if (2..4).contains(&n_c) {
        res.insert("11".to_string(), (0, 0));
        res.insert("001011".to_string(), (0, 1));
        res.insert("10".to_string(), (1, 1));
        res.insert("000111".to_string(), (0, 2));
        res.insert("00111".to_string(), (1, 2));
        res.insert("011".to_string(), (2, 2));
        res.insert("0000111".to_string(), (0, 3));
        res.insert("001010".to_string(), (1, 3));
        res.insert("001001".to_string(), (2, 3));
        res.insert("0101".to_string(), (3, 3));
        res.insert("00000111".to_string(), (0, 4));
        res.insert("000110".to_string(), (1, 4));
        res.insert("000101".to_string(), (2, 4));
        res.insert("0100".to_string(), (3, 4));
        res.insert("00000100".to_string(), (0, 5));
        res.insert("0000110".to_string(), (1, 5));
        res.insert("0000101".to_string(), (2, 5));
        res.insert("00110".to_string(), (3, 5));
        res.insert("000000111".to_string(), (0, 6));
        res.insert("00000110".to_string(), (1, 6));
        res.insert("00000101".to_string(), (2, 6));
        res.insert("001000".to_string(), (3, 6));
        res.insert("00000001111".to_string(), (0, 7));
        res.insert("000000110".to_string(), (1, 7));
        res.insert("000000101".to_string(), (2, 7));
        res.insert("000100".to_string(), (3, 7));
        res.insert("00000001011".to_string(), (0, 8));
        res.insert("00000001110".to_string(), (1, 8));
        res.insert("00000001101".to_string(), (2, 8));
        res.insert("0000100".to_string(), (3, 8));
        res.insert("000000001111".to_string(), (0, 9));
        res.insert("00000001010".to_string(), (1, 9));
        res.insert("00000001001".to_string(), (2, 9));
        res.insert("000000100".to_string(), (3, 9));
        res.insert("000000001011".to_string(), (0, 10));
        res.insert("000000001110".to_string(), (1, 10));
        res.insert("000000001101".to_string(), (2, 10));
        res.insert("00000001100".to_string(), (3, 10));
        res.insert("000000001000".to_string(), (0, 11));
        res.insert("000000001010".to_string(), (1, 11));
        res.insert("000000001001".to_string(), (2, 11));
        res.insert("00000001000".to_string(), (3, 11));
        res.insert("0000000001111".to_string(), (0, 12));
        res.insert("0000000001110".to_string(), (1, 12));
        res.insert("0000000001101".to_string(), (2, 12));
        res.insert("000000001100".to_string(), (3, 12));
        res.insert("0000000001011".to_string(), (0, 13));
        res.insert("0000000001010".to_string(), (1, 13));
        res.insert("0000000001001".to_string(), (2, 13));
        res.insert("0000000001100".to_string(), (3, 13));
        res.insert("0000000000111".to_string(), (0, 14));
        res.insert("00000000001011".to_string(), (1, 14));
        res.insert("0000000000110".to_string(), (2, 14));
        res.insert("0000000001000".to_string(), (3, 14));
        res.insert("00000000001001".to_string(), (0, 15));
        res.insert("00000000001000".to_string(), (1, 15));
        res.insert("00000000001010".to_string(), (2, 15));
        res.insert("0000000000001".to_string(), (3, 15));
        res.insert("00000000000111".to_string(), (0, 16));
        res.insert("00000000000110".to_string(), (1, 16));
        res.insert("00000000000101".to_string(), (2, 16));
        res.insert("00000000000100".to_string(), (3, 16));
    } else if (4..8).contains(&n_c) {
        res.insert("1111".to_string(), (0, 0));
        res.insert("001111".to_string(), (0, 1));
        res.insert("1110".to_string(), (1, 1));
        res.insert("001011".to_string(), (0, 2));
        res.insert("01111".to_string(), (1, 2));
        res.insert("1101".to_string(), (2, 2));
        res.insert("001000".to_string(), (0, 3));
        res.insert("01100".to_string(), (1, 3));
        res.insert("01110".to_string(), (2, 3));
        res.insert("1100".to_string(), (3, 3));
        res.insert("0001111".to_string(), (0, 4));
        res.insert("01010".to_string(), (1, 4));
        res.insert("01011".to_string(), (2, 4));
        res.insert("1011".to_string(), (3, 4));
        res.insert("0001011".to_string(), (0, 5));
        res.insert("01000".to_string(), (1, 5));
        res.insert("01001".to_string(), (2, 5));
        res.insert("1010".to_string(), (3, 5));
        res.insert("0001001".to_string(), (0, 6));
        res.insert("001110".to_string(), (1, 6));
        res.insert("001101".to_string(), (2, 6));
        res.insert("1001".to_string(), (3, 6));
        res.insert("0001000".to_string(), (0, 7));
        res.insert("001010".to_string(), (1, 7));
        res.insert("001001".to_string(), (2, 7));
        res.insert("1000".to_string(), (3, 7));
        res.insert("00001111".to_string(), (0, 8));
        res.insert("0001110".to_string(), (1, 8));
        res.insert("0001101".to_string(), (2, 8));
        res.insert("01101".to_string(), (3, 8));
        res.insert("00001011".to_string(), (0, 9));
        res.insert("00001110".to_string(), (1, 9));
        res.insert("0001010".to_string(), (2, 9));
        res.insert("001100".to_string(), (3, 9));
        res.insert("000001111".to_string(), (0, 10));
        res.insert("00001010".to_string(), (1, 10));
        res.insert("00001101".to_string(), (2, 10));
        res.insert("0001100".to_string(), (3, 10));
        res.insert("000001011".to_string(), (0, 11));
        res.insert("000001110".to_string(), (1, 11));
        res.insert("00001001".to_string(), (2, 11));
        res.insert("00001100".to_string(), (3, 11));
        res.insert("000001000".to_string(), (0, 12));
        res.insert("000001010".to_string(), (1, 12));
        res.insert("000001101".to_string(), (2, 12));
        res.insert("00001000".to_string(), (3, 12));
        res.insert("0000001101".to_string(), (0, 13));
        res.insert("000000111".to_string(), (1, 13));
        res.insert("000001001".to_string(), (2, 13));
        res.insert("000001100".to_string(), (3, 13));
        res.insert("0000001001".to_string(), (0, 14));
        res.insert("0000001100".to_string(), (1, 14));
        res.insert("0000001011".to_string(), (2, 14));
        res.insert("0000001010".to_string(), (3, 14));
        res.insert("0000000101".to_string(), (0, 15));
        res.insert("0000001000".to_string(), (1, 15));
        res.insert("0000000111".to_string(), (2, 15));
        res.insert("0000000110".to_string(), (3, 15));
        res.insert("0000000001".to_string(), (0, 16));
        res.insert("0000000100".to_string(), (1, 16));
        res.insert("0000000011".to_string(), (2, 16));
        res.insert("0000000010".to_string(), (3, 16));
    } else if 8 <= n_c {
        res.insert("000011".to_string(), (0, 0));
        res.insert("000000".to_string(), (0, 1));
        res.insert("000001".to_string(), (1, 1));
        res.insert("000100".to_string(), (0, 2));
        res.insert("000101".to_string(), (1, 2));
        res.insert("000110".to_string(), (2, 2));
        res.insert("001000".to_string(), (0, 3));
        res.insert("001001".to_string(), (1, 3));
        res.insert("001010".to_string(), (2, 3));
        res.insert("001011".to_string(), (3, 3));
        res.insert("001100".to_string(), (0, 4));
        res.insert("001101".to_string(), (1, 4));
        res.insert("001110".to_string(), (2, 4));
        res.insert("001111".to_string(), (3, 4));
        res.insert("010000".to_string(), (0, 5));
        res.insert("010001".to_string(), (1, 5));
        res.insert("010010".to_string(), (2, 5));
        res.insert("010011".to_string(), (3, 5));
        res.insert("010100".to_string(), (0, 6));
        res.insert("010101".to_string(), (1, 6));
        res.insert("010110".to_string(), (2, 6));
        res.insert("010111".to_string(), (3, 6));
        res.insert("011000".to_string(), (0, 7));
        res.insert("011001".to_string(), (1, 7));
        res.insert("011010".to_string(), (2, 7));
        res.insert("011011".to_string(), (3, 7));
        res.insert("011100".to_string(), (0, 8));
        res.insert("011101".to_string(), (1, 8));
        res.insert("011110".to_string(), (2, 8));
        res.insert("011111".to_string(), (3, 8));
        res.insert("100000".to_string(), (0, 9));
        res.insert("100001".to_string(), (1, 9));
        res.insert("100010".to_string(), (2, 9));
        res.insert("100011".to_string(), (3, 9));
        res.insert("100100".to_string(), (0, 10));
        res.insert("100101".to_string(), (1, 10));
        res.insert("100110".to_string(), (2, 10));
        res.insert("100111".to_string(), (3, 10));
        res.insert("101000".to_string(), (0, 11));
        res.insert("101001".to_string(), (1, 11));
        res.insert("101010".to_string(), (2, 11));
        res.insert("101011".to_string(), (3, 11));
        res.insert("101100".to_string(), (0, 12));
        res.insert("101101".to_string(), (1, 12));
        res.insert("101110".to_string(), (2, 12));
        res.insert("101111".to_string(), (3, 12));
        res.insert("110000".to_string(), (0, 13));
        res.insert("110001".to_string(), (1, 13));
        res.insert("110010".to_string(), (2, 13));
        res.insert("110011".to_string(), (3, 13));
        res.insert("110100".to_string(), (0, 14));
        res.insert("110101".to_string(), (1, 14));
        res.insert("110110".to_string(), (2, 14));
        res.insert("110111".to_string(), (3, 14));
        res.insert("111000".to_string(), (0, 15));
        res.insert("111001".to_string(), (1, 15));
        res.insert("111010".to_string(), (2, 15));
        res.insert("111011".to_string(), (3, 15));
        res.insert("111100".to_string(), (0, 16));
        res.insert("111101".to_string(), (1, 16));
        res.insert("111110".to_string(), (2, 16));
        res.insert("111111".to_string(), (3, 16));
    } else if n_c == -1 {
        res.insert("01".to_string(), (0, 0));
        res.insert("000111".to_string(), (0, 1));
        res.insert("1".to_string(), (1, 1));
        res.insert("000100".to_string(), (0, 2));
        res.insert("000110".to_string(), (1, 2));
        res.insert("001".to_string(), (2, 2));
        res.insert("000011".to_string(), (0, 3));
        res.insert("0000011".to_string(), (1, 3));
        res.insert("0000010".to_string(), (2, 3));
        res.insert("000101".to_string(), (3, 3));
        res.insert("000010".to_string(), (0, 4));
        res.insert("00000011".to_string(), (1, 4));
        res.insert("00000010".to_string(), (2, 4));
        res.insert("0000000".to_string(), (3, 4));
    } else if n_c == -2 {
        res.insert("1".to_string(), (0, 0));
        res.insert("0001111".to_string(), (0, 1));
        res.insert("01".to_string(), (1, 1));
        res.insert("0001110".to_string(), (0, 2));
        res.insert("0001101".to_string(), (1, 2));
        res.insert("001".to_string(), (2, 2));
        res.insert("000000111".to_string(), (0, 3));
        res.insert("0001100".to_string(), (1, 3));
        res.insert("0001011".to_string(), (2, 3));
        res.insert("00001".to_string(), (3, 3));
        res.insert("000000110".to_string(), (0, 4));
        res.insert("000000101".to_string(), (1, 4));
        res.insert("0001010".to_string(), (2, 4));
        res.insert("000001".to_string(), (3, 4));
        res.insert("0000000111".to_string(), (0, 5));
        res.insert("0000000110".to_string(), (1, 5));
        res.insert("000000100".to_string(), (2, 5));
        res.insert("0001001".to_string(), (3, 5));
        res.insert("00000000111".to_string(), (0, 6));
        res.insert("00000000110".to_string(), (1, 6));
        res.insert("0000000101".to_string(), (2, 6));
        res.insert("0001000".to_string(), (3, 6));
        res.insert("000000000111".to_string(), (0, 7));
        res.insert("000000000110".to_string(), (1, 7));
        res.insert("00000000101".to_string(), (2, 7));
        res.insert("0000000100".to_string(), (3, 7));
        res.insert("0000000000111".to_string(), (0, 8));
        res.insert("000000000101".to_string(), (1, 8));
        res.insert("000000000100".to_string(), (2, 8));
        res.insert("00000000100".to_string(), (3, 8));
    } else {
        panic!("Wrong n_c value calculated: {}", n_c);
    }
    res
}

/// Implements Tables 9-7, 9-8, and 9-9 to return the relevant total_zeros value
/// depending on the max_num_coeffs and tz_vcl_index
///
/// The hashmap lookup takes in a bit string and provides total_zeros
pub fn create_total_zeros_mappings(
    max_num_coeff: usize,
    tz_vcl_index: usize,
) -> HashMap<String, usize> {
    let mut res = HashMap::new();

    if max_num_coeff == 4 {
        // use Table 9-9 (a)
        match tz_vcl_index {
            1 => {
                res.insert("1".to_string(), 0);
                res.insert("01".to_string(), 1);
                res.insert("001".to_string(), 2);
                res.insert("000".to_string(), 3);
            }
            2 => {
                res.insert("1".to_string(), 0);
                res.insert("01".to_string(), 1);
                res.insert("00".to_string(), 2);
            }
            3 => {
                res.insert("1".to_string(), 0);
                res.insert("0".to_string(), 1);
            }
            _ => panic!("Bad tz_vcl_index value: {}", tz_vcl_index),
        }
    } else if max_num_coeff == 8 {
        // use Table 9-9 (b)
        match tz_vcl_index {
            1 => {
                res.insert("1".to_string(), 0);
                res.insert("010".to_string(), 1);
                res.insert("011".to_string(), 2);
                res.insert("0010".to_string(), 3);
                res.insert("0011".to_string(), 4);
                res.insert("0001".to_string(), 5);
                res.insert("00001".to_string(), 6);
                res.insert("00000".to_string(), 7);
            }
            2 => {
                res.insert("000".to_string(), 0);
                res.insert("01".to_string(), 1);
                res.insert("001".to_string(), 2);
                res.insert("100".to_string(), 3);
                res.insert("101".to_string(), 4);
                res.insert("110".to_string(), 5);
                res.insert("111".to_string(), 6);
            }
            3 => {
                res.insert("000".to_string(), 0);
                res.insert("001".to_string(), 1);
                res.insert("01".to_string(), 2);
                res.insert("10".to_string(), 3);
                res.insert("110".to_string(), 4);
                res.insert("111".to_string(), 5);
            }
            4 => {
                res.insert("110".to_string(), 0);
                res.insert("00".to_string(), 1);
                res.insert("01".to_string(), 2);
                res.insert("10".to_string(), 3);
                res.insert("111".to_string(), 4);
            }
            5 => {
                res.insert("00".to_string(), 0);
                res.insert("01".to_string(), 1);
                res.insert("10".to_string(), 2);
                res.insert("11".to_string(), 3);
            }
            6 => {
                res.insert("00".to_string(), 0);
                res.insert("01".to_string(), 1);
                res.insert("1".to_string(), 2);
            }
            7 => {
                res.insert("0".to_string(), 0);
                res.insert("1".to_string(), 1);
            }
            _ => panic!("Bad tz_vcl_index value: {}", tz_vcl_index),
        }
    } else {
        // use tables 9-7 and 9-8
        match tz_vcl_index {
            // table 9-7
            1 => {
                res.insert("1".to_string(), 0);
                res.insert("011".to_string(), 1);
                res.insert("010".to_string(), 2);
                res.insert("0011".to_string(), 3);
                res.insert("0010".to_string(), 4);
                res.insert("00011".to_string(), 5);
                res.insert("00010".to_string(), 6);
                res.insert("000011".to_string(), 7);
                res.insert("000010".to_string(), 8);
                res.insert("0000011".to_string(), 9);
                res.insert("0000010".to_string(), 10);
                res.insert("00000011".to_string(), 11);
                res.insert("00000010".to_string(), 12);
                res.insert("000000011".to_string(), 13);
                res.insert("000000010".to_string(), 14);
                res.insert("000000001".to_string(), 15);
            }
            2 => {
                res.insert("111".to_string(), 0);
                res.insert("110".to_string(), 1);
                res.insert("101".to_string(), 2);
                res.insert("100".to_string(), 3);
                res.insert("011".to_string(), 4);
                res.insert("0101".to_string(), 5);
                res.insert("0100".to_string(), 6);
                res.insert("0011".to_string(), 7);
                res.insert("0010".to_string(), 8);
                res.insert("00011".to_string(), 9);
                res.insert("00010".to_string(), 10);
                res.insert("000011".to_string(), 11);
                res.insert("000010".to_string(), 12);
                res.insert("000001".to_string(), 13);
                res.insert("000000".to_string(), 14);
            }
            3 => {
                res.insert("0101".to_string(), 0);
                res.insert("111".to_string(), 1);
                res.insert("110".to_string(), 2);
                res.insert("101".to_string(), 3);
                res.insert("0100".to_string(), 4);
                res.insert("0011".to_string(), 5);
                res.insert("100".to_string(), 6);
                res.insert("011".to_string(), 7);
                res.insert("0010".to_string(), 8);
                res.insert("00011".to_string(), 9);
                res.insert("00010".to_string(), 10);
                res.insert("000001".to_string(), 11);
                res.insert("00001".to_string(), 12);
                res.insert("000000".to_string(), 13);
            }
            4 => {
                res.insert("00011".to_string(), 0);
                res.insert("111".to_string(), 1);
                res.insert("0101".to_string(), 2);
                res.insert("0100".to_string(), 3);
                res.insert("110".to_string(), 4);
                res.insert("101".to_string(), 5);
                res.insert("100".to_string(), 6);
                res.insert("0011".to_string(), 7);
                res.insert("011".to_string(), 8);
                res.insert("0010".to_string(), 9);
                res.insert("00010".to_string(), 10);
                res.insert("00001".to_string(), 11);
                res.insert("00000".to_string(), 12);
            }
            5 => {
                res.insert("0101".to_string(), 0);
                res.insert("0100".to_string(), 1);
                res.insert("0011".to_string(), 2);
                res.insert("111".to_string(), 3);
                res.insert("110".to_string(), 4);
                res.insert("101".to_string(), 5);
                res.insert("100".to_string(), 6);
                res.insert("011".to_string(), 7);
                res.insert("0010".to_string(), 8);
                res.insert("00001".to_string(), 9);
                res.insert("0001".to_string(), 10);
                res.insert("00000".to_string(), 11);
            }
            6 => {
                res.insert("000001".to_string(), 0);
                res.insert("00001".to_string(), 1);
                res.insert("111".to_string(), 2);
                res.insert("110".to_string(), 3);
                res.insert("101".to_string(), 4);
                res.insert("100".to_string(), 5);
                res.insert("011".to_string(), 6);
                res.insert("010".to_string(), 7);
                res.insert("0001".to_string(), 8);
                res.insert("001".to_string(), 9);
                res.insert("000000".to_string(), 10);
            }
            7 => {
                res.insert("000001".to_string(), 0);
                res.insert("00001".to_string(), 1);
                res.insert("101".to_string(), 2);
                res.insert("100".to_string(), 3);
                res.insert("011".to_string(), 4);
                res.insert("11".to_string(), 5);
                res.insert("010".to_string(), 6);
                res.insert("0001".to_string(), 7);
                res.insert("001".to_string(), 8);
                res.insert("000000".to_string(), 9);
            }
            // Table 9-8
            8 => {
                res.insert("000001".to_string(), 0);
                res.insert("0001".to_string(), 1);
                res.insert("00001".to_string(), 2);
                res.insert("011".to_string(), 3);
                res.insert("11".to_string(), 4);
                res.insert("10".to_string(), 5);
                res.insert("010".to_string(), 6);
                res.insert("001".to_string(), 7);
                res.insert("000000".to_string(), 8);
            }
            9 => {
                res.insert("000001".to_string(), 0);
                res.insert("000000".to_string(), 1);
                res.insert("0001".to_string(), 2);
                res.insert("11".to_string(), 3);
                res.insert("10".to_string(), 4);
                res.insert("001".to_string(), 5);
                res.insert("01".to_string(), 6);
                res.insert("00001".to_string(), 7);
            }
            10 => {
                res.insert("00001".to_string(), 0);
                res.insert("00000".to_string(), 1);
                res.insert("001".to_string(), 2);
                res.insert("11".to_string(), 3);
                res.insert("10".to_string(), 4);
                res.insert("01".to_string(), 5);
                res.insert("0001".to_string(), 6);
            }
            11 => {
                res.insert("0000".to_string(), 0);
                res.insert("0001".to_string(), 1);
                res.insert("001".to_string(), 2);
                res.insert("010".to_string(), 3);
                res.insert("1".to_string(), 4);
                res.insert("011".to_string(), 5);
            }
            12 => {
                res.insert("0000".to_string(), 0);
                res.insert("0001".to_string(), 1);
                res.insert("01".to_string(), 2);
                res.insert("1".to_string(), 3);
                res.insert("001".to_string(), 4);
            }
            13 => {
                res.insert("000".to_string(), 0);
                res.insert("001".to_string(), 1);
                res.insert("1".to_string(), 2);
                res.insert("01".to_string(), 3);
            }
            14 => {
                res.insert("00".to_string(), 0);
                res.insert("01".to_string(), 1);
                res.insert("1".to_string(), 2);
            }
            15 => {
                res.insert("0".to_string(), 0);
                res.insert("1".to_string(), 1);
            }
            _ => panic!("Bad tz_vcl_index value: {}", tz_vcl_index),
        }
    }

    res
}

/// Implements Table 9-10
pub fn create_run_before_mappings(zeros_left: usize) -> HashMap<String, usize> {
    let mut res = HashMap::new();

    if zeros_left > 6 {
        res.insert("111".to_string(), 0);
        res.insert("110".to_string(), 1);
        res.insert("101".to_string(), 2);
        res.insert("100".to_string(), 3);
        res.insert("011".to_string(), 4);
        res.insert("010".to_string(), 5);
        res.insert("001".to_string(), 6);
        res.insert("0001".to_string(), 7);
        res.insert("00001".to_string(), 8);
        res.insert("000001".to_string(), 9);
        res.insert("0000001".to_string(), 10);
        res.insert("00000001".to_string(), 11);
        res.insert("000000001".to_string(), 12);
        res.insert("0000000001".to_string(), 13);
        res.insert("00000000001".to_string(), 14);
    } else {
        match zeros_left {
            1 => {
                res.insert("1".to_string(), 0);
                res.insert("0".to_string(), 1);
            }
            2 => {
                res.insert("1".to_string(), 0);
                res.insert("01".to_string(), 1);
                res.insert("00".to_string(), 2);
            }
            3 => {
                res.insert("11".to_string(), 0);
                res.insert("10".to_string(), 1);
                res.insert("01".to_string(), 2);
                res.insert("00".to_string(), 3);
            }
            4 => {
                res.insert("11".to_string(), 0);
                res.insert("10".to_string(), 1);
                res.insert("01".to_string(), 2);
                res.insert("001".to_string(), 3);
                res.insert("000".to_string(), 4);
            }
            5 => {
                res.insert("11".to_string(), 0);
                res.insert("10".to_string(), 1);
                res.insert("011".to_string(), 2);
                res.insert("010".to_string(), 3);
                res.insert("001".to_string(), 4);
                res.insert("000".to_string(), 5);
            }
            6 => {
                res.insert("11".to_string(), 0);
                res.insert("000".to_string(), 1);
                res.insert("001".to_string(), 2);
                res.insert("011".to_string(), 3);
                res.insert("010".to_string(), 4);
                res.insert("101".to_string(), 5);
                res.insert("100".to_string(), 6);
            }
            _ => {
                panic!("Bad zeros_left value: {}", zeros_left);
            }
        }
    }

    res
}
