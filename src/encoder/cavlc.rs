//! CAVLC entropy encoding.

use crate::common::cavlc_tables::ENCODE_MAPPED_EXP_GOLOMB_CAT03;
use crate::common::cavlc_tables::ENCODE_MAPPED_EXP_GOLOMB_CAT12;
use crate::common::data_structures::CoeffToken;
use crate::common::data_structures::MacroBlock;
use crate::common::data_structures::MbPartPredMode;
use crate::common::data_structures::MbType;
use crate::common::data_structures::ResidualMode;
use crate::common::data_structures::SliceData;
use crate::common::data_structures::SubMbType;
use crate::common::data_structures::TransformBlock;
use crate::common::data_structures::VideoParameters;
use crate::common::helper::encoder_formatted_print;
use crate::common::helper::is_slice_type;
use crate::encoder::expgolomb;
use log::debug;

use super::binarization_functions::generate_unsigned_binary;

const CAVLC_DEBUG: bool = false;

/// Turns the decoded mb_type number into the actual
/// Type. Based off of Tables 7-11 to 7-14
fn binarize_mb_type(mb_type: MbType, slice_type: u8) -> i32 {
    // some types add a value to the I mb types
    let mut delta = 0;

    if is_slice_type(slice_type, "SI") {
        match mb_type {
            MbType::SI => return 0,
            _ => delta = 1,
        }
    } else if is_slice_type(slice_type, "P") || is_slice_type(slice_type, "SP") {
        // Table 7-13
        match mb_type {
            MbType::PL016x16 => return 0,
            MbType::PL0L016x8 => return 1,
            MbType::PL0L08x16 => return 2,
            MbType::P8x8 => return 3,
            MbType::P8x8ref0 => return 4,
            _ => delta = 5,
        }
    } else if is_slice_type(slice_type, "B") {
        // Table 7-14

        match mb_type {
            MbType::BDirect16x16 => return 0,
            MbType::BL016x16 => return 1,
            MbType::BL116x16 => return 2,
            MbType::BBi16x16 => return 3,
            MbType::BL0L016x8 => return 4,
            MbType::BL0L08x16 => return 5,
            MbType::BL1L116x8 => return 6,
            MbType::BL1L18x16 => return 7,
            MbType::BL0L116x8 => return 8,
            MbType::BL0L18x16 => return 9,
            MbType::BL1L016x8 => return 10,
            MbType::BL1L08x16 => return 11,
            MbType::BL0Bi16x8 => return 12,
            MbType::BL0Bi8x16 => return 13,
            MbType::BL1Bi16x8 => return 14,
            MbType::BL1Bi8x16 => return 15,
            MbType::BBiL016x8 => return 16,
            MbType::BBiL08x16 => return 17,
            MbType::BBiL116x8 => return 18,
            MbType::BBiL18x16 => return 19,
            MbType::BBiBi16x8 => return 20,
            MbType::BBiBi8x16 => return 21,
            MbType::B8x8 => return 22,
            _ => delta = 23,
        }
    }
    // MB Types for intra prediction can be included inside any slice type
    // Table 7-11

    match mb_type {
        MbType::INxN => delta, // no need to offset
        MbType::I16x16_0_0_0 => 1 + delta,
        MbType::I16x16_1_0_0 => 2 + delta,
        MbType::I16x16_2_0_0 => 3 + delta,
        MbType::I16x16_3_0_0 => 4 + delta,
        MbType::I16x16_0_1_0 => 5 + delta,
        MbType::I16x16_1_1_0 => 6 + delta,
        MbType::I16x16_2_1_0 => 7 + delta,
        MbType::I16x16_3_1_0 => 8 + delta,
        MbType::I16x16_0_2_0 => 9 + delta,
        MbType::I16x16_1_2_0 => 10 + delta,
        MbType::I16x16_2_2_0 => 11 + delta,
        MbType::I16x16_3_2_0 => 12 + delta,
        MbType::I16x16_0_0_1 => 13 + delta,
        MbType::I16x16_1_0_1 => 14 + delta,
        MbType::I16x16_2_0_1 => 15 + delta,
        MbType::I16x16_3_0_1 => 16 + delta,
        MbType::I16x16_0_1_1 => 17 + delta,
        MbType::I16x16_1_1_1 => 18 + delta,
        MbType::I16x16_2_1_1 => 19 + delta,
        MbType::I16x16_3_1_1 => 20 + delta,
        MbType::I16x16_0_2_1 => 21 + delta,
        MbType::I16x16_1_2_1 => 22 + delta,
        MbType::I16x16_2_2_1 => 23 + delta,
        MbType::I16x16_3_2_1 => 24 + delta,
        MbType::IPCM => 25 + delta,
        _ => panic!("bad mb_type: {:?}", mb_type),
    }
}

fn binarize_sub_mb_type(sub_mb_type: SubMbType, slice_type: u8) -> i32 {
    if is_slice_type(slice_type, "P") || is_slice_type(slice_type, "SP") {
        match sub_mb_type {
            SubMbType::PL08x8 => 0,
            SubMbType::PL08x4 => 1,
            SubMbType::PL04x8 => 2,
            SubMbType::PL04x4 => 3,
            _ => panic!(
                "binarize_sub_mb_type - Incorrect value provided for P sub mb type: {:?}",
                sub_mb_type
            ),
        }
    } else if is_slice_type(slice_type, "B") {
        match sub_mb_type {
            SubMbType::BDirect8x8 => 0,
            SubMbType::BL08x8 => 1,
            SubMbType::BL18x8 => 2,
            SubMbType::BBi8x8 => 3,
            SubMbType::BL08x4 => 4,
            SubMbType::BL04x8 => 5,
            SubMbType::BL18x4 => 6,
            SubMbType::BL14x8 => 7,
            SubMbType::BBi8x4 => 8,
            SubMbType::BBi4x8 => 9,
            SubMbType::BL04x4 => 10,
            SubMbType::BL14x4 => 11,
            SubMbType::BBi4x4 => 12,
            _ => panic!(
                "binarize_sub_mb_type - Incorrect value provided for B sub mb type: {:?}",
                sub_mb_type
            ),
        }
    } else {
        panic!(
            "binarize_sub_mb_type - Incorrect slice_type provided: {:?}",
            slice_type
        );
    }
}

