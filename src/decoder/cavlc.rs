//! CAVLC entropy decoding.

use crate::common::cavlc_tables::create_coeff_token_mappings;
use crate::common::cavlc_tables::create_run_before_mappings;
use crate::common::cavlc_tables::create_total_zeros_mappings;
use crate::common::cavlc_tables::MAPPED_EXP_GOLOMB_CAT03;
use crate::common::cavlc_tables::MAPPED_EXP_GOLOMB_CAT12;
use crate::common::data_structures::CoeffToken;
use crate::common::data_structures::MacroBlock;
use crate::common::data_structures::MbPartPredMode;
use crate::common::data_structures::MbType;
use crate::common::data_structures::ResidualMode;
use crate::common::data_structures::SliceData;
use crate::common::data_structures::TransformBlock;
use crate::common::data_structures::VideoParameters;
use crate::common::helper::ByteStream;
use crate::decoder::expgolomb::exp_golomb_decode_one_wrapper;
use log::debug;

/// Mapped ExpGolomb decode - mapped to Table 9-4
///
/// chroma_array_type : ChromaArrayType parameter
/// intra_mode : 0 for Intra_4x4/Intra_8x8; 1 for Inter
/// Described in clause 9.1
pub fn mapped_exp_golomb_decode(
    chroma_array_type: u8,
    intra_mode: usize,
    bs: &mut ByteStream,
) -> i32 {
    // first recover the codeNum
    let res: i32;
    let code_num = exp_golomb_decode_one_wrapper(bs, false, 0) as usize;

    // follow the mapping in 9.1.2
    if chroma_array_type == 1 || chroma_array_type == 2 {
        res = MAPPED_EXP_GOLOMB_CAT12[code_num][intra_mode];
    } else if chroma_array_type == 0 || chroma_array_type == 3 {
        res = MAPPED_EXP_GOLOMB_CAT03[code_num][intra_mode];
    } else {
        panic!("Wrong chroma_array_type: {}", chroma_array_type);
    }

    res
}

/// Truncated Exp-Golomb-coded syntax element
///
/// The range of possible values for the syntax element is determined first. The range of this
/// syntax element may be between 0 and x, with x being greater than or equal to 1 and the range
/// is used in the derivation of the value of the syntax element value as follows:
/// - if x is greater than 1, then use ue(v)
/// - if x is equal to 1, then return !read_bits(1)
///
/// Described in clause 9.1
pub fn truncated_exp_golomb_decode(max_value: u32, bs: &mut ByteStream) -> u32 {
    let res: u32 = if max_value == 1 {
        1 - bs.read_bits(1)
    } else {
        exp_golomb_decode_one_wrapper(bs, false, 0) as u32
    };
    res
}

