//! Macroblock syntax element encoding.

use crate::common::data_structures::MacroBlock;
use crate::common::data_structures::MbPartPredMode;
use crate::common::data_structures::MbType;
use crate::common::data_structures::PicParameterSet;
use crate::common::data_structures::ResidualMode;
use crate::common::data_structures::SeqParameterSet;
use crate::common::data_structures::Slice;
use crate::common::data_structures::SubMbType;
use crate::common::data_structures::TransformBlock;
use crate::common::data_structures::VideoParameters;
use crate::common::helper::encoder_formatted_print;
use crate::encoder::cabac;
use crate::encoder::cavlc;
use log::debug;
use std::cmp;

/// Encode the Macroblock elements, including all residual information
pub fn encode_macroblock(
    bitstream_array: &mut Vec<u8>,
    mb: &MacroBlock,
    s: &Slice,
    sps: &SeqParameterSet,
    p: &PicParameterSet,
    vp: &VideoParameters,
    cs: &mut cabac::CABACState,
) {
    // when decoding with mb_skip_run we'll insert empty Macroblocks in the vector for simplicity
    // we can just return if that's the case
    if mb.mb_type == MbType::BSkip || mb.mb_type == MbType::PSkip {
        println!(
            "[WARNING] Trying to encode skip Macroblock type - {:?}",
            mb.mb_type
        );
        return;
    }

    if p.entropy_coding_mode_flag {
        cabac::cabac_encode_mb_type(
            mb.mb_type,
            &s.sh,
            bitstream_array,
            cs,
            s.sd.get_neighbor(mb.mb_idx, false, vp),
        );
    } else {
        cavlc::cavlc_encode_mb_type(mb.mb_type, s.sh.slice_type, bitstream_array);
    }
    if mb.mb_type == MbType::IPCM {
        // Need to align to byte first
        while bitstream_array.len() % 8 != 0 {
            debug!(target: "encode","IPCM aligning zero bit");
            // pcm_align_zero_bit
            bitstream_array.push(0);
        }

        // luma components
        for i in 0..256 {
            let mut pcm_sample_luma_i: Vec<u8> = Vec::new();

            for j in (0..(vp.bit_depth_y)).rev() {
                pcm_sample_luma_i.push(((mb.pcm_sample_luma[i] & (1 << j)) >> j) as u8);
            }

            encoder_formatted_print("pcm luma", &mb.pcm_sample_luma[i], 63);
            bitstream_array.append(&mut pcm_sample_luma_i);
        }

        // chroma components
        for i in 0..2 * vp.mb_width_c * vp.mb_height_c {
            let mut pcm_sample_chroma_i: Vec<u8> = Vec::new();

            for j in (0..(vp.bit_depth_c)).rev() {
                pcm_sample_chroma_i
                    .push(((mb.pcm_sample_chroma[i as usize] & (1 << j)) >> j) as u8);
            }

            encoder_formatted_print("pcm chroma", &mb.pcm_sample_chroma[i as usize], 63);
            bitstream_array.append(&mut pcm_sample_chroma_i);
        }

        if p.entropy_coding_mode_flag {
            // reset the CABAC state after reading the above bytes
            cs.cod_i_offset = 0;
            cs.cod_i_range = 510;
            cs.first_bit_flag = true;
            cs.bits_outstanding = 0;
        }
    } else {
        if mb.mb_type != MbType::INxN
            && mb.mb_part_pred_mode(0) != MbPartPredMode::Intra16x16
            && mb.num_mb_part() == 4
        {
            bitstream_array.append(&mut encode_sub_mb_pred(mb, s, vp, cs));

            // we can ignore setting the noSubMbPartSizeLessThan8x8Flag because
            // it's set by the decoder and not changed later
        } else {
            if p.transform_8x8_mode_flag && mb.mb_type == MbType::INxN {
                if p.entropy_coding_mode_flag {
                    cabac::cabac_encode_transform_size_8x8_flag(
                        mb.transform_size_8x8_flag,
                        &s.sh,
                        bitstream_array,
                        cs,
                        s.sd.get_neighbor(mb.mb_idx, false, vp),
                    );
                } else {
                    cavlc::cavlc_encode_transform_size_8x8_flag(
                        mb.transform_size_8x8_flag,
                        bitstream_array,
                    );
                }
            }

            // mb_pred
            bitstream_array.append(&mut encode_mb_pred(mb, s, vp, cs));
        }

        if mb.mb_part_pred_mode(0) != MbPartPredMode::Intra16x16 {
            if p.entropy_coding_mode_flag {
                cabac::cabac_encode_coded_block_pattern(
                    mb.coded_block_pattern,
                    &s.sh,
                    bitstream_array,
                    cs,
                    &s.sd,
                    vp,
                    mb.mb_idx,
                );
            } else {
                cavlc::cavlc_encode_coded_block_pattern(
                    mb.coded_block_pattern,
                    vp.chroma_array_type,
                    match mb.is_intra_non_mut() {
                        true => 0,
                        false => 1,
                    },
                    bitstream_array,
                );
            }

            if mb.coded_block_pattern_luma > 0
                && p.transform_8x8_mode_flag
                && mb.mb_type != MbType::INxN
                && mb.no_sub_mb_part_size_less_than_8x8_flag
                && (mb.mb_type != MbType::BDirect16x16 || sps.direct_8x8_inference_flag)
            {
                if p.entropy_coding_mode_flag {
                    cabac::cabac_encode_transform_size_8x8_flag(
                        mb.transform_size_8x8_flag,
                        &s.sh,
                        bitstream_array,
                        cs,
                        s.sd.get_neighbor(mb.mb_idx, false, vp),
                    );
                } else {
                    cavlc::cavlc_encode_transform_size_8x8_flag(
                        mb.transform_size_8x8_flag,
                        bitstream_array,
                    );
                }
            }
        }

        if mb.coded_block_pattern_luma > 0
            || mb.coded_block_pattern_chroma > 0
            || mb.mb_part_pred_mode(0) == MbPartPredMode::Intra16x16
        {
            // for bin_idx == 0, need to share previous macroblock if it exists
            let prev_mb: MacroBlock = s.sd.get_previous_macroblock(mb.mb_idx);

            if p.entropy_coding_mode_flag {
                cabac::cabac_encode_mb_qp_delta(
                    mb.mb_qp_delta,
                    &s.sh,
                    bitstream_array,
                    cs,
                    prev_mb,
                );
            } else {
                cavlc::cavlc_encode_mb_qp_delta(mb.mb_qp_delta, bitstream_array);
            }
            bitstream_array.append(&mut encode_residual(mb, vp, 0, 15, s, p, cs));
        }
    }
}

