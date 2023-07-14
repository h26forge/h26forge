//! CABAC entropy encoding.
//!
//! General Model for CABAC Encoding:
//! 1. Binarize the syntax element (Table 9-34)
//! 2. Get the appropriate context model
//! 3. Binary Arithmetic Encode the binarized value based on the context model (described in 9.3.4)
//!
//! NOTE: the bypass encoding engine just means that it's an equal probability encoding,
//!       rather than based on a context model

use crate::common::cabac_tables;
use crate::common::data_structures::MacroBlock;
use crate::common::data_structures::MbPartPredMode;
use crate::common::data_structures::MbType;
use crate::common::data_structures::NeighborMB;
use crate::common::data_structures::SliceData;
use crate::common::data_structures::SliceHeader;
use crate::common::data_structures::SubMbType;
use crate::common::data_structures::TransformBlock;
use crate::common::data_structures::VideoParameters;
use crate::common::helper::clip3;
use crate::common::helper::encoder_formatted_print;
use crate::common::helper::is_slice_type;
use crate::encoder::binarization_functions::*;
use log::debug;
use std::cmp;

const CABAC_DEBUG: bool = false;

/// Maintain the probability state index and the most probable state
#[derive(Debug, Copy, Clone)]
pub struct SyntaxElementState {
    pub p_state_idx: usize, // corresponds to the probability state index
    pub val_mps: u8,        // corresponds to the value of the most probably symbol
}

impl SyntaxElementState {
    fn new(p_state_idx: usize, val_mps: u8) -> SyntaxElementState {
        SyntaxElementState {
            p_state_idx,
            val_mps,
        }
    }
}

/// The entire CABAC state maintained across a slice
pub struct CABACState {
    pub states: Vec<Vec<Vec<SyntaxElementState>>>,
    pub cod_i_range: usize,
    pub cod_i_offset: u32,
    pub first_bit_flag: bool,
    pub bits_outstanding: u32,
    pub bin_counts_in_nal_units: u32,
}

/// Initialize the CABAC State
///
/// Load up all possible ctxIdx values into the state. Similar
/// process to OpenH264.
pub fn initialize_state(nal_unit_count: u32) -> CABACState {
    // Initialize according to 9.3.4.1
    let mut r = CABACState {
        states: Vec::new(),
        cod_i_range: 510,
        cod_i_offset: 0, // same as cod_i_low
        first_bit_flag: true,
        bits_outstanding: 0,
        bin_counts_in_nal_units: nal_unit_count,
    };

    for i in 0..4 {
        // the different cabac_init_idc values
        r.states.push(Vec::new());
        for j in 0..52 {
            // the possible slice_qp_y values
            r.states[i as usize].push(Vec::new());
            for k in 0..cabac_tables::CONTEXT_MODEL_COUNT {
                // cabac_tables::CONTEXT_MODEL_COUNT
                // use slice_qp_y to assign pStateIdx and valMPS values
                let p_state_idx: usize;
                let val_mps: u8;

                let m = cabac_tables::CABAC_INIT_CONSTANTS[k][i][0]; // look up from cabac_table
                let n = cabac_tables::CABAC_INIT_CONSTANTS[k][i][1]; // look up from cabac_table

                let pre_ctx_state = clip3(1, 126, ((m * j) >> 4) + n) as usize;
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

    r
}

/// Follow Figure 9-9
fn put_bit(bit: u8, stream: &mut Vec<u8>, mut cs: &mut CABACState) {
    if cs.first_bit_flag {
        cs.first_bit_flag = false;
    } else {
        stream.push(bit);
    }

    while cs.bits_outstanding > 0 {
        stream.push(1 - bit);
        cs.bits_outstanding -= 1;
    }
}

/// Follow Figure 9-8
/// from_terminate is used to decide when to print out the cod_i_range for debugging
fn renorm(mut cs: &mut CABACState, from_terminate: bool) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();

    while cs.cod_i_range < 256 {
        if cs.cod_i_offset < 256 {
            put_bit(0, &mut res, cs);
        } else if cs.cod_i_offset >= 512 {
            cs.cod_i_offset -= 512;
            put_bit(1, &mut res, cs);
        } else {
            cs.cod_i_offset -= 256;
            cs.bits_outstanding += 1;
        }

        cs.cod_i_range <<= 1;
        cs.cod_i_offset <<= 1;
    }

    if !from_terminate && CABAC_DEBUG {
        debug!(target: "encode","after1 - cod_i_range: {}", cs.cod_i_range);
    }

    res
}

/// Follow figure 9-12
/// only called from encode_terminate
fn encode_flush(mut cs: &mut CABACState) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();

    cs.cod_i_range = 2;

    res.append(&mut renorm(cs, true));
    put_bit(((cs.cod_i_offset >> 9) & 1) as u8, &mut res, cs);

    // the paper writes it as WriteBits(((codILow >> 7) & 3) | 1, 2)
    let val = (cs.cod_i_offset >> 8) & 1;

    // In this flushing procedure, the last bit written by WriteBits( B, N )
    // is equal to 1.
    // When encoding end_of_slice_flag, this last bit is interpreted as the rbsp_stop_one_bit.
    res.push(val as u8);
    res.push(1);

    res
}

/// Follow figure 9-11
fn encode_terminate(bit: &u8, mut cs: &mut CABACState) -> Vec<u8> {
    if CABAC_DEBUG {
        debug!(target: "encode","encode_terminate called");
    }
    cs.cod_i_range -= 2;

    cs.bin_counts_in_nal_units += 1;

    if *bit != 0 {
        cs.cod_i_offset += cs.cod_i_range as u32;
        encode_flush(cs)
    } else {
        renorm(cs, true)
    }
}

fn encode_bypass(bit: u8, mut cs: &mut CABACState) -> Vec<u8> {
    if CABAC_DEBUG {
        debug!(target: "encode","encode_bypass called");
    }
    let mut res: Vec<u8> = Vec::new();

    cs.cod_i_offset <<= 1;

    if bit != 0 {
        cs.cod_i_offset += cs.cod_i_range as u32;
    }

    if cs.cod_i_offset >= 1024 {
        put_bit(1, &mut res, cs);
        cs.cod_i_offset -= 1024;
    } else if cs.cod_i_offset < 512 {
        put_bit(0, &mut res, cs);
    } else {
        cs.cod_i_offset -= 512;
        cs.bits_outstanding += 1;
    }

    cs.bin_counts_in_nal_units += 1;

    res
}

/// Follow Figure 9-7.
fn arithmetic_encode(
    bit: &u8,
    mut cs: &mut CABACState,
    idx1: usize,
    idx2: usize,
    idx3: usize,
) -> Vec<u8> {
    let cur_state = &mut cs.states[idx1][idx2][idx3];
    let q_cod_i_range_idx = (cs.cod_i_range >> 6) & 3;
    let cod_i_range_lps =
        cabac_tables::RANGE_TAB_LPS[cur_state.p_state_idx][q_cod_i_range_idx] as usize;

    if CABAC_DEBUG {
        debug!(target: "encode","before - cod_i_range: {}", cs.cod_i_range);
    }
    cs.cod_i_range -= cod_i_range_lps;

    if *bit != cur_state.val_mps {
        cs.cod_i_offset += cs.cod_i_range as u32;
        cs.cod_i_range = cod_i_range_lps;

        if cur_state.p_state_idx == 0 {
            cur_state.val_mps = 1 - cur_state.val_mps;
        }
        cur_state.p_state_idx = cabac_tables::TRANS_IDX_LPS[cur_state.p_state_idx] as usize;
    } else {
        cur_state.p_state_idx = cabac_tables::TRANS_IDX_MPS[cur_state.p_state_idx] as usize;
    }
    cs.bin_counts_in_nal_units += 1;

    renorm(cs, false) // decides whether to put in a bit or not
}

/// Slice Data - mb_skip_flag
pub fn cabac_encode_mb_skip_flag(
    se_val: bool,
    sh: &SliceHeader,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
    neighbor_info: (MacroBlock, MacroBlock),
) {
    let binarized = match se_val {
        true => 1u8,
        false => 0u8,
    };
    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_mb_skip_flag - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("mb_skip_flag", se_val, 63);
    }

    let mut ctx_idx: usize = 0;

    // binarization is fixed length of 1
    if is_slice_type(sh.slice_type, "P") || is_slice_type(sh.slice_type, "SP") {
        if CABAC_DEBUG {
            debug!(target: "encode","get_ctx_idx - mb_skip_flag (P/SP slices)");
        }
        ctx_idx = 11;
        // Decode according to 9.3.3.1.1.1
        let mut cond_term_flag_a: usize = 1;
        let mut cond_term_flag_b: usize = 1;

        let mb_a: MacroBlock = neighbor_info.0.clone();
        let mb_b: MacroBlock = neighbor_info.1.clone();

        // set cond_term_flag_a
        if !mb_a.available || mb_a.mb_skip_flag {
            cond_term_flag_a = 0;
        }

        if !mb_b.available || mb_b.mb_skip_flag {
            cond_term_flag_b = 0;
        }

        ctx_idx += cond_term_flag_a + cond_term_flag_b;
    } else if is_slice_type(sh.slice_type, "B") {
        if CABAC_DEBUG {
            debug!(target: "encode","get_ctx_idx - mb_skip_flag (B slices)");
        }
        ctx_idx = 24;

        // Decode according to 9.3.3.1.1.1
        let mut cond_term_flag_a: usize = 1;
        let mut cond_term_flag_b: usize = 1;

        let mb_a: MacroBlock = neighbor_info.0.clone();
        let mb_b: MacroBlock = neighbor_info.1.clone();

        // set cond_term_flag_a
        if !mb_a.available || mb_a.mb_skip_flag {
            cond_term_flag_a = 0;
        }

        if !mb_b.available || mb_b.mb_skip_flag {
            cond_term_flag_b = 0;
        }

        ctx_idx += cond_term_flag_a + cond_term_flag_b;
    }
    let idx1 = sh.cabac_init_idc as usize;

    let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;
    if CABAC_DEBUG {
        debug!(target: "encode","cabac_init_idc: {}", idx1);
        debug!(target: "encode","slice_qp_y : {}", idx2);
        debug!(target: "encode","ctx_idx : {}", ctx_idx);
        debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
        debug!(target: "encode","state-3: {:x}", cs.states[idx1][idx2][ctx_idx-3].p_state_idx);
        debug!(target: "encode","state-2: {:x}", cs.states[idx1][idx2][ctx_idx-2].p_state_idx);
        debug!(target: "encode","state-1: {:x}", cs.states[idx1][idx2][ctx_idx-1].p_state_idx);
        debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
        debug!(target: "encode","state+1: {:x}", cs.states[idx1][idx2][ctx_idx+1].p_state_idx);
        debug!(target: "encode","state+2: {:x}", cs.states[idx1][idx2][ctx_idx+2].p_state_idx);
        debug!(target: "encode","state+3: {:x}", cs.states[idx1][idx2][ctx_idx+3].p_state_idx);
    }
    stream.append(&mut arithmetic_encode(&binarized, cs, idx1, idx2, ctx_idx));

    if CABAC_DEBUG {
        debug!(target: "encode",
            "updated state: {:x}",
            cs.states[idx1][idx2][ctx_idx].p_state_idx
        );
    }
}

