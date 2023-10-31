//! CABAC entropy decoding.

use crate::common::cabac_tables;
use crate::common::data_structures::MacroBlock;
use crate::common::data_structures::MbPartPredMode;
use crate::common::data_structures::MbType;
use crate::common::data_structures::NeighborMB;
use crate::common::data_structures::SliceData;
use crate::common::data_structures::SliceHeader;
use crate::common::data_structures::TransformBlock;
use crate::common::data_structures::VideoParameters;
use crate::common::helper::clip3;
use crate::common::helper::is_slice_type;
use crate::common::helper::ByteStream;
use crate::decoder::binarization_functions::*;
use log::debug;
use std::cmp;

// Enable below for cabac state printing
const CABAC_DEBUG: bool = false;

/// Maintain the probability state index and the most probable state
#[derive(Copy, Clone)]
pub struct SyntaxElementState {
    pub p_state_idx: u32, // corresponds to the probability state index
    pub val_mps: u8,      // corresponds to the value of the most probably symbol
}

impl SyntaxElementState {
    fn new(p_state_idx: u32, val_mps: u8) -> SyntaxElementState {
        SyntaxElementState {
            p_state_idx,
            val_mps,
        }
    }
}

/// The entire CABAC state maintained across a slice
pub struct CABACState {
    pub states: Vec<Vec<Vec<SyntaxElementState>>>,
    pub cod_i_range: u32,
    pub cod_i_offset: u32,
}

impl CABACState {
    pub fn new() -> CABACState {
        CABACState {
            states: Vec::new(),
            cod_i_range: 0,
            cod_i_offset: 0,
        }
    }
}