/// Encode MB prediction values
fn encode_mb_pred(
    mb: &MacroBlock,
    s: &Slice,
    vp: &VideoParameters,
    cs: &mut cabac::CABACState,
) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    let mppm = mb.mb_part_pred_mode(0);

    if mppm == MbPartPredMode::Intra4x4
        || mppm == MbPartPredMode::Intra8x8
        || mppm == MbPartPredMode::Intra16x16
    {
        // no Intra4x4 here
        if mppm == MbPartPredMode::Intra4x4 {
            for luma_4x4_blk_idx in 0..16 {
                if vp.entropy_coding_mode_flag {
                    cabac::cabac_encode_prev_intra4x4_pred_mode_flag(
                        mb.prev_intra4x4_pred_mode_flag[luma_4x4_blk_idx],
                        &s.sh,
                        &mut bitstream_array,
                        cs,
                    );
                } else {
                    cavlc::cavlc_encode_prev_intra4x4_pred_mode_flag(
                        mb.prev_intra4x4_pred_mode_flag[luma_4x4_blk_idx],
                        &mut bitstream_array,
                    );
                }
                if !mb.prev_intra4x4_pred_mode_flag[luma_4x4_blk_idx] {
                    if vp.entropy_coding_mode_flag {
                        cabac::cabac_encode_rem_intra4x4_pred_mode(
                            mb.rem_intra4x4_pred_mode[luma_4x4_blk_idx],
                            &s.sh,
                            &mut bitstream_array,
                            cs,
                        );
                    } else {
                        cavlc::cavlc_encode_rem_intra4x4_pred_mode(
                            mb.rem_intra4x4_pred_mode[luma_4x4_blk_idx],
                            &mut bitstream_array,
                        );
                    }
                }
            }
        }

        // Intra8x8
        if mppm == MbPartPredMode::Intra8x8 {
            for luma_8x8_blk_idx in 0..4 {
                if vp.entropy_coding_mode_flag {
                    cabac::cabac_encode_prev_intra8x8_pred_mode_flag(
                        mb.prev_intra8x8_pred_mode_flag[luma_8x8_blk_idx],
                        &s.sh,
                        &mut bitstream_array,
                        cs,
                    );
                } else {
                    cavlc::cavlc_encode_prev_intra8x8_pred_mode_flag(
                        mb.prev_intra8x8_pred_mode_flag[luma_8x8_blk_idx],
                        &mut bitstream_array,
                    );
                }
                if !mb.prev_intra8x8_pred_mode_flag[luma_8x8_blk_idx] {
                    if vp.entropy_coding_mode_flag {
                        cabac::cabac_encode_rem_intra8x8_pred_mode(
                            mb.rem_intra8x8_pred_mode[luma_8x8_blk_idx],
                            &s.sh,
                            &mut bitstream_array,
                            cs,
                        );
                    } else {
                        cavlc::cavlc_encode_rem_intra8x8_pred_mode(
                            mb.rem_intra8x8_pred_mode[luma_8x8_blk_idx],
                            &mut bitstream_array,
                        );
                    }
                }
            }
        }

        if vp.chroma_array_type == 1 || vp.chroma_array_type == 2 {
            if vp.entropy_coding_mode_flag {
                cabac::cabac_encode_intra_chroma_pred_mode(
                    mb.intra_chroma_pred_mode as u32,
                    &s.sh,
                    &mut bitstream_array,
                    cs,
                    s.sd.get_neighbor(mb.mb_idx, false, vp),
                );
            } else {
                cavlc::cavlc_encode_intra_chroma_pred_mode(
                    mb.intra_chroma_pred_mode,
                    &mut bitstream_array,
                );
            }
        }
    } else if mppm != MbPartPredMode::Direct {
        for mb_part_idx in 0..mb.num_mb_part() {
            if (s.sh.num_ref_idx_l0_active_minus1 > 0
                || s.sd.mb_field_decoding_flag[mb.mb_idx] != s.sh.field_pic_flag)
                && mb.mb_part_pred_mode(mb_part_idx) != MbPartPredMode::PredL1
            {
                if vp.entropy_coding_mode_flag {
                    cabac::cabac_encode_ref_idx(
                        mb.ref_idx_l0[mb_part_idx],
                        0,
                        mb_part_idx,
                        &s.sh,
                        &s.sd,
                        mb.mb_idx,
                        &mut bitstream_array,
                        cs,
                        &s.sd
                            .get_neighbor_partitions(mb.mb_idx, mb_part_idx, 0, vp, false),
                    );
                } else {
                    let max_val: u32 =
                        if !s.sh.mbaff_frame_flag || !s.sd.mb_field_decoding_flag[mb.mb_idx] {
                            s.sh.num_ref_idx_l0_active_minus1
                        } else {
                            2 * s.sh.num_ref_idx_l0_active_minus1 + 1
                        };
                    cavlc::cavlc_encode_ref_idx(
                        mb.ref_idx_l0[mb_part_idx],
                        max_val,
                        &mut bitstream_array,
                    );
                }
            }
        }

        for mb_part_idx in 0..mb.num_mb_part() {
            if (s.sh.num_ref_idx_l1_active_minus1 > 0
                || s.sd.mb_field_decoding_flag[mb.mb_idx] != s.sh.field_pic_flag)
                && mb.mb_part_pred_mode(mb_part_idx) != MbPartPredMode::PredL0
            {
                if vp.entropy_coding_mode_flag {
                    cabac::cabac_encode_ref_idx(
                        mb.ref_idx_l1[mb_part_idx],
                        1,
                        mb_part_idx,
                        &s.sh,
                        &s.sd,
                        mb.mb_idx,
                        &mut bitstream_array,
                        cs,
                        &s.sd
                            .get_neighbor_partitions(mb.mb_idx, mb_part_idx, 0, vp, false),
                    );
                } else {
                    let max_val: u32 =
                        if !s.sh.mbaff_frame_flag || !s.sd.mb_field_decoding_flag[mb.mb_idx] {
                            s.sh.num_ref_idx_l1_active_minus1
                        } else {
                            2 * s.sh.num_ref_idx_l1_active_minus1 + 1
                        };
                    cavlc::cavlc_encode_ref_idx(
                        mb.ref_idx_l1[mb_part_idx],
                        max_val,
                        &mut bitstream_array,
                    );
                }
            }
        }

        for mb_part_idx in 0..mb.num_mb_part() {
            if mb.mb_part_pred_mode(mb_part_idx) != MbPartPredMode::PredL1 {
                for comp_idx in 0..2 {
                    if vp.entropy_coding_mode_flag {
                        cabac::cabac_encode_mvd(
                            mb.mvd_l0[mb_part_idx][0][comp_idx],
                            0,
                            mb_part_idx,
                            0,
                            comp_idx,
                            &s.sh,
                            &s.sd,
                            mb.mb_idx,
                            &mut bitstream_array,
                            cs,
                            &s.sd
                                .get_neighbor_partitions(mb.mb_idx, mb_part_idx, 0, vp, false),
                        );
                    } else {
                        cavlc::cavlc_encode_mvd(
                            mb.mvd_l0[mb_part_idx][0][comp_idx],
                            &mut bitstream_array,
                        );
                    }
                }
            }
        }

        for mb_part_idx in 0..mb.num_mb_part() {
            if mb.mb_part_pred_mode(mb_part_idx) != MbPartPredMode::PredL0 {
                for comp_idx in 0..2 {
                    if vp.entropy_coding_mode_flag {
                        cabac::cabac_encode_mvd(
                            mb.mvd_l1[mb_part_idx][0][comp_idx],
                            1,
                            mb_part_idx,
                            0,
                            comp_idx,
                            &s.sh,
                            &s.sd,
                            mb.mb_idx,
                            &mut bitstream_array,
                            cs,
                            &s.sd
                                .get_neighbor_partitions(mb.mb_idx, mb_part_idx, 0, vp, false),
                        );
                    } else {
                        cavlc::cavlc_encode_mvd(
                            mb.mvd_l1[mb_part_idx][0][comp_idx],
                            &mut bitstream_array,
                        );
                    }
                }
            }
        }
    }

    bitstream_array
}