/// Slice Data - mb_field_decoding_flag
pub fn cabac_encode_mb_field_decoding_flag(
    se_val: bool,
    sh: &SliceHeader,
    sd: &SliceData,
    vp: &VideoParameters,
    curr_mb_idx: usize,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
) {
    let binarized = match se_val {
        true => 1u8,
        false => 0u8,
    };
    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_mb_field_decoding_flag - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("mb_field_decoding_flag", se_val, 63);
    }

    let mut ctx_idx: usize = 70;

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

    ctx_idx += cond_term_flag_a + cond_term_flag_b;
    let mut idx1 = sh.cabac_init_idc as usize;

    if !is_slice_type(sh.slice_type, "I") && !is_slice_type(sh.slice_type, "SI") && ctx_idx >= 54 {
        idx1 += 1;
    }

    let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;
    if CABAC_DEBUG {
        debug!(target: "encode","cabac_init_idc: {}", idx1);
        debug!(target: "encode","slice_qp_y : {}", idx2);
        debug!(target: "encode","ctx_idx : {}", ctx_idx);
        debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
        debug!(target: "encode","state-3: {:x}", cs.states[idx1][idx2][ctx_idx-3].p_state_idx);
        debug!(target: "encode","state-2: {:x}", cs.states[idx1][idx2][ctx_idx-2].p_state_idx);
        debug!(target: "encode","state-1: {:x}", cs.states[idx1][idx2][ctx_idx-1].p_state_idx);
        debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
        debug!(target: "encode","state+1: {:x}", cs.states[idx1][idx2][ctx_idx+1].p_state_idx);
        debug!(target: "encode","state+2: {:x}", cs.states[idx1][idx2][ctx_idx+2].p_state_idx);
        debug!(target: "encode","state+3: {:x}", cs.states[idx1][idx2][ctx_idx+3].p_state_idx);
    }
    stream.append(&mut arithmetic_encode(&binarized, cs, idx1, idx2, ctx_idx));

    if CABAC_DEBUG {
        debug!(target: "encode",
            "updated state: {:x}",
            cs.states[idx1][idx2][ctx_idx].p_state_idx
        );
    }
}

/// Slice Data - end_of_slice_flag
pub fn cabac_encode_end_of_slice_flag(se_val: bool, stream: &mut Vec<u8>, cs: &mut CABACState) {
    // binarization is fixed length of 1
    let binarized = match se_val {
        true => 1u8,
        false => 0u8,
    };
    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_end_of_slice_flag - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("end_of_slice_flag", se_val, 63);
    }
    // cabac max_bin_idx_ctx = 0; ctx_idx_offset = 276

    // binary encode with model
    stream.append(&mut encode_terminate(&binarized, cs));
}

/// Macroblock Data - mb_type
pub fn cabac_encode_mb_type(
    se_val: MbType,
    sh: &SliceHeader,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
    neighbor_info: (MacroBlock, MacroBlock),
) {
    // binarization specified in clause 9.3.2.5
    let (binarized, mut encode_suffix, prefix_len) = generate_mb_type_value(se_val, sh);
    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_mb_type - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("mb_type", se_val, 63);
    }
    // cabac
    // -   SI: prefix: max_bin_idx_ctx = 0; ctx_idx_offset = 0
    //         suffix: max_bin_idx_ctx = 6; ctx_idx_offset = 3
    // -    I: max_bin_idx_ctx = 6; ctx_idx_offset = 3
    // - P/SP: prefix: max_bin_idx_ctx = 2; ctx_idx_offset = 14
    //         suffix: max_bin_idx_ctx = 5; ctx_idx_offset = 17
    // -    B: prefix: max_bin_idx_ctx = 3; ctx_idx_offset = 27
    //         suffix: max_bin_idx_ctx = 5; ctx_idx_offset = 32

    let mut ctx_idx_offset = 0;

    // prefix encoding parameters
    if is_slice_type(sh.slice_type, "SI") {
        // prefix and suffix specified in 9.3.2.5
        ctx_idx_offset = 0;
    } else if is_slice_type(sh.slice_type, "I") {
        // follow description in 9.3.2.5
        ctx_idx_offset = 3;
    } else if is_slice_type(sh.slice_type, "P") || is_slice_type(sh.slice_type, "SP") {
        // prefix and suffix specified in 9.3.2.5
        ctx_idx_offset = 14;
    } else if is_slice_type(sh.slice_type, "B") {
        // prefix and suffix specified in 9.3.2.5
        ctx_idx_offset = 27;
    }

    // binary encode with model

    let mut encoded: Vec<u8> = Vec::new();
    let mut bin_idx = 0;

    for b in binarized.iter() {
        // decide whether to go into encode_suffix mode or not
        if bin_idx == prefix_len && encode_suffix {
            bin_idx = 0;
            encode_suffix = false; // to make sure we don't come in here again

            // suffix encoding parameters
            if is_slice_type(sh.slice_type, "SI") {
                // prefix and suffix specified in 9.3.2.5
                ctx_idx_offset = 3
            } else if is_slice_type(sh.slice_type, "I") {
                // follow description in 9.3.2.5
                panic!("cabac_encode_mb_type - no suffix for I slices!");
            } else if is_slice_type(sh.slice_type, "P") || is_slice_type(sh.slice_type, "SP") {
                // prefix and suffix specified in 9.3.2.5
                ctx_idx_offset = 17;
            } else if is_slice_type(sh.slice_type, "B") {
                // prefix and suffix specified in 9.3.2.5
                ctx_idx_offset = 32;
            }
        }

        // Context Model Selection
        let mut ctx_idx = ctx_idx_offset;

        if ctx_idx == 0 {
            // Decode according to 9.3.3.1.1.3

            // A is the block to the left
            // B is the block right above
            let mut cond_term_flag_a: usize = 1;
            let mut cond_term_flag_b: usize = 1;

            let mb_a: MacroBlock = neighbor_info.0.clone();
            let mb_b: MacroBlock = neighbor_info.1.clone();

            // set cond_term_flag_a
            if !mb_a.available || mb_a.mb_type == MbType::SI {
                cond_term_flag_a = 0;
            }

            if !mb_b.available || mb_b.mb_type == MbType::SI {
                cond_term_flag_b = 0;
            }

            ctx_idx += cond_term_flag_a + cond_term_flag_b;
        } else if ctx_idx == 3 {
            // only here for I slices or for SI slice suffix
            if bin_idx == 0 {
                // Getting info for mb_type following clause 9.3.3.1.1.3
                let mut cond_term_flag_a: usize = 1;
                let mut cond_term_flag_b: usize = 1;

                let mb_a: MacroBlock = neighbor_info.0.clone();
                let mb_b: MacroBlock = neighbor_info.1.clone();

                // set cond_term_flag_a
                if !mb_a.available || mb_a.mb_type == MbType::INxN {
                    cond_term_flag_a = 0;
                }

                if !mb_b.available || mb_b.mb_type == MbType::INxN {
                    cond_term_flag_b = 0;
                }

                ctx_idx += cond_term_flag_a + cond_term_flag_b;
            } else if bin_idx == 1 {
                ctx_idx = 276;
            } else if bin_idx == 2 {
                ctx_idx += 3;
            } else if bin_idx == 3 {
                ctx_idx += 4;
            } else if bin_idx == 4 {
                // implement section 9.3.3.1.2
                if binarized[prefix_len + 3] != 0 {
                    ctx_idx += 5;
                } else {
                    ctx_idx += 6;
                }
            } else if bin_idx == 5 {
                // implement section 9.3.3.1.2
                if binarized[prefix_len + 3] != 0 {
                    ctx_idx += 6;
                } else {
                    ctx_idx += 7;
                }
            } else {
                ctx_idx += 7;
            }
        } else if ctx_idx == 14 {
            // only here for P or SP slice prefix
            if bin_idx == 0 {
                ctx_idx += 0;
            } else if bin_idx == 1 {
                ctx_idx += 1;
            } else {
                // implement section 9.3.3.1.2
                let b1 = binarized[prefix_len + 1]; // prefix_len is essentially the 0 index of the suffix
                if b1 != 1 {
                    ctx_idx += 2;
                } else {
                    ctx_idx += 3;
                }
            }
        } else if ctx_idx == 17 {
            // only here for P slice suffix
            if bin_idx == 0 {
                ctx_idx += 0;
            } else if bin_idx == 1 {
                ctx_idx = 276;
            } else if bin_idx == 2 {
                ctx_idx += 1;
            } else if bin_idx == 3 {
                ctx_idx += 2;
            } else if bin_idx == 4 {
                // implement section 9.3.3.1.2
                let b3 = binarized[prefix_len + 3];
                if b3 != 0 {
                    ctx_idx += 2;
                } else {
                    ctx_idx += 3;
                }
            } else if bin_idx >= 5 {
                ctx_idx += 3;
            }
        } else if ctx_idx == 27 {
            // only here for B slice prefix
            if bin_idx == 0 {
                // decode according to 9.3.3.1.3
                // A is the block to the left
                // B is the block right above
                let mut cond_term_flag_a: usize = 1;
                let mut cond_term_flag_b: usize = 1;

                let mb_a: MacroBlock = neighbor_info.0.clone();
                let mb_b: MacroBlock = neighbor_info.1.clone();

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

                ctx_idx += cond_term_flag_a + cond_term_flag_b;
            } else if bin_idx == 1 {
                ctx_idx += 3;
            } else if bin_idx == 2 {
                // implement section 9.3.3.1.2
                let b1 = binarized[1];
                if b1 != 0 {
                    ctx_idx += 4;
                } else {
                    ctx_idx += 5;
                }
            } else if bin_idx >= 3 {
                ctx_idx += 5;
            }
        } else if ctx_idx == 32 {
            // only here for B slice suffix
            if bin_idx == 0 {
                ctx_idx += 0;
            } else if bin_idx == 1 {
                ctx_idx = 276;
            } else if bin_idx == 2 {
                ctx_idx += 1;
            } else if bin_idx == 3 {
                ctx_idx += 2;
            } else if bin_idx == 4 {
                // implement section 9.3.3.1.2
                let b3 = binarized[prefix_len + 3];
                if b3 != 0 {
                    ctx_idx += 2;
                } else {
                    ctx_idx += 3;
                }
            } else if bin_idx >= 5 {
                ctx_idx += 3;
            }
        }

        let idx2: usize = clip3(0, 51, sh.slice_qp_y as i32) as usize;
        if CABAC_DEBUG {
            debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
            debug!(target: "encode","state: {:x}", cs.states[sh.cabac_init_idc as usize][idx2][ctx_idx].p_state_idx);
        }

        // arithmetic encoding
        if ctx_idx == 276 {
            encoded.append(&mut encode_terminate(b, cs));
        } else {
            encoded.append(&mut arithmetic_encode(
                b,
                cs,
                sh.cabac_init_idc as usize,
                idx2,
                ctx_idx,
            ));
        }
        if CABAC_DEBUG {
            debug!(target: "encode",
                "updated state: {:x}",
                cs.states[sh.cabac_init_idc as usize][idx2][ctx_idx].p_state_idx
            );
        }

        bin_idx += 1;
    }

    stream.append(&mut encoded);
}

