//! Macroblock syntax element decoding.

use crate::common::data_structures::MbPartPredMode;
use crate::common::data_structures::MbType;
use crate::common::data_structures::PicParameterSet;
use crate::common::data_structures::ResidualMode;
use crate::common::data_structures::SeqParameterSet;
use crate::common::data_structures::SliceData;
use crate::common::data_structures::SliceHeader;
use crate::common::data_structures::SubMbType;
use crate::common::data_structures::TransformBlock;
use crate::common::data_structures::VideoParameters;
use crate::common::helper::decoder_formatted_print;
use crate::common::helper::is_slice_type;
use crate::common::helper::ByteStream;
use crate::decoder::cabac::cabac_decode;
use crate::decoder::cabac::CABACState;
use crate::decoder::cavlc::cavlc_decode_coeff_token;
use crate::decoder::cavlc::cavlc_decode_level_prefix;
use crate::decoder::cavlc::cavlc_decode_level_suffix;
use crate::decoder::cavlc::cavlc_decode_run_before;
use crate::decoder::cavlc::cavlc_decode_total_zeros;
use crate::decoder::cavlc::mapped_exp_golomb_decode;
use crate::decoder::cavlc::truncated_exp_golomb_decode;
use crate::decoder::expgolomb::exp_golomb_decode_one_wrapper;
use log::debug;
use std::cmp;

/// Turns the decoded mb_type number into the actual
/// Type. Based off of Tables 7-11 to 7-14
fn decode_mb_type(mut res: i32, sh: &SliceHeader) -> MbType {
    if is_slice_type(sh.slice_type, "SI") {
        match res {
            0 => return MbType::SI,
            _ => res -= 1, // Table 7-12: indexed by subtracting 1 from the value of mb_type
        }
    } else if is_slice_type(sh.slice_type, "P") || is_slice_type(sh.slice_type, "SP") {
        // Table 7-13
        match res {
            0 => return MbType::PL016x16,
            1 => return MbType::PL0L016x8,
            2 => return MbType::PL0L08x16,
            3 => return MbType::P8x8,
            4 => return MbType::P8x8ref0, // this should not be allowed
            _ => res -= 5,
        }
    } else if is_slice_type(sh.slice_type, "B") {
        // Table 7-14
        match res {
            0 => return MbType::BDirect16x16,
            1 => return MbType::BL016x16,
            2 => return MbType::BL116x16,
            3 => return MbType::BBi16x16,
            4 => return MbType::BL0L016x8,
            5 => return MbType::BL0L08x16,
            6 => return MbType::BL1L116x8,
            7 => return MbType::BL1L18x16,
            8 => return MbType::BL0L116x8,
            9 => return MbType::BL0L18x16,
            10 => return MbType::BL1L016x8,
            11 => return MbType::BL1L08x16,
            12 => return MbType::BL0Bi16x8,
            13 => return MbType::BL0Bi8x16,
            14 => return MbType::BL1Bi16x8,
            15 => return MbType::BL1Bi8x16,
            16 => return MbType::BBiL016x8,
            17 => return MbType::BBiL08x16,
            18 => return MbType::BBiL116x8,
            19 => return MbType::BBiL18x16,
            20 => return MbType::BBiBi16x8,
            21 => return MbType::BBiBi8x16,
            22 => return MbType::B8x8,
            _ => res -= 23,
        }
    }
    // MB Types for intra prediction can be included inside any slice type
    // Table 7-11
    match res {
        0 => MbType::INxN,
        1 => MbType::I16x16_0_0_0,
        2 => MbType::I16x16_1_0_0,
        3 => MbType::I16x16_2_0_0,
        4 => MbType::I16x16_3_0_0,
        5 => MbType::I16x16_0_1_0,
        6 => MbType::I16x16_1_1_0,
        7 => MbType::I16x16_2_1_0,
        8 => MbType::I16x16_3_1_0,
        9 => MbType::I16x16_0_2_0,
        10 => MbType::I16x16_1_2_0,
        11 => MbType::I16x16_2_2_0,
        12 => MbType::I16x16_3_2_0,
        13 => MbType::I16x16_0_0_1,
        14 => MbType::I16x16_1_0_1,
        15 => MbType::I16x16_2_0_1,
        16 => MbType::I16x16_3_0_1,
        17 => MbType::I16x16_0_1_1,
        18 => MbType::I16x16_1_1_1,
        19 => MbType::I16x16_2_1_1,
        20 => MbType::I16x16_3_1_1,
        21 => MbType::I16x16_0_2_1,
        22 => MbType::I16x16_1_2_1,
        23 => MbType::I16x16_2_2_1,
        24 => MbType::I16x16_3_2_1,
        25 => MbType::IPCM,
        _ => MbType::INONE,
    }
}

/// Turns the decoded sub_mb_type number into the actual
/// Type. Based off of Tables 7-17 to 7-18
fn decode_sub_mb_type(res: i32, sh: &SliceHeader) -> SubMbType {
    // value to sub_mb_type matching done in table 9-38
    if is_slice_type(sh.slice_type, "P") || is_slice_type(sh.slice_type, "SP") {
        match res {
            0 => SubMbType::PL08x8,
            1 => SubMbType::PL08x4,
            2 => SubMbType::PL04x8,
            3 => SubMbType::PL04x4,
            _ => panic!(
                "decode_sub_mb_type - Incorrect value provided for P slice: {:?}",
                res
            ),
        }
    } else if is_slice_type(sh.slice_type, "B") {
        match res {
            0 => SubMbType::BDirect8x8,
            1 => SubMbType::BL08x8,
            2 => SubMbType::BL18x8,
            3 => SubMbType::BBi8x8,
            4 => SubMbType::BL08x4,
            5 => SubMbType::BL04x8,
            6 => SubMbType::BL18x4,
            7 => SubMbType::BL14x8,
            8 => SubMbType::BBi8x4,
            9 => SubMbType::BBi4x8,
            10 => SubMbType::BL04x4,
            11 => SubMbType::BL14x4,
            12 => SubMbType::BBi4x4,
            _ => panic!(
                "decode_sub_mb_type - Incorrect value provided for B slice: {:?}",
                res
            ),
        }
    } else {
        panic!(
            "decode_sub_mb_type - Incorrect slice_type provided: {:?}",
            sh.slice_type
        );
    }
}