/// Encode subMB prediction values
fn encode_sub_mb_pred(
    mb: &MacroBlock,
    s: &Slice,
    vp: &VideoParameters,
    cs: &mut cabac::CABACState,
) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    for mb_part_idx in 0..4 {
        if vp.entropy_coding_mode_flag {
            cabac::cabac_encode_sub_mb_type(
                mb.sub_mb_type[mb_part_idx],
                &s.sh,
                &mut bitstream_array,
                cs,
            );
        } else {
            cavlc::cavlc_encode_sub_mb_type(
                mb.sub_mb_type[mb_part_idx],
                s.sh.slice_type,
                &mut bitstream_array,
            );
        }
    }

    for mb_part_idx in 0..4 {
        if (s.sh.num_ref_idx_l0_active_minus1 > 0
            || s.sd.mb_field_decoding_flag[mb.mb_idx] != s.sh.field_pic_flag)
            && mb.mb_type != MbType::P8x8ref0
            && mb.sub_mb_type[mb_part_idx] != SubMbType::BDirect8x8
            && mb.sub_mb_part_pred_mode(mb_part_idx) != MbPartPredMode::PredL1
        {
            if vp.entropy_coding_mode_flag {
                cabac::cabac_encode_ref_idx(
                    mb.ref_idx_l0[mb_part_idx],
                    0,
                    mb_part_idx,
                    &s.sh,
                    &s.sd,
                    mb.mb_idx,
                    &mut bitstream_array,
                    cs,
                    &s.sd
                        .get_neighbor_partitions(mb.mb_idx, mb_part_idx, 0, vp, false),
                );
            } else {
                let max_val: u32 =
                    if !s.sh.mbaff_frame_flag || !s.sd.mb_field_decoding_flag[mb.mb_idx] {
                        s.sh.num_ref_idx_l0_active_minus1
                    } else {
                        2 * s.sh.num_ref_idx_l0_active_minus1 + 1
                    };
                cavlc::cavlc_encode_ref_idx(
                    mb.ref_idx_l0[mb_part_idx],
                    max_val,
                    &mut bitstream_array,
                );
            }
        }
    }

    for mb_part_idx in 0..4 {
        if (s.sh.num_ref_idx_l1_active_minus1 > 0
            || s.sd.mb_field_decoding_flag[mb.mb_idx] != s.sh.field_pic_flag)
            && mb.sub_mb_type[mb_part_idx] != SubMbType::BDirect8x8
            && mb.sub_mb_part_pred_mode(mb_part_idx) != MbPartPredMode::PredL0
        {
            if vp.entropy_coding_mode_flag {
                cabac::cabac_encode_ref_idx(
                    mb.ref_idx_l1[mb_part_idx],
                    1,
                    mb_part_idx,
                    &s.sh,
                    &s.sd,
                    mb.mb_idx,
                    &mut bitstream_array,
                    cs,
                    &s.sd
                        .get_neighbor_partitions(mb.mb_idx, mb_part_idx, 0, vp, false),
                );
            } else {
                let max_val: u32 =
                    if !s.sh.mbaff_frame_flag || !s.sd.mb_field_decoding_flag[mb.mb_idx] {
                        s.sh.num_ref_idx_l1_active_minus1
                    } else {
                        2 * s.sh.num_ref_idx_l1_active_minus1 + 1
                    };
                cavlc::cavlc_encode_ref_idx(
                    mb.ref_idx_l1[mb_part_idx],
                    max_val,
                    &mut bitstream_array,
                );
            }
        }
    }

    for mb_part_idx in 0..4 {
        if mb.sub_mb_type[mb_part_idx] != SubMbType::BDirect8x8
            && mb.sub_mb_part_pred_mode(mb_part_idx) != MbPartPredMode::PredL1
        {
            for sub_mb_part_idx in 0..mb.num_sub_mb_part(mb_part_idx) {
                for comp_idx in 0..2 {
                    if vp.entropy_coding_mode_flag {
                        cabac::cabac_encode_mvd(
                            mb.mvd_l0[mb_part_idx][sub_mb_part_idx][comp_idx],
                            0,
                            mb_part_idx,
                            sub_mb_part_idx,
                            comp_idx,
                            &s.sh,
                            &s.sd,
                            mb.mb_idx,
                            &mut bitstream_array,
                            cs,
                            &s.sd.get_neighbor_partitions(
                                mb.mb_idx,
                                mb_part_idx,
                                sub_mb_part_idx,
                                vp,
                                false,
                            ),
                        );
                    } else {
                        cavlc::cavlc_encode_mvd(
                            mb.mvd_l0[mb_part_idx][sub_mb_part_idx][comp_idx],
                            &mut bitstream_array,
                        );
                    }
                }
            }
        }
    }

    for mb_part_idx in 0..4 {
        if mb.sub_mb_type[mb_part_idx] != SubMbType::BDirect8x8
            && mb.sub_mb_part_pred_mode(mb_part_idx) != MbPartPredMode::PredL0
        {
            for sub_mb_part_idx in 0..mb.num_sub_mb_part(mb_part_idx) {
                for comp_idx in 0..2 {
                    if vp.entropy_coding_mode_flag {
                        cabac::cabac_encode_mvd(
                            mb.mvd_l1[mb_part_idx][sub_mb_part_idx][comp_idx],
                            1,
                            mb_part_idx,
                            sub_mb_part_idx,
                            comp_idx,
                            &s.sh,
                            &s.sd,
                            mb.mb_idx,
                            &mut bitstream_array,
                            cs,
                            &s.sd.get_neighbor_partitions(
                                mb.mb_idx,
                                mb_part_idx,
                                sub_mb_part_idx,
                                vp,
                                false,
                            ),
                        );
                    } else {
                        cavlc::cavlc_encode_mvd(
                            mb.mvd_l1[mb_part_idx][sub_mb_part_idx][comp_idx],
                            &mut bitstream_array,
                        );
                    }
                }
            }
        }
    }

    bitstream_array
}