/// Macroblock Data - sub_mb_type
pub fn cabac_encode_sub_mb_type(
    se_val: SubMbType,
    sh: &SliceHeader,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
) {
    // binarization specified in clause 9.3.2.5
    let binarized = generate_sub_mb_type_value(se_val);
    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_sub_mb_type - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("sub_mb_type", se_val, 63);
    }
    for (bin_idx, b) in binarized.iter().enumerate() {
        // reset it each time

        let mut ctx_idx = 0;
        if is_slice_type(sh.slice_type, "P") || is_slice_type(sh.slice_type, "SP") {
            ctx_idx = 21;
            if bin_idx == 0 {
                ctx_idx += 0;
            } else if bin_idx == 1 {
                ctx_idx += 1;
            } else if bin_idx == 2 {
                ctx_idx += 2;
            }
        } else if is_slice_type(sh.slice_type, "B") {
            ctx_idx = 36;

            if bin_idx == 0 {
                ctx_idx += 0;
            } else if bin_idx == 1 {
                ctx_idx += 1;
            } else if bin_idx == 2 {
                // implement section 9.3.3.1.2
                let b1 = binarized[1];
                if b1 != 0 {
                    ctx_idx += 2;
                } else {
                    ctx_idx += 3;
                }
            } else if bin_idx == 3 || bin_idx == 4 || bin_idx == 5 {
                ctx_idx += 3;
            }
        }

        let idx1 = sh.cabac_init_idc as usize;
        let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;
        if CABAC_DEBUG {
            debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
            debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
        }
        // binary encode with model
        stream.append(&mut arithmetic_encode(b, cs, idx1, idx2, ctx_idx));
        if CABAC_DEBUG {
            debug!(target: "encode",
                "updated state: {:x}",
                cs.states[idx1][idx2][ctx_idx].p_state_idx
            );
        }
    }
}

/// Macroblock Data - transform_size_8x8_flag
pub fn cabac_encode_transform_size_8x8_flag(
    se_val: bool,
    sh: &SliceHeader,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
    neighbor_info: (MacroBlock, MacroBlock),
) {
    // binarization is fixed length of 1
    let binarized = match se_val {
        true => 1u8,
        false => 0u8,
    };
    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_transform_size_8x8_flag - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("transform_size_8x8_flag", se_val, 63);
    }
    // cabac max_bin_idx_ctx = 0; ctx_idx_offset = 399

    let mut ctx_idx = 399;

    // more specific model selection
    let mut cond_term_flag_a: usize = 1;
    let mut cond_term_flag_b: usize = 1;

    let mb_a: MacroBlock = neighbor_info.0.clone();
    let mb_b: MacroBlock = neighbor_info.1.clone();

    // set cond_term_flag_a
    if !mb_a.available || !mb_a.transform_size_8x8_flag {
        cond_term_flag_a = 0;
    }

    if !mb_b.available || !mb_b.transform_size_8x8_flag {
        cond_term_flag_b = 0;
    }

    ctx_idx += cond_term_flag_a + cond_term_flag_b;

    // binary encode with model

    let mut idx1 = sh.cabac_init_idc as usize;
    let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;

    if !is_slice_type(sh.slice_type, "I") && !is_slice_type(sh.slice_type, "SI") && ctx_idx >= 54 {
        idx1 += 1;
    }

    if CABAC_DEBUG {
        debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
        debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
    }
    stream.append(&mut arithmetic_encode(&binarized, cs, idx1, idx2, ctx_idx));

    if CABAC_DEBUG {
        debug!(target: "encode",
            "updated state: {:x}",
            cs.states[idx1][idx2][ctx_idx].p_state_idx
        );
    }
}

/// Macroblock Data - coded_block_pattern
pub fn cabac_encode_coded_block_pattern(
    se_val: u32,
    sh: &SliceHeader,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
    sd: &SliceData,
    vp: &VideoParameters,
    curr_mb_idx: usize,
) {
    // binarization specified in clause 9.3.2.6

    let binarized = generate_coded_block_pattern_value(se_val);
    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_coded_block_pattern - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("coded_block_pattern", se_val, 63);
    }
    // cabac
    //    prefix: max_bin_idx_ctx = 3; ctx_idx_offset = 73
    //    suffix: max_bin_idx_ctx = 1; ctx_idx_offset = 77

    // first encode the prefix
    let max_bin_idx_ctx = 3;

    for bin_idx in 0..max_bin_idx_ctx + 1 {
        // reset the ctx_idx
        let mut ctx_idx = 73;

        let mut cond_term_flag_a: usize = 1;
        let mut cond_term_flag_b: usize = 1;

        // use 6.4.11.2 to get neighbor luma info
        let res = sd.get_neighbor_8x8_luma_block(curr_mb_idx, true, bin_idx as usize, vp);

        // set cond_term_flag_a
        let mb_a: MacroBlock = res.0;
        let mb_b: MacroBlock = res.1;
        let luma_8x8_blk_idx_a: usize = res.2;
        let luma_8x8_blk_idx_b: usize = res.3;

        let mut prev_decoded_bin_eq_0 = true;

        if luma_8x8_blk_idx_a < binarized.len() {
            prev_decoded_bin_eq_0 = binarized[luma_8x8_blk_idx_a] == 0;
        }

        // see section 9.3.3.1.1.4 for full list of conditions
        if !mb_a.available
            || mb_a.mb_type == MbType::IPCM
            || (mb_a.mb_addr != sd.macroblock_vec[curr_mb_idx].mb_addr
                && mb_a.mb_type != MbType::PSkip
                && mb_a.mb_type != MbType::BSkip
                && (mb_a.coded_block_pattern_luma >> luma_8x8_blk_idx_a) & 1 != 0)
            || (mb_a.mb_addr == sd.macroblock_vec[curr_mb_idx].mb_addr && !prev_decoded_bin_eq_0)
        {
            cond_term_flag_a = 0;
        }

        // reset the value to false just for safesies
        prev_decoded_bin_eq_0 = true;

        if luma_8x8_blk_idx_b < binarized.len() {
            prev_decoded_bin_eq_0 = binarized[luma_8x8_blk_idx_b] == 0;
        }

        if !mb_b.available
            || mb_b.mb_type == MbType::IPCM
            || (mb_b.mb_addr != sd.macroblock_vec[curr_mb_idx].mb_addr
                && mb_b.mb_type != MbType::PSkip
                && mb_b.mb_type != MbType::BSkip
                && (mb_b.coded_block_pattern_luma >> luma_8x8_blk_idx_b) & 1 != 0)
            || (mb_b.mb_addr == sd.macroblock_vec[curr_mb_idx].mb_addr && !prev_decoded_bin_eq_0)
        {
            cond_term_flag_b = 0;
        }

        ctx_idx += cond_term_flag_a + 2 * cond_term_flag_b;

        let mut idx1 = sh.cabac_init_idc as usize;
        let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;
        if CABAC_DEBUG {
            debug!(target: "encode","curr_cpb_ctx : {}", cond_term_flag_a + 2 * cond_term_flag_b );
        }
        if !is_slice_type(sh.slice_type, "I")
            && !is_slice_type(sh.slice_type, "SI")
            && ctx_idx >= 54
        {
            idx1 += 1;
        }
        if CABAC_DEBUG {
            debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
            debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
        }
        // binary encode
        stream.append(&mut arithmetic_encode(
            &binarized[bin_idx],
            cs,
            idx1,
            idx2,
            ctx_idx,
        ));
        if CABAC_DEBUG {
            debug!(target: "encode",
                "updated state: {:x}",
                cs.states[idx1][idx2][ctx_idx].p_state_idx
            );
        }
    }

    // now encode the suffix
    // the values are either 0(0), 1 (10), or 2(11) due to truncated unary value of length 2
    if vp.chroma_array_type != 0 && vp.chroma_array_type != 3 {
        for bin_idx in 0..(binarized.len() - 4) {
            // reset the ctx_idx
            let mut ctx_idx = 77;

            // use the 6.4.11.1 clause to determine neighbor information
            let mut cond_term_flag_a: usize = 1;
            let mut cond_term_flag_b: usize = 1;

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
                cond_term_flag_b = 0;
            }

            ctx_idx += cond_term_flag_a
                + 2 * cond_term_flag_b
                + match bin_idx == 1 {
                    true => 4,
                    _ => 0,
                };

            let mut idx1 = sh.cabac_init_idc as usize;
            let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;

            if !is_slice_type(sh.slice_type, "I")
                && !is_slice_type(sh.slice_type, "SI")
                && ctx_idx >= 54
            {
                idx1 += 1;
            }
            if CABAC_DEBUG {
                debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
                debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
            }
            // binary encode
            stream.append(&mut arithmetic_encode(
                &binarized[bin_idx + 4],
                cs,
                idx1,
                idx2,
                ctx_idx,
            ));
            if CABAC_DEBUG {
                debug!(target: "encode",
                    "updated state: {:x}",
                    cs.states[idx1][idx2][ctx_idx].p_state_idx
                );
            }
        }
    } else if CABAC_DEBUG {
        debug!(target: "encode","no chroma components to encode");
    }
}