/// Follows Section 7.3.5
pub fn decode_macroblock_layer(
    curr_mb_idx: usize,
    mut cabac_state: &mut CABACState,
    mut bs: &mut ByteStream,
    sd: &mut SliceData,
    mut sh: &mut SliceHeader,
    vp: &VideoParameters,
    s: &SeqParameterSet,
    p: &PicParameterSet,
) {
    let res: i32 = if p.entropy_coding_mode_flag {
        cabac_decode(
            "mb_type",
            bs,
            cabac_state,
            curr_mb_idx,
            sh,
            sd,
            vp,
            0,
            Vec::new(),
        )
    } else {
        exp_golomb_decode_one_wrapper(bs, false, 0)
    };

    sd.macroblock_vec[curr_mb_idx].mb_type = decode_mb_type(res, sh);

    decoder_formatted_print("mb_type", &res, 63);
    decoder_formatted_print("mb_type(fancy)", sd.macroblock_vec[curr_mb_idx].mb_type, 63);

    if sd.macroblock_vec[curr_mb_idx].mb_type == MbType::IPCM {
        // zero align
        if bs.byte_offset > 0 {
            bs.byte_offset = 0;
            bs.bytestream.pop_front();
        }

        // pcm_luma_sample

        // fast-path: we're just reading bytes
        if vp.bit_depth_y == 8 {
            for i in 0..256 {
                sd.macroblock_vec[curr_mb_idx]
                    .pcm_sample_luma
                    .push(bs.bytestream[i] as u32);
                decoder_formatted_print(
                    "pcm_byte luma",
                    sd.macroblock_vec[curr_mb_idx].pcm_sample_luma[i],
                    63,
                );
            }
            bs.bytestream.drain(0..256);
        } else {
            // use the complete bit-length
            for i in 0..256 {
                sd.macroblock_vec[curr_mb_idx]
                    .pcm_sample_luma
                    .push(bs.read_bits(vp.bit_depth_y));
                decoder_formatted_print(
                    "pcm_byte luma",
                    sd.macroblock_vec[curr_mb_idx].pcm_sample_luma[i],
                    63,
                );
            }
        }

        if vp.bit_depth_c == 8 {
            let max_chroma_params = (2 * vp.mb_width_c * vp.mb_height_c) as usize;
            for i in 0..max_chroma_params {
                sd.macroblock_vec[curr_mb_idx]
                    .pcm_sample_chroma
                    .push(bs.bytestream[i] as u32);
                decoder_formatted_print(
                    "pcm_byte chroma",
                    sd.macroblock_vec[curr_mb_idx].pcm_sample_chroma[i as usize],
                    63,
                );
            }
            bs.bytestream.drain(0..max_chroma_params);
        } else {
            for i in 0..2 * vp.mb_width_c * vp.mb_height_c {
                sd.macroblock_vec[curr_mb_idx]
                    .pcm_sample_chroma
                    .push(bs.read_bits(vp.bit_depth_c));
                decoder_formatted_print(
                    "pcm_byte chroma",
                    sd.macroblock_vec[curr_mb_idx].pcm_sample_chroma[i as usize],
                    63,
                );
            }
        }

        if p.entropy_coding_mode_flag {
            // reset the CABAC state after reading the above bytes
            cabac_state.cod_i_range = 510;
            cabac_state.cod_i_offset = bs.read_bits(9);
        }
    } else {
        sd.macroblock_vec[curr_mb_idx].no_sub_mb_part_size_less_than_8x8_flag = true;

        if sd.macroblock_vec[curr_mb_idx].mb_type != MbType::INxN
            && sd.macroblock_vec[curr_mb_idx].mb_part_pred_mode(0) != MbPartPredMode::Intra16x16
            && sd.macroblock_vec[curr_mb_idx].num_mb_part() == 4
        {
            decode_sub_mb_pred(curr_mb_idx, bs, cabac_state, sh, sd, vp, p);

            for mb_part_idx in 0..4 {
                if sd.macroblock_vec[curr_mb_idx].sub_mb_type[mb_part_idx] != SubMbType::BDirect8x8
                {
                    if sd.macroblock_vec[curr_mb_idx].num_sub_mb_part(mb_part_idx) > 1 {
                        sd.macroblock_vec[curr_mb_idx].no_sub_mb_part_size_less_than_8x8_flag =
                            false;
                    }
                } else if !s.direct_8x8_inference_flag {
                    sd.macroblock_vec[curr_mb_idx].no_sub_mb_part_size_less_than_8x8_flag = false;
                }
            }
        } else {
            if p.transform_8x8_mode_flag && sd.macroblock_vec[curr_mb_idx].mb_type == MbType::INxN {
                let res: i32 = if p.entropy_coding_mode_flag {
                    cabac_decode(
                        "transform_size_8x8_flag",
                        bs,
                        cabac_state,
                        curr_mb_idx,
                        sh,
                        sd,
                        vp,
                        0,
                        Vec::new(),
                    )
                } else {
                    bs.read_bits(1) as i32
                };

                sd.macroblock_vec[curr_mb_idx].transform_size_8x8_flag = match res {
                    1 => true,
                    _ => false,
                };

                decoder_formatted_print("transform_size_8x8_flag", &res, 63);
            }
            decode_mb_pred(curr_mb_idx, bs, cabac_state, sh, sd, vp, p);
        }
        // equation 7-36
        sd.macroblock_vec[curr_mb_idx].set_cbp_chroma_and_luma();

        if sd.macroblock_vec[curr_mb_idx].mb_part_pred_mode(0) != MbPartPredMode::Intra16x16 {
            let res: i32 = if p.entropy_coding_mode_flag {
                cabac_decode(
                    "coded_block_pattern",
                    bs,
                    cabac_state,
                    curr_mb_idx,
                    sh,
                    sd,
                    vp,
                    0,
                    Vec::new(),
                )
            } else {
                let intra_mode = match sd.macroblock_vec[curr_mb_idx].is_intra() {
                    true => 0,
                    _ => 1,
                };
                mapped_exp_golomb_decode(vp.chroma_array_type, intra_mode, bs)
            };

            sd.macroblock_vec[curr_mb_idx].coded_block_pattern = res as u32;

            decoder_formatted_print(
                "coded_block_pattern",
                &sd.macroblock_vec[curr_mb_idx].coded_block_pattern,
                63,
            );

            // equation 7-36 (we do this again because coded_block_pattern is in the bitstream)
            sd.macroblock_vec[curr_mb_idx].coded_block_pattern_luma =
                sd.macroblock_vec[curr_mb_idx].coded_block_pattern % 16;
            sd.macroblock_vec[curr_mb_idx].coded_block_pattern_chroma =
                sd.macroblock_vec[curr_mb_idx].coded_block_pattern / 16;

            if sd.macroblock_vec[curr_mb_idx].coded_block_pattern_luma > 0
                && p.transform_8x8_mode_flag
                && sd.macroblock_vec[curr_mb_idx].mb_type != MbType::INxN
                && sd.macroblock_vec[curr_mb_idx].no_sub_mb_part_size_less_than_8x8_flag
                && (sd.macroblock_vec[curr_mb_idx].mb_type != MbType::BDirect16x16
                    || s.direct_8x8_inference_flag)
            {
                let res: i32 = if p.entropy_coding_mode_flag {
                    cabac_decode(
                        "transform_size_8x8_flag",
                        bs,
                        cabac_state,
                        curr_mb_idx,
                        sh,
                        sd,
                        vp,
                        0,
                        Vec::new(),
                    )
                } else {
                    bs.read_bits(1) as i32
                };

                sd.macroblock_vec[curr_mb_idx].transform_size_8x8_flag = match res {
                    1 => true,
                    _ => false,
                };

                decoder_formatted_print("transform_size_8x8_flag (inside condition)", &res, 63);
            }
        }
        if sd.macroblock_vec[curr_mb_idx].coded_block_pattern_luma > 0
            || sd.macroblock_vec[curr_mb_idx].coded_block_pattern_chroma > 0
            || sd.macroblock_vec[curr_mb_idx].mb_part_pred_mode(0) == MbPartPredMode::Intra16x16
        {
            let res: i32;

            if p.entropy_coding_mode_flag {
                res = cabac_decode(
                    "mb_qp_delta",
                    bs,
                    cabac_state,
                    curr_mb_idx,
                    sh,
                    sd,
                    vp,
                    0,
                    Vec::new(),
                );
                sd.macroblock_vec[curr_mb_idx].mb_qp_delta = ((res + 1) / 2)
                    * match res % 2 {
                        0 => -1i32,
                        _ => 1i32,
                    };
            } else {
                res = exp_golomb_decode_one_wrapper(bs, true, 0);
                sd.macroblock_vec[curr_mb_idx].mb_qp_delta = res;
            }

            // implement transformation of Table 9-3

            decoder_formatted_print(
                "mb_qp_delta",
                &sd.macroblock_vec[curr_mb_idx].mb_qp_delta,
                63,
            );

            sd.macroblock_vec[curr_mb_idx].qp_y = (sh.qp_y_prev
                + sd.macroblock_vec[curr_mb_idx].mb_qp_delta
                + 52
                + 2 * vp.qp_bd_offset_y)
                % (52 + vp.qp_bd_offset_y)
                - vp.qp_bd_offset_y;
            sh.qp_y_prev = sd.macroblock_vec[curr_mb_idx].qp_y;
            sd.macroblock_vec[curr_mb_idx].qp_y_prime =
                sd.macroblock_vec[curr_mb_idx].qp_y + vp.qp_bd_offset_y;

            if s.qpprime_y_zero_transform_bypass_flag
                && sd.macroblock_vec[curr_mb_idx].qp_y_prime == 0
            {
                sd.macroblock_vec[curr_mb_idx].transform_bypass_mode_flag = true;
            }

            decode_residual(0, 15, bs, cabac_state, curr_mb_idx, sh, sd, vp, p);
        }
    }
}