/// Encode MacroBlock residual information
///
/// NOTE: ctx_block_cat values found in table 9-42
fn encode_residual(
    mb: &MacroBlock,
    vp: &VideoParameters,
    start_idx: usize,
    end_idx: usize,
    s: &Slice,
    p: &PicParameterSet,
    cs: &mut cabac::CABACState,
) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();
    let mut ctx_block_cat_offset: u8 = 0; // used in Luma encoding

    bitstream_array.append(&mut encode_residual_luma(
        mb,
        mb.intra_16x16_dc_level_transform_blocks.clone(),
        mb.intra_16x16_ac_level_transform_blocks.clone(),
        mb.luma_level_4x4_transform_blocks.clone(),
        mb.luma_level_8x8_transform_blocks.clone(),
        ctx_block_cat_offset,
        start_idx,
        end_idx,
        vp,
        s,
        p,
        cs,
    ));

    let mut t_vec = vec![0u8; 4];
    t_vec.extend(&bitstream_array);
    if vp.chroma_array_type == 1 || vp.chroma_array_type == 2 {
        for i_cb_cr in 0..2 {
            if (mb.coded_block_pattern_chroma & 3) > 0 && start_idx == 0 {
                // chroma DC residual present
                let ctx_block_cat = 3;
                debug!(target: "encode", "Chroma DC");

                let additional_inputs: Vec<usize> = vec![i_cb_cr];
                if p.entropy_coding_mode_flag {
                    bitstream_array.extend(encode_residual_block_cabac(
                        mb,
                        mb.chroma_dc_level_transform_blocks[i_cb_cr].clone(),
                        ctx_block_cat,
                        0,
                        4 * mb.num_c8x8 - 1,
                        4 * mb.num_c8x8,
                        vp,
                        s,
                        cs,
                        &additional_inputs,
                    ));
                } else {
                    bitstream_array.extend(encode_residual_block_cavlc(
                        mb,
                        mb.chroma_dc_level_transform_blocks[i_cb_cr].clone(),
                        ResidualMode::ChromaDCLevel,
                        0,
                        4 * mb.num_c8x8 - 1,
                        4 * mb.num_c8x8,
                        vp,
                        s,
                        &additional_inputs,
                    ));
                }
            }
        }
        for i_cb_cr in 0..2 {
            for i_8x8 in 0..mb.num_c8x8 {
                for i_4x4 in 0..4 {
                    if mb.coded_block_pattern_chroma & 2 > 1 {
                        // chroma AC residual present
                        let ctx_block_cat = 4;
                        debug!(target: "encode","Chroma AC");
                        let additional_inputs = [i_8x8 * 4 + i_4x4, i_cb_cr]; //chroma4x4BlkIdx

                        if start_idx == 0 {
                            if p.entropy_coding_mode_flag {
                                bitstream_array.append(&mut encode_residual_block_cabac(
                                    mb,
                                    mb.chroma_ac_level_transform_blocks[i_cb_cr][i_8x8 * 4 + i_4x4]
                                        .clone(),
                                    ctx_block_cat,
                                    0,
                                    end_idx - 1,
                                    15,
                                    vp,
                                    s,
                                    cs,
                                    &additional_inputs,
                                ));
                            } else {
                                bitstream_array.append(&mut encode_residual_block_cavlc(
                                    mb,
                                    mb.chroma_ac_level_transform_blocks[i_cb_cr][i_8x8 * 4 + i_4x4]
                                        .clone(),
                                    ResidualMode::ChromaACLevel,
                                    0,
                                    end_idx - 1,
                                    15,
                                    vp,
                                    s,
                                    &additional_inputs,
                                ));
                            }
                        } else if p.entropy_coding_mode_flag {
                            bitstream_array.append(&mut encode_residual_block_cabac(
                                mb,
                                mb.chroma_ac_level_transform_blocks[i_cb_cr][i_8x8 * 4 + i_4x4]
                                    .clone(),
                                ctx_block_cat,
                                start_idx - 1,
                                end_idx - 1,
                                15,
                                vp,
                                s,
                                cs,
                                &additional_inputs,
                            ));
                        } else {
                            bitstream_array.append(&mut encode_residual_block_cavlc(
                                mb,
                                mb.chroma_ac_level_transform_blocks[i_cb_cr][i_8x8 * 4 + i_4x4]
                                    .clone(),
                                ResidualMode::ChromaACLevel,
                                start_idx - 1,
                                end_idx - 1,
                                15,
                                vp,
                                s,
                                &additional_inputs,
                            ));
                        }
                    }
                }
            }
        }
    } else if vp.chroma_array_type == 3 {
        // Cb values
        ctx_block_cat_offset = 6;
        bitstream_array.append(&mut encode_residual_luma(
            mb,
            mb.cb_intra_16x16_dc_level_transform_blocks.clone(),
            mb.cb_intra_16x16_ac_level_transform_blocks.clone(),
            mb.cb_level_4x4_transform_blocks.clone(),
            mb.cb_level_8x8_transform_blocks.clone(),
            ctx_block_cat_offset,
            start_idx,
            end_idx,
            vp,
            s,
            p,
            cs,
        ));

        // Cr Values
        ctx_block_cat_offset = 10;
        bitstream_array.append(&mut encode_residual_luma(
            mb,
            mb.cr_intra_16x16_dc_level_transform_blocks.clone(),
            mb.cr_intra_16x16_ac_level_transform_blocks.clone(),
            mb.cr_level_4x4_transform_blocks.clone(),
            mb.cr_level_8x8_transform_blocks.clone(),
            ctx_block_cat_offset,
            start_idx,
            end_idx,
            vp,
            s,
            p,
            cs,
        ));
    }

    bitstream_array
}