/// Macroblock Data - mb_qp_delta
pub fn cabac_encode_mb_qp_delta(
    se_val: i32,
    sh: &SliceHeader,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
    prev_mb: MacroBlock,
) {
    // binarization specified in clause 9.3.2.7
    let binarized = generate_mb_qp_delta_value(se_val);
    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_mb_qp_delta - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("mb_qp_delta", se_val, 63);
    }
    // cabac max_bin_idx_ctx = 2; ctx_idx_offset = 60

    for (bin_idx, b) in binarized.iter().enumerate() {
        let mut ctx_idx = 60;
        if bin_idx == 0 {
            // implement section 9.3.3.1.1.5

            if !prev_mb.available
                || prev_mb.mb_type == MbType::PSkip
                || prev_mb.mb_type == MbType::BSkip
                || prev_mb.mb_type == MbType::IPCM
                || (prev_mb.mb_part_pred_mode(0) != MbPartPredMode::Intra16x16
                    && prev_mb.coded_block_pattern_chroma == 0
                    && prev_mb.coded_block_pattern_luma == 0)
                || prev_mb.mb_qp_delta == 0
            {
                ctx_idx += 0;
            } else {
                ctx_idx += 1;
            }
        } else if bin_idx == 1 {
            ctx_idx += 2;
        } else if bin_idx >= 2 {
            // Spec shows up to 6
            ctx_idx += 3;
        }

        let mut idx1 = sh.cabac_init_idc as usize;
        let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;

        if !is_slice_type(sh.slice_type, "I")
            && !is_slice_type(sh.slice_type, "SI")
            && ctx_idx >= 54
        {
            idx1 += 1;
        }
        if CABAC_DEBUG {
            debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
            debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
        }
        // binary encode with model
        stream.append(&mut arithmetic_encode(b, cs, idx1, idx2, ctx_idx));
        if CABAC_DEBUG {
            debug!(target: "encode",
                "updated state: {:x}",
                cs.states[idx1][idx2][ctx_idx].p_state_idx
            );
        }
    }
}

/// Macroblock Prediction - intra4x4_pred_mode_flag
pub fn cabac_encode_prev_intra4x4_pred_mode_flag(
    se_val: bool,
    sh: &SliceHeader,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
) {
    // binarization is fixed length of 1
    let binarized = match se_val {
        true => 1u8,
        false => 0u8,
    };

    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_prev_intra4x4_pred_mode_flag - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("prev_intra4x4_pred_mode_flag", se_val, 63);
    }

    // cabac max_bin_idx_ctx = 0; ctx_idx_offset = 68
    let ctx_idx = 68;

    let mut idx1 = sh.cabac_init_idc as usize;
    let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;

    if !is_slice_type(sh.slice_type, "I") && !is_slice_type(sh.slice_type, "SI") && ctx_idx >= 54 {
        idx1 += 1;
    }
    if CABAC_DEBUG {
        debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
        debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
    }
    // binary encode with model
    stream.append(&mut arithmetic_encode(&binarized, cs, idx1, idx2, ctx_idx));

    if CABAC_DEBUG {
        debug!(target: "encode",
            "updated state: {:x}",
            cs.states[idx1][idx2][ctx_idx].p_state_idx
        );
    }
}

/// Macroblock Prediction - intra4x4_pred_mode
pub fn cabac_encode_rem_intra4x4_pred_mode(
    se_val: u32,
    sh: &SliceHeader,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
) {
    // binarization is fixed length of 3
    let mut binarized = generate_fixed_length_value(se_val, 3);

    // reverse it because the bitstream is lsb to msb
    binarized.reverse();
    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_rem_intra4x4_pred_mode - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("rem_intra4x4_pred_mode", se_val, 63);
    }
    // cabac max_bin_idx_ctx = 0; ctx_idx_offset = 69
    let ctx_idx = 69;

    // binary encode with model
    for b in binarized.iter() {
        let mut idx1 = sh.cabac_init_idc as usize;
        let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;

        if !is_slice_type(sh.slice_type, "I")
            && !is_slice_type(sh.slice_type, "SI")
            && ctx_idx >= 54
        {
            idx1 += 1;
        }
        if CABAC_DEBUG {
            debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
            debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
        }
        stream.append(&mut arithmetic_encode(b, cs, idx1, idx2, ctx_idx));
        if CABAC_DEBUG {
            debug!(target: "encode",
                "updated state: {:x}",
                cs.states[idx1][idx2][ctx_idx].p_state_idx
            );
        }
    }
}

/// Macroblock Prediction - intra8x8_pred_mode_flag
pub fn cabac_encode_prev_intra8x8_pred_mode_flag(
    se_val: bool,
    sh: &SliceHeader,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
) {
    // binarization is fixed length of 1
    let binarized = match se_val {
        true => 1u8,
        false => 0u8,
    };

    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_prev_intra8x8_pred_mode_flag - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("prev_intra8x8_pred_mode_flag", se_val, 63);
    }
    // cabac max_bin_idx_ctx = 0; ctx_idx_offset = 68
    let ctx_idx = 68;

    let mut idx1 = sh.cabac_init_idc as usize;
    let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;

    if !is_slice_type(sh.slice_type, "I") && !is_slice_type(sh.slice_type, "SI") && ctx_idx >= 54 {
        idx1 += 1;
    }
    if CABAC_DEBUG {
        debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
        debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
    }
    // binary encode with model
    stream.append(&mut arithmetic_encode(&binarized, cs, idx1, idx2, ctx_idx));
    if CABAC_DEBUG {
        debug!(target: "encode",
            "updated state: {:x}",
            cs.states[idx1][idx2][ctx_idx].p_state_idx
        );
    }
}

/// Macroblock Prediction - intra8x8_pred_mode
pub fn cabac_encode_rem_intra8x8_pred_mode(
    se_val: u32,
    sh: &SliceHeader,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
) {
    // binarization is fixed length of 3
    let mut binarized = generate_fixed_length_value(se_val, 3);

    // reverse it because the bitstream is lsb to msb
    binarized.reverse();
    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_rem_intra8x8_pred_mode - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("rem_intra8x8_pred_mode", se_val, 63);
    }
    // cabac max_bin_idx_ctx = 0; ctx_idx_offset = 69
    let ctx_idx = 69;

    // binary encode with model
    for b in binarized.iter() {
        let mut idx1 = sh.cabac_init_idc as usize;
        let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;

        if !is_slice_type(sh.slice_type, "I")
            && !is_slice_type(sh.slice_type, "SI")
            && ctx_idx >= 54
        {
            idx1 += 1;
        }
        if CABAC_DEBUG {
            debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
            debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
        }
        stream.append(&mut arithmetic_encode(b, cs, idx1, idx2, ctx_idx));
        if CABAC_DEBUG {
            debug!(target: "encode",
                "updated state: {:x}",
                cs.states[idx1][idx2][ctx_idx].p_state_idx
            );
        }
    }
}