/// Follows Section 7.3.5.1
fn decode_mb_pred(
    curr_mb_idx: usize,
    bs: &mut ByteStream,
    cabac_state: &mut CABACState,
    sh: &SliceHeader,
    sd: &mut SliceData,
    vp: &VideoParameters,
    p: &PicParameterSet,
) {
    let mpp_mode = sd.macroblock_vec[curr_mb_idx].mb_part_pred_mode(0);

    if mpp_mode == MbPartPredMode::Intra4x4
        || mpp_mode == MbPartPredMode::Intra8x8
        || mpp_mode == MbPartPredMode::Intra16x16
    {
        if mpp_mode == MbPartPredMode::Intra4x4 {
            for luma4x4blkidx in 0..16 {
                //luma4x4BlkIdx
                let res: i32 = if p.entropy_coding_mode_flag {
                    cabac_decode(
                        "prev_intra4x4_pred_mode_flag",
                        bs,
                        cabac_state,
                        curr_mb_idx,
                        sh,
                        sd,
                        vp,
                        0,
                        Vec::new(),
                    )
                } else {
                    bs.read_bits(1) as i32
                };

                sd.macroblock_vec[curr_mb_idx].prev_intra4x4_pred_mode_flag[luma4x4blkidx] =
                    match res {
                        1 => true,
                        _ => false,
                    };

                if !sd.macroblock_vec[curr_mb_idx].prev_intra4x4_pred_mode_flag[luma4x4blkidx] {
                    let res: u32 = if p.entropy_coding_mode_flag {
                        cabac_decode(
                            "rem_intra4x4_pred_mode",
                            bs,
                            cabac_state,
                            curr_mb_idx,
                            sh,
                            sd,
                            vp,
                            0,
                            Vec::new(),
                        ) as u32
                    } else {
                        bs.read_bits(3)
                    };

                    sd.macroblock_vec[curr_mb_idx].rem_intra4x4_pred_mode[luma4x4blkidx] = res;

                    decoder_formatted_print(
                        "intra4x4_pred_mode - read_ipred_4x4_modes",
                        &sd.macroblock_vec[curr_mb_idx].rem_intra4x4_pred_mode[luma4x4blkidx],
                        63,
                    );
                } else {
                    decoder_formatted_print(
                        "intra4x4_pred_mode - prev_intra4x4_pred_mode_flag",
                        match &sd.macroblock_vec[curr_mb_idx].prev_intra4x4_pred_mode_flag
                            [luma4x4blkidx]
                        {
                            true => 1,
                            false => 0,
                        },
                        63,
                    );
                }
            }
        }
        if mpp_mode == MbPartPredMode::Intra8x8 {
            for luma8x8blkidx in 0..4 {
                let res: u32 = if p.entropy_coding_mode_flag {
                    cabac_decode(
                        "prev_intra8x8_pred_mode_flag",
                        bs,
                        cabac_state,
                        curr_mb_idx,
                        sh,
                        sd,
                        vp,
                        0,
                        Vec::new(),
                    ) as u32
                } else {
                    bs.read_bits(1)
                };

                sd.macroblock_vec[curr_mb_idx].prev_intra8x8_pred_mode_flag[luma8x8blkidx] =
                    match res {
                        1 => true,
                        _ => false,
                    };

                if !sd.macroblock_vec[curr_mb_idx].prev_intra8x8_pred_mode_flag[luma8x8blkidx] {
                    let res: u32 = if p.entropy_coding_mode_flag {
                        cabac_decode(
                            "rem_intra8x8_pred_mode",
                            bs,
                            cabac_state,
                            curr_mb_idx,
                            sh,
                            sd,
                            vp,
                            0,
                            Vec::new(),
                        ) as u32
                    } else {
                        bs.read_bits(3)
                    };

                    sd.macroblock_vec[curr_mb_idx].rem_intra8x8_pred_mode[luma8x8blkidx] = res;
                    decoder_formatted_print(
                        "intra4x4_pred_mode - read_ipred_8x8_modes",
                        &sd.macroblock_vec[curr_mb_idx].rem_intra8x8_pred_mode[luma8x8blkidx],
                        63,
                    );
                } else {
                    decoder_formatted_print("prev_intra8x8_pred_mode_flag", &res, 63);
                }
            }
        }
        if vp.chroma_array_type == 1 || vp.chroma_array_type == 2 {
            let res: i32 = if p.entropy_coding_mode_flag {
                cabac_decode(
                    "intra_chroma_pred_mode",
                    bs,
                    cabac_state,
                    curr_mb_idx,
                    sh,
                    sd,
                    vp,
                    0,
                    Vec::new(),
                )
            } else {
                exp_golomb_decode_one_wrapper(bs, false, 0)
            };

            sd.macroblock_vec[curr_mb_idx].intra_chroma_pred_mode = res as u8;
            decoder_formatted_print(
                "intra_chroma_pred_mode",
                &sd.macroblock_vec[curr_mb_idx].intra_chroma_pred_mode,
                63,
            );
        }
    } else if mpp_mode != MbPartPredMode::Direct {
        for mb_part_idx in 0..sd.macroblock_vec[curr_mb_idx].num_mb_part() {
            if (sh.num_ref_idx_l0_active_minus1 > 0
                || sd.mb_field_decoding_flag[curr_mb_idx] != sh.field_pic_flag)
                && sd.macroblock_vec[curr_mb_idx].mb_part_pred_mode(mb_part_idx)
                    != MbPartPredMode::PredL1
            {
                let additional_inputs = vec![mb_part_idx];
                let res: u32 = if p.entropy_coding_mode_flag {
                    cabac_decode(
                        "ref_idx_l0",
                        bs,
                        cabac_state,
                        curr_mb_idx,
                        sh,
                        sd,
                        vp,
                        0,
                        additional_inputs,
                    ) as u32
                } else {
                    let max_val: u32 =
                        if !sh.mbaff_frame_flag || !sd.mb_field_decoding_flag[curr_mb_idx] {
                            sh.num_ref_idx_l0_active_minus1
                        } else {
                            2 * sh.num_ref_idx_l0_active_minus1 + 1
                        };
                    truncated_exp_golomb_decode(max_val, bs) as u32
                };

                sd.macroblock_vec[curr_mb_idx].ref_idx_l0[mb_part_idx] = res;
                decoder_formatted_print(
                    "ref_idx_l0",
                    &sd.macroblock_vec[curr_mb_idx].ref_idx_l0[mb_part_idx],
                    63,
                );
            }
        }
        for mb_part_idx in 0..sd.macroblock_vec[curr_mb_idx].num_mb_part() {
            if (sh.num_ref_idx_l1_active_minus1 > 0
                || sd.mb_field_decoding_flag[curr_mb_idx] != sh.field_pic_flag)
                && sd.macroblock_vec[curr_mb_idx].mb_part_pred_mode(mb_part_idx)
                    != MbPartPredMode::PredL0
            {
                let additional_inputs = vec![mb_part_idx];
                let res: u32 = if p.entropy_coding_mode_flag {
                    cabac_decode(
                        "ref_idx_l1",
                        bs,
                        cabac_state,
                        curr_mb_idx,
                        sh,
                        sd,
                        vp,
                        0,
                        additional_inputs,
                    ) as u32
                } else {
                    let max_val: u32 =
                        if !sh.mbaff_frame_flag || !sd.mb_field_decoding_flag[curr_mb_idx] {
                            sh.num_ref_idx_l1_active_minus1
                        } else {
                            2 * sh.num_ref_idx_l1_active_minus1 + 1
                        };

                    truncated_exp_golomb_decode(max_val, bs) as u32
                };

                sd.macroblock_vec[curr_mb_idx].ref_idx_l1[mb_part_idx] = res;

                decoder_formatted_print(
                    "ref_idx_l1",
                    &sd.macroblock_vec[curr_mb_idx].ref_idx_l1[mb_part_idx],
                    63,
                );
            }
        }
        for mb_part_idx in 0..sd.macroblock_vec[curr_mb_idx].num_mb_part() {
            if sd.macroblock_vec[curr_mb_idx].mb_part_pred_mode(mb_part_idx)
                != MbPartPredMode::PredL1
            {
                for comp_idx in 0..2 {
                    let name = format!("mvd_l0_{}", comp_idx);
                    let additional_inputs = vec![mb_part_idx];
                    let res: i32 = if p.entropy_coding_mode_flag {
                        cabac_decode(
                            name.as_str(),
                            bs,
                            cabac_state,
                            curr_mb_idx,
                            sh,
                            sd,
                            vp,
                            0,
                            additional_inputs,
                        )
                    } else {
                        exp_golomb_decode_one_wrapper(bs, true, 0)
                    };

                    sd.macroblock_vec[curr_mb_idx].mvd_l0[mb_part_idx][0][comp_idx] = res;

                    decoder_formatted_print(
                        name.as_str(),
                        &sd.macroblock_vec[curr_mb_idx].mvd_l0[mb_part_idx][0][comp_idx],
                        63,
                    );
                }
            }
        }
        for mb_part_idx in 0..sd.macroblock_vec[curr_mb_idx].num_mb_part() {
            if sd.macroblock_vec[curr_mb_idx].mb_part_pred_mode(mb_part_idx)
                != MbPartPredMode::PredL0
            {
                for comp_idx in 0..2 {
                    let name = format!("mvd_l1_{}", comp_idx);
                    let additional_inputs = vec![mb_part_idx];

                    let res: i32 = if p.entropy_coding_mode_flag {
                        cabac_decode(
                            name.as_str(),
                            bs,
                            cabac_state,
                            curr_mb_idx,
                            sh,
                            sd,
                            vp,
                            0,
                            additional_inputs,
                        )
                    } else {
                        exp_golomb_decode_one_wrapper(bs, true, 0)
                    };

                    sd.macroblock_vec[curr_mb_idx].mvd_l1[mb_part_idx][0][comp_idx] = res;

                    decoder_formatted_print(
                        name.as_str(),
                        &sd.macroblock_vec[curr_mb_idx].mvd_l0[mb_part_idx][0][comp_idx],
                        63,
                    );
                }
            }
        }
    }
}