/// Encode residual luma components
fn encode_residual_luma(
    mb: &MacroBlock,
    i16x16_dc_level: TransformBlock,
    i16x16_ac_level: Vec<TransformBlock>,
    level_4x4: Vec<TransformBlock>,
    level_8x8: Vec<TransformBlock>,
    ctx_block_cat_offset: u8,
    start_idx: usize,
    end_idx: usize,
    vp: &VideoParameters,
    s: &Slice,
    p: &PicParameterSet,
    cs: &mut cabac::CABACState,
) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    if start_idx == 0 && mb.mb_part_pred_mode(0) == MbPartPredMode::Intra16x16 {
        debug!(target: "encode","Intra16x16 Pred Mode && DC levels");
        let ctx_block_cat = ctx_block_cat_offset; // no offset
        let additional_inputs = vec![0];
        if p.entropy_coding_mode_flag {
            bitstream_array.extend(encode_residual_block_cabac(
                mb,
                i16x16_dc_level,
                ctx_block_cat,
                0,
                15,
                16,
                vp,
                s,
                cs,
                &additional_inputs,
            ));
        } else {
            bitstream_array.extend(encode_residual_block_cavlc(
                mb,
                i16x16_dc_level,
                ResidualMode::Intra16x16DCLevel,
                0,
                15,
                16,
                vp,
                s,
                &additional_inputs,
            ));
        }
    }
    for i_8x8 in 0..4 {
        // VLC will always go here, so no need to CAVLC-case the else
        if !mb.transform_size_8x8_flag || !p.entropy_coding_mode_flag {
            for i_4x4 in 0..4 {
                let additional_inputs = [i_8x8 * 4 + i_4x4];

                if mb.coded_block_pattern_luma & (1 << i_8x8) > 0 {
                    if mb.mb_part_pred_mode(0) == MbPartPredMode::Intra16x16 {
                        debug!(target: "encode","Intra16x16 Pred Mode && AC levels");

                        let ctx_block_cat = 1 + ctx_block_cat_offset;

                        if start_idx == 0 {
                            if p.entropy_coding_mode_flag {
                                bitstream_array.extend(encode_residual_block_cabac(
                                    mb,
                                    i16x16_ac_level[i_8x8 * 4 + i_4x4].clone(),
                                    ctx_block_cat,
                                    0,
                                    end_idx - 1,
                                    15,
                                    vp,
                                    s,
                                    cs,
                                    &additional_inputs,
                                ));
                            } else {
                                bitstream_array.extend(encode_residual_block_cavlc(
                                    mb,
                                    i16x16_ac_level[i_8x8 * 4 + i_4x4].clone(),
                                    ResidualMode::Intra16x16ACLevel,
                                    0,
                                    end_idx - 1,
                                    15,
                                    vp,
                                    s,
                                    &additional_inputs,
                                ));
                            }
                        } else if p.entropy_coding_mode_flag {
                            bitstream_array.extend(encode_residual_block_cabac(
                                mb,
                                i16x16_ac_level[i_8x8 * 4 + i_4x4].clone(),
                                ctx_block_cat,
                                start_idx - 1,
                                end_idx - 1,
                                15,
                                vp,
                                s,
                                cs,
                                &additional_inputs,
                            ));
                        } else {
                            bitstream_array.extend(encode_residual_block_cavlc(
                                mb,
                                i16x16_ac_level[i_8x8 * 4 + i_4x4].clone(),
                                ResidualMode::Intra16x16ACLevel,
                                start_idx - 1,
                                end_idx - 1,
                                15,
                                vp,
                                s,
                                &additional_inputs,
                            ));
                        }
                    } else {
                        let ctx_block_cat = 2 + ctx_block_cat_offset;

                        debug!(target: "encode","Level4x4");
                        if p.entropy_coding_mode_flag {
                            bitstream_array.extend(encode_residual_block_cabac(
                                mb,
                                level_4x4[i_8x8 * 4 + i_4x4].clone(),
                                ctx_block_cat,
                                start_idx,
                                end_idx,
                                16,
                                vp,
                                s,
                                cs,
                                &additional_inputs,
                            ));
                        } else {
                            bitstream_array.extend(encode_residual_block_cavlc(
                                mb,
                                level_4x4[i_8x8 * 4 + i_4x4].clone(),
                                ResidualMode::LumaLevel4x4,
                                start_idx,
                                end_idx,
                                16,
                                vp,
                                s,
                                &additional_inputs,
                            ));
                        }
                    }
                }
            }
        } else if mb.coded_block_pattern_luma & (1 << i_8x8) > 0 {
            let ctx_block_cat = 5 + match ctx_block_cat_offset {
                6 => 4,
                10 => 8,
                _ => 0,
            };

            debug!(target: "encode","Level8x8");
            let additional_inputs = vec![i_8x8];
            bitstream_array.extend(encode_residual_block_cabac(
                mb,
                level_8x8[i_8x8].clone(),
                ctx_block_cat,
                4 * start_idx,
                4 * end_idx + 3,
                64,
                vp,
                s,
                cs,
                &additional_inputs,
            ));
        }
    }

    bitstream_array
}