/// Macroblock Prediction - intra_chroma_pred_mode
pub fn cabac_encode_intra_chroma_pred_mode(
    se_val: u32,
    sh: &SliceHeader,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
    neighbor_info: (MacroBlock, MacroBlock),
) {
    // binarization is truncated unary of max 3
    let binarized = generate_truncated_unary_value(se_val, 3);
    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_intra_chroma_pred_mode - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("intra_chroma_pred_mode", se_val, 63);
    }

    // cabac max_bin_idx_ctx = 1; ctx_idx_offset = 64
    for (bin_idx, b) in binarized.iter().enumerate() {
        // reset it each time

        let mut ctx_idx = 64;
        if bin_idx == 0 {
            // use the 6.4.11.1 clause to determine neighbor information
            let mut cond_term_flag_a: usize = 1;
            let mut cond_term_flag_b: usize = 1;

            let mut mb_a: MacroBlock = neighbor_info.0.clone();
            let mut mb_b: MacroBlock = neighbor_info.1.clone();

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

            ctx_idx += cond_term_flag_a + cond_term_flag_b;
        } else if bin_idx == 1 || bin_idx == 2 {
            ctx_idx += 3;
        }

        let mut idx1 = sh.cabac_init_idc as usize;
        let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;

        if !is_slice_type(sh.slice_type, "I")
            && !is_slice_type(sh.slice_type, "SI")
            && ctx_idx >= 54
        {
            idx1 += 1;
        }
        if CABAC_DEBUG {
            debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
            debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
        }
        // binary encode with model
        stream.append(&mut arithmetic_encode(b, cs, idx1, idx2, ctx_idx));
        if CABAC_DEBUG {
            debug!(target: "encode",
                "updated state: {:x}",
                cs.states[idx1][idx2][ctx_idx].p_state_idx
            );
        }
    }
}