/// Follows Section 7.3.5.2
fn decode_sub_mb_pred(
    curr_mb_idx: usize,
    bs: &mut ByteStream,
    cabac_state: &mut CABACState,
    sh: &SliceHeader,
    sd: &mut SliceData,
    vp: &VideoParameters,
    p: &PicParameterSet,
) {
    for mb_part_idx in 0..4 {
        let res: i32 = if p.entropy_coding_mode_flag {
            cabac_decode(
                "sub_mb_type",
                bs,
                cabac_state,
                curr_mb_idx,
                sh,
                sd,
                vp,
                0,
                Vec::new(),
            )
        } else {
            exp_golomb_decode_one_wrapper(bs, false, 0)
        };

        sd.macroblock_vec[curr_mb_idx].sub_mb_type[mb_part_idx] = decode_sub_mb_type(res, sh);

        decoder_formatted_print("sub_mb_type", &res, 63);
        decoder_formatted_print(
            "sub_mb_type(fancy)",
            sd.macroblock_vec[curr_mb_idx].sub_mb_type[mb_part_idx],
            63,
        );
    }

    for mb_part_idx in 0..4 {
        if (sh.num_ref_idx_l0_active_minus1 > 0
            || sd.mb_field_decoding_flag[curr_mb_idx] != sh.field_pic_flag)
            && sd.macroblock_vec[curr_mb_idx].mb_type != MbType::P8x8ref0
            && sd.macroblock_vec[curr_mb_idx].sub_mb_type[mb_part_idx] != SubMbType::BDirect8x8
            && sd.macroblock_vec[curr_mb_idx].sub_mb_part_pred_mode(mb_part_idx)
                != MbPartPredMode::PredL1
        {
            let additional_inputs = vec![mb_part_idx];
            let res: u32 = if p.entropy_coding_mode_flag {
                cabac_decode(
                    "ref_idx_l0",
                    bs,
                    cabac_state,
                    curr_mb_idx,
                    sh,
                    sd,
                    vp,
                    0,
                    additional_inputs,
                ) as u32
            } else {
                let max_val: u32 =
                    if !sh.mbaff_frame_flag || !sd.mb_field_decoding_flag[curr_mb_idx] {
                        sh.num_ref_idx_l0_active_minus1
                    } else {
                        2 * sh.num_ref_idx_l0_active_minus1 + 1
                    };

                truncated_exp_golomb_decode(max_val, bs) as u32
            };

            sd.macroblock_vec[curr_mb_idx].ref_idx_l0[mb_part_idx] = res;
            decoder_formatted_print(
                "(sub) ref_idx_l0",
                &sd.macroblock_vec[curr_mb_idx].ref_idx_l0[mb_part_idx],
                63,
            );
        }
    }

    for mb_part_idx in 0..4 {
        if (sh.num_ref_idx_l1_active_minus1 > 0
            || sd.mb_field_decoding_flag[curr_mb_idx] != sh.field_pic_flag)
            && sd.macroblock_vec[curr_mb_idx].sub_mb_type[mb_part_idx] != SubMbType::BDirect8x8
            && sd.macroblock_vec[curr_mb_idx].sub_mb_part_pred_mode(mb_part_idx)
                != MbPartPredMode::PredL0
        {
            let additional_inputs = vec![mb_part_idx];
            let res: u32 = if p.entropy_coding_mode_flag {
                cabac_decode(
                    "ref_idx_l1",
                    bs,
                    cabac_state,
                    curr_mb_idx,
                    sh,
                    sd,
                    vp,
                    0,
                    additional_inputs,
                ) as u32
            } else {
                let max_val: u32 =
                    if !sh.mbaff_frame_flag || !sd.mb_field_decoding_flag[curr_mb_idx] {
                        sh.num_ref_idx_l1_active_minus1
                    } else {
                        2 * sh.num_ref_idx_l1_active_minus1 + 1
                    };
                truncated_exp_golomb_decode(max_val, bs) as u32
            };

            sd.macroblock_vec[curr_mb_idx].ref_idx_l1[mb_part_idx] = res;

            decoder_formatted_print(
                "(sub) ref_idx_l1",
                &sd.macroblock_vec[curr_mb_idx].ref_idx_l1[mb_part_idx],
                63,
            );
        }
    }

    for mb_part_idx in 0..4 {
        if sd.macroblock_vec[curr_mb_idx].sub_mb_type[mb_part_idx] != SubMbType::BDirect8x8
            && sd.macroblock_vec[curr_mb_idx].sub_mb_part_pred_mode(mb_part_idx)
                != MbPartPredMode::PredL1
        {
            for sub_mb_part_idx in 0..sd.macroblock_vec[curr_mb_idx].num_sub_mb_part(mb_part_idx) {
                for comp_idx in 0..2 {
                    let name = format!("mvd_l0_{}", comp_idx);
                    let additional_inputs = vec![mb_part_idx, sub_mb_part_idx];
                    let res: i32 = if p.entropy_coding_mode_flag {
                        cabac_decode(
                            name.as_str(),
                            bs,
                            cabac_state,
                            curr_mb_idx,
                            sh,
                            sd,
                            vp,
                            0,
                            additional_inputs,
                        )
                    } else {
                        exp_golomb_decode_one_wrapper(bs, true, 0)
                    };

                    sd.macroblock_vec[curr_mb_idx].mvd_l0[mb_part_idx][sub_mb_part_idx][comp_idx] =
                        res;

                    decoder_formatted_print(
                        format!("(sub) {}", name.as_str()).as_str(),
                        &sd.macroblock_vec[curr_mb_idx].mvd_l0[mb_part_idx][sub_mb_part_idx]
                            [comp_idx],
                        63,
                    );
                }
            }
        }
    }

    for mb_part_idx in 0..4 {
        if sd.macroblock_vec[curr_mb_idx].sub_mb_type[mb_part_idx] != SubMbType::BDirect8x8
            && sd.macroblock_vec[curr_mb_idx].sub_mb_part_pred_mode(mb_part_idx)
                != MbPartPredMode::PredL0
        {
            for sub_mb_part_idx in 0..sd.macroblock_vec[curr_mb_idx].num_sub_mb_part(mb_part_idx) {
                for comp_idx in 0..2 {
                    let name = format!("mvd_l1_{}", comp_idx);
                    let additional_inputs = vec![mb_part_idx, sub_mb_part_idx];
                    let res: i32 = if p.entropy_coding_mode_flag {
                        cabac_decode(
                            name.as_str(),
                            bs,
                            cabac_state,
                            curr_mb_idx,
                            sh,
                            sd,
                            vp,
                            0,
                            additional_inputs,
                        )
                    } else {
                        exp_golomb_decode_one_wrapper(bs, true, 0)
                    };

                    sd.macroblock_vec[curr_mb_idx].mvd_l1[mb_part_idx][sub_mb_part_idx][comp_idx] =
                        res;

                    decoder_formatted_print(
                        format!("(sub) {}", name.as_str()).as_str(),
                        &sd.macroblock_vec[curr_mb_idx].mvd_l1[mb_part_idx][sub_mb_part_idx]
                            [comp_idx],
                        63,
                    );
                }
            }
        }
    }
}