impl Default for CABACState {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the CABAC State
///
/// Load up all possible ctxIdx values into the state. Similar
/// process to OpenH264.
pub fn initialize_state(bs: &mut ByteStream) -> CABACState {
    let mut r = CABACState::new();

    for i in 0..4 {
        // the different cabac_init_idc values
        r.states.push(Vec::new());
        for j in 0..52 {
            // the possible slice_qp_y values
            r.states[i as usize].push(Vec::new());
            for k in 0..cabac_tables::CONTEXT_MODEL_COUNT {
                // cabac_tables::CONTEXT_MODEL_COUNT
                // use slice_qp_y to assign pStateIdx and valMPS values
                let p_state_idx: u32;
                let val_mps: u8;

                let m = cabac_tables::CABAC_INIT_CONSTANTS[k][i][0]; // look up from cabac_table
                let n = cabac_tables::CABAC_INIT_CONSTANTS[k][i][1]; // look up from cabac_table

                let pre_ctx_state = clip3(1, 126, ((m * j) >> 4) + n) as u32;
                if pre_ctx_state <= 63 {
                    p_state_idx = 63 - pre_ctx_state;
                    val_mps = 0;
                } else {
                    p_state_idx = pre_ctx_state - 64;
                    val_mps = 1;
                }

                r.states[i as usize][j as usize]
                    .push(SyntaxElementState::new(p_state_idx, val_mps));
            }
        }
    }

    r.cod_i_range = 510;
    r.cod_i_offset = bs.read_bits(9);

    r
}

/// Described in section 9.3.2 of the spec
///
/// Input to this process is a request for a syntax element, ctx_block_category and slice header information
/// Output of this process is the maxBinIdxCtx, ctxIdxOffset, and bypass flag
fn get_binarization_params(
    syntax_element: &str,
    ctx_block_cat: u8,
    sh: &SliceHeader,
    sd: &SliceData,
    curr_mb_idx: usize,
) -> (u32, u32, bool) {
    let mut max_bin_idx_ctx: u32 = 0;
    let mut ctx_idx_offset: u32 = 0;
    let mut bypass_flag: bool = true;

    // from Table 9-11 in H.264 Spec
    // We use this to run the binarization and get the contextID which is used to choose m,n values
    match syntax_element {
        // slice_data()
        "mb_skip_flag" => {
            if is_slice_type(sh.slice_type, "P") || is_slice_type(sh.slice_type, "SP") {
                max_bin_idx_ctx = 0;
                ctx_idx_offset = 11;
                bypass_flag = false;
            } else if is_slice_type(sh.slice_type, "B") {
                max_bin_idx_ctx = 0;
                ctx_idx_offset = 24;
                bypass_flag = false;
            } else {
                panic!("MB_SKIP_FLAG DECODING ERROR");
            }
        }
        "mb_field_decoding_flag" => {
            max_bin_idx_ctx = 0;
            ctx_idx_offset = 70;
            bypass_flag = false;
        }
        "transform_size_8x8_flag" => {
            max_bin_idx_ctx = 0;
            ctx_idx_offset = 399;
            bypass_flag = false;
        }
        "mb_qp_delta" => {
            // specified in 9.3.2.7
            max_bin_idx_ctx = 2;
            ctx_idx_offset = 60;
            bypass_flag = false;
        }
        // mb_pred()
        "prev_intra4x4_pred_mode_flag" => {
            max_bin_idx_ctx = 0;
            ctx_idx_offset = 68;
            bypass_flag = false;
        }
        "rem_intra4x4_pred_mode" => {
            max_bin_idx_ctx = 0;
            ctx_idx_offset = 69;
            bypass_flag = false;
        }
        "prev_intra8x8_pred_mode_flag" => {
            max_bin_idx_ctx = 0;
            ctx_idx_offset = 68;
            bypass_flag = false;
        }
        "rem_intra8x8_pred_mode" => {
            max_bin_idx_ctx = 0;
            ctx_idx_offset = 69;
            bypass_flag = false;
        }
        "intra_chroma_pred_mode" => {
            max_bin_idx_ctx = 1;
            ctx_idx_offset = 64;
            bypass_flag = false;
        }
        // mb_pred() and sub_mb_pred()
        "ref_idx_l0" => {
            max_bin_idx_ctx = 2;
            ctx_idx_offset = 54;
            bypass_flag = false;
        }
        "ref_idx_l1" => {
            max_bin_idx_ctx = 2;
            ctx_idx_offset = 54;
            bypass_flag = false;
        }
        // sub_mb_pred()
        "sub_mb_type" => {
            // binarization specified in 9.3.2.5
            if is_slice_type(sh.slice_type, "P") || is_slice_type(sh.slice_type, "SP") {
                max_bin_idx_ctx = 2;
                ctx_idx_offset = 21;
                bypass_flag = false;
            } else if is_slice_type(sh.slice_type, "B") {
                max_bin_idx_ctx = 3;
                ctx_idx_offset = 36;
                bypass_flag = false;
            }
        }
        // residual_block_cabac()
        "coded_block_flag" => {
            // first determine ctxBlockCat by section 9.3.3.1.3
            max_bin_idx_ctx = 0;
            if ctx_block_cat < 5 {
                ctx_idx_offset = 85;
            } else if 5 < ctx_block_cat && ctx_block_cat < 9 {
                ctx_idx_offset = 460;
            } else if 9 < ctx_block_cat && ctx_block_cat < 13 {
                ctx_idx_offset = 472;
            } else if ctx_block_cat == 5 || ctx_block_cat == 9 || ctx_block_cat == 13 {
                ctx_idx_offset = 1012;
            }
            bypass_flag = false;
        }
        "significant_coeff_flag" => {
            max_bin_idx_ctx = 0;
            if sh.field_pic_flag || sd.mb_field_decoding_flag[curr_mb_idx] {
                // this means that the slice is part of a coded field
                if ctx_block_cat < 5 {
                    ctx_idx_offset = 277;
                } else if ctx_block_cat == 5 {
                    ctx_idx_offset = 436;
                } else if 5 < ctx_block_cat && ctx_block_cat < 9 {
                    ctx_idx_offset = 776;
                } else if 9 < ctx_block_cat && ctx_block_cat < 13 {
                    ctx_idx_offset = 820;
                } else if ctx_block_cat == 9 {
                    ctx_idx_offset = 675;
                } else if ctx_block_cat == 13 {
                    ctx_idx_offset = 733;
                }
            } else {
                // else it's part of a coded frame
                if ctx_block_cat < 5 {
                    ctx_idx_offset = 105;
                } else if ctx_block_cat == 5 {
                    ctx_idx_offset = 402;
                } else if 5 < ctx_block_cat && ctx_block_cat < 9 {
                    ctx_idx_offset = 484;
                } else if 9 < ctx_block_cat && ctx_block_cat < 13 {
                    ctx_idx_offset = 528;
                } else if ctx_block_cat == 9 {
                    ctx_idx_offset = 660;
                } else if ctx_block_cat == 13 {
                    ctx_idx_offset = 718;
                }
            }
            bypass_flag = false;
        }
        "last_significant_coeff_flag" => {
            if sh.field_pic_flag || sd.mb_field_decoding_flag[curr_mb_idx] {
                // this means that the slice is part of a coded field
                if ctx_block_cat < 5 {
                    ctx_idx_offset = 338;
                } else if ctx_block_cat == 5 {
                    ctx_idx_offset = 451;
                } else if 5 < ctx_block_cat && ctx_block_cat < 9 {
                    ctx_idx_offset = 864;
                } else if 9 < ctx_block_cat && ctx_block_cat < 13 {
                    ctx_idx_offset = 908;
                } else if ctx_block_cat == 9 {
                    ctx_idx_offset = 699;
                } else if ctx_block_cat == 13 {
                    ctx_idx_offset = 757;
                }
            } else {
                // else it's part of a coded frame
                if ctx_block_cat < 5 {
                    ctx_idx_offset = 166;
                } else if ctx_block_cat == 5 {
                    ctx_idx_offset = 417;
                } else if 5 < ctx_block_cat && ctx_block_cat < 9 {
                    ctx_idx_offset = 572;
                } else if 9 < ctx_block_cat && ctx_block_cat < 13 {
                    ctx_idx_offset = 616;
                } else if ctx_block_cat == 9 {
                    ctx_idx_offset = 690;
                } else if ctx_block_cat == 13 {
                    ctx_idx_offset = 748;
                }
            }
            bypass_flag = false;
        }
        "end_of_slice_flag" => {
            ctx_idx_offset = 276;
            bypass_flag = false;
        }
        "coeff_sign_flag" => {
            max_bin_idx_ctx = 0;
            bypass_flag = true;
        }
        _ => {
            panic!("get_binarization_params - {} not found", syntax_element);
        }
    }

    (max_bin_idx_ctx, ctx_idx_offset, bypass_flag)
}

/// Relies on getting information from the surrounding macroblocks to get
/// the content index for the current macroblock
fn get_ctx_idx(
    syntax_element: &str,
    curr_bin_idx: u32,
    max_bin_idx_ctx: u32,
    ctx_idx_offset: u32,
    curr_mb_idx: usize,
    sh: &SliceHeader,
    sd: &mut SliceData,
    ctx_block_cat: u8,
    additional_inputs: &[usize],
    vp: &VideoParameters,
) -> usize {
    let mut ctx_idx: usize = ctx_idx_offset as usize;
    let mut ctx_idx_inc: usize = 0;

    // "All bins with (curr)binIdx greater than maxBinIdxCtx are parsed using the value of ctxIdx being assigned to binIdx equal to maxBinIdxCtx"
    let bin_idx = match curr_bin_idx > max_bin_idx_ctx {
        true => max_bin_idx_ctx,
        false => curr_bin_idx,
    };

    let mut in_table: bool = true;

    // Table 9-39
    match ctx_idx_offset {
        0 => {
            if bin_idx > 0 {
                panic!(
                    "GET_CTX_IDX: Incorrect ctx_idx_offset and bin_idx combination: {} and {}",
                    ctx_idx_offset, bin_idx
                );
            }
            // Decode according to 9.3.3.1.1.3

            // A is the block to the left
            // B is the block right above
            let mut cond_term_flag_a: usize = 1;
            let mut cond_term_flag_b: usize = 1;

            let res = sd.get_neighbor(curr_mb_idx, false, vp);

            let mb_a: MacroBlock = res.0;
            let mb_b: MacroBlock = res.1;

            if !mb_a.available || mb_a.mb_type == MbType::SI {
                cond_term_flag_a = 0;
            }

            if !mb_b.available || mb_b.mb_type == MbType::SI {
                cond_term_flag_b = 0;
            }

            ctx_idx_inc = cond_term_flag_a + cond_term_flag_b;
        }
        3 => {
            if bin_idx == 0 {
                // Getting info for mb_type following clause 9.3.3.1.1.3

                // A is the block to the left
                // B is the block right above
                let mut cond_term_flag_a: usize = 1;
                let mut cond_term_flag_b: usize = 1;

                let res = sd.get_neighbor(curr_mb_idx, false, vp);

                let mb_a: MacroBlock = res.0;
                let mb_b: MacroBlock = res.1;

                if !mb_a.available || mb_a.mb_type == MbType::INxN {
                    cond_term_flag_a = 0;
                }

                if !mb_b.available || mb_b.mb_type == MbType::INxN {
                    cond_term_flag_b = 0;
                }

                ctx_idx_inc = cond_term_flag_a + cond_term_flag_b;
            } else if bin_idx == 1 {
                return 276;
            } else if bin_idx == 2 {
                ctx_idx_inc = 3;
            } else if bin_idx == 3 {
                ctx_idx_inc = 4;
            } else if bin_idx == 4 {
                // Section 9.3.3.1.2
                let b3 = additional_inputs[0];
                if b3 != 0 {
                    ctx_idx_inc = 5;
                } else {
                    ctx_idx_inc = 6;
                }
            } else if bin_idx == 5 {
                // Section 9.3.3.1.2
                let b3 = additional_inputs[0];
                if b3 != 0 {
                    ctx_idx_inc = 6;
                } else {
                    ctx_idx_inc = 7;
                }
            } else {
                ctx_idx_inc = 7;
            }
        }
        11 => {
            // Decode according to 9.3.3.1.1.1
            let mut cond_term_flag_a: usize = 1;
            let mut cond_term_flag_b: usize = 1;

            // NOTE: given section 7.4.4, mb_field_decoding_flag must be inferred
            //
            // get_neighbor requires knowing the current
            // mb_field_decoding_flag, but it has not been
            // decoded yet from the stream.

            // Use clause 6.4.11.1 to get neighbors
            let res = sd.get_neighbor(curr_mb_idx, false, vp);
            let mb_a: MacroBlock = res.0;
            let mb_b: MacroBlock = res.1;

            // set cond_term_flag_a
            if !mb_a.available || mb_a.mb_skip_flag {
                cond_term_flag_a = 0;
            }

            if !mb_b.available || mb_b.mb_skip_flag {
                cond_term_flag_b = 0;
            }

            ctx_idx_inc = cond_term_flag_a + cond_term_flag_b;
        }
        14 => {
            if bin_idx > 2 {
                panic!(
                    "GET_CTX_IDX: Incorrect ctx_idx_offset and bin_idx combination: {} and {}",
                    ctx_idx_offset, bin_idx
                );
            }
            if CABAC_DEBUG {
                debug!(target: "decode","get_ctx_idx - mb_type (P, SP slices Prefix)");
            }

            if bin_idx == 0 {
                ctx_idx_inc = 0;
            } else if bin_idx == 1 {
                ctx_idx_inc = 1;
            } else {
                // Section 9.3.3.1.2
                let b1 = additional_inputs[0];
                if b1 != 1 {
                    ctx_idx_inc = 2;
                } else {
                    ctx_idx_inc = 3;
                }
            }
        }
        17 => {
            if CABAC_DEBUG {
                debug!(target: "decode","get_ctx_idx - mb_type (P, SP slices Suffix) - bin_idx: {}", bin_idx);
            }

            if bin_idx == 0 {
                ctx_idx_inc = 0;
            } else if bin_idx == 1 {
                return 276;
            } else if bin_idx == 2 {
                ctx_idx_inc = 1;
            } else if bin_idx == 3 {
                ctx_idx_inc = 2;
            } else if bin_idx == 4 {
                // Section 9.3.3.1.2
                let b3 = additional_inputs[0];

                if b3 != 0 {
                    ctx_idx_inc = 2;
                } else {
                    ctx_idx_inc = 3;
                }
            } else if bin_idx >= 5 {
                ctx_idx_inc = 3;
            }
            if CABAC_DEBUG {
                debug!(target: "decode","get_ctx_idx - mb_type (P, SP slices Suffix) - ctx_idx_inc value: {}", ctx_idx_inc);
            }
        }
        21 => {
            if bin_idx > 2 {
                panic!(
                    "GET_CTX_IDX: Incorrect ctx_idx_offset and bin_idx combination: {} and {}",
                    ctx_idx_offset, bin_idx
                );
            }
            if bin_idx == 0 {
                ctx_idx_inc = 0;
            } else if bin_idx == 1 {
                ctx_idx_inc = 1;
            } else if bin_idx == 2 {
                ctx_idx_inc = 2;
            }
        }
        24 => {
            if bin_idx > 0 {
                panic!(
                    "GET_CTX_IDX: Incorrect ctx_idx_offset and bin_idx combination: {} and {}",
                    ctx_idx_offset, bin_idx
                );
            }
            // Decode according to 9.3.3.1.1.1
            let mut cond_term_flag_a: usize = 1;
            let mut cond_term_flag_b: usize = 1;

            let res = sd.get_neighbor(curr_mb_idx, false, vp);
            let mb_a: MacroBlock = res.0;
            let mb_b: MacroBlock = res.1;

            debug!(target: "decode","mb_skip_flag (B slices) - mb_a.mb_addr {} and !mb_a.available {} and mb_a.mb_skip_flag {}", mb_a.mb_addr, !mb_a.available, mb_a.mb_skip_flag);
            debug!(target: "decode","mb_skip_flag (B slices) - mb_b.mb_addr {} and !mb_b.available {} and  mb_b.mb_skip_flag {}", mb_b.mb_addr, !mb_b.available, mb_b.mb_skip_flag);

            // set cond_term_flag_a
            if !mb_a.available || mb_a.mb_skip_flag {
                cond_term_flag_a = 0;
            }

            if !mb_b.available || mb_b.mb_skip_flag {
                cond_term_flag_b = 0;
            }

            ctx_idx_inc = cond_term_flag_a + cond_term_flag_b;
        }
        27 => {
            if CABAC_DEBUG {
                debug!(target: "decode","get_ctx_idx - mb_type (B slices Prefix)");
            }

            if bin_idx == 0 {
                // decode according to 9.3.3.1.3
                // A is the block to the left
                // B is the block right above
                let mut cond_term_flag_a: usize = 1;
                let mut cond_term_flag_b: usize = 1;

                let res = sd.get_neighbor(curr_mb_idx, false, vp);
                let mb_a: MacroBlock = res.0;
                let mb_b: MacroBlock = res.1;

                // set cond_term_flag_a
                if !mb_a.available
                    || mb_a.mb_type == MbType::BSkip
                    || mb_a.mb_type == MbType::BDirect16x16
                {
                    cond_term_flag_a = 0;
                }

                if !mb_b.available
                    || mb_b.mb_type == MbType::BSkip
                    || mb_b.mb_type == MbType::BDirect16x16
                {
                    cond_term_flag_b = 0;
                }

                ctx_idx_inc = cond_term_flag_a + cond_term_flag_b;
            } else if bin_idx == 1 {
                ctx_idx_inc = 3;
            } else if bin_idx == 2 {
                // Section 9.3.3.1.2
                let b1 = additional_inputs[0];
                if b1 != 0 {
                    ctx_idx_inc = 4;
                } else {
                    ctx_idx_inc = 5;
                }
            } else if bin_idx >= 3 {
                ctx_idx_inc = 5;
            }
        }
        32 => {
            if CABAC_DEBUG {
                debug!(target: "decode","get_ctx_idx - mb_type (B slices Suffix)");
            }

            if bin_idx == 0 {
                ctx_idx_inc = 0;
            } else if bin_idx == 1 {
                return 276;
            } else if bin_idx == 2 {
                ctx_idx_inc = 1;
            } else if bin_idx == 3 {
                ctx_idx_inc = 2;
            } else if bin_idx == 4 {
                // Section 9.3.3.1.2
                let b3 = additional_inputs[0];
                if b3 != 0 {
                    ctx_idx_inc = 2;
                } else {
                    ctx_idx_inc = 3;
                }
            } else if bin_idx >= 5 {
                ctx_idx_inc = 3;
            }
        }
        36 => {
            if bin_idx > 5 {
                panic!(
                    "GET_CTX_IDX: Incorrect ctx_idx_offset and bin_idx combination: {} and {}",
                    ctx_idx_offset, bin_idx
                );
            }
            if bin_idx == 0 {
                ctx_idx_inc = 0;
            } else if bin_idx == 1 {
                ctx_idx_inc = 1;
            } else if bin_idx == 2 {
                // Section 9.3.3.1.2
                let b1 = additional_inputs[0];
                if b1 != 0 {
                    ctx_idx_inc = 2;
                } else {
                    ctx_idx_inc = 3;
                }
            } else if bin_idx >= 3 {
                ctx_idx_inc = 3;
            }
        }
        54 => {
            if bin_idx == 0 {
                // Section 9.3.3.1.1.6

                let mut ref_idx_zero_flag_a = false;
                let mut ref_idx_zero_flag_b = false;

                let mut pred_mode_equal_flag_a = true;
                let mut pred_mode_equal_flag_b = true;

                let mut cond_term_flag_a = 1;
                let mut cond_term_flag_b = 1;

                let mb_part_idx = additional_inputs[0];
                let sub_mb_part_idx = 0;

                // use 6.4.11.7 for neighbor info
                let res =
                    sd.get_neighbor_partitions(curr_mb_idx, mb_part_idx, sub_mb_part_idx, vp, true);

                let mb_a: MacroBlock = res.0;
                let mb_b: MacroBlock = res.1;
                let mb_part_idx_a = res.4;
                let mb_part_idx_b = res.5;

                if mb_a.available {
                    if syntax_element == "ref_idx_l0" {
                        if sh.mbaff_frame_flag
                            && !sd.mb_field_decoding_flag[curr_mb_idx]
                            && sd.mb_field_decoding_flag[mb_a.mb_idx]
                        {
                            ref_idx_zero_flag_a = mb_a.ref_idx_l0[mb_part_idx_a] <= 1;
                        // equation 9-12
                        } else {
                            ref_idx_zero_flag_a = mb_a.ref_idx_l0[mb_part_idx_a] == 0;
                            // equation 9-13
                        }
                    } else {
                        if sh.mbaff_frame_flag
                            && !sd.mb_field_decoding_flag[curr_mb_idx]
                            && sd.mb_field_decoding_flag[mb_a.mb_idx]
                        {
                            ref_idx_zero_flag_a = mb_a.ref_idx_l1[mb_part_idx_a] <= 1;
                        } else {
                            ref_idx_zero_flag_a = mb_a.ref_idx_l1[mb_part_idx_a] == 0;
                        }
                    }
                }

                if mb_b.available {
                    if syntax_element == "ref_idx_l0" {
                        if sh.mbaff_frame_flag
                            && !sd.mb_field_decoding_flag[curr_mb_idx]
                            && sd.mb_field_decoding_flag[mb_b.mb_idx]
                        {
                            ref_idx_zero_flag_b = mb_b.ref_idx_l0[mb_part_idx_b] <= 1;
                        } else {
                            ref_idx_zero_flag_b = mb_b.ref_idx_l0[mb_part_idx_b] == 0;
                        }
                    } else {
                        if sh.mbaff_frame_flag
                            && !sd.mb_field_decoding_flag[curr_mb_idx]
                            && sd.mb_field_decoding_flag[mb_b.mb_idx]
                        {
                            ref_idx_zero_flag_b = mb_b.ref_idx_l1[mb_part_idx_b] <= 1;
                        } else {
                            ref_idx_zero_flag_b = mb_b.ref_idx_l1[mb_part_idx_b] == 0;
                        }
                    }
                }

                if mb_a.mb_type == MbType::BDirect16x16 || mb_a.mb_type == MbType::BSkip {
                    pred_mode_equal_flag_a = false;
                } else if mb_a.mb_type == MbType::P8x8 || mb_a.mb_type == MbType::B8x8 {
                    if syntax_element == "ref_idx_l0" {
                        if mb_a.sub_mb_part_pred_mode(mb_part_idx_a) != MbPartPredMode::PredL0
                            && mb_a.sub_mb_part_pred_mode(mb_part_idx_a) != MbPartPredMode::BiPred
                        {
                            pred_mode_equal_flag_a = false;
                        }
                    } else if mb_a.sub_mb_part_pred_mode(mb_part_idx_a) != MbPartPredMode::PredL1
                        && mb_a.sub_mb_part_pred_mode(mb_part_idx_a) != MbPartPredMode::BiPred
                    {
                        pred_mode_equal_flag_a = false;
                    }
                } else if syntax_element == "ref_idx_l0" {
                    if mb_a.mb_part_pred_mode(mb_part_idx_a) != MbPartPredMode::PredL0
                        && mb_a.mb_part_pred_mode(mb_part_idx_a) != MbPartPredMode::BiPred
                    {
                        pred_mode_equal_flag_a = false;
                    }
                } else if mb_a.mb_part_pred_mode(mb_part_idx_a) != MbPartPredMode::PredL1
                    && mb_a.mb_part_pred_mode(mb_part_idx_a) != MbPartPredMode::BiPred
                {
                    pred_mode_equal_flag_a = false;
                }

                if mb_b.mb_type == MbType::BDirect16x16 || mb_b.mb_type == MbType::BSkip {
                    pred_mode_equal_flag_b = false;
                } else if mb_b.mb_type == MbType::P8x8 || mb_b.mb_type == MbType::B8x8 {
                    if syntax_element == "ref_idx_l0" {
                        if mb_b.sub_mb_part_pred_mode(mb_part_idx_b) != MbPartPredMode::PredL0
                            && mb_b.sub_mb_part_pred_mode(mb_part_idx_b) != MbPartPredMode::BiPred
                        {
                            pred_mode_equal_flag_b = false;
                        }
                    } else if mb_b.sub_mb_part_pred_mode(mb_part_idx_b) != MbPartPredMode::PredL1
                        && mb_b.sub_mb_part_pred_mode(mb_part_idx_b) != MbPartPredMode::BiPred
                    {
                        pred_mode_equal_flag_b = false;
                    }
                } else if syntax_element == "ref_idx_l0" {
                    if mb_b.mb_part_pred_mode(mb_part_idx_b) != MbPartPredMode::PredL0
                        && mb_b.mb_part_pred_mode(mb_part_idx_b) != MbPartPredMode::BiPred
                    {
                        pred_mode_equal_flag_b = false;
                    }
                } else if mb_b.mb_part_pred_mode(mb_part_idx_b) != MbPartPredMode::PredL1
                    && mb_b.mb_part_pred_mode(mb_part_idx_b) != MbPartPredMode::BiPred
                {
                    pred_mode_equal_flag_b = false;
                }

                if !mb_a.available
                    || mb_a.mb_type == MbType::PSkip
                    || mb_a.mb_type == MbType::BSkip
                    || mb_a.is_intra_non_mut()
                    || !pred_mode_equal_flag_a
                    || ref_idx_zero_flag_a
                {
                    cond_term_flag_a = 0;
                }

                if !mb_b.available
                    || mb_b.mb_type == MbType::PSkip
                    || mb_b.mb_type == MbType::BSkip
                    || mb_b.is_intra_non_mut()
                    || !pred_mode_equal_flag_b
                    || ref_idx_zero_flag_b
                {
                    cond_term_flag_b = 0;
                }
                ctx_idx_inc = cond_term_flag_a + 2 * cond_term_flag_b;
            } else if bin_idx == 1 {
                ctx_idx_inc = 4;
            } else if bin_idx >= 2 {
                ctx_idx_inc = 5;
            }
        }
        60 => {
            // Table 9-39
            if bin_idx == 0 {
                // Section 9.3.3.1.1.5

                let prev_mb = sd.get_previous_macroblock(curr_mb_idx);
                // if any of the following conditions are true, ctxIdxInc is set equal to 0:
                if !prev_mb.available
                    || prev_mb.mb_type == MbType::PSkip
                    || prev_mb.mb_type == MbType::BSkip
                    || prev_mb.mb_type == MbType::IPCM
                    || (prev_mb.mb_part_pred_mode(0) != MbPartPredMode::Intra16x16
                        && prev_mb.coded_block_pattern_chroma == 0
                        && prev_mb.coded_block_pattern_luma == 0)
                    || prev_mb.mb_qp_delta == 0
                {
                    ctx_idx_inc = 0;
                } else {
                    ctx_idx_inc = 1;
                }
            } else if bin_idx == 1 {
                ctx_idx_inc = 2;
            } else if bin_idx >= 2 {
                ctx_idx_inc = 3;
            }
        }
        64 => {
            if bin_idx > 2 {
                panic!(
                    "GET_CTX_IDX: Incorrect ctx_idx_offset and bin_idx combination: {} and {}",
                    ctx_idx_offset, bin_idx
                );
            }
            if bin_idx == 0 {
                let mut cond_term_flag_a: usize = 1;
                let mut cond_term_flag_b: usize = 1;

                // use the 6.4.11.1 clause to determine neighbor information
                let res = sd.get_neighbor(curr_mb_idx, false, vp);
                let mut mb_a: MacroBlock = res.0;
                let mut mb_b: MacroBlock = res.1;

                // set cond_term_flag_a
                if !mb_a.available
                    || mb_a.is_inter()
                    || mb_a.mb_type == MbType::IPCM
                    || mb_a.intra_chroma_pred_mode == 0
                {
                    cond_term_flag_a = 0;
                }

                if !mb_b.available
                    || mb_b.is_inter()
                    || mb_b.mb_type == MbType::IPCM
                    || mb_b.intra_chroma_pred_mode == 0
                {
                    cond_term_flag_b = 0;
                }

                ctx_idx_inc = cond_term_flag_a + cond_term_flag_b;
            } else if bin_idx == 1 || bin_idx == 2 {
                ctx_idx_inc = 3;
            }
        }
        68 => {
            if bin_idx > 0 {
                panic!(
                    "GET_CTX_IDX: Incorrect ctx_idx_offset and bin_idx combination: {} and {}",
                    ctx_idx_offset, bin_idx
                );
            }
            ctx_idx_inc = 0;
        }
        69 => {
            if bin_idx > 2 {
                panic!(
                    "GET_CTX_IDX: Incorrect ctx_idx_offset and bin_idx combination: {} and {}",
                    ctx_idx_offset, bin_idx
                );
            }
            ctx_idx_inc = 0;
        }
        70 => {
            if bin_idx > 0 {
                panic!(
                    "GET_CTX_IDX: Incorrect ctx_idx_offset and bin_idx combination: {} and {}",
                    ctx_idx_offset, bin_idx
                );
            }
            // Implements clause 9.3.3.1.1.2
            let mut cond_term_flag_a: usize = 1;
            let mut cond_term_flag_b: usize = 1;

            // use 6.4.10 to get MBAFF address
            let mb_a = sd.get_neighbor_mbaff_macroblock(curr_mb_idx, NeighborMB::MbAddrA, vp);
            let mb_b = sd.get_neighbor_mbaff_macroblock(curr_mb_idx, NeighborMB::MbAddrB, vp);

            // if mb_a.mb_addr and mb_a.mb_addr+1 have mb_type equal to PSkip or BSkip the inference
            // rule for mb_field_decoding_flag as specified in clause 7.4.4. for macroblock MbAddrN

            if !mb_a.available || !sd.mb_field_decoding_flag[mb_a.mb_idx] {
                cond_term_flag_a = 0;
            }

            if !mb_b.available || !sd.mb_field_decoding_flag[mb_b.mb_idx] {
                cond_term_flag_b = 0;
            }

            ctx_idx_inc = cond_term_flag_a + cond_term_flag_b;
        }
        73 => {
            if bin_idx > 3 {
                panic!(
                    "GET_CTX_IDX: Incorrect ctx_idx_offset and bin_idx combination: {} and {}",
                    ctx_idx_offset, bin_idx
                );
            }

            let mut cond_term_flag_a: usize = 1;
            let mut cond_term_flag_b: usize = 1;

            // use the 6.4.11.2 clause to determine neighbor information
            let res = sd.get_neighbor_8x8_luma_block(curr_mb_idx, true, bin_idx as usize, vp);

            // set cond_term_flag_a

            let mb_a: MacroBlock = res.0;
            let mb_b: MacroBlock = res.1;
            let luma_8x8_blk_idx_a: usize = res.2;
            let luma_8x8_blk_idx_b: usize = res.3;

            let mut prev_decoded_bin_eq_0 = true;

            if luma_8x8_blk_idx_a < additional_inputs.len() {
                prev_decoded_bin_eq_0 = additional_inputs[luma_8x8_blk_idx_a] == 0;
            }

            // see section 9.3.3.1.1.4 for full list of conditions
            if !mb_a.available
                || mb_a.mb_type == MbType::IPCM
                || (mb_a.mb_addr != sd.macroblock_vec[curr_mb_idx].mb_addr
                    && mb_a.mb_type != MbType::PSkip
                    && mb_a.mb_type != MbType::BSkip
                    && (mb_a.coded_block_pattern_luma >> luma_8x8_blk_idx_a) & 1 != 0)
                || (mb_a.mb_addr == sd.macroblock_vec[curr_mb_idx].mb_addr
                    && !prev_decoded_bin_eq_0)
            // the second condition is that the previously decoded bit is not equal to 0
            {
                if CABAC_DEBUG {
                    debug!(target: "decode","!mb_a.available: {}", !mb_a.available);
                    debug!(target: "decode","mb_a.mb_type == MbType::IPCM: {}", mb_a.mb_type == MbType::IPCM);
                    debug!(target: "decode","(mb_a.mb_addr != sd.macroblock_vec[curr_mb_idx].mb_addr && mb_a.mb_type != MbType::PSkip && mb_a.mb_type != MbType::BSkip && (mb_a.coded_block_pattern_luma >> luma_8x8_blk_idx_a) & 1 != 0) : {}", (mb_a.mb_addr != sd.macroblock_vec[curr_mb_idx].mb_addr && mb_a.mb_type != MbType::PSkip && mb_a.mb_type != MbType::BSkip && (mb_a.coded_block_pattern_luma >> luma_8x8_blk_idx_a) & 1 != 0) );
                    debug!(target: "decode","mb_a.mb_addr: {}", mb_a.mb_addr);
                    debug!(target: "decode","sd.macroblock_vec[curr_mb_idx].mb_addr: {}", sd.macroblock_vec[curr_mb_idx].mb_addr);
                    debug!(target: "decode","mb_a.mb_addr == sd.macroblock_vec[curr_mb_idx].mb_addr && !prev_decoded_bin_eq_0): {}", mb_a.mb_addr == sd.macroblock_vec[curr_mb_idx].mb_addr && !prev_decoded_bin_eq_0);
                }

                cond_term_flag_a = 0;
            }

            // reset the value to false just for safesies
            prev_decoded_bin_eq_0 = true;

            if luma_8x8_blk_idx_b < additional_inputs.len() {
                prev_decoded_bin_eq_0 = additional_inputs[luma_8x8_blk_idx_b] == 0;
            }

            if !mb_b.available
                || mb_b.mb_type == MbType::IPCM
                || (mb_b.mb_addr != sd.macroblock_vec[curr_mb_idx].mb_addr
                    && mb_b.mb_type != MbType::PSkip
                    && mb_b.mb_type != MbType::BSkip
                    && (mb_b.coded_block_pattern_luma >> luma_8x8_blk_idx_b) & 1 != 0)
                || (mb_b.mb_addr == sd.macroblock_vec[curr_mb_idx].mb_addr
                    && !prev_decoded_bin_eq_0)
            {
                if CABAC_DEBUG {
                    debug!(target: "decode","!mb_b.available: {}", !mb_b.available);
                    debug!(target: "decode","mb_b.mb_type == MbType::IPCM: {}", mb_b.mb_type == MbType::IPCM);
                    debug!(target: "decode","(mb_b.mb_addr != sd.macroblock_vec[curr_mb_idx].mb_addr && mb_b.mb_type != MbType::PSkip && mb_b.mb_type != MbType::BSkip && (mb_b.coded_block_pattern_luma >> luma_8x8_blk_idx_a) & 1 != 0) : {}", (mb_b.mb_addr != sd.macroblock_vec[curr_mb_idx].mb_addr && mb_b.mb_type != MbType::PSkip && mb_b.mb_type != MbType::BSkip && (mb_b.coded_block_pattern_luma >> luma_8x8_blk_idx_b) & 1 != 0) );
                    debug!(target: "decode","mb_b.mb_addr == sd.macroblock_vec[curr_mb_idx].mb_addr && !prev_decoded_bin_eq_0): {}", mb_b.mb_addr == sd.macroblock_vec[curr_mb_idx].mb_addr && !prev_decoded_bin_eq_0);
                }
                cond_term_flag_b = 0;
            }

            ctx_idx_inc = cond_term_flag_a + 2 * cond_term_flag_b;
            if CABAC_DEBUG {
                debug!(target: "decode","get_ctx_idx - coded_block_pattern - Prefix - ctx_idx_inc: {}", ctx_idx_inc);
            }
        }
        77 => {
            if CABAC_DEBUG {
                debug!(target: "decode","get_ctx_idx - coded_block_pattern - Suffix - bin_idx {}", bin_idx);
            }
            if bin_idx > 1 {
                panic!(
                    "GET_CTX_IDX: Incorrect ctx_idx_offset and bin_idx combination: {} and {}",
                    ctx_idx_offset, bin_idx
                );
            }

            let mut cond_term_flag_a: usize = 1;
            let mut cond_term_flag_b: usize = 1;

            // use the 6.4.11.1 clause to determine neighbor information
            let res = sd.get_neighbor(curr_mb_idx, false, vp);

            let mb_a: MacroBlock = res.0;
            let mb_b: MacroBlock = res.1;

            // set cond_term_flag_a
            if mb_a.available && mb_a.mb_type == MbType::IPCM {
                cond_term_flag_a = 1;
            } else if !mb_a.available
                || mb_a.mb_type == MbType::PSkip
                || mb_a.mb_type == MbType::BSkip
                || (bin_idx == 0 && mb_a.coded_block_pattern_chroma == 0)
                || (bin_idx == 1 && mb_a.coded_block_pattern_chroma != 2)
            {
                if CABAC_DEBUG {
                    debug!(target: "decode","!mb_a.available: {}", !mb_a.available);
                    debug!(target: "decode","mb_a.mb_type == MbType::PSkip: {}", mb_a.mb_type == MbType::PSkip);
                    debug!(target: "decode","mb_a.mb_type == MbType::BSkip: {}", mb_a.mb_type == MbType::BSkip);
                    debug!(target: "decode","(bin_idx == 0 && mb_a.coded_block_pattern_chroma == 0): {}", (bin_idx == 0 && mb_a.coded_block_pattern_chroma == 0));
                    debug!(target: "decode","(bin_idx == 1 && mb_a.coded_block_pattern_chroma != 2): {}", (bin_idx == 1 && mb_a.coded_block_pattern_chroma != 2));
                }
                cond_term_flag_a = 0;
            }
            if mb_b.available && mb_b.mb_type == MbType::IPCM {
                cond_term_flag_b = 1;
            } else if !mb_b.available
                || mb_b.mb_type == MbType::PSkip
                || mb_b.mb_type == MbType::BSkip
                || (bin_idx == 0 && mb_b.coded_block_pattern_chroma == 0)
                || (bin_idx == 1 && mb_b.coded_block_pattern_chroma != 2)
            {
                if CABAC_DEBUG {
                    debug!(target: "decode","!mb_b.available: {}", !mb_b.available);
                    debug!(target: "decode","mb_b.mb_type == MbType::PSkip: {}", mb_b.mb_type == MbType::PSkip);
                    debug!(target: "decode","mb_b.mb_type == MbType::BSkip: {}", mb_b.mb_type == MbType::BSkip);
                    debug!(target: "decode","(bin_idx == 0 && mb_b.coded_block_pattern_chroma == 0): {}", (bin_idx == 0 && mb_b.coded_block_pattern_chroma == 0));
                    debug!(target: "decode","(bin_idx == 1 && mb_b.coded_block_pattern_chroma != 2): {}", (bin_idx == 1 && mb_b.coded_block_pattern_chroma != 2));
                }

                cond_term_flag_b = 0;
            }

            ctx_idx_inc = cond_term_flag_a
                + 2 * cond_term_flag_b
                + match bin_idx == 1 {
                    true => 4,
                    _ => 0,
                };
        }
        276 => {
            if bin_idx > 0 {
                panic!(
                    "GET_CTX_IDX: Incorrect ctx_idx_offset and bin_idx combination: {} and {}",
                    ctx_idx_offset, bin_idx
                );
            }
            ctx_idx_inc = 0;
        }
        399 => {
            if bin_idx > 0 {
                panic!(
                    "GET_CTX_IDX: Incorrect ctx_idx_offset and bin_idx combination: {} and {}",
                    ctx_idx_offset, bin_idx
                );
            }
            // Defined in clause 9.3.3.1.1.10
            let mut cond_term_flag_a: usize = 1;
            let mut cond_term_flag_b: usize = 1;

            // use the 6.4.11.1 clause to determine neighbor information
            let res = sd.get_neighbor(curr_mb_idx, false, vp);

            let mb_a: MacroBlock = res.0;
            let mb_b: MacroBlock = res.1;

            if !mb_a.available || !mb_a.transform_size_8x8_flag {
                cond_term_flag_a = 0;
            }

            if !mb_b.available || !mb_b.transform_size_8x8_flag {
                cond_term_flag_b = 0;
            }

            ctx_idx_inc = cond_term_flag_a + cond_term_flag_b;
        }
        _ => {
            in_table = false;
        }
    }

    if in_table {
        ctx_idx += ctx_idx_inc;
    } else {
        // ctx_block_cat lookups here
        // should only be coded_block_flag, significant_coeff_flag, last_significant_coeff_flag, and coeff_abs_level_minus1

        // Table 9-40 lookup
        let ctx_block_cat_offset: usize;

        if syntax_element == "coded_block_flag" {
            if CABAC_DEBUG {
                debug!(
                    "get_ctx_idx - coded_block_flag - ctx_block_cat value: {}",
                    ctx_block_cat
                );
            }

            ctx_block_cat_offset =
                cabac_tables::CTX_BLOCK_CAT_OFFSET_CODED_BLOCK_FLAG[ctx_block_cat as usize];

            // specified in section 9.3.3.1.1.9
            let mut trans_block_a: TransformBlock = TransformBlock::new();
            let mut trans_block_b: TransformBlock = TransformBlock::new();
            let mut mb_a: MacroBlock = MacroBlock::new();
            let mut mb_b: MacroBlock = MacroBlock::new();
            let cond_term_flag_a: usize;
            let cond_term_flag_b: usize;

            // additional input depending on the value of ctx_block_cat
            // Table 9-42 mentions the appropriate block to copy from
            if ctx_block_cat == 0 || ctx_block_cat == 6 || ctx_block_cat == 10 {
                // no additional input required

                // use 6.4.11.1 to get neighbor info
                let res = sd.get_neighbor(curr_mb_idx, false, vp);
                mb_a = res.0;
                mb_b = res.1;

                if CABAC_DEBUG {
                    debug!(target: "decode","mb_a.mb_addr {}, mb_b.mb_addr {}",
                            mb_a.mb_addr,
                            mb_b.mb_addr);
                }

                if mb_a.available && mb_a.mb_part_pred_mode(0) == MbPartPredMode::Intra16x16 {
                    if ctx_block_cat == 0 {
                        trans_block_a = mb_a.intra_16x16_dc_level_transform_blocks.clone();
                    } else if ctx_block_cat == 6 {
                        trans_block_a = mb_a.cb_intra_16x16_dc_level_transform_blocks.clone();
                    } else if ctx_block_cat == 10 {
                        trans_block_a = mb_a.cr_intra_16x16_dc_level_transform_blocks.clone();
                    }
                }

                if mb_b.available && mb_b.mb_part_pred_mode(0) == MbPartPredMode::Intra16x16 {
                    if ctx_block_cat == 0 {
                        trans_block_b = mb_b.intra_16x16_dc_level_transform_blocks.clone();
                    } else if ctx_block_cat == 6 {
                        trans_block_b = mb_b.cb_intra_16x16_dc_level_transform_blocks.clone();
                    } else if ctx_block_cat == 10 {
                        trans_block_b = mb_b.cr_intra_16x16_dc_level_transform_blocks.clone();
                    }
                }
            } else if ctx_block_cat == 1 || ctx_block_cat == 2 {
                let luma_4x4_blk_idx = additional_inputs[0];

                // Use 6.4.11.4 to get neighbor luma blocks
                let res = sd.get_neighbor_4x4_luma_block(curr_mb_idx, true, luma_4x4_blk_idx, vp);

                mb_a = res.0;
                mb_b = res.1;
                let luma_4x4_blk_idx_a: usize = res.2;
                let luma_4x4_blk_idx_b: usize = res.3;

                if mb_a.available
                    && mb_a.mb_type != MbType::PSkip
                    && mb_a.mb_type != MbType::BSkip
                    && mb_a.mb_type != MbType::IPCM
                    && (mb_a.coded_block_pattern_luma >> (luma_4x4_blk_idx_a >> 2)) & 1 != 0
                    && !mb_a.transform_size_8x8_flag
                {
                    // our 4x4 look up value differs based on mb part pred mode
                    if mb_a.mb_part_pred_mode(0) == MbPartPredMode::Intra16x16 {
                        trans_block_a =
                            mb_a.intra_16x16_ac_level_transform_blocks[luma_4x4_blk_idx_a].clone();
                    } else {
                        trans_block_a =
                            mb_a.luma_level_4x4_transform_blocks[luma_4x4_blk_idx_a].clone();
                    }
                } else if mb_a.available
                    && mb_a.mb_type != MbType::PSkip
                    && mb_a.mb_type != MbType::BSkip
                    && (mb_a.coded_block_pattern_luma >> (luma_4x4_blk_idx_a >> 2)) & 1 != 0
                    && mb_a.transform_size_8x8_flag
                {
                    trans_block_a =
                        mb_a.luma_level_8x8_transform_blocks[luma_4x4_blk_idx_a >> 2].clone();
                }

                if mb_b.available
                    && mb_b.mb_type != MbType::PSkip
                    && mb_b.mb_type != MbType::BSkip
                    && mb_b.mb_type != MbType::IPCM
                    && (mb_b.coded_block_pattern_luma >> (luma_4x4_blk_idx_b >> 2)) & 1 != 0
                    && !mb_b.transform_size_8x8_flag
                {
                    if mb_b.mb_part_pred_mode(0) == MbPartPredMode::Intra16x16 {
                        trans_block_b =
                            mb_b.intra_16x16_ac_level_transform_blocks[luma_4x4_blk_idx_b].clone();
                    } else {
                        trans_block_b =
                            mb_b.luma_level_4x4_transform_blocks[luma_4x4_blk_idx_b].clone();
                    }
                } else if mb_b.available
                    && mb_b.mb_type != MbType::PSkip
                    && mb_b.mb_type != MbType::BSkip
                    && (mb_b.coded_block_pattern_luma >> (luma_4x4_blk_idx_b >> 2)) & 1 != 0
                    && mb_b.transform_size_8x8_flag
                {
                    trans_block_b =
                        mb_b.luma_level_8x8_transform_blocks[luma_4x4_blk_idx_b >> 2].clone();
                }
            } else if ctx_block_cat == 3 {
                let icb_cr = additional_inputs[0];

                // use 6.4.11.1 to get neighbor info
                let res = sd.get_neighbor(curr_mb_idx, false, vp);
                mb_a = res.0;
                mb_b = res.1;

                if mb_a.available
                    && mb_a.mb_type != MbType::PSkip
                    && mb_a.mb_type != MbType::BSkip
                    && mb_a.mb_type != MbType::IPCM
                    && mb_a.coded_block_pattern_chroma != 0
                {
                    trans_block_a = mb_a.chroma_dc_level_transform_blocks[icb_cr].clone();
                }

                if mb_b.available
                    && mb_b.mb_type != MbType::PSkip
                    && mb_b.mb_type != MbType::BSkip
                    && mb_b.mb_type != MbType::IPCM
                    && mb_b.coded_block_pattern_chroma != 0
                {
                    trans_block_b = mb_b.chroma_dc_level_transform_blocks[icb_cr].clone();
                }
            } else if ctx_block_cat == 4 {
                let chroma_4x4_blk_idx = additional_inputs[0];
                let icb_cr = additional_inputs[1];

                // use 6.4.11.5 to get neighbor chroma blocks
                let res = sd.get_neighbor_4x4_chroma_block(curr_mb_idx, chroma_4x4_blk_idx, vp);

                mb_a = res.0;
                mb_b = res.1;
                let chroma_4x4_blk_idx_a: usize = res.2;
                let chroma_4x4_blk_idx_b: usize = res.3;

                if mb_a.available
                    && mb_a.mb_type != MbType::PSkip
                    && mb_a.mb_type != MbType::BSkip
                    && mb_a.mb_type != MbType::IPCM
                    && mb_a.coded_block_pattern_chroma == 2
                {
                    trans_block_a =
                        mb_a.chroma_ac_level_transform_blocks[icb_cr][chroma_4x4_blk_idx_a].clone();
                }

                if mb_b.available
                    && mb_b.mb_type != MbType::PSkip
                    && mb_b.mb_type != MbType::BSkip
                    && mb_b.mb_type != MbType::IPCM
                    && mb_b.coded_block_pattern_chroma == 2
                {
                    trans_block_b =
                        mb_b.chroma_ac_level_transform_blocks[icb_cr][chroma_4x4_blk_idx_b].clone();
                }
            } else if ctx_block_cat == 5 {
                let luma_8x8_blk_idx = additional_inputs[0];

                // use 6.4.11.2 to get neighbor luma info
                let res = sd.get_neighbor_8x8_luma_block(curr_mb_idx, true, luma_8x8_blk_idx, vp);

                mb_a = res.0;
                mb_b = res.1;
                let luma_8x8_blk_idx_a: usize = res.2;
                let luma_8x8_blk_idx_b: usize = res.3;

                if CABAC_DEBUG {
                    debug!(target: "decode","luma_8x8_blk_idx {}, mb_a.mb_addr {}, luma_8x8_blk_idx_a {}, mb_b.mb_addr {}, luma_8x8_blk_idx_b {}",
                            luma_8x8_blk_idx,
                            mb_a.mb_addr,
                            luma_8x8_blk_idx_a,
                            mb_b.mb_addr,
                            luma_8x8_blk_idx_b);
                }

                if mb_a.available
                    && mb_a.mb_type != MbType::PSkip
                    && mb_a.mb_type != MbType::BSkip
                    && mb_a.mb_type != MbType::IPCM
                    && (mb_a.coded_block_pattern_luma >> luma_8x8_blk_idx_a) & 1 != 0 // The spec just says luma_8x8_blk_idx, which seems to be a typo
                    && mb_a.transform_size_8x8_flag
                {
                    trans_block_a =
                        mb_a.luma_level_8x8_transform_blocks[luma_8x8_blk_idx_a].clone();
                }

                if mb_b.available
                    && mb_b.mb_type != MbType::PSkip
                    && mb_b.mb_type != MbType::BSkip
                    && mb_b.mb_type != MbType::IPCM
                    && (mb_b.coded_block_pattern_luma >> luma_8x8_blk_idx_b) & 1 != 0 // The spec just says luma_8x8_blk_idx, which seems to be a typo
                    && mb_b.transform_size_8x8_flag
                {
                    trans_block_b =
                        mb_b.luma_level_8x8_transform_blocks[luma_8x8_blk_idx_b].clone();
                }
            } else if ctx_block_cat == 7 || ctx_block_cat == 8 {
                let cb_4x4_blk_idx = additional_inputs[0];

                // use 6.4.11.5 to get neighbor luma info
                //let res = sd.get_neighbor_4x4_chroma_block(curr_mb_idx, cb_4x4_blk_idx, vp);
                // The Spec says to use the above algorithm for neighbor calculation but that leads to the incorrect answer.
                // This path is for CrCb decoding which would only occur for YUV444 video (ChromaArrayType == 3).
                // The above neighbor algorithm claims to only be for YUV420 or YUV422.

                // Try other neighbor algorithm - 6.4.11.6
                let res = sd.get_neighbor_4x4_cr_cb_blocks_info(curr_mb_idx, cb_4x4_blk_idx, vp);

                mb_a = res.0;
                mb_b = res.1;
                let cb_4x4_blk_idx_a: usize = res.2;
                let cb_4x4_blk_idx_b: usize = res.3;

                if CABAC_DEBUG {
                    debug!(target: "decode","cb_4x4_blk_idx {}, mb_a.mb_addr {}, cb_4x4_blk_idx_a {}, mb_b.mb_addr {}, cb_4x4_blk_idx_b {}",
                            cb_4x4_blk_idx,
                            mb_a.mb_addr,
                            cb_4x4_blk_idx_a,
                            mb_b.mb_addr,
                            cb_4x4_blk_idx_b);
                }

                // difference between two cases is IPCM and transform_size_8x8_flag
                if mb_a.available
                    && mb_a.mb_type != MbType::PSkip
                    && mb_a.mb_type != MbType::BSkip
                    && mb_a.mb_type != MbType::IPCM
                    && (mb_a.coded_block_pattern_luma >> (cb_4x4_blk_idx_a >> 2)) & 1 != 0
                    && !mb_a.transform_size_8x8_flag
                {
                    if ctx_block_cat == 7 {
                        if CABAC_DEBUG {
                            debug!(target: "decode","get_ctx_idx - mb_a copying from CB intra AC");
                        }
                        trans_block_a =
                            mb_a.cb_intra_16x16_ac_level_transform_blocks[cb_4x4_blk_idx_a].clone();
                    } else {
                        if CABAC_DEBUG {
                            debug!(target: "decode","get_ctx_idx - mb_a copying from CB level4x4");
                        }
                        trans_block_a =
                            mb_a.cb_level_4x4_transform_blocks[cb_4x4_blk_idx_a].clone();
                    }
                } else if mb_a.available
                    && mb_a.mb_type != MbType::PSkip
                    && mb_a.mb_type != MbType::BSkip
                    && (mb_a.coded_block_pattern_luma >> (cb_4x4_blk_idx_a >> 2)) & 1 != 0
                    && mb_a.transform_size_8x8_flag
                {
                    if CABAC_DEBUG {
                        debug!(target: "decode","get_ctx_idx - mb_a copying from CB level8x8");
                    }
                    trans_block_a =
                        mb_a.cb_level_8x8_transform_blocks[cb_4x4_blk_idx_a >> 2].clone();
                }

                if mb_b.available
                    && mb_b.mb_type != MbType::PSkip
                    && mb_b.mb_type != MbType::BSkip
                    && mb_b.mb_type != MbType::IPCM
                    && (mb_b.coded_block_pattern_luma >> (cb_4x4_blk_idx_b >> 2)) & 1 != 0
                    && !mb_b.transform_size_8x8_flag
                {
                    if ctx_block_cat == 7 {
                        if CABAC_DEBUG {
                            debug!(target: "decode","get_ctx_idx - mb_b copying from CB intra AC");
                        }
                        trans_block_b =
                            mb_b.cb_intra_16x16_ac_level_transform_blocks[cb_4x4_blk_idx_b].clone();
                    } else {
                        if CABAC_DEBUG {
                            debug!(target: "decode","get_ctx_idx - mb_b copying from CB level4x4");
                        }
                        trans_block_b =
                            mb_b.cb_level_4x4_transform_blocks[cb_4x4_blk_idx_b].clone();
                    }
                } else if mb_b.available
                    && mb_b.mb_type != MbType::PSkip
                    && mb_b.mb_type != MbType::BSkip
                    && (mb_b.coded_block_pattern_luma >> (cb_4x4_blk_idx_b >> 2)) & 1 != 0
                    && mb_b.transform_size_8x8_flag
                {
                    if CABAC_DEBUG {
                        debug!(target: "decode","get_ctx_idx - mb_b copying from CB level8x8");
                    }
                    trans_block_b =
                        mb_b.cb_level_8x8_transform_blocks[cb_4x4_blk_idx_b >> 2].clone();
                }
            } else if ctx_block_cat == 9 {
                let cb_8x8_blk_idx = additional_inputs[0];

                // use 6.4.11.3
                let res = sd.get_neighbor_8x8_cr_cb_block(curr_mb_idx, cb_8x8_blk_idx, vp);

                mb_a = res.0;
                mb_b = res.1;
                let cb_8x8_blk_idx_a: usize = res.2;
                let cb_8x8_blk_idx_b: usize = res.3;

                if mb_a.available
                    && mb_a.mb_type != MbType::PSkip
                    && mb_a.mb_type != MbType::BSkip
                    && mb_a.mb_type != MbType::IPCM
                    && (mb_a.coded_block_pattern_luma >> cb_8x8_blk_idx_a) & 1 != 0
                    && mb_a.transform_size_8x8_flag
                {
                    trans_block_a = mb_a.cb_level_8x8_transform_blocks[cb_8x8_blk_idx_a].clone();
                }

                if mb_b.available
                    && mb_b.mb_type != MbType::PSkip
                    && mb_b.mb_type != MbType::BSkip
                    && mb_b.mb_type != MbType::IPCM
                    && (mb_b.coded_block_pattern_luma >> cb_8x8_blk_idx_b) & 1 != 0
                    && mb_b.transform_size_8x8_flag
                {
                    trans_block_b = mb_b.cb_level_8x8_transform_blocks[cb_8x8_blk_idx_b].clone();
                }
            } else if ctx_block_cat == 11 || ctx_block_cat == 12 {
                let cr_4x4_blk_idx = additional_inputs[0];

                // use 6.4.11.5 to get neighbor luma info
                //let res = sd.get_neighbor_4x4_chroma_block(curr_mb_idx, cr_4x4_blk_idx, vp);
                // Try other neighbor algorithm - 6.4.11.6
                let res = sd.get_neighbor_4x4_cr_cb_blocks_info(curr_mb_idx, cr_4x4_blk_idx, vp);

                mb_a = res.0;
                mb_b = res.1;
                let cr_4x4_blk_idx_a: usize = res.2;
                let cr_4x4_blk_idx_b: usize = res.3;

                // difference between two cases is IPCM and transform_size_8x8_flag
                if mb_a.available
                    && mb_a.mb_type != MbType::PSkip
                    && mb_a.mb_type != MbType::BSkip
                    && mb_a.mb_type != MbType::IPCM
                    && (mb_a.coded_block_pattern_luma >> (cr_4x4_blk_idx_a >> 2)) & 1 != 0
                    && !mb_a.transform_size_8x8_flag
                {
                    if ctx_block_cat == 11 {
                        trans_block_a =
                            mb_a.cr_intra_16x16_ac_level_transform_blocks[cr_4x4_blk_idx_a].clone();
                    } else {
                        trans_block_a =
                            mb_a.cr_level_4x4_transform_blocks[cr_4x4_blk_idx_a].clone();
                    }
                } else if mb_a.available
                    && mb_a.mb_type != MbType::PSkip
                    && mb_a.mb_type != MbType::BSkip
                    && (mb_a.coded_block_pattern_luma >> (cr_4x4_blk_idx_a >> 2)) & 1 != 0
                    && mb_a.transform_size_8x8_flag
                {
                    trans_block_a =
                        mb_a.cr_level_8x8_transform_blocks[cr_4x4_blk_idx_a >> 2].clone();
                }

                if mb_b.available
                    && mb_b.mb_type != MbType::PSkip
                    && mb_b.mb_type != MbType::BSkip
                    && mb_b.mb_type != MbType::IPCM
                    && (mb_b.coded_block_pattern_luma >> (cr_4x4_blk_idx_b >> 2)) & 1 != 0
                    && !mb_b.transform_size_8x8_flag
                {
                    if ctx_block_cat == 11 {
                        trans_block_b =
                            mb_b.cr_intra_16x16_ac_level_transform_blocks[cr_4x4_blk_idx_b].clone();
                    } else {
                        trans_block_b =
                            mb_b.cr_level_4x4_transform_blocks[cr_4x4_blk_idx_b].clone();
                    }
                } else if mb_b.available
                    && mb_b.mb_type != MbType::PSkip
                    && mb_b.mb_type != MbType::BSkip
                    && (mb_b.coded_block_pattern_luma >> (cr_4x4_blk_idx_b >> 2)) & 1 != 0
                    && mb_b.transform_size_8x8_flag
                {
                    trans_block_b =
                        mb_b.cr_level_8x8_transform_blocks[cr_4x4_blk_idx_b >> 2].clone();
                }
            } else if ctx_block_cat == 13 {
                let cr_8x8_blk_idx = additional_inputs[0];

                // use 6.4.11.3
                let res = sd.get_neighbor_8x8_cr_cb_block(curr_mb_idx, cr_8x8_blk_idx, vp);

                mb_a = res.0;
                mb_b = res.1;
                let cr_8x8_blk_idx_a: usize = res.2;
                let cr_8x8_blk_idx_b: usize = res.3;

                if mb_a.available
                    && mb_a.mb_type != MbType::PSkip
                    && mb_a.mb_type != MbType::BSkip
                    && mb_a.mb_type != MbType::IPCM
                    && (mb_a.coded_block_pattern_luma >> cr_8x8_blk_idx_a) & 1 != 0
                    && mb_a.transform_size_8x8_flag
                {
                    trans_block_a = mb_a.cr_level_8x8_transform_blocks[cr_8x8_blk_idx_a].clone();
                }

                if mb_b.available
                    && mb_b.mb_type != MbType::PSkip
                    && mb_b.mb_type != MbType::BSkip
                    && mb_b.mb_type != MbType::IPCM
                    && (mb_b.coded_block_pattern_luma >> cr_8x8_blk_idx_b) & 1 != 0
                    && mb_b.transform_size_8x8_flag
                {
                    trans_block_b = mb_b.cr_level_8x8_transform_blocks[cr_8x8_blk_idx_b].clone();
                }
            }

            // send cond_term_flag_a
            if (!mb_a.available && sd.macroblock_vec[curr_mb_idx].is_inter())
                || (mb_a.available && !trans_block_a.available && mb_a.mb_type != MbType::IPCM)
                    // nal_unit_type check is to check whether slice data partitioning is in use (nal_unit_type is in the range of 2 through 4, inclusive)
                || (sd.macroblock_vec[curr_mb_idx].is_intra() && vp.pps_constrained_intra_pred_flag && mb_a.available && mb_a.is_inter() && vp.nal_unit_type > 1 && vp.nal_unit_type < 5)
            {
                if CABAC_DEBUG {
                    debug!(target: "decode","get_ctx_idx - coded_block_flag - (!mb_a.available && sd.macroblock_vec[curr_mb_idx].is_inter()) - {}", (!mb_a.available && sd.macroblock_vec[curr_mb_idx].is_inter()));
                    debug!(target: "decode","get_ctx_idx - coded_block_flag - (mb_a.available && !trans_block_a.available && mb_a.mb_type != MbType::IPCM) - {}", (mb_a.available && !trans_block_a.available && mb_a.mb_type != MbType::IPCM));
                    debug!(target: "decode","get_ctx_idx - coded_block_flag - (sd.macroblock_vec[curr_mb_idx].is_intra() && vp.pps_constrained_intra_pred_flag && mb_a.available && mb_a.is_inter() && vp.nal_unit_type > 1 && vp.nal_unit_type < 5) - {}",
                            (sd.macroblock_vec[curr_mb_idx].is_intra() && vp.pps_constrained_intra_pred_flag && mb_a.available && mb_a.is_inter() && vp.nal_unit_type > 1 && vp.nal_unit_type < 5));

                    debug!(target: "decode","get_ctx_idx - coded_block_flag - cond_term_flag_a is getting set to 0");
                }
                cond_term_flag_a = 0;
            } else if (!mb_a.available && sd.macroblock_vec[curr_mb_idx].is_intra())
                || mb_a.mb_type == MbType::IPCM
            {
                if CABAC_DEBUG {
                    debug!(target: "decode","get_ctx_idx - coded_block_flag - cond_term_flag_a is getting set to 1");
                }
                cond_term_flag_a = 1;
            } else {
                if CABAC_DEBUG {
                    debug!(target: "decode","get_ctx_idx - coded_block_flag - cond_term_flag_a is getting set to prev value");
                }
                cond_term_flag_a = match trans_block_a.coded_block_flag {
                    true => 1,
                    false => 0,
                };
            }

            // send cond_term_flag_b
            if (!mb_b.available && sd.macroblock_vec[curr_mb_idx].is_inter()) ||
                (mb_b.available && !trans_block_b.available && mb_b.mb_type != MbType::IPCM) ||
                    // nal_unit_type check is to check whether slice data partitioning is in use (nal_unit_type is in the range of 2 through 4, inclusive)
                (sd.macroblock_vec[curr_mb_idx].is_intra() && vp.pps_constrained_intra_pred_flag && mb_b.available && mb_b.is_inter() && vp.nal_unit_type > 1 && vp.nal_unit_type < 5)
            {
                if CABAC_DEBUG {
                    debug!(target: "decode","get_ctx_idx - coded_block_flag - (!mb_b.available && sd.macroblock_vec[curr_mb_idx].is_inter()) - {}", (!mb_b.available && sd.macroblock_vec[curr_mb_idx].is_inter()));
                    debug!(target: "decode","get_ctx_idx - coded_block_flag - (mb_b.available && !trans_block_b.available && mb_b.mb_type != MbType::IPCM) - {}", (mb_b.available && !trans_block_b.available && mb_b.mb_type != MbType::IPCM));
                    debug!(target: "decode","get_ctx_idx - coded_block_flag - (sd.macroblock_vec[curr_mb_idx].is_intra() && vp.pps_constrained_intra_pred_flag && mb_b.available && mb_b.is_inter() && vp.nal_unit_type > 1 && vp.nal_unit_type < 5) - {}",
                                (sd.macroblock_vec[curr_mb_idx].is_intra() && vp.pps_constrained_intra_pred_flag && mb_b.available && mb_b.is_inter() && vp.nal_unit_type > 1 && vp.nal_unit_type < 5));

                    debug!(target: "decode","get_ctx_idx - coded_block_flag - cond_term_flag_b is getting set to 0");
                }
                cond_term_flag_b = 0;
            } else if (!mb_b.available && sd.macroblock_vec[curr_mb_idx].is_intra())
                || mb_b.mb_type == MbType::IPCM
            {
                if CABAC_DEBUG {
                    debug!(target: "decode","get_ctx_idx - coded_block_flag - cond_term_flag_b is getting set to 1");
                }
                cond_term_flag_b = 1;
            } else {
                if CABAC_DEBUG {
                    debug!(target: "decode","get_ctx_idx - coded_block_flag - cond_term_flag_b is getting set to prev value");
                }
                cond_term_flag_b = match trans_block_b.coded_block_flag {
                    true => 1,
                    _ => 0,
                };
            }

            ctx_idx_inc = cond_term_flag_a + 2 * cond_term_flag_b;
            if CABAC_DEBUG {
                debug!(target: "decode","get_ctx_idx - coded_block_flag - ctx_idx_inc value: {}", ctx_idx_inc);
            }
        } else if syntax_element == "significant_coeff_flag" {
            ctx_block_cat_offset =
                cabac_tables::CTX_BLOCK_CAT_OFFSET_SIGNIFICANT_COEFF_FLAG[ctx_block_cat as usize];

            let level_list_idx = additional_inputs[0];
            if ctx_block_cat != 3 && ctx_block_cat != 5 && ctx_block_cat != 9 && ctx_block_cat != 13
            {
                ctx_idx_inc = level_list_idx;
            } else if ctx_block_cat == 3 {
                let num_c8x8 = additional_inputs[1];

                ctx_idx_inc = cmp::min(level_list_idx / num_c8x8, 2);
            } else if ctx_block_cat == 5 || ctx_block_cat == 9 || ctx_block_cat == 13 {
                // use Table 9-43
                if sh.field_pic_flag || sd.mb_field_decoding_flag[curr_mb_idx] {
                    // field coded slice
                    ctx_idx_inc = cabac_tables::CTX_IDX_INC_SIGNIFICANT_COEFF_FLAG_FIELD_CODED
                        [level_list_idx as usize];
                } else {
                    ctx_idx_inc = cabac_tables::CTX_IDX_INC_SIGNIFICANT_COEFF_FLAG_FRAME_CODED
                        [level_list_idx as usize];
                }
            }
        } else if syntax_element == "last_significant_coeff_flag" {
            ctx_block_cat_offset = cabac_tables::CTX_BLOCK_CAT_OFFSET_LAST_SIGNIFICANT_COEFF_FLAG
                [ctx_block_cat as usize];
            let level_list_idx = additional_inputs[0];
            if ctx_block_cat != 3 && ctx_block_cat != 5 && ctx_block_cat != 9 && ctx_block_cat != 13
            {
                ctx_idx_inc = level_list_idx;
            } else if ctx_block_cat == 3 {
                let num_c8x8 = additional_inputs[1];

                ctx_idx_inc = cmp::min(level_list_idx / num_c8x8, 2);
            } else if ctx_block_cat == 5 || ctx_block_cat == 9 || ctx_block_cat == 13 {
                // use Table 9-43
                ctx_idx_inc = cabac_tables::CTX_IDX_INC_LAST_SIGNIFICANT_COEFF_FLAG[level_list_idx];
            }
        } else if syntax_element == "coeff_abs_level_minus1" {
            ctx_block_cat_offset =
                cabac_tables::CTX_BLOCK_CAT_OFFSET_COEFF_ABS_LEVEL_MINUS1[ctx_block_cat as usize];

            let num_decode_abs_level_eq_1 = additional_inputs[0];
            let num_decode_abs_level_gt_1 = additional_inputs[1];

            if bin_idx == 0 {
                ctx_idx_inc = match num_decode_abs_level_gt_1 != 0 {
                    true => 0,
                    false => cmp::min(4, 1 + num_decode_abs_level_eq_1),
                };
            } else {
                ctx_idx_inc = 5 + cmp::min(
                    4 - match ctx_block_cat == 3 {
                        true => 1,
                        false => 0,
                    },
                    num_decode_abs_level_gt_1,
                );
            }
        } else {
            panic!("get_ctx_idx - Unknown syntax_element being decoded!");
        }
        ctx_idx += ctx_block_cat_offset + ctx_idx_inc;
    }

    ctx_idx
}

/// Given a bitstream value, produces the decoded syntax elements
/// by walking the decoding tree.
fn debinarization(syntax_element: &str, bitstream: &[u8], sh: &SliceHeader) -> Option<i32> {
    let res: i32;

    match syntax_element {
        // slice_data()
        "mb_skip_flag" => {
            if is_slice_type(sh.slice_type, "P")
                || is_slice_type(sh.slice_type, "SP")
                || is_slice_type(sh.slice_type, "B")
            {
                res = bitstream[0] as i32; // Fixed-length binarization process
            } else {
                panic!("MB_SKIP_FLAG DECODING ERROR");
            }
        }
        "mb_field_decoding_flag" => {
            res = bitstream[0] as i32; // Fixed-length binarization process
        }
        // macroblock_layer()
        "mb_type_prefix" => {
            // prefix and suffix specified in 9.3.2.5
            if is_slice_type(sh.slice_type, "SI")
                || is_slice_type(sh.slice_type, "P")
                || is_slice_type(sh.slice_type, "SP")
            {
                res = bitstream[0] as i32;
            } else if is_slice_type(sh.slice_type, "I") || is_slice_type(sh.slice_type, "B") {
                return read_mb_types(&bitstream, sh.slice_type);
            } else {
                panic!("MB_TYPE_PREFIX DECODING ERROR");
            }
        }
        "mb_type_suffix" => {
            // prefix and suffix specified in 9.3.2.5
            // We're only here because we need to decode I slice types
            if is_slice_type(sh.slice_type, "P") || is_slice_type(sh.slice_type, "SP") {
                if bitstream[0] == 1 {
                    return read_mb_types(&bitstream[1..], 2); // 2 is I slice type
                } else {
                    return read_mb_types(&bitstream[1..], sh.slice_type);
                }
            } else {
                return read_mb_types(&bitstream, 2); // 2 is I slice type
            }
        }
        "transform_size_8x8_flag" => {
            res = bitstream[0] as i32;
        }
        "coded_block_pattern_luma" => {
            // Specified in clause 9.3.2.6
            if bitstream.len() != 4 {
                return None;
            } else {
                res = ((bitstream[3] << 3)
                    | (bitstream[2] << 2)
                    | (bitstream[1] << 1)
                    | bitstream[0]) as i32;
            }
        }
        "coded_block_pattern_chroma" => {
            // Specified in clause 9.3.2.6
            return read_truncated_unary_value(2, &bitstream);
        }
        "mb_qp_delta" => {
            // specified in 9.3.2.7
            // Table 9-4
            return read_unary_value(&bitstream);
        }
        // mb_pred()
        "prev_intra4x4_pred_mode_flag" => {
            res = bitstream[0] as i32;
        }
        "rem_intra4x4_pred_mode" => {
            if bitstream.len() < 3 {
                return None;
            } else {
                res = ((bitstream[2] << 2) | (bitstream[1] << 1) | bitstream[0]) as i32;
            }
        }
        "prev_intra8x8_pred_mode_flag" => {
            res = bitstream[0] as i32;
        }
        "rem_intra8x8_pred_mode" => {
            if bitstream.len() < 3 {
                return None;
            } else {
                res = ((bitstream[2] << 2) | (bitstream[1] << 1) | bitstream[0]) as i32;
            }
        }
        "intra_chroma_pred_mode" => {
            return read_truncated_unary_value(3, &bitstream);
        }
        // mb_pred() and sub_mb_pred()
        "ref_idx_l0" => {
            return read_unary_value(&bitstream);
        }
        "ref_idx_l1" => {
            return read_unary_value(&bitstream);
        }
        "mvd_l0_0" => {
            return read_uegk(true, 9, 3, &bitstream);
        }
        "mvd_l1_0" => {
            return read_uegk(true, 9, 3, &bitstream);
        }
        "mvd_l0_1" => {
            return read_uegk(true, 9, 3, &bitstream);
        }
        "mvd_l1_1" => {
            return read_uegk(true, 9, 3, &bitstream);
        }
        // sub_mb_pred()
        "sub_mb_type" => {
            return read_sub_mb_types(bitstream, sh.slice_type);
        }
        // residual_block_cabac()
        "coded_block_flag" => {
            res = bitstream[0] as i32;
        }
        "significant_coeff_flag" => {
            res = bitstream[0] as i32;
        }
        "last_significant_coeff_flag" => {
            res = bitstream[0] as i32;
        }
        "coeff_abs_level_minus1" => {
            return read_uegk(false, 14, 0, &bitstream);
        }
        "end_of_slice_flag" => {
            res = bitstream[0] as i32;
        }
        _ => {
            panic!("debinarization - {} not found", syntax_element);
        }
    }
    Some(res)
}

/// arith_decode - part of cabac_decode
///
/// Input:
///  - input_val: content to decode
///  - ctx_idx : context model index
///  - cod_I_range:
///  - cod_I_offset:
///
/// Output:
///  - Option<i32> : the decoded binary value. Returns None if ctx_idx is the terminate condition
///  - u32: updated cod_I_range
///  - u32: updated cod_I_offset
fn arith_decode(
    idx1: usize,
    idx2: usize,
    idx3: usize,
    bs: &mut ByteStream,
    cs: &mut CABACState,
) -> u8 {
    // decode_decision()

    let bin_val: u8;
    let cabac_se_state = &mut cs.states[idx1][idx2][idx3];
    let cod_i_range_idx: usize = ((cs.cod_i_range >> 6) as usize) & 3;
    let cod_i_range_lps: u32 =
        cabac_tables::RANGE_TAB_LPS[cabac_se_state.p_state_idx as usize][cod_i_range_idx];
    if CABAC_DEBUG {
        debug!(target: "decode","before - cod_i_range: {}", cs.cod_i_range);
    }

    cs.cod_i_range -= cod_i_range_lps;

    if cs.cod_i_offset >= cs.cod_i_range {
        bin_val = 1 - cabac_se_state.val_mps;
        cs.cod_i_offset -= cs.cod_i_range;
        cs.cod_i_range = cod_i_range_lps;
    } else {
        bin_val = cabac_se_state.val_mps;
    }

    // state transition process
    if bin_val == cabac_se_state.val_mps {
        cabac_se_state.p_state_idx =
            cabac_tables::TRANS_IDX_MPS[cabac_se_state.p_state_idx as usize].into();
    } else {
        if cabac_se_state.p_state_idx == 0 {
            cabac_se_state.val_mps = 1 - cabac_se_state.val_mps;
        }
        cabac_se_state.p_state_idx =
            cabac_tables::TRANS_IDX_LPS[cabac_se_state.p_state_idx as usize].into();
    }

    // renormalization process
    while cs.cod_i_range < 256 {
        cs.cod_i_range <<= 1;
        cs.cod_i_offset <<= 1;
        cs.cod_i_offset |= bs.read_bits(1);
    }
    if CABAC_DEBUG {
        debug!(target: "decode","after1 - cod_i_range: {}", cs.cod_i_range);
    }
    // the bitstream shall not contain data that result in a value of
    // cod_i_offset being greater than or equal to cod_i_range upon
    // completion of this process
    assert!(cs.cod_i_offset < cs.cod_i_range);

    bin_val
}

/// arithmetic decoding with p=0.5
fn decode_bypass(bs: &mut ByteStream, cs: &mut CABACState) -> u8 {
    if CABAC_DEBUG {
        debug!(target: "decode","decode_bypass called");
    }
    let bin_val: u8;

    cs.cod_i_offset <<= 1;
    cs.cod_i_offset |= bs.read_bits(1);

    if cs.cod_i_offset >= cs.cod_i_range {
        bin_val = 1;
        cs.cod_i_offset -= cs.cod_i_range;
    } else {
        bin_val = 0;
    }
    bin_val
}

fn decode_terminate(bs: &mut ByteStream, cs: &mut CABACState) -> u8 {
    if CABAC_DEBUG {
        debug!(target: "decode","decode_terminate called");
    }
    let bin_val: u8;

    cs.cod_i_range -= 2;

    if cs.cod_i_offset >= cs.cod_i_range {
        bin_val = 1;
    } else {
        bin_val = 0;

        // renormalization process
        while cs.cod_i_range < 256 {
            cs.cod_i_range <<= 1;
            cs.cod_i_offset <<= 1;
            cs.cod_i_offset |= bs.read_bits(1);
        }
    }

    bin_val
}

/// Because the coded block pattern has its own unique binarization process we split it out
/// to its own function
fn cabac_decode_cbp(
    syntax_element: &str,
    bs: &mut ByteStream,
    state: &mut CABACState,
    curr_mb_idx: usize,
    sh: &SliceHeader,
    sd: &mut SliceData,
    vp: &VideoParameters,
) -> i32 {
    // first decode the prefix
    let mut decoded: Vec<u8> = Vec::new();
    let mut res: i32;

    // Specified in clause 9.3.2.6
    // first decode the prefix
    let mut max_bin_idx_ctx = 3;
    let mut ctx_idx_offset = 73;
    let mut additional_inputs: Vec<usize> = Vec::new(); // used to keep track of decoded bins

    for bin_idx in 0..max_bin_idx_ctx + 1 {
        let ctx_idx = get_ctx_idx(
            syntax_element,
            bin_idx,
            max_bin_idx_ctx,
            ctx_idx_offset,
            curr_mb_idx,
            sh,
            sd,
            0,
            &additional_inputs,
            vp,
        );

        let mut idx1 = sh.cabac_init_idc as usize;
        let idx2 = sh.slice_qp_y as usize;

        if !is_slice_type(sh.slice_type, "I")
            && !is_slice_type(sh.slice_type, "SI")
            && ctx_idx >= 54
        {
            idx1 += 1;
        }

        if CABAC_DEBUG {
            debug!(target: "decode","cod_i_offset: {:x}", state.cod_i_offset << 7);
            debug!(target: "decode","state ctx_idx-2: {:x}", state.states[idx1][idx2][ctx_idx-2].p_state_idx);
            debug!(target: "decode","state ctx_idx-1: {:x}", state.states[idx1][idx2][ctx_idx-1].p_state_idx);
            debug!(target: "decode","state: {:x}", state.states[idx1][idx2][ctx_idx].p_state_idx);

            if ctx_idx + 1 < cabac_tables::CONTEXT_MODEL_COUNT {
                debug!(target: "decode","state ctx_idx+1: {:x}", state.states[idx1][idx2][ctx_idx+1].p_state_idx);
            }

            if ctx_idx + 2 < cabac_tables::CONTEXT_MODEL_COUNT {
                debug!(target: "decode","state ctx_idx+2: {:x}", state.states[idx1][idx2][ctx_idx+2].p_state_idx);
            }

            debug!(target: "decode","Indices:");
            debug!(target: "decode","\tcabac_init_idc: {}", idx1);
            debug!(target: "decode","\tslice_qp_y: {}", idx2);
            debug!(target: "decode","\tctx_idx: {}", ctx_idx);
        }

        let ad = arith_decode(idx1, idx2, ctx_idx, bs, state);
        if CABAC_DEBUG {
            debug!(
                "updated state: {:x}",
                state.states[idx1][idx2][ctx_idx].p_state_idx
            );
        }
        decoded.push(ad);
        additional_inputs.push(ad as usize);
    }
    match debinarization(&(syntax_element.to_owned() + "_luma"), &decoded, sh) {
        Some(x) => {
            res = x;
        }
        _ => panic!("cabac_decode_cbp - issue debinarization luma"),
    }
    if CABAC_DEBUG {
        debug!(target: "decode","yoo cbp_luma: {:?}", decoded);
    }

    // suffix decode
    if vp.chroma_array_type != 0 && vp.chroma_array_type != 3 {
        max_bin_idx_ctx = 1;
        ctx_idx_offset = 77;
        decoded.clear(); // reset our bitstream
        for bin_idx in 0..max_bin_idx_ctx + 1 {
            let ctx_idx = get_ctx_idx(
                syntax_element,
                bin_idx,
                max_bin_idx_ctx,
                ctx_idx_offset,
                curr_mb_idx,
                sh,
                sd,
                0,
                &[],
                vp,
            );

            let mut idx1 = sh.cabac_init_idc as usize;
            let idx2 = sh.slice_qp_y as usize;

            if !is_slice_type(sh.slice_type, "I")
                && !is_slice_type(sh.slice_type, "SI")
                && ctx_idx >= 54
            {
                idx1 += 1;
            }

            if CABAC_DEBUG {
                debug!(target: "decode","cod_i_offset: {:x}", state.cod_i_offset << 7);
                debug!(target: "decode","state ctx_idx-3: {:x}", state.states[idx1][idx2][ctx_idx-3].p_state_idx);
                debug!(target: "decode","state ctx_idx-2: {:x}", state.states[idx1][idx2][ctx_idx-2].p_state_idx);
                debug!(target: "decode","state ctx_idx-1: {:x}", state.states[idx1][idx2][ctx_idx-1].p_state_idx);
                debug!(target: "decode","state: {:x}", state.states[idx1][idx2][ctx_idx].p_state_idx);
                if ctx_idx + 1 < cabac_tables::CONTEXT_MODEL_COUNT {
                    debug!(target: "decode","state ctx_idx+1: {:x}", state.states[idx1][idx2][ctx_idx+1].p_state_idx);
                }
                if ctx_idx + 2 < cabac_tables::CONTEXT_MODEL_COUNT {
                    debug!(target: "decode","state ctx_idx+2: {:x}", state.states[idx1][idx2][ctx_idx+2].p_state_idx);
                }
                if ctx_idx + 3 < cabac_tables::CONTEXT_MODEL_COUNT {
                    debug!(target: "decode","state ctx_idx+3: {:x}", state.states[idx1][idx2][ctx_idx+3].p_state_idx);
                }
                debug!(target: "decode","Indices:");
                debug!(target: "decode","\tcabac_init_idc: {}", idx1);
                debug!(target: "decode","\tslice_qp_y: {}", idx2);
                debug!(target: "decode","\tctx_idx: {}", ctx_idx);
            }

            let ad = arith_decode(idx1, idx2, ctx_idx, bs, state);
            if CABAC_DEBUG {
                debug!(
                    "updated state: {:x}",
                    state.states[idx1][idx2][ctx_idx].p_state_idx
                );
            }

            decoded.push(ad);
            if let Some(x) = debinarization(&(syntax_element.to_owned() + "_chroma"), &decoded, sh)
            {
                // Table 7-15 - Chroma value can't be equal to 3 so if that's the case then we set it equal to 2
                if x == 3 {
                    res += 32; // Chroma is at the start, and set equal to 2 << 4
                } else {
                    res += x << 4; // Chroma is at the start
                }
                break; // once we get the value, we can just quit and not read any more (specifically when the value is 0)
            }
        }
        if CABAC_DEBUG {
            debug!(target: "decode","yoo cbp_chroma: {:?}", decoded);
        }
    }
    if CABAC_DEBUG {
        debug!(target: "decode","yoo cbp_total: {:?}", res);
    }
    res
}

/// Own function to decode Luma coefficient values
fn cabac_decode_coeff(
    syntax_element: &str,
    bs: &mut ByteStream,
    state: &mut CABACState,
    curr_mb_idx: usize,
    sh: &SliceHeader,
    sd: &mut SliceData,
    vp: &VideoParameters,
    ctx_block_cat: u8,
    additional_inputs: &[usize],
) -> i32 {
    // first decode the prefix
    let mut decoded: Vec<u8> = Vec::new();
    let mut res: i32 = 0;

    let max_bin_idx_ctx = 1;
    let mut ctx_idx_offset = 0;

    // Table 9-34
    if ctx_block_cat < 5 {
        ctx_idx_offset = 227;
    } else if ctx_block_cat == 5 {
        ctx_idx_offset = 426;
    } else if 5 < ctx_block_cat && ctx_block_cat < 9 {
        ctx_idx_offset = 952;
    } else if 9 < ctx_block_cat && ctx_block_cat < 13 {
        ctx_idx_offset = 982;
    } else if ctx_block_cat == 9 {
        ctx_idx_offset = 708;
    } else if ctx_block_cat == 13 {
        ctx_idx_offset = 766;
    }

    let mut done_decoding = false;
    let mut bin_idx = 0;

    while !done_decoding {
        if bin_idx == 14 {
            // we reach the end of the prefix values
            break;
        }
        let ctx_idx = get_ctx_idx(
            syntax_element,
            bin_idx,
            max_bin_idx_ctx,
            ctx_idx_offset,
            curr_mb_idx,
            sh,
            sd,
            ctx_block_cat,
            &additional_inputs,
            vp,
        );

        let mut idx1 = sh.cabac_init_idc as usize;
        let idx2 = sh.slice_qp_y as usize;

        if !is_slice_type(sh.slice_type, "I")
            && !is_slice_type(sh.slice_type, "SI")
            && ctx_idx >= 54
        {
            idx1 += 1;
        }
        if CABAC_DEBUG {
            debug!(target: "decode","cod_i_offset: {:x}", state.cod_i_offset << 7);
            debug!(target: "decode","state: {:x}", state.states[idx1][idx2][ctx_idx].p_state_idx);
        }
        let ad = arith_decode(idx1, idx2, ctx_idx, bs, state);
        if CABAC_DEBUG {
            debug!(
                "updated state: {:x}",
                state.states[idx1][idx2][ctx_idx].p_state_idx
            );
        }
        decoded.push(ad);
        match debinarization(syntax_element, &decoded, sh) {
            Some(x) => {
                res = x;
                done_decoding = true;
            }
            _ => bin_idx += 1, // just continue if the result is None
        }
    }

    // if the suffix exists
    if bin_idx == 14 {
        // next decode the suffix - uses DecodeBypass

        done_decoding = false;
        let mut suffix: i32 = 0;

        while !done_decoding {
            let ad = decode_bypass(bs, state) as u8;
            decoded.push(ad);
            if let Some(x) = debinarization(syntax_element, &decoded, sh) {
                suffix = x;
                done_decoding = true;
            }
        }

        res = (res << 4) | suffix;
    }

    res
}

/// mb_type is broken up into prefix and suffix so we handle those cases here
fn cabac_decode_mbtype(
    syntax_element: &str,
    bs: &mut ByteStream,
    state: &mut CABACState,
    curr_mb_idx: usize,
    sh: &SliceHeader,
    sd: &mut SliceData,
    vp: &VideoParameters,
    ctx_block_cat: u8,
    mut additional_inputs: Vec<usize>,
) -> i32 {
    let mut decoded: Vec<u8> = Vec::new();
    let mut res: i32 = 0;
    let mut decode_suffix = true;

    let mut max_bin_idx_ctx: u32 = 0;
    let mut ctx_idx_offset: u32 = 0;

    // prefix decoding parameters
    if is_slice_type(sh.slice_type, "SI") {
        // prefix and suffix specified in 9.3.2.5
        max_bin_idx_ctx = 0;
        ctx_idx_offset = 0;
    } else if is_slice_type(sh.slice_type, "I") {
        // follow description in 9.3.2.5
        max_bin_idx_ctx = 6;
        ctx_idx_offset = 3;
        decode_suffix = false;
    } else if is_slice_type(sh.slice_type, "P") || is_slice_type(sh.slice_type, "SP") {
        // prefix and suffix specified in 9.3.2.5
        max_bin_idx_ctx = 2;
        ctx_idx_offset = 14;
    } else if is_slice_type(sh.slice_type, "B") {
        // prefix and suffix specified in 9.3.2.5
        max_bin_idx_ctx = 3;
        ctx_idx_offset = 27;
    }

    let mut bin_idx = 0;
    let mut still_decoding = true;

    // This does a bit-wise arithmetic decoding, and performs a debinarization on the recovered bitstream
    while still_decoding {
        // provide additional inputs for 9.3.3.1.2 (mb_type decoding)
        if ctx_idx_offset == 3 {
            if bin_idx == 4 || bin_idx == 5 {
                additional_inputs.push(decoded[3] as usize);
            } else {
                additional_inputs.clear();
            }
        } else if ctx_idx_offset == 14 {
            if bin_idx == 2 {
                additional_inputs.push(decoded[1] as usize);
            } else {
                additional_inputs.clear();
            }
        } else if ctx_idx_offset == 17 {
            if bin_idx == 4 {
                additional_inputs.push(decoded[3] as usize);
            } else {
                additional_inputs.clear();
            }
        } else if ctx_idx_offset == 27 {
            if bin_idx == 2 {
                additional_inputs.push(decoded[1] as usize);
            } else {
                additional_inputs.clear();
            }
        } else if ctx_idx_offset == 32 {
            if bin_idx == 4 {
                additional_inputs.push(decoded[3] as usize);
            } else {
                additional_inputs.clear();
            }
        } else if ctx_idx_offset == 36 {
            if bin_idx == 2 {
                additional_inputs.push(decoded[1] as usize);
            } else {
                additional_inputs.clear();
            }
        }

        let ctx_idx = get_ctx_idx(
            syntax_element,
            bin_idx,
            max_bin_idx_ctx,
            ctx_idx_offset,
            curr_mb_idx,
            sh,
            sd,
            ctx_block_cat,
            &additional_inputs,
            vp,
        );
        if ctx_idx == 276 {
            // DecodeTerminate
            let ad = decode_terminate(bs, state);
            decoded.push(ad);
        } else {
            let mut idx1 = sh.cabac_init_idc as usize;
            let idx2 = sh.slice_qp_y as usize;
            // due to CABAC State table set up, the first index is for I slices, and subsequent ones are for P/B slices
            // with cabac_init_idc shifted by one. We compensate that here by adding 1 for all ctx_idx >= 54
            if !is_slice_type(sh.slice_type, "I")
                && !is_slice_type(sh.slice_type, "SI")
                && ctx_idx >= 54
            {
                idx1 += 1;
            }

            if CABAC_DEBUG {
                debug!(target: "decode","cod_i_offset: {:x}", state.cod_i_offset << 7);
                debug!(target: "decode","state ctx_idx-2: {:x}", state.states[idx1][idx2][ctx_idx-2].p_state_idx);
                debug!(target: "decode","state ctx_idx-1: {:x}", state.states[idx1][idx2][ctx_idx-1].p_state_idx);
                debug!(target: "decode","idx1 : {}, idx2 : {}, idx3 : {}", idx1, idx2, ctx_idx);
                debug!(target: "decode","state: {:x}", state.states[idx1][idx2][ctx_idx].p_state_idx);

                if ctx_idx + 1 < cabac_tables::CONTEXT_MODEL_COUNT {
                    debug!(target: "decode","state ctx_idx+1: {:x}", state.states[idx1][idx2][ctx_idx+1].p_state_idx);
                }

                if ctx_idx + 2 < cabac_tables::CONTEXT_MODEL_COUNT {
                    debug!(target: "decode","state ctx_idx+2: {:x}", state.states[idx1][idx2][ctx_idx+2].p_state_idx);
                }
            }

            let ad = arith_decode(idx1, idx2, ctx_idx, bs, state);
            if CABAC_DEBUG {
                debug!(
                    "updated state: {:x}",
                    state.states[idx1][idx2][ctx_idx].p_state_idx
                );
            }
            decoded.push(ad);
        }

        match debinarization(&(syntax_element.to_owned() + "_prefix"), &decoded, sh) {
            Some(x) => {
                res = x;
                still_decoding = false;
                bin_idx += 1;
            }
            _ => bin_idx += 1, // just continue if the result is None
        }
    }

    let mut suffix_offset_value = 0;
    if is_slice_type(sh.slice_type, "SI") {
        // prefix and suffix specified in 9.3.2.5

        // if 1, then we have to decode the rest as an I slice value
        if res != 0 {
            max_bin_idx_ctx = 6;
            ctx_idx_offset = 3;

            suffix_offset_value = 1;
            bin_idx = 0;
        } else {
            decode_suffix = false;
        }
        decoded.clear();
    } else if is_slice_type(sh.slice_type, "P") || is_slice_type(sh.slice_type, "SP") {
        // if the prefix exists then we change the conditions, else keep it the same while decoding
        if res == 1 {
            suffix_offset_value = 5;
            max_bin_idx_ctx = 5;
            ctx_idx_offset = 17;
            bin_idx = 0;
        }
    } else if is_slice_type(sh.slice_type, "B") {
        // prefix and suffix specified in 9.3.2.5

        // if it's 100, then we have to decode the prefix which contains I slice values
        if res == 100 {
            max_bin_idx_ctx = 5;
            ctx_idx_offset = 32;
            suffix_offset_value = 23;
            bin_idx = 0;
        } else {
            decode_suffix = false;
        }
        decoded.clear();
    }

    if decode_suffix {
        // suffix decoding parameters
        let mut still_decoding = true;

        // This does a bit-wise arithmetic decoding, and performs a debinarization on the recovered bitstream
        while still_decoding {
            // provide additional inputs for 9.3.3.1.2 (mb_type decoding)
            if ctx_idx_offset == 3 {
                if bin_idx == 4 || bin_idx == 5 {
                    additional_inputs.push(decoded[3] as usize);
                } else {
                    additional_inputs.clear();
                }
            } else if ctx_idx_offset == 14 {
                if bin_idx == 2 {
                    additional_inputs.push(decoded[1] as usize);
                } else {
                    additional_inputs.clear();
                }
            } else if ctx_idx_offset == 17 {
                if bin_idx == 4 {
                    // because we kept the initial bit, b3 for the suffix is decoded[4]
                    additional_inputs.push(decoded[4] as usize);
                } else {
                    additional_inputs.clear();
                }
            } else if ctx_idx_offset == 27 {
                if bin_idx == 2 {
                    additional_inputs.push(decoded[1] as usize);
                } else {
                    additional_inputs.clear();
                }
            } else if ctx_idx_offset == 32 {
                if bin_idx == 4 {
                    additional_inputs.push(decoded[3] as usize);
                } else {
                    additional_inputs.clear();
                }
            } else if ctx_idx_offset == 36 {
                if bin_idx == 2 {
                    additional_inputs.push(decoded[1] as usize);
                } else {
                    additional_inputs.clear();
                }
            }

            let ctx_idx = get_ctx_idx(
                syntax_element,
                bin_idx,
                max_bin_idx_ctx,
                ctx_idx_offset,
                curr_mb_idx,
                sh,
                sd,
                ctx_block_cat,
                &additional_inputs,
                vp,
            );
            if ctx_idx == 276 {
                // DecodeTerminate
                let ad = decode_terminate(bs, state);
                decoded.push(ad);
            } else {
                let mut idx1 = sh.cabac_init_idc as usize;
                let idx2 = sh.slice_qp_y as usize;
                // due to CABAC State table set up, the first index is for I slices, and subsequent ones are for P/B slices
                // with cabac_init_idc shifted by one. We compensate that here by adding 1 for all ctx_idx >= 54
                if !is_slice_type(sh.slice_type, "I")
                    && !is_slice_type(sh.slice_type, "SI")
                    && ctx_idx >= 54
                {
                    idx1 += 1;
                }

                if CABAC_DEBUG {
                    debug!(target: "decode","cod_i_offset: {:x}", state.cod_i_offset << 7);
                    debug!(target: "decode","state ctx_idx-2: {:x}", state.states[idx1][idx2][ctx_idx-2].p_state_idx);
                    debug!(target: "decode","state ctx_idx-1: {:x}", state.states[idx1][idx2][ctx_idx-1].p_state_idx);
                    debug!(target: "decode","state: {:x}", state.states[idx1][idx2][ctx_idx].p_state_idx);

                    if ctx_idx + 1 < cabac_tables::CONTEXT_MODEL_COUNT {
                        debug!(target: "decode","state ctx_idx+1: {:x}", state.states[idx1][idx2][ctx_idx+1].p_state_idx);
                    }

                    if ctx_idx + 2 < cabac_tables::CONTEXT_MODEL_COUNT {
                        debug!(target: "decode","state ctx_idx+2: {:x}", state.states[idx1][idx2][ctx_idx+2].p_state_idx);
                    }

                    debug!(target: "decode","Indices:");
                    debug!(target: "decode","\tcabac_init_idc: {}", idx1);
                    debug!(target: "decode","\tslice_qp_y: {}", idx2);
                    debug!(target: "decode","\tctx_idx: {}", ctx_idx);
                }

                let ad = arith_decode(idx1, idx2, ctx_idx, bs, state);
                if CABAC_DEBUG {
                    debug!(
                        "updated state: {:x}",
                        state.states[idx1][idx2][ctx_idx].p_state_idx
                    );
                }

                decoded.push(ad);
            }

            match debinarization(&(syntax_element.to_owned() + "_suffix"), &decoded, sh) {
                Some(x) => {
                    res = x + suffix_offset_value;
                    still_decoding = false;
                }
                _ => bin_idx += 1, // just continue if the result is None
            }
        }
    }

    res
}

/// Similar to the coeffs above, mvd uses UEGK split between prefix and suffix. We use a separate
/// decode method to handle this element
fn cabac_decode_mvd(
    syntax_element: &str,
    bs: &mut ByteStream,
    state: &mut CABACState,
    curr_mb_idx: usize,
    sh: &SliceHeader,
    sd: &mut SliceData,
    vp: &VideoParameters,
    additional_inputs: &[usize],
) -> i32 {
    let mut decoded: Vec<u8> = Vec::new();
    let mut res: i32 = 0;

    let mut ctx_idx: usize;

    let mut bin_idx = 0;
    let mut still_decoding = true;

    // This does a bit-wise arithmetic decoding, and performs a debinarization on the recovered bitstream
    while still_decoding {
        if syntax_element == "mvd_l0_0" || syntax_element == "mvd_l1_0" {
            ctx_idx = 40;
            if bin_idx == 0 {
                // Section 9.3.3.1.1.7
                let comp_idx: usize = 0;

                let pred_mode_equal_flag_a: usize;
                let pred_mode_equal_flag_b: usize;
                let abs_mvd_comp_b: u32;
                let abs_mvd_comp_a: u32;
                let mb_part_idx = additional_inputs[0];
                let mut sub_mb_part_idx: usize = 0;

                if additional_inputs.len() == 2 {
                    sub_mb_part_idx = additional_inputs[1];
                }

                // use 6.4.11.7 for neighbor info
                let res =
                    sd.get_neighbor_partitions(curr_mb_idx, mb_part_idx, sub_mb_part_idx, vp, true);

                let mb_a: MacroBlock = res.0;
                let mb_b: MacroBlock = res.1;
                let mb_part_idx_a = res.4;
                let mb_part_idx_b = res.5;
                let sub_mb_part_idx_a = res.8;
                let sub_mb_part_idx_b = res.9;

                // set pred_mode_equal_flag_a
                if mb_a.mb_type == MbType::BDirect16x16 || mb_a.mb_type == MbType::BSkip {
                    pred_mode_equal_flag_a = 0;
                } else if mb_a.mb_type == MbType::P8x8 || mb_a.mb_type == MbType::B8x8 {
                    let cur_sub_pred_mode = mb_a.sub_mb_part_pred_mode(mb_part_idx_a);

                    if (syntax_element == "mvd_l1_0"
                        && cur_sub_pred_mode != MbPartPredMode::PredL1
                        && cur_sub_pred_mode != MbPartPredMode::BiPred)
                        || (syntax_element == "mvd_l0_0"
                            && cur_sub_pred_mode != MbPartPredMode::PredL0
                            && cur_sub_pred_mode != MbPartPredMode::BiPred)
                    {
                        pred_mode_equal_flag_a = 0;
                    } else {
                        pred_mode_equal_flag_a = 1;
                    }
                } else {
                    let cur_pred_mode = mb_a.mb_part_pred_mode(mb_part_idx_a);
                    if (syntax_element == "mvd_l1_0"
                        && cur_pred_mode != MbPartPredMode::PredL1
                        && cur_pred_mode != MbPartPredMode::BiPred)
                        || (syntax_element == "mvd_l0_0"
                            && cur_pred_mode != MbPartPredMode::PredL0
                            && cur_pred_mode != MbPartPredMode::BiPred)
                    {
                        pred_mode_equal_flag_a = 0;
                    } else {
                        pred_mode_equal_flag_a = 1;
                    }
                }

                // set pred_mode_equal_flag_b
                if mb_b.mb_type == MbType::BDirect16x16 || mb_b.mb_type == MbType::BSkip {
                    pred_mode_equal_flag_b = 0;
                } else if mb_b.mb_type == MbType::P8x8 || mb_b.mb_type == MbType::B8x8 {
                    let cur_sub_pred_mode = mb_b.sub_mb_part_pred_mode(mb_part_idx_b);

                    if (syntax_element == "mvd_l1_0"
                        && cur_sub_pred_mode != MbPartPredMode::PredL1
                        && cur_sub_pred_mode != MbPartPredMode::BiPred)
                        || (syntax_element == "mvd_l0_0"
                            && cur_sub_pred_mode != MbPartPredMode::PredL0
                            && cur_sub_pred_mode != MbPartPredMode::BiPred)
                    {
                        pred_mode_equal_flag_b = 0;
                    } else {
                        pred_mode_equal_flag_b = 1;
                    }
                } else {
                    let cur_pred_mode = mb_b.mb_part_pred_mode(mb_part_idx_b);
                    if (syntax_element == "mvd_l1_0"
                        && cur_pred_mode != MbPartPredMode::PredL1
                        && cur_pred_mode != MbPartPredMode::BiPred)
                        || (syntax_element == "mvd_l0_0"
                            && cur_pred_mode != MbPartPredMode::PredL0
                            && cur_pred_mode != MbPartPredMode::BiPred)
                    {
                        pred_mode_equal_flag_b = 0;
                    } else {
                        pred_mode_equal_flag_b = 1;
                    }
                }

                if !mb_a.available
                    || mb_a.mb_type == MbType::PSkip
                    || mb_a.mb_type == MbType::BSkip
                    || mb_a.is_intra_non_mut()
                    || pred_mode_equal_flag_a == 0
                {
                    abs_mvd_comp_a = 0;
                } else {
                    if comp_idx == 1
                        && sh.mbaff_frame_flag
                        && !sd.mb_field_decoding_flag[curr_mb_idx]
                        && sd.mb_field_decoding_flag[mb_a.mb_idx]
                    {
                        if syntax_element == "mvd_l1_0" {
                            abs_mvd_comp_a = mb_a.mvd_l1[mb_part_idx_a][sub_mb_part_idx_a][comp_idx]
                                .abs() as u32
                                * 2;
                        } else {
                            abs_mvd_comp_a = mb_a.mvd_l0[mb_part_idx_a][sub_mb_part_idx_a][comp_idx]
                                .abs() as u32
                                * 2;
                        }
                    } else if comp_idx == 1
                        && sh.mbaff_frame_flag
                        && sd.mb_field_decoding_flag[curr_mb_idx]
                        && !sd.mb_field_decoding_flag[mb_a.mb_idx]
                    {
                        if syntax_element == "mvd_l1_0" {
                            abs_mvd_comp_a = mb_a.mvd_l1[mb_part_idx_a][sub_mb_part_idx_a][comp_idx]
                                .abs() as u32
                                / 2;
                        } else {
                            abs_mvd_comp_a = mb_a.mvd_l0[mb_part_idx_a][sub_mb_part_idx_a][comp_idx]
                                .abs() as u32
                                / 2;
                        }
                    } else {
                        if syntax_element == "mvd_l1_0" {
                            abs_mvd_comp_a = mb_a.mvd_l1[mb_part_idx_a][sub_mb_part_idx_a][comp_idx]
                                .abs() as u32;
                        } else {
                            abs_mvd_comp_a = mb_a.mvd_l0[mb_part_idx_a][sub_mb_part_idx_a][comp_idx]
                                .abs() as u32;
                        }
                    }
                }

                if !mb_b.available
                    || mb_b.mb_type == MbType::PSkip
                    || mb_b.mb_type == MbType::BSkip
                    || mb_b.is_intra_non_mut()
                    || pred_mode_equal_flag_b == 0
                {
                    abs_mvd_comp_b = 0;
                } else {
                    if comp_idx == 1
                        && sh.mbaff_frame_flag
                        && !sd.mb_field_decoding_flag[curr_mb_idx]
                        && sd.mb_field_decoding_flag[mb_b.mb_idx]
                    {
                        if syntax_element == "mvd_l1_0" {
                            abs_mvd_comp_b = mb_b.mvd_l1[mb_part_idx_b][sub_mb_part_idx_b][comp_idx]
                                .abs() as u32
                                * 2;
                        } else {
                            abs_mvd_comp_b = mb_b.mvd_l0[mb_part_idx_b][sub_mb_part_idx_b][comp_idx]
                                .abs() as u32
                                * 2;
                        }
                    } else if comp_idx == 1
                        && sh.mbaff_frame_flag
                        && sd.mb_field_decoding_flag[curr_mb_idx]
                        && !sd.mb_field_decoding_flag[mb_b.mb_idx]
                    {
                        if syntax_element == "mvd_l1_0" {
                            abs_mvd_comp_b = mb_b.mvd_l1[mb_part_idx_b][sub_mb_part_idx_b][comp_idx]
                                .abs() as u32
                                / 2;
                        } else {
                            abs_mvd_comp_b = mb_b.mvd_l0[mb_part_idx_b][sub_mb_part_idx_b][comp_idx]
                                .abs() as u32
                                / 2;
                        }
                    } else {
                        if syntax_element == "mvd_l1_0" {
                            abs_mvd_comp_b = mb_b.mvd_l1[mb_part_idx_b][sub_mb_part_idx_b][comp_idx]
                                .abs() as u32;
                        } else {
                            abs_mvd_comp_b = mb_b.mvd_l0[mb_part_idx_b][sub_mb_part_idx_b][comp_idx]
                                .abs() as u32;
                        }
                    }
                }

                if abs_mvd_comp_a + abs_mvd_comp_b > 32 {
                    ctx_idx += 2;
                } else if abs_mvd_comp_a + abs_mvd_comp_b > 2 {
                    ctx_idx += 1;
                }
            }
        } else {
            ctx_idx = 47;
            if bin_idx == 0 {
                // Section 9.3.3.1.1.7
                let comp_idx: usize = 1;

                let pred_mode_equal_flag_a: usize;
                let pred_mode_equal_flag_b: usize;
                let abs_mvd_comp_a: u32;
                let abs_mvd_comp_b: u32;
                let mb_part_idx = additional_inputs[0];
                let mut sub_mb_part_idx: usize = 0;

                if additional_inputs.len() == 2 {
                    sub_mb_part_idx = additional_inputs[1];
                }

                // use 6.4.11.7 for neighbor info
                let res =
                    sd.get_neighbor_partitions(curr_mb_idx, mb_part_idx, sub_mb_part_idx, vp, true);

                let mb_a: MacroBlock = res.0;
                let mb_b: MacroBlock = res.1;
                let mb_part_idx_a = res.4;
                let mb_part_idx_b = res.5;
                let sub_mb_part_idx_a = res.8;
                let sub_mb_part_idx_b = res.9;

                // set pred_mode_equal_flag_a
                if mb_a.mb_type == MbType::BDirect16x16 || mb_a.mb_type == MbType::BSkip {
                    pred_mode_equal_flag_a = 0;
                } else if mb_a.mb_type == MbType::P8x8 || mb_a.mb_type == MbType::B8x8 {
                    let cur_sub_pred_mode = mb_a.sub_mb_part_pred_mode(mb_part_idx_a);

                    if (syntax_element == "mvd_l1_1"
                        && cur_sub_pred_mode != MbPartPredMode::PredL1
                        && cur_sub_pred_mode != MbPartPredMode::BiPred)
                        || (syntax_element == "mvd_l0_1"
                            && cur_sub_pred_mode != MbPartPredMode::PredL0
                            && cur_sub_pred_mode != MbPartPredMode::BiPred)
                    {
                        pred_mode_equal_flag_a = 0;
                    } else {
                        pred_mode_equal_flag_a = 1;
                    }
                } else {
                    let cur_pred_mode = mb_a.mb_part_pred_mode(mb_part_idx_a);

                    if (syntax_element == "mvd_l1_1"
                        && cur_pred_mode != MbPartPredMode::PredL1
                        && cur_pred_mode != MbPartPredMode::BiPred)
                        || (syntax_element == "mvd_l0_1"
                            && cur_pred_mode != MbPartPredMode::PredL0
                            && cur_pred_mode != MbPartPredMode::BiPred)
                    {
                        pred_mode_equal_flag_a = 0;
                    } else {
                        pred_mode_equal_flag_a = 1;
                    }
                }

                // set pred_mode_equal_flag_b
                if mb_b.mb_type == MbType::BDirect16x16 || mb_b.mb_type == MbType::BSkip {
                    pred_mode_equal_flag_b = 0;
                } else if mb_b.mb_type == MbType::P8x8 || mb_b.mb_type == MbType::B8x8 {
                    let cur_sub_pred_mode = mb_b.sub_mb_part_pred_mode(mb_part_idx_b);

                    if (syntax_element == "mvd_l1_1"
                        && cur_sub_pred_mode != MbPartPredMode::PredL1
                        && cur_sub_pred_mode != MbPartPredMode::BiPred)
                        || (syntax_element == "mvd_l0_1"
                            && cur_sub_pred_mode != MbPartPredMode::PredL0
                            && cur_sub_pred_mode != MbPartPredMode::BiPred)
                    {
                        pred_mode_equal_flag_b = 0;
                    } else {
                        pred_mode_equal_flag_b = 1;
                    }
                } else {
                    let cur_pred_mode = mb_b.mb_part_pred_mode(mb_part_idx_b);

                    if (syntax_element == "mvd_l1_1"
                        && cur_pred_mode != MbPartPredMode::PredL1
                        && cur_pred_mode != MbPartPredMode::BiPred)
                        || (syntax_element == "mvd_l0_1"
                            && cur_pred_mode != MbPartPredMode::PredL0
                            && cur_pred_mode != MbPartPredMode::BiPred)
                    {
                        pred_mode_equal_flag_b = 0;
                    } else {
                        pred_mode_equal_flag_b = 1;
                    }
                }

                if !mb_a.available
                    || mb_a.mb_type == MbType::PSkip
                    || mb_a.mb_type == MbType::BSkip
                    || mb_a.is_intra_non_mut()
                    || pred_mode_equal_flag_a == 0
                {
                    abs_mvd_comp_a = 0;
                } else {
                    if comp_idx == 1
                        && sh.mbaff_frame_flag
                        && !sd.mb_field_decoding_flag[curr_mb_idx]
                        && sd.mb_field_decoding_flag[mb_a.mb_idx]
                    {
                        if syntax_element == "mvd_l1_1" {
                            abs_mvd_comp_a = mb_a.mvd_l1[mb_part_idx_a][sub_mb_part_idx_a][comp_idx]
                                .abs() as u32
                                * 2;
                        } else {
                            abs_mvd_comp_a = mb_a.mvd_l0[mb_part_idx_a][sub_mb_part_idx_a][comp_idx]
                                .abs() as u32
                                * 2;
                        }
                    } else if comp_idx == 1
                        && sh.mbaff_frame_flag
                        && sd.mb_field_decoding_flag[curr_mb_idx]
                        && !sd.mb_field_decoding_flag[mb_a.mb_idx]
                    {
                        if syntax_element == "mvd_l1_1" {
                            abs_mvd_comp_a = mb_a.mvd_l1[mb_part_idx_a][sub_mb_part_idx_a][comp_idx]
                                .abs() as u32
                                / 2;
                        } else {
                            abs_mvd_comp_a = mb_a.mvd_l0[mb_part_idx_a][sub_mb_part_idx_a][comp_idx]
                                .abs() as u32
                                / 2;
                        }
                    } else {
                        if syntax_element == "mvd_l1_1" {
                            abs_mvd_comp_a = mb_a.mvd_l1[mb_part_idx_a][sub_mb_part_idx_a][comp_idx]
                                .abs() as u32;
                        } else {
                            abs_mvd_comp_a = mb_a.mvd_l0[mb_part_idx_a][sub_mb_part_idx_a][comp_idx]
                                .abs() as u32;
                        }
                    }
                }

                if !mb_b.available
                    || mb_b.mb_type == MbType::PSkip
                    || mb_b.mb_type == MbType::BSkip
                    || mb_b.is_intra_non_mut()
                    || pred_mode_equal_flag_b == 0
                {
                    abs_mvd_comp_b = 0;
                } else {
                    if comp_idx == 1
                        && sh.mbaff_frame_flag
                        && !sd.mb_field_decoding_flag[curr_mb_idx]
                        && sd.mb_field_decoding_flag[mb_b.mb_idx]
                    {
                        if syntax_element == "mvd_l1_1" {
                            abs_mvd_comp_b = mb_b.mvd_l1[mb_part_idx_b][sub_mb_part_idx_b][comp_idx]
                                .abs() as u32
                                * 2;
                        } else {
                            abs_mvd_comp_b = mb_b.mvd_l0[mb_part_idx_b][sub_mb_part_idx_b][comp_idx]
                                .abs() as u32
                                * 2;
                        }
                    } else if comp_idx == 1
                        && sh.mbaff_frame_flag
                        && sd.mb_field_decoding_flag[curr_mb_idx]
                        && !sd.mb_field_decoding_flag[mb_b.mb_idx]
                    {
                        if syntax_element == "mvd_l1_1" {
                            abs_mvd_comp_b = mb_b.mvd_l1[mb_part_idx_b][sub_mb_part_idx_b][comp_idx]
                                .abs() as u32
                                / 2;
                        } else {
                            abs_mvd_comp_b = mb_b.mvd_l0[mb_part_idx_b][sub_mb_part_idx_b][comp_idx]
                                .abs() as u32
                                / 2;
                        }
                    } else {
                        if syntax_element == "mvd_l1_1" {
                            abs_mvd_comp_b = mb_b.mvd_l1[mb_part_idx_b][sub_mb_part_idx_b][comp_idx]
                                .abs() as u32;
                        } else {
                            abs_mvd_comp_b = mb_b.mvd_l0[mb_part_idx_b][sub_mb_part_idx_b][comp_idx]
                                .abs() as u32;
                        }
                    }
                }

                if abs_mvd_comp_a + abs_mvd_comp_b > 32 {
                    ctx_idx += 2;
                } else if abs_mvd_comp_a + abs_mvd_comp_b > 2 {
                    ctx_idx += 1;
                } // else less than 2, so offset is 0
            }
        }

        if bin_idx == 1 {
            ctx_idx += 3;
        } else if bin_idx == 2 {
            ctx_idx += 4;
        } else if bin_idx == 3 {
            ctx_idx += 5;
        } else if bin_idx >= 4 {
            ctx_idx += 6;
        }

        let idx1 = sh.cabac_init_idc as usize;
        let idx2 = sh.slice_qp_y as usize;

        if CABAC_DEBUG {
            debug!(target: "decode","cod_i_offset: {:x}", state.cod_i_offset << 7);
            debug!(target: "decode","state: {:x}", state.states[idx1][idx2][ctx_idx].p_state_idx);
        }

        let ad = arith_decode(idx1, idx2, ctx_idx, bs, state);
        if CABAC_DEBUG {
            debug!(
                "updated state: {:x}",
                state.states[idx1][idx2][ctx_idx].p_state_idx
            );
        }
        decoded.push(ad);

        match read_truncated_unary_value(9, &decoded) {
            Some(x) => {
                if CABAC_DEBUG {
                    debug!(target: "decode","decoded: {:?}", decoded);
                }
                res = x;
                still_decoding = false;
                bin_idx += 1;
            }
            _ => bin_idx += 1, // just continue if the result is None
        }
    }

    // Implement exp_golomb_decode_eq_prob from the JM software (see cabac.c:static unsigned int exp_golomb_decode_eq_prob)
    if res == 2i32.pow(9) - 1 {
        let mut k = 3; // K value
        let mut symbol = 0;
        let mut binary_symbol = 0;

        // there seems to be an extra check at the start
        while {
            let ad = decode_bypass(bs, state);
            if ad == 1 {
                symbol += 1 << k;
                k += 1;
            }
            ad != 0
        } {}
        k -= 1;
        while k >= 0 {
            let ad = decode_bypass(bs, state);
            if ad == 1 {
                binary_symbol |= 1 << k;
            }
            k -= 1;
        }
        res = symbol + binary_symbol + 9; // the 9 is the truncated unary value that we have read
    }
    if res != 0 {
        let ad = decode_bypass(bs, state);
        if ad == 1 {
            res = -res;
        }
    }

    res
}

/// CABAC decode a syntax element
///
/// Decoding is split into three steps:
/// 1. Context model selection for arithmetic decoding
/// 2. Context model updates
/// 3. Debinarization
///
/// First a binarization is derived for the syntax_element
/// The binarization and the sequence of parsed bins determines the decoding process
///
///
/// Takes in a request for a syntax element, the bytestream, and values of prior
/// parsed syntax elements
///
/// Outputs the syntax element
///
/// Returns list of decoded values
pub fn cabac_decode(
    syntax_element: &str,
    bs: &mut ByteStream,
    state: &mut CABACState,
    curr_mb_idx: usize,
    sh: &SliceHeader,
    sd: &mut SliceData,
    vp: &VideoParameters,
    ctx_block_cat: u8,
    mut additional_inputs: Vec<usize>,
) -> i32 {
    if CABAC_DEBUG {
        debug!(target: "decode","");
        debug!(target: "decode","");
        debug!(target: "decode","--START READING SYNTAX ELEMENT");
    }
    // harder cases
    if syntax_element == "coded_block_pattern" {
        return cabac_decode_cbp(syntax_element, bs, state, curr_mb_idx, sh, sd, vp);
    } else if syntax_element == "coeff_abs_level_minus1" {
        return cabac_decode_coeff(
            syntax_element,
            bs,
            state,
            curr_mb_idx,
            sh,
            sd,
            vp,
            ctx_block_cat,
            &additional_inputs,
        );
    } else if syntax_element == "mb_type" {
        return cabac_decode_mbtype(
            syntax_element,
            bs,
            state,
            curr_mb_idx,
            sh,
            sd,
            vp,
            ctx_block_cat,
            additional_inputs,
        );
    } else if syntax_element == "mvd_l0_0"
        || syntax_element == "mvd_l0_1"
        || syntax_element == "mvd_l1_0"
        || syntax_element == "mvd_l1_1"
    {
        return cabac_decode_mvd(
            syntax_element,
            bs,
            state,
            curr_mb_idx,
            sh,
            sd,
            vp,
            &additional_inputs,
        );
    }

    let mut decoded: Vec<u8> = Vec::new();
    let mut res: i32 = 0;

    let (max_bin_idx_ctx, ctx_idx_offset, bypass_flag) =
        get_binarization_params(syntax_element, ctx_block_cat, sh, sd, curr_mb_idx);
    // if we're not bypassing, then we're arithmetic decoding
    if !bypass_flag {
        let mut bin_idx = 0;
        let mut still_decoding = true;

        // This does a bit-wise arithmetic decoding, and performs a debinarization on the recovered bitstream
        while still_decoding {
            if syntax_element == "sub_mb_type" {
                // add additional inputs for B slice sub_mb_type
                if bin_idx == 2 && is_slice_type(sh.slice_type, "B") {
                    additional_inputs.push(decoded[1] as usize);
                }
            }

            let ctx_idx = get_ctx_idx(
                syntax_element,
                bin_idx,
                max_bin_idx_ctx,
                ctx_idx_offset,
                curr_mb_idx,
                sh,
                sd,
                ctx_block_cat,
                &additional_inputs,
                vp,
            );

            if ctx_idx == 276 {
                // DecodeTerminate
                let ad = decode_terminate(bs, state);
                decoded.push(ad);
            } else {
                let mut idx1 = sh.cabac_init_idc as usize;
                let idx2 = sh.slice_qp_y as usize;
                // due to CABAC State table set up, the first index is for I slices, and subsequent ones are for P/B slices
                // with cabac_init_idc shifted by one. We compensate that here by adding 1 for all ctx_idx >= 54
                if !is_slice_type(sh.slice_type, "I")
                    && !is_slice_type(sh.slice_type, "SI")
                    && ctx_idx >= 54
                {
                    idx1 += 1;
                }

                if CABAC_DEBUG {
                    debug!(target: "decode","cod_i_offset: {:x}", state.cod_i_offset << 7);
                    debug!(target: "decode","cabac_init_idc: {}", idx1);
                    debug!(target: "decode","slice_qp_y: {}", idx2);
                    debug!(target: "decode","ctx_idx: {}", ctx_idx);
                    debug!(target: "decode","state ctx_idx-3: {:x}", state.states[idx1][idx2][ctx_idx-3].p_state_idx);
                    debug!(target: "decode","state ctx_idx-2: {:x}", state.states[idx1][idx2][ctx_idx-2].p_state_idx);
                    debug!(target: "decode","state ctx_idx-1: {:x}", state.states[idx1][idx2][ctx_idx-1].p_state_idx);
                    debug!(target: "decode","state: {:x}", state.states[idx1][idx2][ctx_idx].p_state_idx);
                    if ctx_idx + 1 < cabac_tables::CONTEXT_MODEL_COUNT {
                        debug!(target: "decode","state ctx_idx+1: {:x}", state.states[idx1][idx2][ctx_idx+1].p_state_idx);
                    }
                    if ctx_idx + 2 < cabac_tables::CONTEXT_MODEL_COUNT {
                        debug!(target: "decode","state ctx_idx+2: {:x}", state.states[idx1][idx2][ctx_idx+2].p_state_idx);
                    }
                    if ctx_idx + 3 < cabac_tables::CONTEXT_MODEL_COUNT {
                        debug!(target: "decode","state ctx_idx+3: {:x}", state.states[idx1][idx2][ctx_idx+3].p_state_idx);
                    }
                }
                let ad = arith_decode(idx1, idx2, ctx_idx, bs, state);
                if CABAC_DEBUG {
                    debug!(
                        "updated state: {:x}",
                        state.states[idx1][idx2][ctx_idx].p_state_idx
                    );
                }

                decoded.push(ad);
            }

            match debinarization(syntax_element, &decoded, sh) {
                Some(x) => {
                    res = x;
                    still_decoding = false;
                }
                _ => bin_idx += 1, // just continue if the result is None
            }
        }
    } else {
        // bypass context model approach
        res = decode_bypass(bs, state) as i32;
    }
    res
}