/// CABAC encoding of residual elements
fn encode_residual_block_cabac(
    mb: &MacroBlock,
    tb: TransformBlock,
    ctx_block_cat: u8,
    start_idx: usize,
    end_idx: usize,
    max_num_coeff: usize,
    vp: &VideoParameters,
    s: &Slice,
    cs: &mut cabac::CABACState,
    additional_inputs: &[usize],
) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    if max_num_coeff != 64 || vp.chroma_array_type == 3 {
        cabac::cabac_encode_coded_block_flag(
            tb.coded_block_flag,
            ctx_block_cat,
            &s.sh,
            &mut bitstream_array,
            cs,
            &s.sd,
            vp,
            mb.mb_idx,
            additional_inputs,
        );
    }

    // this is to get levelListIdx and NumC8x8, as dictated in section 9.3.3.1.3
    let mut additional_inputs: Vec<usize> = Vec::new();
    additional_inputs.push(0);
    additional_inputs.push(max_num_coeff / 4); // maxNumCoeff = NumC8x8*4 whenever we're parsing ChromaDCLevel

    if tb.coded_block_flag {
        let mut num_coeff = end_idx + 1;
        let mut i = start_idx;
        while i < num_coeff - 1 {
            additional_inputs[0] = i; // this is the levelListIdx
            cabac::cabac_encode_significant_coeff_flag(
                tb.significant_coeff_flag[i],
                ctx_block_cat,
                &s.sh,
                &mut bitstream_array,
                cs,
                &additional_inputs,
                s.sd.mb_field_decoding_flag[mb.mb_idx],
            );
            if tb.significant_coeff_flag[i] {
                cabac::cabac_encode_last_significant_coeff_flag(
                    tb.last_significant_coeff_flag[i],
                    ctx_block_cat,
                    &s.sh,
                    &mut bitstream_array,
                    cs,
                    &additional_inputs,
                    s.sd.mb_field_decoding_flag[mb.mb_idx],
                );
                if tb.last_significant_coeff_flag[i] {
                    num_coeff = i + 1;
                }
            }
            i += 1;
        }

        // time to reset additional_input to have numDecodeAbsLevelEq1 and numDecodeAbsLevelGt1
        let mut num_decode_abs_level_eq_1 = 0; // according to reference implementations, this seems to be equal to 1?
        let mut num_decode_abs_level_gt_1 = 0;

        // for the first one clear it and set it to
        additional_inputs.clear();
        additional_inputs.push(0);
        additional_inputs.push(0);
        cabac::cabac_encode_coeff_abs_level_minus1(
            tb.coeff_abs_level_minus1[num_coeff - 1],
            ctx_block_cat,
            &s.sh,
            &mut bitstream_array,
            cs,
            &additional_inputs,
        );
        cabac::cabac_encode_coeff_sign_flag(
            tb.coeff_sign_flag[num_coeff - 1],
            &mut bitstream_array,
            cs,
        );

        // we do +1 because we compare the coeff value, not the coeff value minus one
        if tb.coeff_abs_level_minus1[num_coeff - 1] + 1 == 1 {
            num_decode_abs_level_eq_1 += 1;
        } else if tb.coeff_abs_level_minus1[num_coeff - 1] + 1 > 1 {
            num_decode_abs_level_gt_1 += 1;
        }

        let coeff_level = (tb.coeff_abs_level_minus1[num_coeff - 1] + 1) as i32
            * match tb.coeff_sign_flag[num_coeff - 1] {
                true => -1i32,
                false => 1i32,
            };
        encoder_formatted_print("coeff_level", coeff_level, 63);

        if num_coeff > 1 {
            for i in (start_idx..=num_coeff - 2).rev() {
                additional_inputs[0] = num_decode_abs_level_eq_1;
                additional_inputs[1] = num_decode_abs_level_gt_1;
                if tb.significant_coeff_flag[i] {
                    cabac::cabac_encode_coeff_abs_level_minus1(
                        tb.coeff_abs_level_minus1[i],
                        ctx_block_cat,
                        &s.sh,
                        &mut bitstream_array,
                        cs,
                        &additional_inputs,
                    );
                    cabac::cabac_encode_coeff_sign_flag(
                        tb.coeff_sign_flag[i],
                        &mut bitstream_array,
                        cs,
                    );

                    // we do +1 because we compare the coeff value, not the coeff value minus one
                    if tb.coeff_abs_level_minus1[i] + 1 == 1 {
                        num_decode_abs_level_eq_1 += 1;
                    } else if tb.coeff_abs_level_minus1[i] + 1 > 1 {
                        num_decode_abs_level_gt_1 += 1;
                    }

                    let coeff_level = (tb.coeff_abs_level_minus1[i] + 1) as i32
                        * match tb.coeff_sign_flag[i] {
                            true => -1i32,
                            false => 1i32,
                        };
                    encoder_formatted_print("coeff_level", coeff_level, 63);
                }
            }
        }
    }

    bitstream_array
}