/// Follows Section 7.3.5.3
fn decode_residual(
    start_idx: usize,
    end_idx: usize,
    bs: &mut ByteStream,
    cabac_state: &mut CABACState,
    curr_mb_idx: usize,
    sh: &SliceHeader,
    sd: &mut SliceData,
    vp: &VideoParameters,
    p: &PicParameterSet,
) {
    let mut i16x16dclevel: Vec<i32> = Vec::new();
    let mut i16x16aclevel: Vec<Vec<i32>> = Vec::new();
    let mut level4x4: Vec<Vec<i32>> = Vec::new();
    let mut level8x8: Vec<Vec<i32>> = Vec::new();

    let mut i16x16dclevel_transform_block: TransformBlock = TransformBlock::new();
    let mut i16x16aclevel_transform_block: Vec<TransformBlock> = Vec::new();
    let mut level4x4_transform_block: Vec<TransformBlock> = Vec::new();
    let mut level8x8_transform_block: Vec<TransformBlock> = Vec::new();

    let mut ctx_block_cat: u8;
    let mut ctx_block_cat_offset: u8 = 0; // used in Luma decoding

    decode_residual_luma(
        &mut i16x16dclevel,
        &mut i16x16aclevel,
        &mut level4x4,
        &mut level8x8,
        start_idx,
        end_idx,
        bs,
        cabac_state,
        ctx_block_cat_offset,
        curr_mb_idx,
        sh,
        sd,
        vp,
        p,
        &mut i16x16dclevel_transform_block,
        &mut i16x16aclevel_transform_block,
        &mut level4x4_transform_block,
        &mut level8x8_transform_block,
    );

    // these are used in section 8.5 to reconstruct the image

    if vp.chroma_array_type == 1 || vp.chroma_array_type == 2 {
        for i_cb_cr in 0..2 {
            // DC residue
            sd.macroblock_vec[curr_mb_idx]
                .chroma_dc_level
                .push(Vec::new());
            sd.macroblock_vec[curr_mb_idx]
                .chroma_dc_level_transform_blocks
                .push(TransformBlock::new());

            if (sd.macroblock_vec[curr_mb_idx].coded_block_pattern_chroma & 3) > 0 && start_idx == 0
            {
                let mut res = sd.macroblock_vec[curr_mb_idx].chroma_dc_level[i_cb_cr].clone();
                let mut res2 = sd.macroblock_vec[curr_mb_idx].chroma_dc_level_transform_blocks
                    [i_cb_cr]
                    .clone();
                debug!(target: "decode","Chroma DC");

                ctx_block_cat = 3;
                let additional_inputs: Vec<usize> = vec![i_cb_cr];

                if p.entropy_coding_mode_flag {
                    decode_residual_block_cabac(
                        &mut res,
                        0,
                        4 * sd.macroblock_vec[curr_mb_idx].num_c8x8 - 1,
                        4 * sd.macroblock_vec[curr_mb_idx].num_c8x8,
                        bs,
                        cabac_state,
                        ctx_block_cat,
                        curr_mb_idx,
                        sh,
                        sd,
                        vp,
                        additional_inputs,
                        &mut res2,
                    );
                } else {
                    decode_residual_block_cavlc(
                        &mut res,
                        0,
                        4 * sd.macroblock_vec[curr_mb_idx].num_c8x8 - 1,
                        4 * sd.macroblock_vec[curr_mb_idx].num_c8x8,
                        bs,
                        ResidualMode::ChromaDCLevel,
                        curr_mb_idx,
                        sh,
                        sd,
                        vp,
                        &additional_inputs,
                        &mut res2,
                    );
                }

                sd.macroblock_vec[curr_mb_idx].chroma_dc_level[i_cb_cr] = res.clone();
                sd.macroblock_vec[curr_mb_idx].chroma_dc_level_transform_blocks[i_cb_cr] =
                    res2.clone();
            } else {
                for _ in 0..4 * sd.macroblock_vec[curr_mb_idx].num_c8x8 {
                    sd.macroblock_vec[curr_mb_idx].chroma_dc_level[i_cb_cr].push(0);
                }
            }
        }
        for i_cb_cr in 0..2 {
            // AC Residue
            sd.macroblock_vec[curr_mb_idx]
                .chroma_ac_level
                .push(Vec::new());
            sd.macroblock_vec[curr_mb_idx]
                .chroma_ac_level_transform_blocks
                .push(Vec::new());
            for i8x8 in 0..sd.macroblock_vec[curr_mb_idx].num_c8x8 {
                for i4x4 in 0..4 {
                    sd.macroblock_vec[curr_mb_idx].chroma_ac_level[i_cb_cr].push(Vec::new());
                    sd.macroblock_vec[curr_mb_idx].chroma_ac_level_transform_blocks[i_cb_cr]
                        .push(TransformBlock::new());
                    if sd.macroblock_vec[curr_mb_idx].coded_block_pattern_chroma & 2 > 0 {
                        let mut res = sd.macroblock_vec[curr_mb_idx].chroma_ac_level[i_cb_cr]
                            [(i8x8 * 4 + i4x4) as usize]
                            .clone();
                        let mut res2 = sd.macroblock_vec[curr_mb_idx]
                            .chroma_ac_level_transform_blocks[i_cb_cr]
                            [(i8x8 * 4 + i4x4) as usize]
                            .clone();
                        debug!(target: "decode","Chroma AC");
                        ctx_block_cat = 4;

                        let additional_inputs: Vec<usize> =
                            vec![(i8x8 * 4 + i4x4) as usize, i_cb_cr]; // first index is chroma4x4BlkIdx

                        if start_idx == 0 {
                            if p.entropy_coding_mode_flag {
                                decode_residual_block_cabac(
                                    &mut res,
                                    0,
                                    end_idx - 1,
                                    15,
                                    bs,
                                    cabac_state,
                                    ctx_block_cat,
                                    curr_mb_idx,
                                    sh,
                                    sd,
                                    vp,
                                    additional_inputs.clone(),
                                    &mut res2,
                                );
                            } else {
                                decode_residual_block_cavlc(
                                    &mut res,
                                    0,
                                    end_idx - 1,
                                    15,
                                    bs,
                                    ResidualMode::ChromaACLevel,
                                    curr_mb_idx,
                                    sh,
                                    sd,
                                    vp,
                                    &additional_inputs,
                                    &mut res2,
                                );
                            }
                        } else if p.entropy_coding_mode_flag {
                            decode_residual_block_cabac(
                                &mut res,
                                start_idx - 1,
                                end_idx - 1,
                                15,
                                bs,
                                cabac_state,
                                ctx_block_cat,
                                curr_mb_idx,
                                sh,
                                sd,
                                vp,
                                additional_inputs.clone(),
                                &mut res2,
                            );
                        } else {
                            decode_residual_block_cavlc(
                                &mut res,
                                start_idx - 1,
                                end_idx - 1,
                                15,
                                bs,
                                ResidualMode::ChromaACLevel,
                                curr_mb_idx,
                                sh,
                                sd,
                                vp,
                                &additional_inputs,
                                &mut res2,
                            );
                        }

                        sd.macroblock_vec[curr_mb_idx].chroma_ac_level[i_cb_cr]
                            [(i8x8 * 4 + i4x4) as usize] = res.clone();
                        sd.macroblock_vec[curr_mb_idx].chroma_ac_level_transform_blocks[i_cb_cr]
                            [(i8x8 * 4 + i4x4) as usize] = res2.clone();
                    } else {
                        for _ in 0..15 {
                            sd.macroblock_vec[curr_mb_idx].chroma_ac_level[i_cb_cr]
                                [(i8x8 * 4 + i4x4) as usize]
                                .push(0);
                        }
                    }
                }
            }
        }
    } else if vp.chroma_array_type == 3 {
        debug!(target: "decode","Chroma_array_type == 3");
        // First get the Cb values
        i16x16dclevel.clear();
        i16x16aclevel.clear();
        level4x4.clear();
        level8x8.clear();
        i16x16dclevel_transform_block = TransformBlock::new();
        i16x16aclevel_transform_block = Vec::new();
        level4x4_transform_block = Vec::new();
        level8x8_transform_block = Vec::new();

        debug!(target: "decode","Cb Components");
        ctx_block_cat_offset = 6;
        decode_residual_luma(
            &mut i16x16dclevel,
            &mut i16x16aclevel,
            &mut level4x4,
            &mut level8x8,
            start_idx,
            end_idx,
            bs,
            cabac_state,
            ctx_block_cat_offset,
            curr_mb_idx,
            sh,
            sd,
            vp,
            p,
            &mut i16x16dclevel_transform_block,
            &mut i16x16aclevel_transform_block,
            &mut level4x4_transform_block,
            &mut level8x8_transform_block,
        );

        // Now the Cr values
        i16x16dclevel.clear();
        i16x16aclevel.clear();
        level4x4.clear();
        level8x8.clear();
        i16x16dclevel_transform_block = TransformBlock::new();
        i16x16aclevel_transform_block = Vec::new();
        level4x4_transform_block = Vec::new();
        level8x8_transform_block = Vec::new();

        debug!(target: "decode","Cr Components");
        ctx_block_cat_offset = 10;
        decode_residual_luma(
            &mut i16x16dclevel,
            &mut i16x16aclevel,
            &mut level4x4,
            &mut level8x8,
            start_idx,
            end_idx,
            bs,
            cabac_state,
            ctx_block_cat_offset,
            curr_mb_idx,
            sh,
            sd,
            vp,
            p,
            &mut i16x16dclevel_transform_block,
            &mut i16x16aclevel_transform_block,
            &mut level4x4_transform_block,
            &mut level8x8_transform_block,
        );
    }
}