/// CoeffToken decoding
///
/// Inputs to this process are bits from slice data, a maximum number of non-zero
/// transform coefficient levels maxNumCoeff, the luma block index luma4x4BlkIdx
/// or the chroma block index chroma4x4BlkIdx, cb4x4BlkIdx or cr4x4BlkIdx of the
/// current block of transform coefficient levels.
/// Outputs of this process are TotalCoeff( coeff_token ), TrailingOnes( coeff_token ),
/// and the variable nC.
pub fn cavlc_decode_coeff_token(
    residual_mode: ResidualMode,
    curr_mb_idx: usize,
    sd: &mut SliceData,
    mut blk_idx: usize,
    i_cb_cr: usize,
    bs: &mut ByteStream,
    vp: &VideoParameters,
) -> CoeffToken {
    // First calculate nC, then return the total_coeff and trailing_ones
    let n_c: i8;

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

        debug!(target: "decode","blk_idx - {}", blk_idx);

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
        debug!(target: "decode","blk_idx_a {}, blk_idx_b {}", blk_idx_a, blk_idx_b);
        // step 5
        let mut available_flag_a: bool = true;
        let mut available_flag_b: bool = true;

        if !mb_a.available
            || (sd.macroblock_vec[curr_mb_idx].is_intra()
                && vp.pps_constrained_intra_pred_flag
                && mb_a.is_inter()
                && 2 <= vp.nal_unit_type
                && vp.nal_unit_type <= 4)
        {
            available_flag_a = false;
        }

        if !mb_b.available
            || (sd.macroblock_vec[curr_mb_idx].is_intra()
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

        debug!(target: "decode","available_flag_a - {}; available_flag_b - {}", available_flag_a, available_flag_b);

        if available_flag_a {
            if mb_a.mb_type == MbType::PSkip
                || mb_a.mb_type == MbType::BSkip
                || (mb_a.mb_type != MbType::IPCM && mb_a.ac_resid_all_zero(ac_mode, blk_idx_a))
            {
                debug!(target: "decode","mb_a.ac_resid_all_zero(ac_mode): {}", mb_a.ac_resid_all_zero(ac_mode, blk_idx_a));
                n_a = 0;
            } else if mb_a.mb_type == MbType::IPCM {
                debug!(target: "decode","mb_a.mb_type == MbType::IPCM");
                n_a = 16;
            } else {
                n_a = blk_a.coeff_token.total_coeff;
                debug!(target: "decode","n_a - using previous value - {}", n_a);
            }
        }

        if available_flag_b {
            if mb_b.mb_type == MbType::PSkip
                || mb_b.mb_type == MbType::BSkip
                || (mb_b.mb_type != MbType::IPCM && mb_b.ac_resid_all_zero(ac_mode, blk_idx_b))
            {
                debug!(target: "decode","mb_b.ac_resid_all_zero(ac_mode): {}", mb_b.ac_resid_all_zero(ac_mode, blk_idx_b));
                n_b = 0;
            } else if mb_b.mb_type == MbType::IPCM {
                debug!(target: "decode","mb_b.mb_type == MbType::IPCM");
                n_b = 16;
            } else {
                n_b = blk_b.coeff_token.total_coeff;
                debug!(target: "decode","n_b - using previous value - {}", n_b);
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
    debug!(target: "decode","using n_c value - {}", n_c);
    let coeff_token_mapping = create_coeff_token_mappings(n_c);

    let mut key: String = String::new();
    let total_coeff: usize;
    let trailing_ones: usize;
    loop {
        key += &bs.read_bits(1).to_string();
        debug!(target: "decode","cavlc_decode_coeff_token - key: {:?}", key);
        if coeff_token_mapping.contains_key(&key) {
            let res = coeff_token_mapping[&key];
            trailing_ones = res.0;
            total_coeff = res.1;
            break;
        }

        if key.len() > 16 {
            panic!("key not found in coeff_token_mapping: {:?}", key);
        }
    }

    // NOTE: if max_num_coeff == 15 then total_coeff cannot be equal to 16 for bitstream conformance

    CoeffToken {
        total_coeff,
        trailing_ones,
        n_c,
    }
}

/// Level prefix decoding
///
/// Decodes level_prefix by counting the number of zeros in the stream
/// Described in section 9.2.2.1
pub fn cavlc_decode_level_prefix(bs: &mut ByteStream) -> u32 {
    let mut leading_zeros = 0;

    // see section 9.2.2.1 and Table 9-6 as an example
    let mut cur_bit = bs.read_bits(1);
    while cur_bit == 0 {
        leading_zeros += 1;
        cur_bit = bs.read_bits(1);
    }

    leading_zeros
}

/// Level suffix decoding
///
/// Described in section 9.2.2
pub fn cavlc_decode_level_suffix(
    suffix_length: u32,
    level_prefix: u32,
    bs: &mut ByteStream,
) -> u32 {
    let level_suffix_size: u32 = if level_prefix == 14 && suffix_length == 0 {
        4
    } else if level_prefix >= 15 {
        level_prefix - 3
    } else {
        suffix_length
    };
    debug!(target: "decode","cavlc_decode_level_suffix - level_suffix_size: {}", level_suffix_size);

    let level_suffix: u32 = if level_suffix_size > 0 {
        bs.read_bits(level_suffix_size as u8)
    } else {
        0
    };

    level_suffix
}

/// Total zeroes decoding
///
/// Described in section 9.2.3 - returns the total_zeros for run information
///
/// tz_vlc_index is set equal to TotalCoeff(coeff_token) and is used to index into the tables
/// max_num_coeff is used to decide which table
/// bs is the bitstream
pub fn cavlc_decode_total_zeros(
    tz_vlc_index: usize,
    max_num_coeff: usize,
    bs: &mut ByteStream,
) -> usize {
    let total_zeros: usize;
    debug!(target: "decode","cavlc_decode_total_zeros - max_num_coeff: {}; tz_vlc_index : {}", max_num_coeff, tz_vlc_index);
    let total_zeros_mapping = create_total_zeros_mappings(max_num_coeff, tz_vlc_index);

    let mut key: String = String::new();
    loop {
        key += &bs.read_bits(1).to_string();
        debug!(target: "decode","cavlc_decode_total_zeros - key: {:?}", key);
        if total_zeros_mapping.contains_key(&key) {
            total_zeros = total_zeros_mapping[&key];
            break;
        }
    }

    total_zeros
}

/// Run before decoding
///
/// Described in 9.2.3 and Table 9-10
pub fn cavlc_decode_run_before(zeros_left: usize, bs: &mut ByteStream) -> usize {
    let run_before: usize;

    let run_before_mapping = create_run_before_mappings(zeros_left);

    let mut key: String = String::new();
    loop {
        key += &bs.read_bits(1).to_string();
        debug!(target: "decode","cavlc_decode_run_before - key: {:?}", key);
        if run_before_mapping.contains_key(&key) {
            run_before = run_before_mapping[&key];
            break;
        }
    }

    run_before
}