/// Macroblock Data - mb_type
pub fn cavlc_encode_mb_type(se_val: MbType, slice_type: u8, stream: &mut Vec<u8>) {
    // ue(v)
    let encoded = binarize_mb_type(se_val, slice_type);

    if CAVLC_DEBUG {
        debug!(target: "encode","\t cavlc_encode_mb_type - Se_val is {:?} and the binarized value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("mb_type", se_val, 63);
    }

    stream.append(&mut expgolomb::exp_golomb_encode_one(
        encoded, false, 0, false,
    ));
}

/// Macroblock Data - transform_size_8x8_flag
pub fn cavlc_encode_transform_size_8x8_flag(se_val: bool, stream: &mut Vec<u8>) {
    // u(1)
    let encoded = match se_val {
        true => 1,
        false => 0,
    };
    if CAVLC_DEBUG {
        debug!(target: "encode","\t transform_size_8x8_flag - Se_val is {:?} and the binarized value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("transform_size_8x8_flag", se_val, 63);
    }
    stream.push(encoded);
}

/// Macroblock Data - coded_block_pattern
pub fn cavlc_encode_coded_block_pattern(
    se_val: u32,
    chroma_array_type: u8,
    intra_mode: u8,
    stream: &mut Vec<u8>,
) {
    // uses me(v)
    let mapped: i32;
    // last check is because we treat oob chroma_array_type as 4:2:0
    if chroma_array_type == 1 || chroma_array_type == 2 || chroma_array_type > 4 {
        if se_val > 47 {
            debug!(target: "encode","[WARNING] coded_block_pattern {} is out of bounds - max is 47", se_val);
        }
        mapped = ENCODE_MAPPED_EXP_GOLOMB_CAT12[(se_val % 48) as usize][intra_mode as usize];
    } else {
        // if chroma_array_type == 0 || chroma_array_type == 3 {
        if se_val > 15 {
            debug!(target: "encode","[WARNING] coded_block_pattern {} is out of bounds - max is 15", se_val);
        }
        mapped = ENCODE_MAPPED_EXP_GOLOMB_CAT03[(se_val % 16) as usize][intra_mode as usize];
    } //else {
      //panic!("Wrong chroma_array_type: {}", chroma_array_type);
      //}

    let mut encoded = expgolomb::exp_golomb_encode_one(mapped, false, 0, false);
    if CAVLC_DEBUG {
        debug!(target: "encode","\t coded_block_pattern - Se_val is {:?} and the binarized value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("coded_block_pattern", se_val, 63);
    }

    stream.append(&mut encoded);
}

/// Macroblock Data - mb_qp_delta
pub fn cavlc_encode_mb_qp_delta(se_val: i32, stream: &mut Vec<u8>) {
    // se(v)
    let mut encoded = expgolomb::exp_golomb_encode_one(se_val, true, 0, false);
    if CAVLC_DEBUG {
        debug!(target: "encode","\t mb_qp_delta - Se_val is {:?} and the binarized value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("mb_qp_delta", se_val, 63);
    }
    stream.append(&mut encoded);
}

/// Macroblock Prediction - intra4x4_pred_mode_flag
pub fn cavlc_encode_prev_intra4x4_pred_mode_flag(se_val: bool, stream: &mut Vec<u8>) {
    // u(1)
    let encoded = match se_val {
        true => 1,
        false => 0,
    };
    if CAVLC_DEBUG {
        debug!(target: "encode","\t prev_intra4x4_pred_mode_flag - Se_val is {:?} and the binarized value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("prev_intra4x4_pred_mode_flag", se_val, 63);
    }
    stream.push(encoded);
}

/// Macroblock Prediction - intra4x4_pred_mode
pub fn cavlc_encode_rem_intra4x4_pred_mode(se_val: u32, stream: &mut Vec<u8>) {
    // u(3)
    let mut encoded: Vec<u8> = generate_unsigned_binary(se_val, 3);
    if CAVLC_DEBUG {
        debug!(target: "encode","\t rem_intra4x4_pred_mod - Se_val is {:?} and the binarized value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("rem_intra4x4_pred_mode", se_val, 63);
    }
    stream.append(&mut encoded);
}

/// Macroblock Prediction - intra8x8_pred_mode_flag
pub fn cavlc_encode_prev_intra8x8_pred_mode_flag(se_val: bool, stream: &mut Vec<u8>) {
    // u(1)
    let encoded = match se_val {
        true => 1,
        false => 0,
    };
    if CAVLC_DEBUG {
        debug!(target: "encode","\t prev_intra8x8_pred_mode_flag - Se_val is {:?} and the binarized value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("prev_intra8x8_pred_mode_flag", se_val, 63);
    }
    stream.push(encoded);
}

/// Macroblock Prediction - intra8x8_pred_mode
pub fn cavlc_encode_rem_intra8x8_pred_mode(se_val: u32, stream: &mut Vec<u8>) {
    // u(3)
    let mut encoded: Vec<u8> = generate_unsigned_binary(se_val, 3);

    if CAVLC_DEBUG {
        debug!(target: "encode","\t rem_intra8x8_pred_mod - Se_val is {:?} and the binarized value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("rem_intra8x8_pred_mode", se_val, 63);
    }
    stream.append(&mut encoded);
}

/// Macroblock Prediction - intra_chroma_pred_mode
pub fn cavlc_encode_intra_chroma_pred_mode(se_val: u8, stream: &mut Vec<u8>) {
    // ue(v)
    let mut encoded = expgolomb::exp_golomb_encode_one(se_val as i32, false, 0, false);
    if CAVLC_DEBUG {
        debug!(target: "encode","\t intra_chroma_pred_mode - Se_val is {:?} and the binarized value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("intra_chroma_pred_mode", se_val, 63);
    }
    stream.append(&mut encoded);
}

/// Macroblock Prediction - ref_idx
pub fn cavlc_encode_ref_idx(se_val: u32, max_value: u32, stream: &mut Vec<u8>) {
    // te(v)
    if max_value == 1 {
        match se_val {
            0 => stream.push(1),
            1 => stream.push(0),
            _ => {
                debug!(target: "encode","[WARNING] cavlc_encode_ref_idx - bad value for se_val {} with max_value {}", se_val, max_value);
                debug!(target: "encode","[WARNING]                        defaulting to value 0");
                stream.push(1);
            }
        }
        if !CAVLC_DEBUG {
            encoder_formatted_print("ref_idx_l[][]", se_val, 63);
        }
    } else {
        let mut encoded = expgolomb::exp_golomb_encode_one(se_val as i32, false, 0, false);
        if CAVLC_DEBUG {
            debug!(target: "encode","\t ref_idx - Se_val is {:?} and the binarized value is {:?}", se_val, encoded);
        } else {
            encoder_formatted_print("ref_idx_l[][]", se_val, 63);
        }
        stream.append(&mut encoded);
    }
}

/// Macroblock Prediction - mvd
pub fn cavlc_encode_mvd(se_val: i32, stream: &mut Vec<u8>) {
    // se(v)
    let mut encoded = expgolomb::exp_golomb_encode_one(se_val, true, 0, false);
    if CAVLC_DEBUG {
        debug!(target: "encode","\t encode_mvd - Se_val is {:?} and the binarized value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("mvd_l[][]", se_val, 63);
    }
    stream.append(&mut encoded);
}

/// Macroblock Data - sub_mb_type
pub fn cavlc_encode_sub_mb_type(se_val: SubMbType, slice_type: u8, stream: &mut Vec<u8>) {
    // ue(v)
    let encoded = binarize_sub_mb_type(se_val, slice_type);
    if CAVLC_DEBUG {
        debug!(target: "encode","\t encode_sub_mb_type - Se_val is {:?} and the binarized value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("sub_mb_type", se_val, 63);
    }
    stream.append(&mut expgolomb::exp_golomb_encode_one(
        encoded, false, 0, false,
    ));
}

/// Macroblock Residual Data - coeff_token
pub fn cavlc_encode_coeff_token(
    se_val: &CoeffToken,
    cur_blk_idx: usize,
    i_cb_cr: usize,
    residual_mode: ResidualMode,
    curr_mb_idx: usize,
    sd: &SliceData,
    vp: &VideoParameters,
    stream: &mut Vec<u8>,
) {
    // ce(v)

    let n_c: i8;
    let mut blk_idx = cur_blk_idx;
    if residual_mode == ResidualMode::ChromaDCLevel {
        if vp.chroma_array_type == 1 {
            n_c = -1;
        } else {
            n_c = -2;
        }
    } else {
        // step 1, 2, and 3
        if residual_mode == ResidualMode::Intra16x16DCLevel
            || residual_mode == ResidualMode::CbIntra16x16DCLevel
            || residual_mode == ResidualMode::CrIntra16x16DCLevel
        {
            blk_idx = 0;
        }

        if CAVLC_DEBUG {
            debug!(target: "encode","blk_idx - {}", blk_idx);
        }

        let mut mb_a: MacroBlock = MacroBlock::new();
        let mut mb_b: MacroBlock = MacroBlock::new();
        let mut blk_idx_a: usize = 0;
        let mut blk_idx_b: usize = 0;
        let mut blk_a: TransformBlock = TransformBlock::new();
        let mut blk_b: TransformBlock = TransformBlock::new();

        // 0 - luma mode; 1 - cb chroma_mode; 2 - cr chroma mode; 3 - chroma ac mode
        let mut ac_mode = 0;

        // step 4
        match residual_mode {
            ResidualMode::Intra16x16DCLevel
            | ResidualMode::Intra16x16ACLevel
            | ResidualMode::LumaLevel4x4 => {
                let res = sd.get_neighbor_4x4_luma_block(curr_mb_idx, true, blk_idx, vp);
                mb_a = res.0;
                mb_b = res.1;
                blk_idx_a = res.2;
                blk_idx_b = res.3;

                if mb_a.available && (mb_a.coded_block_pattern_luma >> (blk_idx_a >> 2)) & 1 != 0 {
                    if mb_a.mb_part_pred_mode(0) == MbPartPredMode::Intra16x16 {
                        blk_a = mb_a.intra_16x16_ac_level_transform_blocks[blk_idx_a].clone();
                    } else {
                        blk_a = mb_a.luma_level_4x4_transform_blocks[blk_idx_a].clone();
                    }
                } else {
                    blk_a = TransformBlock::new();
                }

                if mb_b.available && (mb_b.coded_block_pattern_luma >> (blk_idx_b >> 2)) & 1 != 0 {
                    if mb_b.mb_part_pred_mode(0) == MbPartPredMode::Intra16x16 {
                        blk_b = mb_b.intra_16x16_ac_level_transform_blocks[blk_idx_b].clone();
                    } else {
                        blk_b = mb_b.luma_level_4x4_transform_blocks[blk_idx_b].clone();
                    }
                } else {
                    blk_b = TransformBlock::new();
                }

                ac_mode = 0;
            }
            ResidualMode::CbIntra16x16DCLevel
            | ResidualMode::CbIntra16x16ACLevel
            | ResidualMode::CbLevel4x4 => {
                let _res = sd.get_neighbor_4x4_cr_cb_blocks_info(curr_mb_idx, blk_idx, vp);
                //mb_a = res.0;
                //mb_b = res.1;
                //blk_idx_a = res.2;
                //blk_idx_b = res.3;

                // TODO: CAVLC 422/444
                panic!("sd.get_neighbor_4x4_cr_cb_blocks_info(curr_mb_idx, blk_idx, vp); - no TransformBlock yet");

                //ac_mode = 1;
            }
            ResidualMode::CrIntra16x16DCLevel
            | ResidualMode::CrIntra16x16ACLevel
            | ResidualMode::CrLevel4x4 => {
                let _res = sd.get_neighbor_4x4_cr_cb_blocks_info(curr_mb_idx, blk_idx, vp);
                //mb_a = res.0;
                //mb_b = res.1;
                //blk_idx_a = res.2;
                //blk_idx_b = res.3;

                // TODO: CAVLC 422/444
                panic!("sd.get_neighbor_4x4_cr_cb_blocks_info(curr_mb_idx, blk_idx, vp); - no TransformBlock yet");

                //ac_mode = 2;
            }
            ResidualMode::ChromaACLevel => {
                let res = sd.get_neighbor_4x4_chroma_block(curr_mb_idx, blk_idx, vp);

                mb_a = res.0;
                mb_b = res.1;
                blk_idx_a = res.2;
                blk_idx_b = res.3;

                if mb_a.available && mb_a.coded_block_pattern_chroma == 2 {
                    blk_a = mb_a.chroma_ac_level_transform_blocks[i_cb_cr][blk_idx_a].clone();
                } else {
                    blk_a = TransformBlock::new();
                }

                if mb_b.available && mb_b.coded_block_pattern_chroma == 2 {
                    blk_b = mb_b.chroma_ac_level_transform_blocks[i_cb_cr][blk_idx_b].clone();
                } else {
                    blk_b = TransformBlock::new();
                }

                ac_mode = 3;
            }
            _ => (),
        }
        if CAVLC_DEBUG {
            debug!(target: "encode","blk_idx_a {}, blk_idx_b {}", blk_idx_a, blk_idx_b);
        }
        // step 5
        let mut available_flag_a: bool = true;
        let mut available_flag_b: bool = true;

        if !mb_a.available
            || (sd.macroblock_vec[curr_mb_idx].is_intra_non_mut()
                && vp.pps_constrained_intra_pred_flag
                && mb_a.is_inter()
                && 2 <= vp.nal_unit_type
                && vp.nal_unit_type <= 4)
        {
            available_flag_a = false;
        }

        if !mb_b.available
            || (sd.macroblock_vec[curr_mb_idx].is_intra_non_mut()
                && vp.pps_constrained_intra_pred_flag
                && mb_b.is_inter()
                && 2 <= vp.nal_unit_type
                && vp.nal_unit_type <= 4)
        {
            available_flag_b = false;
        }

        // step 6
        let mut n_a = 0;
        let mut n_b = 0;

        if CAVLC_DEBUG {
            debug!(target: "encode","available_flag_a - {}; available_flag_b - {}", available_flag_a, available_flag_b);
        }

        if available_flag_a {
            if mb_a.mb_type == MbType::PSkip
                || mb_a.mb_type == MbType::BSkip
                || (mb_a.mb_type != MbType::IPCM && mb_a.ac_resid_all_zero(ac_mode, blk_idx_a))
            {
                if CAVLC_DEBUG {
                    debug!(target: "encode","mb_a.ac_resid_all_zero(ac_mode): {}", mb_a.ac_resid_all_zero(ac_mode, blk_idx_a));
                }
                n_a = 0;
            } else if mb_a.mb_type == MbType::IPCM {
                if CAVLC_DEBUG {
                    debug!(target: "encode","mb_a.mb_type == MbType::IPCM");
                }
                n_a = 16;
            } else {
                n_a = blk_a.coeff_token.total_coeff;
                if CAVLC_DEBUG {
                    debug!(target: "encode","n_a - using previous value - {}", n_a);
                }
            }
        }

        if available_flag_b {
            if mb_b.mb_type == MbType::PSkip
                || mb_b.mb_type == MbType::BSkip
                || (mb_b.mb_type != MbType::IPCM && mb_b.ac_resid_all_zero(ac_mode, blk_idx_b))
            {
                if CAVLC_DEBUG {
                    debug!(target: "encode","mb_b.ac_resid_all_zero(ac_mode): {}", mb_b.ac_resid_all_zero(ac_mode, blk_idx_b));
                }
                n_b = 0;
            } else if mb_b.mb_type == MbType::IPCM {
                if CAVLC_DEBUG {
                    debug!(target: "encode","mb_b.mb_type == MbType::IPCM");
                }
                n_b = 16;
            } else {
                n_b = blk_b.coeff_token.total_coeff;
                if CAVLC_DEBUG {
                    debug!(target: "encode","n_b - using previous value - {}", n_b);
                }
            }
        }

        // step 7
        if available_flag_a && available_flag_b {
            n_c = ((n_a + n_b + 1) >> 1) as i8;
        } else if available_flag_a {
            n_c = n_a as i8;
        } else if available_flag_b {
            n_c = n_b as i8;
        } else {
            n_c = 0;
        }
    }

    if CAVLC_DEBUG {
        debug!(target: "encode","using n_c value - {}", n_c);
    }

    let mut encoded: Vec<u8> = Vec::new();

    if (0..2).contains(&n_c) {
        match se_val.total_coeff {
            0 => match se_val.trailing_ones {
                0 => {
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            1 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            2 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            3 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }

                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            4 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }

                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            5 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }

                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            6 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }

                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            7 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }

                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            8 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }

                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            9 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }

                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            10 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }

                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            11 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }

                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            12 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }

                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            13 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }

                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            14 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }

                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            15 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }

                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            16 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            _ => panic!(
                "bad n_c {} and total_coeff {} combination",
                n_c, se_val.total_coeff
            ),
        }
    } else if (2..4).contains(&n_c) {
        match se_val.total_coeff {
            0 => match se_val.trailing_ones {
                0 => {
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            1 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(1);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            2 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            3 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            4 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            5 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            6 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            7 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            8 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            9 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            10 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            11 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            12 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            13 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            14 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            15 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            16 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            _ => panic!(
                "bad n_c {} and total_coeff {} combination",
                n_c, se_val.total_coeff
            ),
        }
    } else if (4..8).contains(&n_c) {
        match se_val.total_coeff {
            0 => match se_val.trailing_ones {
                0 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            1 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            2 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            3 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            4 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            5 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            6 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            7 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            8 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            9 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            10 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            11 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            12 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            13 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            14 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            15 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            16 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            _ => panic!(
                "bad n_c {} and total_coeff {} combination",
                n_c, se_val.total_coeff
            ),
        }
    } else if 8 <= n_c {
        match se_val.total_coeff {
            0 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            1 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            2 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            3 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            4 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            5 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            6 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            7 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            8 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            9 => match se_val.trailing_ones {
                0 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            10 => match se_val.trailing_ones {
                0 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            11 => match se_val.trailing_ones {
                0 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            12 => match se_val.trailing_ones {
                0 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            13 => match se_val.trailing_ones {
                0 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            14 => match se_val.trailing_ones {
                0 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            15 => match se_val.trailing_ones {
                0 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            16 => match se_val.trailing_ones {
                0 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            _ => panic!(
                "bad n_c {} and total_coeff {} combination",
                n_c, se_val.total_coeff
            ),
        }
    } else if n_c == -1 {
        match se_val.total_coeff {
            0 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            1 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            2 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            3 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            4 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            _ => panic!(
                "bad n_c {} and total_coeff {} combination",
                n_c, se_val.total_coeff
            ),
        }
    } else if n_c == -2 {
        match se_val.total_coeff {
            0 => match se_val.trailing_ones {
                0 => {
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            1 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            2 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            3 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            4 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            5 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            6 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            7 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            8 => match se_val.trailing_ones {
                0 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "bad n_c {} total_coeff {} and trailing_ones {} combination",
                    n_c, se_val.total_coeff, se_val.trailing_ones
                ),
            },
            _ => panic!(
                "bad n_c {} and total_coeff {} combination",
                n_c, se_val.total_coeff
            ),
        }
    } else {
        panic!("Wrong n_c value calculated: {}", n_c);
    }

    if CAVLC_DEBUG {
        debug!(target: "encode","\t encode_coeff_token - Se_val is {:?} and encoded value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("coeff_token", se_val, 63);
    }

    stream.append(&mut encoded);
}

/// Macroblock Residual Data - trailing_ones_sign_flag
pub fn cavlc_encode_trailing_ones_sign_flag(se_val: bool, stream: &mut Vec<u8>) {
    // u(1)
    let encoded = match se_val {
        true => 1,
        false => 0,
    };
    if CAVLC_DEBUG {
        debug!(target: "encode","\t trailing_ones_sign_flag - Se_val is {:?} and the binarized value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("trailing_ones_sign_flag", se_val, 63);
    }
    stream.push(encoded);
}

/// Macroblock Residual Data - level_prefix
pub fn cavlc_encode_level_prefix(se_val: u32, stream: &mut Vec<u8>) {
    // ce(v)
    // se_val is the number of zeros in the bitstream, with a 1 at the end
    let mut encoded: Vec<u8> = vec![0; se_val as usize];
    encoded.push(1);
    if CAVLC_DEBUG {
        debug!(target: "encode","\t level_prefix - Se_val is {:?} and the binarized value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("level_prefix", se_val, 63);
    }
    stream.append(&mut encoded);
}

/// Macroblock Residual Data - level_suffix
pub fn cavlc_encode_level_suffix(
    se_val: u32,
    suffix_length: u32,
    level_prefix: u32,
    stream: &mut Vec<u8>,
) {
    // u(v)

    let level_suffix_size: u32;

    if level_prefix == 14 && suffix_length == 0 {
        level_suffix_size = 4;
    } else if level_prefix >= 15 {
        level_suffix_size = level_prefix - 3;
    } else {
        level_suffix_size = suffix_length;
    }
    if CAVLC_DEBUG {
        debug!(target: "encode","\t level_suffix - level_suffix_size is {:?}", level_suffix_size);
    }

    let mut encoded = generate_unsigned_binary(se_val, level_suffix_size as usize);

    if CAVLC_DEBUG {
        debug!(target: "encode","\t level_suffix - Se_val is {:?} and encoded value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("level_suffix_size", level_suffix_size, 63);
        encoder_formatted_print("level_suffix", se_val, 63);
    }

    stream.append(&mut encoded);
}

/// Macroblock Residual Data - total_zeros
pub fn cavlc_encode_total_zeros(
    se_val: usize,
    max_num_coeff: usize,
    tz_vcl_index: usize,
    stream: &mut Vec<u8>,
) {
    // ce(v)

    let mut encoded: Vec<u8> = Vec::new();

    if max_num_coeff == 4 {
        // use Table 9-9 (a)
        match tz_vcl_index {
            1 =>{
                match se_val {
                    0 => {encoded.push(1); },
                    1 => {encoded.push(0); encoded.push(1); },
                    2 => {encoded.push(0); encoded.push(0); encoded.push(1);},
                    3 => {encoded.push(0); encoded.push(0); encoded.push(0);},
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            2 => {
                match se_val {
                    0 => {encoded.push(1); },
                    1 => {encoded.push(0); encoded.push(1); },
                    2 => {encoded.push(0); encoded.push(0); },
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            3 => {
                match se_val {
                    0 => {encoded.push(1); },
                    1 => {encoded.push(0); },
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            _ => panic!("Bad tz_vcl_index value: {}", tz_vcl_index)
        }
    } else if max_num_coeff == 8 {
        // use Table 9-9 (b)
        match tz_vcl_index {
            1 =>{
                match se_val {
                    0 => {encoded.push(1);  },
                    1 => {encoded.push(0); encoded.push(1); encoded.push(0);  },
                    2 => {encoded.push(0); encoded.push(1); encoded.push(1);  },
                    3 => {encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(0); },
                    4 => {encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(1); },
                    5 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); },
                    6 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);},
                    7 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0);},
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            2 => {
                match se_val {
                    0 => {encoded.push(0); encoded.push(0); encoded.push(0);},
                    1 => {encoded.push(0); encoded.push(1); },
                    2 => {encoded.push(0); encoded.push(0); encoded.push(1);},
                    3 => {encoded.push(1); encoded.push(0); encoded.push(0);},
                    4 => {encoded.push(1); encoded.push(0); encoded.push(1);},
                    5 => {encoded.push(1); encoded.push(1); encoded.push(0);},
                    6 => {encoded.push(1); encoded.push(1); encoded.push(1);},
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            3 => {
                match se_val {
                    0 => {encoded.push(0); encoded.push(0); encoded.push(0);},
                    1 => {encoded.push(0); encoded.push(0); encoded.push(1);},
                    2 => {encoded.push(0); encoded.push(1);  },
                    3 => {encoded.push(1); encoded.push(0);  },
                    4 => {encoded.push(1); encoded.push(1); encoded.push(0);},
                    5 => {encoded.push(1); encoded.push(1); encoded.push(1);},
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            4 => {
                match se_val {
                    0 => {encoded.push(1); encoded.push(1); encoded.push(0);}
                    1 => {encoded.push(0); encoded.push(0); }
                    2 => {encoded.push(0); encoded.push(1); }
                    3 => {encoded.push(1); encoded.push(0); }
                    4 => {encoded.push(1); encoded.push(1); encoded.push(1);}
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            5 => {
                match se_val {
                    0 => {encoded.push(0); encoded.push(0);},
                    1 => {encoded.push(0); encoded.push(1);},
                    2 => {encoded.push(1); encoded.push(0);},
                    3 => {encoded.push(1); encoded.push(1);},
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            6 => {
                match se_val {
                    0 => {encoded.push(0); encoded.push(0);},
                    1 => {encoded.push(0); encoded.push(1);},
                    2 => {encoded.push(1); },
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            7 => {
                match se_val {
                    0 => {encoded.push(0);},
                    1 => {encoded.push(1);},
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            _ => panic!("Bad tz_vcl_index value: {}", tz_vcl_index)
        }
    } else {
        // use tables 9-7 and 9-8
        match tz_vcl_index {
            // table 9-7
            1 =>{
                match se_val {
                    0  => {encoded.push(1);        },
                    1  => {encoded.push(0); encoded.push(1); encoded.push(1);      },
                    2  => {encoded.push(0); encoded.push(1); encoded.push(0);      },
                    3  => {encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(1);     },
                    4  => {encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(0);     },
                    5  => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(1);    },
                    6  => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(0);    },
                    7  => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(1);   },
                    8  => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(0);   },
                    9  => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(1);  },
                    10 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(0);  },
                    11 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(1); },
                    12 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(0); },
                    13 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(1);},
                    14 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(0);},
                    15 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);},
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }


            },
            2 => {
                match se_val {
                    0 =>  {encoded.push(1); encoded.push(1); encoded.push(1);   },
                    1 =>  {encoded.push(1); encoded.push(1); encoded.push(0);   },
                    2 =>  {encoded.push(1); encoded.push(0); encoded.push(1);   },
                    3 =>  {encoded.push(1); encoded.push(0); encoded.push(0);   },
                    4 =>  {encoded.push(0); encoded.push(1); encoded.push(1);   },
                    5 =>  {encoded.push(0); encoded.push(1); encoded.push(0); encoded.push(1);  },
                    6 =>  {encoded.push(0); encoded.push(1); encoded.push(0); encoded.push(0);  },
                    7 =>  {encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(1);  },
                    8 =>  {encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(0);  },
                    9 =>  {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(1); },
                    10 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(0); },
                    11 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(1);},
                    12 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(0);},
                    13 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);},
                    14 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0);},
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }

            },
            3 => {
                match se_val {
                    0 =>  {encoded.push(0); encoded.push(1); encoded.push(0); encoded.push(1);  },
                    1 =>  {encoded.push(1); encoded.push(1); encoded.push(1);   },
                    2 =>  {encoded.push(1); encoded.push(1); encoded.push(0);   },
                    3 =>  {encoded.push(1); encoded.push(0); encoded.push(1);   },
                    4 =>  {encoded.push(0); encoded.push(1); encoded.push(0); encoded.push(0);  },
                    5 =>  {encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(1);  },
                    6 =>  {encoded.push(1); encoded.push(0); encoded.push(0);   },
                    7 =>  {encoded.push(0); encoded.push(1); encoded.push(1);   },
                    8 =>  {encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(0);  },
                    9 =>  {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(1); },
                    10 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(0); },
                    11 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);},
                    12 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); },
                    13 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0);},
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }

            },
            4 => {
                match se_val {
                    0 =>  {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(1);},
                    1 =>  {encoded.push(1); encoded.push(1); encoded.push(1);  },
                    2 =>  {encoded.push(0); encoded.push(1); encoded.push(0); encoded.push(1); },
                    3 =>  {encoded.push(0); encoded.push(1); encoded.push(0); encoded.push(0); },
                    4 =>  {encoded.push(1); encoded.push(1); encoded.push(0);  },
                    5 =>  {encoded.push(1); encoded.push(0); encoded.push(1);  },
                    6 =>  {encoded.push(1); encoded.push(0); encoded.push(0);  },
                    7 =>  {encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(1); },
                    8 =>  {encoded.push(0); encoded.push(1); encoded.push(1);  },
                    9 =>  {encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(0); },
                    10 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(0);},
                    11 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);},
                    12 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0);},
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }

            },
            5 => {
                match se_val {
                    0 =>  {encoded.push(0); encoded.push(1); encoded.push(0); encoded.push(1); },
                    1 =>  {encoded.push(0); encoded.push(1); encoded.push(0); encoded.push(0); },
                    2 =>  {encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(1); },
                    3 =>  {encoded.push(1); encoded.push(1); encoded.push(1);  },
                    4 =>  {encoded.push(1); encoded.push(1); encoded.push(0);  },
                    5 =>  {encoded.push(1); encoded.push(0); encoded.push(1);  },
                    6 =>  {encoded.push(1); encoded.push(0); encoded.push(0);  },
                    7 =>  {encoded.push(0); encoded.push(1); encoded.push(1);  },
                    8 =>  {encoded.push(0); encoded.push(0); encoded.push(1); encoded.push(0); },
                    9 =>  {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);},
                    10 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); },
                    11 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0);},
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            6 => {
                match se_val {
                    0 =>  {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);},
                    1 =>  {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); },
                    2 =>  {encoded.push(1); encoded.push(1); encoded.push(1);   },
                    3 =>  {encoded.push(1); encoded.push(1); encoded.push(0);   },
                    4 =>  {encoded.push(1); encoded.push(0); encoded.push(1);   },
                    5 =>  {encoded.push(1); encoded.push(0); encoded.push(0);   },
                    6 =>  {encoded.push(0); encoded.push(1); encoded.push(1);   },
                    7 =>  {encoded.push(0); encoded.push(1); encoded.push(0);   },
                    8 =>  {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);  },
                    9 =>  {encoded.push(0); encoded.push(0); encoded.push(1);   },
                    10 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0);},
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }

            },
            7 => {
                match se_val {
                    0 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);},
                    1 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); },
                    2 => {encoded.push(1); encoded.push(0); encoded.push(1);   },
                    3 => {encoded.push(1); encoded.push(0); encoded.push(0);   },
                    4 => {encoded.push(0); encoded.push(1); encoded.push(1);   },
                    5 => {encoded.push(1); encoded.push(1);    },
                    6 => {encoded.push(0); encoded.push(1); encoded.push(0);   },
                    7 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);  },
                    8 => {encoded.push(0); encoded.push(0); encoded.push(1);   },
                    9 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0);},
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            // Table 9-8
            8 => {
                match se_val {
                    0 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);},
                    1 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);  },
                    2 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); },
                    3 => {encoded.push(0); encoded.push(1); encoded.push(1);   },
                    4 => {encoded.push(1); encoded.push(1);    },
                    5 => {encoded.push(1); encoded.push(0);    },
                    6 => {encoded.push(0); encoded.push(1); encoded.push(0);   },
                    7 => {encoded.push(0); encoded.push(0); encoded.push(1);   },
                    8 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0);},
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            9 => {
                match se_val {
                    0 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);},
                    1 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0);},
                    2 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);  },
                    3 => {encoded.push(1); encoded.push(1);    },
                    4 => {encoded.push(1); encoded.push(0);    },
                    5 => {encoded.push(0); encoded.push(0); encoded.push(1);   },
                    6 => {encoded.push(0); encoded.push(1);    },
                    7 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); },
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            10 => {
                match se_val {
                    0 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);},
                    1 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0);},
                    2 => {encoded.push(0); encoded.push(0); encoded.push(1);  },
                    3 => {encoded.push(1); encoded.push(1);   },
                    4 => {encoded.push(1); encoded.push(0);   },
                    5 => {encoded.push(0); encoded.push(1);   },
                    6 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1); },
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            11 => {
                match se_val {
                    0 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0);},
                    1 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);},
                    2 => {encoded.push(0); encoded.push(0); encoded.push(1); },
                    3 => {encoded.push(0); encoded.push(1); encoded.push(0); },
                    4 => {encoded.push(1);   },
                    5 => {encoded.push(0); encoded.push(1); encoded.push(1); },
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            12 => {
                match se_val {
                    0 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(0);},
                    1 => {encoded.push(0); encoded.push(0); encoded.push(0); encoded.push(1);},
                    2 => {encoded.push(0); encoded.push(1); },
                    3 => {encoded.push(1); },
                    4 => {encoded.push(0); encoded.push(0); encoded.push(1);},
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            13 => {
                match se_val {
                    0 => {encoded.push(0); encoded.push(0); encoded.push(0);},
                    1 => {encoded.push(0); encoded.push(0); encoded.push(1);},
                    2 => {encoded.push(1); },
                    3 => {encoded.push(0); encoded.push(1); },
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            14 => {
                match se_val {
                    0 => {encoded.push(0); encoded.push(0);},
                    1 => {encoded.push(0); encoded.push(1);},
                    2 => {encoded.push(1); },
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            15 => {
                match se_val {
                    0 => encoded.push(0),
                    1 => encoded.push(1),
                    _ => panic!("cavlc_encode_total_zeros - bad max_num_coeff {} and tz_vcl_index {} and total_zeros {}", max_num_coeff, tz_vcl_index, se_val),
                }
            },
            _ => panic!("Bad tz_vcl_index value: {}", tz_vcl_index)
        }
    }

    if CAVLC_DEBUG {
        debug!(target: "encode","\t total_zeros - Se_val is {:?} and encoded value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("total_zeros", se_val, 63);
    }
    stream.append(&mut encoded);
}

/// Macroblock Residual Data - run_before
pub fn cavlc_encode_run_before(se_val: usize, zeros_left: i32, stream: &mut Vec<u8>) {
    // ce(v)

    let mut encoded: Vec<u8> = Vec::new();

    if zeros_left > 6 {
        match se_val {
            0 => {
                encoded.push(1);
                encoded.push(1);
                encoded.push(1);
            }
            1 => {
                encoded.push(1);
                encoded.push(1);
                encoded.push(0);
            }
            2 => {
                encoded.push(1);
                encoded.push(0);
                encoded.push(1);
            }
            3 => {
                encoded.push(1);
                encoded.push(0);
                encoded.push(0);
            }
            4 => {
                encoded.push(0);
                encoded.push(1);
                encoded.push(1);
            }
            5 => {
                encoded.push(0);
                encoded.push(1);
                encoded.push(0);
            }
            6 => {
                encoded.push(0);
                encoded.push(0);
                encoded.push(1);
            }
            7 => {
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(1);
            }
            8 => {
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(1);
            }
            9 => {
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(1);
            }
            10 => {
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(1);
            }
            11 => {
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(1);
            }
            12 => {
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(1);
            }
            13 => {
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(1);
            }
            14 => {
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(0);
                encoded.push(1);
            }
            _ => panic!(
                "cavlc_encode_run_before - bad values of zeros_left {} and run_before {}",
                zeros_left, se_val
            ),
        }
    } else {
        match zeros_left {
            1 => match se_val {
                0 => encoded.push(1),
                1 => encoded.push(0),
                _ => panic!(
                    "cavlc_encode_run_before - bad values of zeros_left {} and run_before {}",
                    zeros_left, se_val
                ),
            },
            2 => match se_val {
                0 => {
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(1);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "cavlc_encode_run_before - bad values of zeros_left {} and run_before {}",
                    zeros_left, se_val
                ),
            },
            3 => match se_val {
                0 => {
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "cavlc_encode_run_before - bad values of zeros_left {} and run_before {}",
                    zeros_left, se_val
                ),
            },
            4 => match se_val {
                0 => {
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                4 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "cavlc_encode_run_before - bad values of zeros_left {} and run_before {}",
                    zeros_left, se_val
                ),
            },
            5 => match se_val {
                0 => {
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(1);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                4 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                5 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "cavlc_encode_run_before - bad values of zeros_left {} and run_before {}",
                    zeros_left, se_val
                ),
            },
            6 => match se_val {
                0 => {
                    encoded.push(1);
                    encoded.push(1);
                }
                1 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(0);
                }
                2 => {
                    encoded.push(0);
                    encoded.push(0);
                    encoded.push(1);
                }
                3 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(1);
                }
                4 => {
                    encoded.push(0);
                    encoded.push(1);
                    encoded.push(0);
                }
                5 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(1);
                }
                6 => {
                    encoded.push(1);
                    encoded.push(0);
                    encoded.push(0);
                }
                _ => panic!(
                    "cavlc_encode_run_before - bad values of zeros_left {} and run_before {}",
                    zeros_left, se_val
                ),
            },
            _ => {
                panic!("Bad zeros_left value: {}", zeros_left);
            }
        }
    }

    if CAVLC_DEBUG {
        debug!(target: "encode","\t run_before - Se_val is {:?} and encoded value is {:?}", se_val, encoded);
    } else {
        encoder_formatted_print("run_before", se_val, 63);
    }

    stream.append(&mut encoded);
}