/// CAVLC encoding of residual elements
fn encode_residual_block_cavlc(
    mb: &MacroBlock,
    tb: TransformBlock,
    residual_mode: ResidualMode,
    start_idx: usize,
    end_idx: usize,
    max_num_coeff: usize,
    vp: &VideoParameters,
    s: &Slice,
    additional_inputs: &[usize],
) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    let mut i_cb_cr = 0;

    if additional_inputs.len() > 1 {
        i_cb_cr = additional_inputs[1];
    }

    cavlc::cavlc_encode_coeff_token(
        &tb.coeff_token,
        additional_inputs[0],
        i_cb_cr,
        residual_mode,
        mb.mb_idx,
        &s.sd,
        vp,
        &mut bitstream_array,
    );

    let mut suffix_length = 0;
    let mut level_code: i32;

    if tb.coeff_token.total_coeff > 0 {
        if tb.coeff_token.total_coeff > 10 && tb.coeff_token.trailing_ones < 3 {
            suffix_length = 1;
        }

        for i in 0..tb.coeff_token.total_coeff {
            if i < tb.coeff_token.trailing_ones {
                cavlc::cavlc_encode_trailing_ones_sign_flag(
                    tb.trailing_ones_sign_flag[i],
                    &mut bitstream_array,
                );
            } else {
                cavlc::cavlc_encode_level_prefix(tb.level_prefix[i], &mut bitstream_array);
                level_code = cmp::min(15, tb.level_prefix[i] as i32) << suffix_length;
                if suffix_length > 0 || tb.level_prefix[i] >= 14 {
                    cavlc::cavlc_encode_level_suffix(
                        tb.level_suffix[i],
                        suffix_length,
                        tb.level_prefix[i],
                        &mut bitstream_array,
                    );
                }
                if suffix_length == 0 {
                    suffix_length = 1;
                }
                if tb.level_prefix[i] >= 15 && suffix_length == 0 {
                    level_code += 15;
                }
                if tb.level_prefix[i] >= 16 {
                    level_code += (1 << (tb.level_prefix[i] - 3)) - 4096;
                }
                if i == tb.coeff_token.trailing_ones && tb.coeff_token.trailing_ones < 3 {
                    level_code += 2;
                }

                let level_val: i32 = if level_code % 2 == 0 {
                    (level_code + 2) >> 1
                } else {
                    (-level_code - 1) >> 1
                };
                if level_val.abs() > (3 << (suffix_length - 1)) && suffix_length < 6 {
                    suffix_length += 1;
                }
                encoder_formatted_print("level_val", level_val, 63);
            }
        }
        let mut zeros_left: i32 = 0;
        if tb.coeff_token.total_coeff < end_idx - start_idx + 1 {
            cavlc::cavlc_encode_total_zeros(
                tb.total_zeros,
                max_num_coeff,
                tb.coeff_token.total_coeff,
                &mut bitstream_array,
            );
            zeros_left = tb.total_zeros as i32;
        }
        for i in 0..tb.coeff_token.total_coeff - 1 {
            let mut run_val: i32 = 0;
            if zeros_left > 0 {
                cavlc::cavlc_encode_run_before(tb.run_before[i], zeros_left, &mut bitstream_array);
                run_val = tb.run_before[i] as i32;
            }

            // TODO: check for potential underflow
            zeros_left -= run_val;
        }
    }

    bitstream_array
}