/// Follows Section 7.3.5.3.1
fn decode_residual_luma(
    i16x16dclevel: &mut Vec<i32>,
    i16x16aclevel: &mut Vec<Vec<i32>>,
    level4x4: &mut Vec<Vec<i32>>,
    level8x8: &mut Vec<Vec<i32>>,
    start_idx: usize,
    end_idx: usize,
    bs: &mut ByteStream,
    cabac_state: &mut CABACState,
    ctx_block_cat_offset: u8,
    curr_mb_idx: usize,
    sh: &SliceHeader,
    sd: &mut SliceData,
    vp: &VideoParameters,
    p: &PicParameterSet,
    i16x16dclevel_transform_block: &mut TransformBlock,
    i16x16aclevel_transform_block: &mut Vec<TransformBlock>,
    level4x4_transform_block: &mut Vec<TransformBlock>,
    level8x8_transform_block: &mut Vec<TransformBlock>,
) {
    debug!(target: "decode","Decoding Residual Luma components");
    let mut ctx_block_cat: u8; // Values are derived from Table 9-42

    if start_idx == 0
        && sd.macroblock_vec[curr_mb_idx].mb_part_pred_mode(0) == MbPartPredMode::Intra16x16
    {
        debug!(target: "decode","Luma DC");
        ctx_block_cat = ctx_block_cat_offset; // no modification
        let additional_inputs = vec![0];

        if p.entropy_coding_mode_flag {
            decode_residual_block_cabac(
                i16x16dclevel,
                0,
                15,
                16,
                bs,
                cabac_state,
                ctx_block_cat,
                curr_mb_idx,
                sh,
                sd,
                vp,
                additional_inputs.clone(),
                i16x16dclevel_transform_block,
            );
        } else {
            decode_residual_block_cavlc(
                i16x16dclevel,
                0,
                15,
                16,
                bs,
                ResidualMode::LumaLevel4x4,
                curr_mb_idx,
                sh,
                sd,
                vp,
                &additional_inputs,
                i16x16dclevel_transform_block,
            );
        }

        if ctx_block_cat_offset == 0 {
            sd.macroblock_vec[curr_mb_idx].intra_16x16_dc_level = i16x16dclevel.clone();
            sd.macroblock_vec[curr_mb_idx].intra_16x16_dc_level_transform_blocks =
                i16x16dclevel_transform_block.clone();
        } else if ctx_block_cat_offset == 6 {
            sd.macroblock_vec[curr_mb_idx].cb_intra_16x16_dc_level = i16x16dclevel.clone();
            sd.macroblock_vec[curr_mb_idx].cb_intra_16x16_dc_level_transform_blocks =
                i16x16dclevel_transform_block.clone();
        } else if ctx_block_cat_offset == 10 {
            sd.macroblock_vec[curr_mb_idx].cr_intra_16x16_dc_level = i16x16dclevel.clone();
            sd.macroblock_vec[curr_mb_idx].cr_intra_16x16_dc_level_transform_blocks =
                i16x16dclevel_transform_block.clone();
        }
    }
    for i8x8 in 0..4 {
        level8x8.push(Vec::new());
        level8x8_transform_block.push(TransformBlock::new());
        if !sd.macroblock_vec[curr_mb_idx].transform_size_8x8_flag || !p.entropy_coding_mode_flag {
            for i4x4 in 0..4 {
                i16x16aclevel.push(Vec::new());
                i16x16aclevel_transform_block.push(TransformBlock::new());
                level4x4.push(Vec::new());
                level4x4_transform_block.push(TransformBlock::new());

                let additional_inputs: Vec<usize> = vec![i8x8 * 4 + i4x4];

                if ((sd.macroblock_vec[curr_mb_idx].coded_block_pattern_luma & (1 << i8x8)) >> i8x8)
                    > 0
                {
                    if sd.macroblock_vec[curr_mb_idx].mb_part_pred_mode(0)
                        == MbPartPredMode::Intra16x16
                    {
                        debug!(target: "decode","Luma AC - i8x8 {} and i4x4 {}", i8x8, i4x4);
                        ctx_block_cat = 1 + ctx_block_cat_offset;
                        if start_idx == 0 {
                            if p.entropy_coding_mode_flag {
                                decode_residual_block_cabac(
                                    &mut i16x16aclevel[(i8x8 * 4 + i4x4) as usize],
                                    0,
                                    end_idx - 1,
                                    15,
                                    bs,
                                    cabac_state,
                                    ctx_block_cat,
                                    curr_mb_idx,
                                    sh,
                                    sd,
                                    vp,
                                    additional_inputs.clone(),
                                    &mut i16x16aclevel_transform_block[(i8x8 * 4 + i4x4) as usize],
                                );
                            } else {
                                decode_residual_block_cavlc(
                                    &mut i16x16aclevel[(i8x8 * 4 + i4x4) as usize],
                                    0,
                                    end_idx - 1,
                                    15,
                                    bs,
                                    ResidualMode::Intra16x16ACLevel,
                                    curr_mb_idx,
                                    sh,
                                    sd,
                                    vp,
                                    &additional_inputs,
                                    &mut i16x16aclevel_transform_block[(i8x8 * 4 + i4x4) as usize],
                                );
                            }
                        } else if p.entropy_coding_mode_flag {
                            decode_residual_block_cabac(
                                &mut i16x16aclevel[(i8x8 * 4 + i4x4) as usize],
                                start_idx - 1,
                                end_idx - 1,
                                15,
                                bs,
                                cabac_state,
                                ctx_block_cat,
                                curr_mb_idx,
                                sh,
                                sd,
                                vp,
                                additional_inputs.clone(),
                                &mut i16x16aclevel_transform_block[(i8x8 * 4 + i4x4) as usize],
                            );
                        } else {
                            decode_residual_block_cavlc(
                                &mut i16x16aclevel[(i8x8 * 4 + i4x4) as usize],
                                start_idx - 1,
                                end_idx - 1,
                                15,
                                bs,
                                ResidualMode::Intra16x16ACLevel,
                                curr_mb_idx,
                                sh,
                                sd,
                                vp,
                                &additional_inputs,
                                &mut i16x16aclevel_transform_block[(i8x8 * 4 + i4x4) as usize],
                            );
                        }
                        // To test, copy over the values
                        level4x4_transform_block[(i8x8 * 4 + i4x4) as usize] =
                            i16x16aclevel_transform_block[(i8x8 * 4 + i4x4) as usize].clone();
                    } else {
                        debug!(target: "decode","Luma4x4");
                        ctx_block_cat = 2 + ctx_block_cat_offset;
                        if p.entropy_coding_mode_flag {
                            decode_residual_block_cabac(
                                &mut level4x4[(i8x8 * 4 + i4x4) as usize],
                                start_idx,
                                end_idx,
                                16,
                                bs,
                                cabac_state,
                                ctx_block_cat,
                                curr_mb_idx,
                                sh,
                                sd,
                                vp,
                                additional_inputs.clone(),
                                &mut level4x4_transform_block[(i8x8 * 4 + i4x4) as usize],
                            );
                        } else {
                            decode_residual_block_cavlc(
                                &mut level4x4[(i8x8 * 4 + i4x4) as usize],
                                start_idx,
                                end_idx,
                                16,
                                bs,
                                ResidualMode::LumaLevel4x4,
                                curr_mb_idx,
                                sh,
                                sd,
                                vp,
                                &additional_inputs,
                                &mut level4x4_transform_block[(i8x8 * 4 + i4x4) as usize],
                            );
                        }
                        // To test, copy over the values
                        i16x16aclevel_transform_block[(i8x8 * 4 + i4x4) as usize] =
                            level4x4_transform_block[(i8x8 * 4 + i4x4) as usize].clone();
                    }
                } else if sd.macroblock_vec[curr_mb_idx].mb_part_pred_mode(0)
                    == MbPartPredMode::Intra16x16
                {
                    debug!(target: "decode","Luma AC - skipping");
                    for _ in 0..15 {
                        i16x16aclevel[i8x8 * 4 + i4x4].push(0);
                    }
                } else {
                    debug!(target: "decode","Luma4x4 - skipping");
                    for _ in 0..16 {
                        level4x4[i8x8 * 4 + i4x4].push(0);
                    }
                }

                if sd.macroblock_vec[curr_mb_idx].transform_size_8x8_flag
                    && !p.entropy_coding_mode_flag
                {
                    debug!(target: "decode","Luma8x8 - copying");
                    level8x8[i8x8] = vec![0; 64];
                    for i in 0..16 {
                        level8x8[i8x8][4 * i + i4x4] = level4x4[i8x8 * 4 + i4x4][i];
                    }
                }

                // overwrite with most up to date values
                if ctx_block_cat_offset == 0 {
                    sd.macroblock_vec[curr_mb_idx].intra_16x16_ac_level = i16x16aclevel.clone();
                    sd.macroblock_vec[curr_mb_idx].luma_level_4x4 = level4x4.clone();

                    sd.macroblock_vec[curr_mb_idx].intra_16x16_ac_level_transform_blocks =
                        i16x16aclevel_transform_block.clone();
                    sd.macroblock_vec[curr_mb_idx].luma_level_4x4_transform_blocks =
                        level4x4_transform_block.clone();
                } else if ctx_block_cat_offset == 6 {
                    sd.macroblock_vec[curr_mb_idx].cb_intra_16x16_ac_level = i16x16aclevel.clone();
                    sd.macroblock_vec[curr_mb_idx].cb_level_4x4 = level4x4.clone();

                    sd.macroblock_vec[curr_mb_idx].cb_intra_16x16_ac_level_transform_blocks =
                        i16x16aclevel_transform_block.clone();
                    sd.macroblock_vec[curr_mb_idx].cb_level_4x4_transform_blocks =
                        level4x4_transform_block.clone();
                } else if ctx_block_cat_offset == 10 {
                    sd.macroblock_vec[curr_mb_idx].cr_intra_16x16_ac_level = i16x16aclevel.clone();
                    sd.macroblock_vec[curr_mb_idx].cr_level_4x4 = level4x4.clone();

                    sd.macroblock_vec[curr_mb_idx].cr_intra_16x16_ac_level_transform_blocks =
                        i16x16aclevel_transform_block.clone();
                    sd.macroblock_vec[curr_mb_idx].cr_level_4x4_transform_blocks =
                        level4x4_transform_block.clone();
                }
            }
        } else if (sd.macroblock_vec[curr_mb_idx].coded_block_pattern_luma & (1 << i8x8)) >> i8x8
            > 0
        {
            // put in empty values
            for i4x4 in 0..4 {
                i16x16aclevel.push(Vec::new());
                i16x16aclevel_transform_block.push(TransformBlock::new());
                level4x4.push(Vec::new());
                level4x4_transform_block.push(TransformBlock::new());
                for _ in 0..15 {
                    i16x16aclevel[(i8x8 * 4 + i4x4) as usize].push(0);
                }
                for _ in 0..16 {
                    level4x4[i8x8 * 4 + i4x4].push(0);
                }
                if ctx_block_cat_offset == 0 {
                    sd.macroblock_vec[curr_mb_idx].intra_16x16_ac_level = i16x16aclevel.clone();
                    sd.macroblock_vec[curr_mb_idx].luma_level_4x4 = level4x4.clone();
                    sd.macroblock_vec[curr_mb_idx].intra_16x16_ac_level_transform_blocks =
                        i16x16aclevel_transform_block.clone();
                    sd.macroblock_vec[curr_mb_idx].luma_level_4x4_transform_blocks =
                        level4x4_transform_block.clone();
                } else if ctx_block_cat_offset == 6 {
                    sd.macroblock_vec[curr_mb_idx].cb_intra_16x16_ac_level = i16x16aclevel.clone();
                    sd.macroblock_vec[curr_mb_idx].cb_level_4x4 = level4x4.clone();
                    sd.macroblock_vec[curr_mb_idx].cb_intra_16x16_ac_level_transform_blocks =
                        i16x16aclevel_transform_block.clone();
                    sd.macroblock_vec[curr_mb_idx].cb_level_4x4_transform_blocks =
                        level4x4_transform_block.clone();
                } else if ctx_block_cat_offset == 10 {
                    sd.macroblock_vec[curr_mb_idx].cr_intra_16x16_ac_level = i16x16aclevel.clone();
                    sd.macroblock_vec[curr_mb_idx].cr_level_4x4 = level4x4.clone();
                    sd.macroblock_vec[curr_mb_idx].cr_intra_16x16_ac_level_transform_blocks =
                        i16x16aclevel_transform_block.clone();
                    sd.macroblock_vec[curr_mb_idx].cr_level_4x4_transform_blocks =
                        level4x4_transform_block.clone();
                }
            }
            debug!(target: "decode","Luma8x8");

            ctx_block_cat = 5;
            if ctx_block_cat_offset == 6 {
                ctx_block_cat += 4;
            } else if ctx_block_cat_offset == 10 {
                ctx_block_cat += 8;
            }

            let additional_inputs: Vec<usize> = vec![(i8x8) as usize];

            if p.entropy_coding_mode_flag {
                decode_residual_block_cabac(
                    &mut level8x8[i8x8],
                    4 * start_idx,
                    4 * end_idx + 3,
                    64,
                    bs,
                    cabac_state,
                    ctx_block_cat,
                    curr_mb_idx,
                    sh,
                    sd,
                    vp,
                    additional_inputs.clone(),
                    &mut level8x8_transform_block[i8x8],
                );
            } else {
                decode_residual_block_cavlc(
                    &mut level8x8[i8x8],
                    4 * start_idx,
                    4 * end_idx + 3,
                    64,
                    bs,
                    ResidualMode::LumaLevel4x4,
                    curr_mb_idx,
                    sh,
                    sd,
                    vp,
                    &additional_inputs,
                    &mut level8x8_transform_block[i8x8],
                );
            }
        } else {
            // insert empty values
            for i4x4 in 0..4 {
                i16x16aclevel.push(Vec::new());
                i16x16aclevel_transform_block.push(TransformBlock::new());
                level4x4.push(Vec::new());
                level4x4_transform_block.push(TransformBlock::new());
                for _ in 0..15 {
                    i16x16aclevel[(i8x8 * 4 + i4x4) as usize].push(0);
                }
                for _ in 0..16 {
                    level4x4[i8x8 * 4 + i4x4].push(0);
                }
                if ctx_block_cat_offset == 0 {
                    sd.macroblock_vec[curr_mb_idx].intra_16x16_ac_level = i16x16aclevel.clone();
                    sd.macroblock_vec[curr_mb_idx].luma_level_4x4 = level4x4.clone();
                    sd.macroblock_vec[curr_mb_idx].intra_16x16_ac_level_transform_blocks =
                        i16x16aclevel_transform_block.clone();
                    sd.macroblock_vec[curr_mb_idx].luma_level_4x4_transform_blocks =
                        level4x4_transform_block.clone();
                } else if ctx_block_cat_offset == 6 {
                    sd.macroblock_vec[curr_mb_idx].cb_intra_16x16_ac_level = i16x16aclevel.clone();
                    sd.macroblock_vec[curr_mb_idx].cb_level_4x4 = level4x4.clone();
                    sd.macroblock_vec[curr_mb_idx].cb_intra_16x16_ac_level_transform_blocks =
                        i16x16aclevel_transform_block.clone();
                    sd.macroblock_vec[curr_mb_idx].cb_level_4x4_transform_blocks =
                        level4x4_transform_block.clone();
                } else if ctx_block_cat_offset == 10 {
                    sd.macroblock_vec[curr_mb_idx].cr_intra_16x16_ac_level = i16x16aclevel.clone();
                    sd.macroblock_vec[curr_mb_idx].cr_level_4x4 = level4x4.clone();
                    sd.macroblock_vec[curr_mb_idx].cr_intra_16x16_ac_level_transform_blocks =
                        i16x16aclevel_transform_block.clone();
                    sd.macroblock_vec[curr_mb_idx].cr_level_4x4_transform_blocks =
                        level4x4_transform_block.clone();
                }
            }
            for _ in 0..64 {
                level8x8[i8x8].push(0);
            }
        }

        if ctx_block_cat_offset == 0 {
            sd.macroblock_vec[curr_mb_idx].luma_level_8x8 = level8x8.clone();
            sd.macroblock_vec[curr_mb_idx].luma_level_8x8_transform_blocks =
                level8x8_transform_block.clone();
        } else if ctx_block_cat_offset == 6 {
            sd.macroblock_vec[curr_mb_idx].cb_level_8x8 = level8x8.clone();
            sd.macroblock_vec[curr_mb_idx].cb_level_8x8_transform_blocks =
                level8x8_transform_block.clone();
        } else if ctx_block_cat_offset == 10 {
            sd.macroblock_vec[curr_mb_idx].cr_level_8x8 = level8x8.clone();
            sd.macroblock_vec[curr_mb_idx].cr_level_8x8_transform_blocks =
                level8x8_transform_block.clone();
        }
    }
}