/// Macroblock Prediction - ref_idx
pub fn cabac_encode_ref_idx(
    se_val: u32,
    list_idx: usize,
    mb_part_idx: usize,
    sh: &SliceHeader,
    sd: &SliceData,
    curr_mb_idx: usize,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
    neighbor_info: &(
        MacroBlock,
        MacroBlock,
        MacroBlock,
        MacroBlock,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
    ),
) {
    // binarization is unary value
    let binarized = generate_unary_value(se_val);

    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_ref_idx_l{}[{}] - Se_val is {:?}, and the binarized value is {:?}", list_idx, mb_part_idx, se_val, binarized);
    } else {
        encoder_formatted_print("ref_idx_l[][]", se_val, 63);
    }

    // cabac max_bin_idx_ctx = 2; ctx_idx_offset = 54

    let ctx_idx_offset = 54;

    // binary encode with model
    for (bin_idx, b) in binarized.iter().enumerate() {
        let mut ctx_idx = ctx_idx_offset;

        if bin_idx == 0 {
            // implement section 9.3.3.1.1.6
            let mut ref_idx_zero_flag_a = false;
            let mut ref_idx_zero_flag_b = false;

            let mut pred_mode_equal_flag_a = true;
            let mut pred_mode_equal_flag_b = true;

            let mut cond_term_flag_a = 1;
            let mut cond_term_flag_b = 1;

            // use 6.4.11.7 for neighbor info
            let mb_a: MacroBlock = neighbor_info.0.clone();
            let mb_b: MacroBlock = neighbor_info.1.clone();
            let mb_part_idx_a = neighbor_info.4;
            let mb_part_idx_b = neighbor_info.5;

            if mb_a.available {
                if list_idx == 0 {
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
                if list_idx == 0 {
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
                if list_idx == 0 {
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
            } else if list_idx == 0 {
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
                if list_idx == 0 {
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
            } else if list_idx == 0 {
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

            ctx_idx += cond_term_flag_a + 2 * cond_term_flag_b;
        } else if bin_idx == 1 {
            ctx_idx += 4;
        } else if bin_idx >= 2 {
            ctx_idx += 5;
        }

        let mut idx1 = sh.cabac_init_idc as usize;
        let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;

        if !is_slice_type(sh.slice_type, "I")
            && !is_slice_type(sh.slice_type, "SI")
            && ctx_idx >= 54
        {
            idx1 += 1;
        }
        if CABAC_DEBUG {
            debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
            debug!(target: "encode","state-3: {:x}", cs.states[idx1][idx2][ctx_idx-3].p_state_idx);
            debug!(target: "encode","state-2: {:x}", cs.states[idx1][idx2][ctx_idx-2].p_state_idx);
            debug!(target: "encode","state-1: {:x}", cs.states[idx1][idx2][ctx_idx-1].p_state_idx);
            debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
            debug!(target: "encode","state+1: {:x}", cs.states[idx1][idx2][ctx_idx+1].p_state_idx);
            debug!(target: "encode","state+2: {:x}", cs.states[idx1][idx2][ctx_idx+2].p_state_idx);
            debug!(target: "encode","state+3: {:x}", cs.states[idx1][idx2][ctx_idx+3].p_state_idx);
        }
        stream.append(&mut arithmetic_encode(b, cs, idx1, idx2, ctx_idx));
        if CABAC_DEBUG {
            debug!(target: "encode",
                "updated state: {:x}",
                cs.states[idx1][idx2][ctx_idx].p_state_idx
            );
        }
    }
}

/// Macroblock Prediction - mvd
pub fn cabac_encode_mvd(
    se_val: i32,
    list_idx: usize,
    mb_part_idx: usize,
    sub_mb_part_idx: usize,
    comp_idx: usize,
    sh: &SliceHeader,
    sd: &SliceData,
    curr_mb_idx: usize,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
    neighbor_info: &(
        MacroBlock,
        MacroBlock,
        MacroBlock,
        MacroBlock,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
        usize,
    ),
) {
    // binarization is uegk value
    let (binarized, suffix_exist) = generate_uegk(se_val, 9, 3, true);

    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_mvd_l{}_{}_{}_{} - Se_val is {:?} and the binarized value is {:?}", list_idx, comp_idx, sub_mb_part_idx, mb_part_idx, se_val, binarized);
    } else {
        encoder_formatted_print("mvd_l[][]", se_val, 63);
    }
    // encode prefix
    let ctx_idx_offset: usize = if comp_idx == 0 { 40 } else { 47 };

    for (bin_idx, b) in binarized.iter().enumerate() {
        if bin_idx > 8 {
            break; // this means the suffix exists
        }
        let mut ctx_idx = ctx_idx_offset;

        if comp_idx == 0 {
            if bin_idx == 0 {
                // implement section 9.3.3.1.1.7

                let pred_mode_equal_flag_a: usize;
                let pred_mode_equal_flag_b: usize;
                let abs_mvd_comp_b: u32;
                let abs_mvd_comp_a: u32;

                // use 6.4.11.7 for neighbor info
                let mut mb_a: MacroBlock = neighbor_info.0.clone();
                let mut mb_b: MacroBlock = neighbor_info.1.clone();
                let mb_part_idx_a = neighbor_info.4;
                let mb_part_idx_b = neighbor_info.5;
                let sub_mb_part_idx_a = neighbor_info.8;
                let sub_mb_part_idx_b = neighbor_info.9;

                // set pred_mode_equal_flag_a
                if mb_a.mb_type == MbType::BDirect16x16 || mb_a.mb_type == MbType::BSkip {
                    pred_mode_equal_flag_a = 0;
                } else if mb_a.mb_type == MbType::P8x8 || mb_a.mb_type == MbType::B8x8 {
                    let cur_sub_pred_mode = mb_a.sub_mb_part_pred_mode(mb_part_idx_a);

                    if (list_idx == 1
                        && cur_sub_pred_mode != MbPartPredMode::PredL1
                        && cur_sub_pred_mode != MbPartPredMode::BiPred)
                        || (list_idx == 0
                            && cur_sub_pred_mode != MbPartPredMode::PredL0
                            && cur_sub_pred_mode != MbPartPredMode::BiPred)
                    {
                        pred_mode_equal_flag_a = 0;
                    } else {
                        pred_mode_equal_flag_a = 1;
                    }
                } else {
                    let cur_pred_mode = mb_a.mb_part_pred_mode(mb_part_idx_a);
                    if (list_idx == 1
                        && cur_pred_mode != MbPartPredMode::PredL1
                        && cur_pred_mode != MbPartPredMode::BiPred)
                        || (list_idx == 0
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

                    if (list_idx == 1
                        && cur_sub_pred_mode != MbPartPredMode::PredL1
                        && cur_sub_pred_mode != MbPartPredMode::BiPred)
                        || (list_idx == 0
                            && cur_sub_pred_mode != MbPartPredMode::PredL0
                            && cur_sub_pred_mode != MbPartPredMode::BiPred)
                    {
                        pred_mode_equal_flag_b = 0;
                    } else {
                        pred_mode_equal_flag_b = 1;
                    }
                } else {
                    let cur_pred_mode = mb_b.mb_part_pred_mode(mb_part_idx_b);
                    if (list_idx == 1
                        && cur_pred_mode != MbPartPredMode::PredL1
                        && cur_pred_mode != MbPartPredMode::BiPred)
                        || (list_idx == 0
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
                    || mb_a.is_intra()
                    || pred_mode_equal_flag_a == 0
                {
                    abs_mvd_comp_a = 0;
                } else {
                    if comp_idx == 1
                        && sh.mbaff_frame_flag
                        && !sd.mb_field_decoding_flag[curr_mb_idx]
                        && sd.mb_field_decoding_flag[mb_a.mb_idx]
                    {
                        if list_idx == 1 {
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
                        if list_idx == 1 {
                            abs_mvd_comp_a = mb_a.mvd_l1[mb_part_idx_a][sub_mb_part_idx_a][comp_idx]
                                .abs() as u32
                                / 2;
                        } else {
                            abs_mvd_comp_a = mb_a.mvd_l0[mb_part_idx_a][sub_mb_part_idx_a][comp_idx]
                                .abs() as u32
                                / 2;
                        }
                    } else {
                        if list_idx == 1 {
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
                    || mb_b.is_intra()
                    || pred_mode_equal_flag_b == 0
                {
                    abs_mvd_comp_b = 0;
                } else {
                    if comp_idx == 1
                        && sh.mbaff_frame_flag
                        && !sd.mb_field_decoding_flag[curr_mb_idx]
                        && sd.mb_field_decoding_flag[mb_b.mb_idx]
                    {
                        if list_idx == 1 {
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
                        if list_idx == 1 {
                            abs_mvd_comp_b = mb_b.mvd_l1[mb_part_idx_b][sub_mb_part_idx_b][comp_idx]
                                .abs() as u32
                                / 2;
                        } else {
                            abs_mvd_comp_b = mb_b.mvd_l0[mb_part_idx_b][sub_mb_part_idx_b][comp_idx]
                                .abs() as u32
                                / 2;
                        }
                    } else {
                        if list_idx == 1 {
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
        } else if bin_idx == 0 {
            // implement section 9.3.3.1.1.7
            let comp_idx: usize = 1;

            let pred_mode_equal_flag_a: usize;
            let pred_mode_equal_flag_b: usize;
            let abs_mvd_comp_a: u32;
            let abs_mvd_comp_b: u32;

            let mut mb_a: MacroBlock = neighbor_info.0.clone();
            let mut mb_b: MacroBlock = neighbor_info.1.clone();
            let mb_part_idx_a = neighbor_info.4;
            let mb_part_idx_b = neighbor_info.5;
            let sub_mb_part_idx_a = neighbor_info.8;
            let sub_mb_part_idx_b = neighbor_info.9;

            // set pred_mode_equal_flag_a
            if mb_a.mb_type == MbType::BDirect16x16 || mb_a.mb_type == MbType::BSkip {
                pred_mode_equal_flag_a = 0;
            } else if mb_a.mb_type == MbType::P8x8 || mb_a.mb_type == MbType::B8x8 {
                let cur_sub_pred_mode = mb_a.sub_mb_part_pred_mode(mb_part_idx_a);

                if (list_idx == 1
                    && cur_sub_pred_mode != MbPartPredMode::PredL1
                    && cur_sub_pred_mode != MbPartPredMode::BiPred)
                    || (list_idx == 0
                        && cur_sub_pred_mode != MbPartPredMode::PredL0
                        && cur_sub_pred_mode != MbPartPredMode::BiPred)
                {
                    pred_mode_equal_flag_a = 0;
                } else {
                    pred_mode_equal_flag_a = 1;
                }
            } else {
                let cur_pred_mode = mb_a.mb_part_pred_mode(mb_part_idx_a);
                if (list_idx == 1
                    && cur_pred_mode != MbPartPredMode::PredL1
                    && cur_pred_mode != MbPartPredMode::BiPred)
                    || (list_idx == 0
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

                if (list_idx == 1
                    && cur_sub_pred_mode != MbPartPredMode::PredL1
                    && cur_sub_pred_mode != MbPartPredMode::BiPred)
                    || (list_idx == 0
                        && cur_sub_pred_mode != MbPartPredMode::PredL0
                        && cur_sub_pred_mode != MbPartPredMode::BiPred)
                {
                    pred_mode_equal_flag_b = 0;
                } else {
                    pred_mode_equal_flag_b = 1;
                }
            } else {
                let cur_pred_mode = mb_b.mb_part_pred_mode(mb_part_idx_b);

                if (list_idx == 1
                    && cur_pred_mode != MbPartPredMode::PredL1
                    && cur_pred_mode != MbPartPredMode::BiPred)
                    || (list_idx == 0
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
                || mb_a.is_intra()
                || pred_mode_equal_flag_a == 0
            {
                abs_mvd_comp_a = 0;
            } else {
                if comp_idx == 1
                    && sh.mbaff_frame_flag
                    && !sd.mb_field_decoding_flag[curr_mb_idx]
                    && sd.mb_field_decoding_flag[mb_a.mb_idx]
                {
                    if list_idx == 1 {
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
                    if list_idx == 1 {
                        abs_mvd_comp_a = mb_a.mvd_l1[mb_part_idx_a][sub_mb_part_idx_a][comp_idx]
                            .abs() as u32
                            / 2;
                    } else {
                        abs_mvd_comp_a = mb_a.mvd_l0[mb_part_idx_a][sub_mb_part_idx_a][comp_idx]
                            .abs() as u32
                            / 2;
                    }
                } else {
                    if list_idx == 1 {
                        abs_mvd_comp_a =
                            mb_a.mvd_l1[mb_part_idx_a][sub_mb_part_idx_a][comp_idx].abs() as u32;
                    } else {
                        abs_mvd_comp_a =
                            mb_a.mvd_l0[mb_part_idx_a][sub_mb_part_idx_a][comp_idx].abs() as u32;
                    }
                }
            }

            if !mb_b.available
                || mb_b.mb_type == MbType::PSkip
                || mb_b.mb_type == MbType::BSkip
                || mb_b.is_intra()
                || pred_mode_equal_flag_b == 0
            {
                abs_mvd_comp_b = 0;
            } else {
                if comp_idx == 1
                    && sh.mbaff_frame_flag
                    && !sd.mb_field_decoding_flag[curr_mb_idx]
                    && sd.mb_field_decoding_flag[mb_b.mb_idx]
                {
                    if list_idx == 1 {
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
                    if list_idx == 1 {
                        abs_mvd_comp_b = mb_b.mvd_l1[mb_part_idx_b][sub_mb_part_idx_b][comp_idx]
                            .abs() as u32
                            / 2;
                    } else {
                        abs_mvd_comp_b = mb_b.mvd_l0[mb_part_idx_b][sub_mb_part_idx_b][comp_idx]
                            .abs() as u32
                            / 2;
                    }
                } else {
                    if list_idx == 1 {
                        abs_mvd_comp_b =
                            mb_b.mvd_l1[mb_part_idx_b][sub_mb_part_idx_b][comp_idx].abs() as u32;
                    } else {
                        abs_mvd_comp_b =
                            mb_b.mvd_l0[mb_part_idx_b][sub_mb_part_idx_b][comp_idx].abs() as u32;
                    }
                }
            }

            if abs_mvd_comp_a + abs_mvd_comp_b > 32 {
                ctx_idx += 2;
            } else if abs_mvd_comp_a + abs_mvd_comp_b > 2 {
                ctx_idx += 1;
            }
        }

        if bin_idx == 1 {
            ctx_idx += 3;
        } else if bin_idx == 2 {
            ctx_idx += 4;
        } else if bin_idx == 3 {
            ctx_idx += 5;
        } else if bin_idx == 4 || bin_idx == 5 || bin_idx >= 6 {
            ctx_idx += 6;
        }

        let mut idx1 = sh.cabac_init_idc as usize;
        let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;

        if !is_slice_type(sh.slice_type, "I")
            && !is_slice_type(sh.slice_type, "SI")
            && ctx_idx >= 54
        {
            idx1 += 1;
        }
        if CABAC_DEBUG {
            debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
            debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
        }
        stream.append(&mut arithmetic_encode(b, cs, idx1, idx2, ctx_idx));
        if CABAC_DEBUG {
            debug!(target: "encode",
                "updated state: {:x}",
                cs.states[idx1][idx2][ctx_idx].p_state_idx
            );
        }
    }

    // encode suffix
    if suffix_exist {
        for b in binarized[9..].iter().copied() {
            stream.append(&mut encode_bypass(b, cs));
        }
    }

    // encode sign bit
    if se_val != 0 {
        if se_val < 0 {
            stream.append(&mut encode_bypass(1, cs));
        } else {
            stream.append(&mut encode_bypass(0, cs));
        }
    }
}

/// Macroblock Residual Data - coded_block_flag
pub fn cabac_encode_coded_block_flag(
    se_val: bool,
    ctx_block_cat: u8,
    sh: &SliceHeader,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
    sd: &SliceData,
    vp: &VideoParameters,
    curr_mb_idx: usize,
    additional_inputs: &[usize],
) {
    // binarization is fixed length of 1
    let binarized = match se_val {
        true => 1u8,
        false => 0u8,
    };

    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_coded_block_flag - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("coded_block_flag", se_val, 63);
    }
    // cabac
    // match ctx_block_cat {
    //         < 5: max_bin_idx_ctx = 0; ctx_idx_offset = 85
    //        == 5: max_bin_idx_ctx = 0; ctx_idx_offset = 1012
    //   5 < x < 9: max_bin_idx_ctx = 0; ctx_idx_offset = 460
    //        == 9: max_bin_idx_ctx = 0; ctx_idx_offset = 1012
    //  9 < x < 13: max_bin_idx_ctx = 0; ctx_idx_offset = 472
    //       == 13: max_bin_idx_ctx = 0; ctx_idx_offset = 1012
    //}

    let mut ctx_idx = 85;

    if ctx_block_cat < 5 {
        ctx_idx = 85;
    } else if 5 < ctx_block_cat && ctx_block_cat < 9 {
        ctx_idx = 460;
    } else if 9 < ctx_block_cat && ctx_block_cat < 13 {
        ctx_idx = 472;
    } else if ctx_block_cat == 5 || ctx_block_cat == 9 || ctx_block_cat == 13 {
        ctx_idx = 1012;
    }

    let ctx_block_cat_offset =
        cabac_tables::CTX_BLOCK_CAT_OFFSET_CODED_BLOCK_FLAG[ctx_block_cat as usize];

    // specified in section 9.3.3.1.1.9
    let mut trans_block_a: TransformBlock = TransformBlock::new();
    let mut trans_block_b: TransformBlock = TransformBlock::new();
    let mut mb_a: MacroBlock = MacroBlock::new();
    let mut mb_b: MacroBlock = MacroBlock::new();
    let cond_term_flag_a: usize;
    let cond_term_flag_b: usize;

    // additional input depending on the value of ctx_block_cat
    if ctx_block_cat == 0 || ctx_block_cat == 6 || ctx_block_cat == 10 {
        // no additional input required

        // use 6.4.11.1 to get neighbor info
        let res = sd.get_neighbor(curr_mb_idx, false, vp);
        mb_a = res.0;
        mb_b = res.1;

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

        // use 6.4.11.4 to get neighbor luma blocks
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
                trans_block_a = mb_a.luma_level_4x4_transform_blocks[luma_4x4_blk_idx_a].clone();
            }
        } else if mb_a.available
            && mb_a.mb_type != MbType::PSkip
            && mb_a.mb_type != MbType::BSkip
            && (mb_a.coded_block_pattern_luma >> (luma_4x4_blk_idx_a >> 2)) & 1 != 0
            && mb_a.transform_size_8x8_flag
        {
            trans_block_a = mb_a.luma_level_8x8_transform_blocks[luma_4x4_blk_idx_a >> 2].clone();
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
                trans_block_b = mb_b.luma_level_4x4_transform_blocks[luma_4x4_blk_idx_b].clone();
            }
        } else if mb_b.available
            && mb_b.mb_type != MbType::PSkip
            && mb_b.mb_type != MbType::BSkip
            && (mb_b.coded_block_pattern_luma >> (luma_4x4_blk_idx_b >> 2)) & 1 != 0
            && mb_b.transform_size_8x8_flag
        {
            trans_block_b = mb_b.luma_level_8x8_transform_blocks[luma_4x4_blk_idx_b >> 2].clone();
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

    // use 6.4.11.5 to get neighbor chroma info
    } else if ctx_block_cat == 5 {
        let luma_8x8_blk_idx = additional_inputs[0];

        // use 6.4.11.2 to get neighbor luma info
        let res = sd.get_neighbor_8x8_luma_block(curr_mb_idx, true, luma_8x8_blk_idx, vp);

        mb_a = res.0;
        mb_b = res.1;
        let luma_8x8_blk_idx_a: usize = res.2;
        let luma_8x8_blk_idx_b: usize = res.3;

        if mb_a.available
            && mb_a.mb_type != MbType::PSkip
            && mb_a.mb_type != MbType::BSkip
            && mb_a.mb_type != MbType::IPCM
            && (mb_a.coded_block_pattern_luma >> luma_8x8_blk_idx_a) & 1 != 0 // The spec just says luma_8x8_blk_idx, which seems to be a typo
            && mb_a.transform_size_8x8_flag
        {
            trans_block_a = mb_a.luma_level_8x8_transform_blocks[luma_8x8_blk_idx_a].clone();
        }

        if mb_b.available
            && mb_b.mb_type != MbType::PSkip
            && mb_b.mb_type != MbType::BSkip
            && mb_b.mb_type != MbType::IPCM
            && (mb_b.coded_block_pattern_luma >> luma_8x8_blk_idx_b) & 1 != 0 // The spec just says luma_8x8_blk_idx, which seems to be a typo
            && mb_b.transform_size_8x8_flag
        {
            trans_block_b = mb_b.luma_level_8x8_transform_blocks[luma_8x8_blk_idx_b].clone();
        }
    } else if ctx_block_cat == 7 || ctx_block_cat == 8 {
        let cb_4x4_blk_idx = additional_inputs[0];

        // use 6.4.11.5 to get neighbor luma info
        //let res = sd.get_neighbor_4x4_chroma_block(curr_mb_idx, cb_4x4_blk_idx, vp);
        // Try other neighbor algorithm - 6.4.11.6
        let res = sd.get_neighbor_4x4_cr_cb_blocks_info(curr_mb_idx, cb_4x4_blk_idx, vp);

        mb_a = res.0;
        mb_b = res.1;
        let cb_4x4_blk_idx_a: usize = res.2;
        let cb_4x4_blk_idx_b: usize = res.3;

        // difference between two cases is IPCM and transform_size_8x8_flag
        if mb_a.available
            && mb_a.mb_type != MbType::PSkip
            && mb_a.mb_type != MbType::BSkip
            && mb_a.mb_type != MbType::IPCM
            && (mb_a.coded_block_pattern_luma >> (cb_4x4_blk_idx_a >> 2)) & 1 != 0
            && !mb_a.transform_size_8x8_flag
        {
            if ctx_block_cat == 7 {
                trans_block_a =
                    mb_a.cb_intra_16x16_ac_level_transform_blocks[cb_4x4_blk_idx_a].clone();
            } else {
                trans_block_a = mb_a.cb_level_4x4_transform_blocks[cb_4x4_blk_idx_a].clone();
            }
        } else if mb_a.available
            && mb_a.mb_type != MbType::PSkip
            && mb_a.mb_type != MbType::BSkip
            && (mb_a.coded_block_pattern_luma >> (cb_4x4_blk_idx_a >> 2)) & 1 != 0
            && mb_a.transform_size_8x8_flag
        {
            trans_block_a = mb_a.cb_level_8x8_transform_blocks[cb_4x4_blk_idx_a >> 2].clone();
        }

        if mb_b.available
            && mb_b.mb_type != MbType::PSkip
            && mb_b.mb_type != MbType::BSkip
            && mb_b.mb_type != MbType::IPCM
            && (mb_b.coded_block_pattern_luma >> (cb_4x4_blk_idx_b >> 2)) & 1 != 0
            && !mb_b.transform_size_8x8_flag
        {
            if ctx_block_cat == 7 {
                trans_block_b =
                    mb_b.cb_intra_16x16_ac_level_transform_blocks[cb_4x4_blk_idx_b].clone();
            } else {
                trans_block_b = mb_b.cb_level_4x4_transform_blocks[cb_4x4_blk_idx_b].clone();
            }
        } else if mb_b.available
            && mb_b.mb_type != MbType::PSkip
            && mb_b.mb_type != MbType::BSkip
            && (mb_b.coded_block_pattern_luma >> (cb_4x4_blk_idx_b >> 2)) & 1 != 0
            && mb_b.transform_size_8x8_flag
        {
            trans_block_b = mb_b.cb_level_8x8_transform_blocks[cb_4x4_blk_idx_b >> 2].clone();
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
                trans_block_a = mb_a.cr_level_4x4_transform_blocks[cr_4x4_blk_idx_a].clone();
            }
        } else if mb_a.available
            && mb_a.mb_type != MbType::PSkip
            && mb_a.mb_type != MbType::BSkip
            && (mb_a.coded_block_pattern_luma >> (cr_4x4_blk_idx_a >> 2)) & 1 != 0
            && mb_a.transform_size_8x8_flag
        {
            trans_block_a = mb_a.cr_level_8x8_transform_blocks[cr_4x4_blk_idx_a >> 2].clone();
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
                trans_block_b = mb_b.cr_level_4x4_transform_blocks[cr_4x4_blk_idx_b].clone();
            }
        } else if mb_b.available
            && mb_b.mb_type != MbType::PSkip
            && mb_b.mb_type != MbType::BSkip
            && (mb_b.coded_block_pattern_luma >> (cr_4x4_blk_idx_b >> 2)) & 1 != 0
            && mb_b.transform_size_8x8_flag
        {
            trans_block_b = mb_b.cr_level_8x8_transform_blocks[cr_4x4_blk_idx_b >> 2].clone();
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
    if (!mb_a.available && sd.macroblock_vec[curr_mb_idx].clone().is_inter())
        || (mb_a.available && !trans_block_a.available && mb_a.mb_type != MbType::IPCM)
        // nal_unit_type check is to check whether slice data partitioning is in use (nal_unit_type is in the range of 2 through 4, inclusive)
        || (sd.macroblock_vec[curr_mb_idx].clone().is_intra() && vp.pps_constrained_intra_pred_flag && mb_a.available && mb_a.is_inter() && vp.nal_unit_type > 1 && vp.nal_unit_type < 5)
    {
        cond_term_flag_a = 0;
    } else if (!mb_a.available && sd.macroblock_vec[curr_mb_idx].clone().is_intra())
        || mb_a.mb_type == MbType::IPCM
    {
        cond_term_flag_a = 1;
    } else {
        cond_term_flag_a = match trans_block_a.coded_block_flag {
            true => 1,
            false => 0,
        };
    }

    // send cond_term_flag_b
    if (!mb_b.available && sd.macroblock_vec[curr_mb_idx].clone().is_inter())
        || (mb_b.available && !trans_block_b.available && mb_b.mb_type != MbType::IPCM)||
        // nal_unit_type check is to check whether slice data partitioning is in use (nal_unit_type is in the range of 2 through 4, inclusive)
        (sd.macroblock_vec[curr_mb_idx].clone().is_intra() && vp.pps_constrained_intra_pred_flag && mb_b.available && mb_b.is_inter() && vp.nal_unit_type > 1 && vp.nal_unit_type < 5)
    {
        cond_term_flag_b = 0;
    } else if (!mb_b.available && sd.macroblock_vec[curr_mb_idx].clone().is_intra())
        || mb_b.mb_type == MbType::IPCM
    {
        cond_term_flag_b = 1;
    } else {
        cond_term_flag_b = match trans_block_b.coded_block_flag {
            true => 1,
            _ => 0,
        };
    }

    let ctx_idx_inc: usize = cond_term_flag_a + 2 * cond_term_flag_b;

    ctx_idx += ctx_block_cat_offset + ctx_idx_inc;

    // binary encode with model
    let mut idx1 = sh.cabac_init_idc as usize;
    let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;

    if !is_slice_type(sh.slice_type, "I") && !is_slice_type(sh.slice_type, "SI") && ctx_idx >= 54 {
        idx1 += 1;
    }
    if CABAC_DEBUG {
        debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
        debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
    }
    stream.append(&mut arithmetic_encode(&binarized, cs, idx1, idx2, ctx_idx));
    if CABAC_DEBUG {
        debug!(target: "encode",
            "updated state: {:x}",
            cs.states[idx1][idx2][ctx_idx].p_state_idx
        );
    }
}

/// Macroblock Residual Data - significant_coeff_flag
pub fn cabac_encode_significant_coeff_flag(
    se_val: bool,
    ctx_block_cat: u8,
    sh: &SliceHeader,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
    additional_inputs: &[usize],
    curr_mb_field_decoding_flag: bool,
) {
    // binarization is fixed length of 1
    let binarized = match se_val {
        true => 1u8,
        false => 0u8,
    };

    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_significant_coeff_flag - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("significant_coeff_flag", se_val, 63);
    }
    let mut ctx_idx_offset: usize = 0;
    let mut ctx_idx_inc: usize = 0;
    let ctx_block_cat_offset =
        cabac_tables::CTX_BLOCK_CAT_OFFSET_SIGNIFICANT_COEFF_FLAG[ctx_block_cat as usize];

    if sh.field_pic_flag || curr_mb_field_decoding_flag {
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

    let level_list_idx = additional_inputs[0];
    if ctx_block_cat != 3 && ctx_block_cat != 5 && ctx_block_cat != 9 && ctx_block_cat != 13 {
        ctx_idx_inc = level_list_idx;
    } else if ctx_block_cat == 3 {
        let num_c8x8 = additional_inputs[1];

        ctx_idx_inc = cmp::min(level_list_idx / num_c8x8, 2);
    } else if ctx_block_cat == 5 || ctx_block_cat == 9 || ctx_block_cat == 13 {
        // use Table 9-43
        if sh.field_pic_flag || curr_mb_field_decoding_flag {
            // field coded slice
            ctx_idx_inc = cabac_tables::CTX_IDX_INC_SIGNIFICANT_COEFF_FLAG_FIELD_CODED
                [level_list_idx as usize];
        } else {
            ctx_idx_inc = cabac_tables::CTX_IDX_INC_SIGNIFICANT_COEFF_FLAG_FRAME_CODED
                [level_list_idx as usize];
        }
    }
    let ctx_idx = ctx_idx_offset + ctx_block_cat_offset + ctx_idx_inc;
    // binary encode with model
    let mut idx1 = sh.cabac_init_idc as usize;
    let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;

    if !is_slice_type(sh.slice_type, "I") && !is_slice_type(sh.slice_type, "SI") && ctx_idx >= 54 {
        idx1 += 1;
    }
    if CABAC_DEBUG {
        debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
        debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
    }
    stream.append(&mut arithmetic_encode(&binarized, cs, idx1, idx2, ctx_idx));

    if CABAC_DEBUG {
        debug!(target: "encode",
                "updated state: {:x}",
                cs.states[idx1][idx2][ctx_idx].p_state_idx
        );
    }
}

/// Macroblock Residual Data - last_significant_coeff_flag
pub fn cabac_encode_last_significant_coeff_flag(
    se_val: bool,
    ctx_block_cat: u8,
    sh: &SliceHeader,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
    additional_inputs: &[usize],
    curr_mb_field_decoding_flag: bool,
) {
    // binarization is fixed length of 1
    let binarized = match se_val {
        true => 1u8,
        false => 0u8,
    };

    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_last_significant_coeff_flag - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("last_significant_coeff_flag", se_val, 63);
    }
    let mut ctx_idx_offset: usize = 0;
    let mut ctx_idx_inc: usize = 0;

    if sh.field_pic_flag || curr_mb_field_decoding_flag {
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

    let ctx_block_cat_offset =
        cabac_tables::CTX_BLOCK_CAT_OFFSET_LAST_SIGNIFICANT_COEFF_FLAG[ctx_block_cat as usize];
    let level_list_idx = additional_inputs[0];
    if ctx_block_cat != 3 && ctx_block_cat != 5 && ctx_block_cat != 9 && ctx_block_cat != 13 {
        ctx_idx_inc = level_list_idx;
    } else if ctx_block_cat == 3 {
        let num_c8x8 = additional_inputs[1];

        ctx_idx_inc = cmp::min(level_list_idx / num_c8x8, 2);
    } else if ctx_block_cat == 5 || ctx_block_cat == 9 || ctx_block_cat == 13 {
        // use Table 9-43
        ctx_idx_inc = cabac_tables::CTX_IDX_INC_LAST_SIGNIFICANT_COEFF_FLAG[level_list_idx];
    }

    let ctx_idx = ctx_idx_offset + ctx_block_cat_offset + ctx_idx_inc;

    // binary encode with model
    let mut idx1 = sh.cabac_init_idc as usize;
    let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;

    if !is_slice_type(sh.slice_type, "I") && !is_slice_type(sh.slice_type, "SI") && ctx_idx >= 54 {
        idx1 += 1;
    }
    if CABAC_DEBUG {
        debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
        debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
    }
    stream.append(&mut arithmetic_encode(&binarized, cs, idx1, idx2, ctx_idx));

    if CABAC_DEBUG {
        debug!(target: "encode",
            "updated state: {:x}",
            cs.states[idx1][idx2][ctx_idx].p_state_idx
        );
    }
}

/// Macroblock Residual Data - coeff_abs_level_minus1
pub fn cabac_encode_coeff_abs_level_minus1(
    se_val: u32,
    ctx_block_cat: u8,
    sh: &SliceHeader,
    stream: &mut Vec<u8>,
    cs: &mut CABACState,
    additional_inputs: &[usize],
) {
    // binarization is given by UEG0 with signed_val_flag = 0 && u_coff = 14

    let binarized = generate_coeff_abs_level_minus1_value(se_val);

    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_coeff_abs_level_minus1 - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("coeff_abs_level_minus1", se_val, 63);
    }
    // cabac
    // match ctx_block_cat {
    //         < 5: prefix: max_bin_idx_ctx = 1; ctx_idx_offset = 277
    //              suffix: max_bin_idx_ctx = na; use DecodeBypass
    //        == 5: prefix: max_bin_idx_ctx = 1; ctx_idx_offset = 426
    //              suffix: max_bin_idx_ctx = na; use DecodeBypass
    //   5 < x < 9: prefix: max_bin_idx_ctx = 1; ctx_idx_offset = 952
    //              suffix: max_bin_idx_ctx = na; use DecodeBypass
    //        == 9: prefix: max_bin_idx_ctx = 1; ctx_idx_offset = 708
    //              suffix: max_bin_idx_ctx = na; use DecodeBypass
    //  9 < x < 13: prefix: max_bin_idx_ctx = 1; ctx_idx_offset = 982
    //              suffix: max_bin_idx_ctx = na; use DecodeBypass
    //       == 13: prefix: max_bin_idx_ctx = 1; ctx_idx_offset = 766
    //              suffix: max_bin_idx_ctx = na; use DecodeBypass
    //}
    let mut ctx_idx_offset: usize = 0;
    let mut ctx_idx_inc: usize;

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

    let ctx_block_cat_offset =
        cabac_tables::CTX_BLOCK_CAT_OFFSET_COEFF_ABS_LEVEL_MINUS1[ctx_block_cat as usize];

    let num_decode_abs_level_eq_1 = additional_inputs[0];
    let num_decode_abs_level_gt_1 = additional_inputs[1];

    for bin_idx in 0..binarized.len() {
        if bin_idx == 14 {
            // we need to encode the suffix using encode_bypass
            break;
        }

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

        let ctx_idx = ctx_idx_offset + ctx_block_cat_offset + ctx_idx_inc;

        // binary encode with model
        let mut idx1 = sh.cabac_init_idc as usize;
        let idx2 = clip3(0, 51, sh.slice_qp_y as i32) as usize;

        if !is_slice_type(sh.slice_type, "I")
            && !is_slice_type(sh.slice_type, "SI")
            && ctx_idx >= 54
        {
            idx1 += 1;
        }
        if CABAC_DEBUG {
            debug!(target: "encode","cod_i_offset: {:x}", cs.cod_i_offset << 7);
            debug!(target: "encode","state: {:x}", cs.states[idx1][idx2][ctx_idx].p_state_idx);
        }
        stream.append(&mut arithmetic_encode(
            &binarized[bin_idx],
            cs,
            idx1,
            idx2,
            ctx_idx,
        ));
        if CABAC_DEBUG {
            debug!(target: "encode",
                "updated state: {:x}",
                cs.states[idx1][idx2][ctx_idx].p_state_idx
            );
        }
    }

    // encode the prefix using the encode_bypass
    if binarized.len() > 14 {
        if CABAC_DEBUG {
            debug!(target: "encode","\tcabac_encode_coeff_abs_level_minus1 - encoding suffix with decode_bypass");
        }
        for bin_idx in 14..binarized.len() {
            stream.append(&mut encode_bypass(binarized[bin_idx], cs));
        }
    }
}

/// Macroblock Residual Data - coeff_sign_flag
pub fn cabac_encode_coeff_sign_flag(se_val: bool, stream: &mut Vec<u8>, cs: &mut CABACState) {
    // binarization is fixed length of 1
    let binarized = match se_val {
        true => 1u8,
        false => 0u8,
    };

    if CABAC_DEBUG {
        debug!(target: "encode","\tcabac_encode_coeff_sign_flag - Se_val is {:?} and the binarized value is {:?}", se_val, binarized);
    } else {
        encoder_formatted_print("coeff_sign_flag", se_val, 63);
    }
    // cabac max_bin_idx_ctx = 0; use DecodeBypass
    stream.append(&mut encode_bypass(binarized, cs));
}
