//! Exp-Golomb entropy encoding.

/// Exp-Golomb encode
pub fn exp_golomb_encode_one(val: i32, signed: bool, k: usize, reversed: bool) -> Vec<u8> {
    // Generalization of order k

    // encode floor(x/2^k) using order-0 exp-golomb
    let mut working_num = val >> k;

    if signed {
        if val <= 0 {
            working_num = val * -2;
        } else {
            working_num = 2 * val - 1;
        }
    }
    // write x+1 in binary
    let bin_string: String = format!("{:b}", working_num + 1);

    let bits_written = bin_string.len() - 1;

    // write binary length-1 number of 0s in front
    let mut res = if reversed {
        // NOTE 2 of the H.264 Spec Section 9 (around 9-6) claims that 0s and 1s are reversed when doing UEGk encoding
        vec![1; bits_written]
    } else {
        vec![0; bits_written]
    };

    let mut first = true;

    for c in bin_string.chars() {
        if c == '0' {
            res.push(0);
        } else if first && reversed {
            // the first bit denoting a start point has to be 0
            first = false;
            res.push(0);
        } else {
            res.push(1);
        }
    }

    // encode x mod 2^k in binary
    if k != 0 {
        let remainder = val % (1 << k);
        let mut remainder_string: String = format!("{:b}", remainder);

        while remainder_string.len() < k {
            remainder_string.insert(0, '0');
        }

        for c in remainder_string.chars() {
            if c == '0' {
                res.push(0);
            } else {
                res.push(1);
            }
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exp_golomb_encode_one() {
        let test_cases = [
            (0i32, vec![1u8]),
            (1i32, vec![0u8, 1u8, 0u8]),
            (2i32, vec![0u8, 1u8, 1u8]),
        ];

        for r in test_cases.iter() {
            let result = exp_golomb_encode_one(r.0, false, 0, false);
            println!("Expected: {:?}, got: {:?}", r.1, result);
            assert_eq!(result, r.1);
        }
    }

    #[test]
    fn test_exp_golomb_encode_one_signed() {
        let test_cases = [
            (0i32, vec![1u8]),
            (1i32, vec![0u8, 1u8, 0u8]),
            (-1i32, vec![0u8, 1u8, 1u8]),
        ];

        for r in test_cases.iter() {
            let result = exp_golomb_encode_one(r.0, true, 0, false);
            println!("Expected: {:?}, got: {:?}", r.1, result);
            assert_eq!(result, r.1);
        }
    }

    #[test]
    fn test_exp_golomb_encode_no_stream_simple() {
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
            let result = exp_golomb_encode_one(t.0, t.2, 0, false);
            println!("Expected: {:?}, got: {:?}", t.1, result);
            assert_eq!(result, t.1);
        }
    }

    #[test]
    fn test_exp_golomb_encode_one_simple_k() {
        // (integer, binary stream to decode, k value, reversed or not)
        let test_cases = [
            (0i32, vec![1u8], 0, false),
            (1i32, vec![0u8, 1], 1, true),
            (2i32, vec![0u8, 1, 0], 2, true),
            (0i32, vec![0u8, 0, 0, 0], 3, true),
            (1i32, vec![0u8, 0, 0, 1], 3, true),
            (2i32, vec![0u8, 0, 1, 0], 3, true),
            (13i32, vec![1u8, 0, 0, 1, 0, 1], 3, true),
            (24i32, vec![1u8, 1, 0, 0, 0, 0, 0, 0], 3, true),
        ];

        for t in test_cases.iter() {
            let result = exp_golomb_encode_one(t.0, false, t.2, t.3);
            println!("Expected: {:?}, got: {:?}", t.1, result);
            assert_eq!(result, t.1);
        }
    }
}