/// Follows Section 7.3.5.3.2
fn decode_residual_block_cabac(
    coeff_level: &mut Vec<i32>,
    start_idx: usize,
    end_idx: usize,
    max_num_coeff: usize,
    bs: &mut ByteStream,
    cabac_state: &mut CABACState,
    ctx_block_cat: u8,
    curr_mb_idx: usize,
    sh: &SliceHeader,
    sd: &mut SliceData,
    vp: &VideoParameters,
    mut additional_inputs: Vec<usize>,
    mut cur_transform_block: &mut TransformBlock,
) {
    // additional inputs starts off with containing the BlkIdx, which is necessary for coded_block_flag
    if max_num_coeff != 64 || vp.chroma_array_type == 3 {
        let r = cabac_decode(
            "coded_block_flag",
            bs,
            cabac_state,
            curr_mb_idx,
            sh,
            sd,
            vp,
            ctx_block_cat,
            additional_inputs.clone(),
        );
        cur_transform_block.coded_block_flag = match r {
            1 => true,
            _ => false,
        };
    }
    while coeff_level.len() < max_num_coeff {
        coeff_level.push(0);
        cur_transform_block.significant_coeff_flag.push(false);
        cur_transform_block.last_significant_coeff_flag.push(false);
        cur_transform_block.coeff_abs_level_minus1.push(0);
        cur_transform_block.coeff_sign_flag.push(false);
    }

    // this is to get levelListIdx and NumC8x8, as dictated in section 9.3.3.1.3
    additional_inputs.clear();
    additional_inputs.push(0);
    additional_inputs.push(max_num_coeff / 4); // maxNumCoeff = NumC8x8*4 whenever we're parsing ChromaDCLevel

    if cur_transform_block.coded_block_flag {
        cur_transform_block.available = true;

        let mut num_coeff = end_idx + 1;
        let mut i = start_idx;
        while i < num_coeff - 1 {
            additional_inputs[0] = i; // this is the levelListIdx
            let res = cabac_decode(
                "significant_coeff_flag",
                bs,
                cabac_state,
                curr_mb_idx,
                sh,
                sd,
                vp,
                ctx_block_cat,
                additional_inputs.clone(),
            );
            cur_transform_block.significant_coeff_flag[i] = match res {
                1 => true,
                _ => false,
            };
            decoder_formatted_print("significant_coeff_flag", &res, 63);

            if cur_transform_block.significant_coeff_flag[i] {
                let res = cabac_decode(
                    "last_significant_coeff_flag",
                    bs,
                    cabac_state,
                    curr_mb_idx,
                    sh,
                    sd,
                    vp,
                    ctx_block_cat,
                    additional_inputs.clone(),
                );
                cur_transform_block.last_significant_coeff_flag[i] = match res {
                    1 => true,
                    _ => false,
                };
                decoder_formatted_print("last_significant_coeff_flag", &res, 63);

                if cur_transform_block.last_significant_coeff_flag[i] {
                    num_coeff = i + 1;
                }
            }
            i += 1;
        }

        // time to reset additional_input to have numDecodeAbsLevelEq1 and numDecodeAbsLevelGt1
        let mut num_decode_abs_level_eq_1 = 0; // according to reference implementations, this seems to be equal to 1?
        let mut num_decode_abs_level_gt_1 = 0;

        // for the first one clear it and set it to 1 and 0
        additional_inputs.clear();
        additional_inputs.push(0);
        additional_inputs.push(0);

        cur_transform_block.coeff_abs_level_minus1[num_coeff - 1] = cabac_decode(
            "coeff_abs_level_minus1",
            bs,
            cabac_state,
            curr_mb_idx,
            sh,
            sd,
            vp,
            ctx_block_cat,
            additional_inputs.clone(),
        ) as u32;

        // we do +1 because we compare the coeff value, not the coeff value minus one
        if cur_transform_block.coeff_abs_level_minus1[num_coeff - 1] + 1 == 1 {
            num_decode_abs_level_eq_1 += 1;
        } else if cur_transform_block.coeff_abs_level_minus1[num_coeff - 1] + 1 > 1 {
            num_decode_abs_level_gt_1 += 1;
        }
        let r = cabac_decode(
            "coeff_sign_flag",
            bs,
            cabac_state,
            curr_mb_idx,
            sh,
            sd,
            vp,
            ctx_block_cat,
            additional_inputs.clone(),
        );
        cur_transform_block.coeff_sign_flag[num_coeff - 1] = match r {
            1 => true,
            _ => false,
        };

        coeff_level[num_coeff - 1] =
            (cur_transform_block.coeff_abs_level_minus1[num_coeff - 1] as i32 + 1)
                * (1 - 2 * match cur_transform_block.coeff_sign_flag[num_coeff - 1] {
                    true => 1,
                    false => 0,
                });
        // only do this if there are more to parse
        if num_coeff > 1 {
            for i in (start_idx..=(num_coeff - 2)).rev() {
                additional_inputs[0] = num_decode_abs_level_eq_1;
                additional_inputs[1] = num_decode_abs_level_gt_1;

                if cur_transform_block.significant_coeff_flag[i] {
                    cur_transform_block.coeff_abs_level_minus1[i] = cabac_decode(
                        "coeff_abs_level_minus1",
                        bs,
                        cabac_state,
                        curr_mb_idx,
                        sh,
                        sd,
                        vp,
                        ctx_block_cat,
                        additional_inputs.clone(),
                    ) as u32;

                    // we do +1 because we compare the coeff value, not the coeff value minus one
                    if cur_transform_block.coeff_abs_level_minus1[i] + 1 == 1 {
                        num_decode_abs_level_eq_1 += 1;
                    } else if cur_transform_block.coeff_abs_level_minus1[i] + 1 > 1 {
                        num_decode_abs_level_gt_1 += 1;
                    }

                    let r = cabac_decode(
                        "coeff_sign_flag",
                        bs,
                        cabac_state,
                        curr_mb_idx,
                        sh,
                        sd,
                        vp,
                        ctx_block_cat,
                        additional_inputs.clone(),
                    );
                    cur_transform_block.coeff_sign_flag[i] = match r {
                        1 => true,
                        _ => false,
                    };

                    coeff_level[i] = (cur_transform_block.coeff_abs_level_minus1[i] as i32 + 1)
                        * (1 - 2 * match cur_transform_block.coeff_sign_flag[i] {
                            true => 1,
                            false => 0,
                        });
                }
            }
        }
        // output it in usage order vs decoding order
        for i in 0..num_coeff {
            if cur_transform_block.significant_coeff_flag[i] {
                decoder_formatted_print("coeff_level", coeff_level[i], 63);
            }
        }
    }
}

/// Follows Section 7.3.5.3.3
fn decode_residual_block_cavlc(
    coeff_level: &mut Vec<i32>,
    start_idx: usize,
    end_idx: usize,
    max_num_coeff: usize,
    bs: &mut ByteStream,
    residual_mode: ResidualMode,
    curr_mb_idx: usize,
    _sh: &SliceHeader,
    sd: &mut SliceData,
    vp: &VideoParameters,
    additional_inputs: &[usize],
    mut cur_transform_block: &mut TransformBlock,
) {
    let mut level_val: Vec<i32> = Vec::new();
    let mut run_val: Vec<usize> = Vec::new();
    let mut suffix_length: u32 = 0;
    let mut level_code: i32;
    let mut zeros_left;
    let mut i_cb_cr = 0;

    cur_transform_block.available = true;

    for _ in 0..max_num_coeff {
        coeff_level.push(0);
    }

    if additional_inputs.len() > 1 {
        i_cb_cr = additional_inputs[1];
    }

    let coeff_token = cavlc_decode_coeff_token(
        residual_mode,
        curr_mb_idx,
        sd,
        additional_inputs[0],
        i_cb_cr,
        bs,
        vp,
    );
    cur_transform_block.coeff_token = coeff_token.clone();

    decoder_formatted_print("coeff_token.n_c", coeff_token.n_c, 63);
    decoder_formatted_print("coeff_token.trailing_ones", coeff_token.trailing_ones, 63);
    decoder_formatted_print("coeff_token.total_coeff", coeff_token.total_coeff, 63);

    if coeff_token.total_coeff > 0 {
        if coeff_token.total_coeff > 10 && coeff_token.trailing_ones < 3 {
            suffix_length = 1;
        }

        debug!(target: "decode","------");
        for i in 0..coeff_token.total_coeff {
            // to ensure we have something at each ith position
            level_val.push(0);
            cur_transform_block.trailing_ones_sign_flag.push(false);
            cur_transform_block.level_prefix.push(0);
            cur_transform_block.level_suffix.push(0);

            if i < coeff_token.trailing_ones {
                let trailing_ones_sign_flag: i32 = bs.read_bits(1) as i32;
                cur_transform_block.trailing_ones_sign_flag[i] = match trailing_ones_sign_flag {
                    0 => false,
                    _ => true,
                };

                decoder_formatted_print("trailing_ones_sign_flag", trailing_ones_sign_flag, 63);
                level_val[i] = 1 - 2 * trailing_ones_sign_flag;
            } else {
                let level_prefix = cavlc_decode_level_prefix(bs);
                cur_transform_block.level_prefix[i] = level_prefix;

                decoder_formatted_print("level_prefix", level_prefix, 63);
                level_code = cmp::min(15, level_prefix as i32) << suffix_length;
                if suffix_length > 0 || level_prefix >= 14 {
                    let level_suffix = cavlc_decode_level_suffix(suffix_length, level_prefix, bs);
                    cur_transform_block.level_suffix[i] = level_suffix;

                    decoder_formatted_print("level_suffix", level_suffix, 63);
                    level_code += level_suffix as i32;
                }

                if level_prefix >= 15 && suffix_length == 0 {
                    level_code += 15;
                }
                if level_prefix >= 16 {
                    level_code += (1 << (level_prefix - 3)) - 4096;
                }
                if i == coeff_token.trailing_ones && coeff_token.trailing_ones < 3 {
                    level_code += 2;
                }
                if level_code % 2 == 0 {
                    level_val[i] = (level_code + 2) >> 1;
                } else {
                    level_val[i] = (-level_code - 1) >> 1;
                }
                if suffix_length == 0 {
                    suffix_length = 1;
                }
                if level_val[i].abs() > (3 << (suffix_length - 1)) && suffix_length < 6 {
                    suffix_length += 1;
                }
            }
            decoder_formatted_print("level_val", level_val[i], 63);
            debug!(target: "decode","------");
        }
        if coeff_token.total_coeff < (end_idx - start_idx + 1) {
            let total_zeros = cavlc_decode_total_zeros(coeff_token.total_coeff, max_num_coeff, bs);
            cur_transform_block.total_zeros = total_zeros;

            decoder_formatted_print("total_zeros", total_zeros, 63);
            zeros_left = total_zeros;
        } else {
            zeros_left = 0;
        }
        for i in 0..coeff_token.total_coeff - 1 {
            // to ensure we have something at each ith position
            run_val.push(0);
            cur_transform_block.run_before.push(0);

            if zeros_left > 0 {
                let run_before = cavlc_decode_run_before(zeros_left, bs);
                cur_transform_block.run_before[i] = run_before;

                decoder_formatted_print("run_before", run_before, 63);
                run_val[i] = run_before;
            } // else leave at 0
            zeros_left -= run_val[i]; // conformance specifies that this should always be greater than or equal to 0
        }
        run_val.push(zeros_left);
        let mut coeff_num = 0;
        for i in (0..=coeff_token.total_coeff - 1).rev() {
            if i == coeff_token.total_coeff - 1 {
                coeff_num += run_val[i]; // ignore the +1 for the first time around
            } else {
                coeff_num += run_val[i] + 1;
            }

            coeff_level[start_idx + coeff_num] = level_val[i];
            decoder_formatted_print(
                format!("coeff_level[{}]", start_idx + coeff_num).as_str(),
                coeff_level[start_idx + coeff_num],
                63,
            );
        }
    }
}
