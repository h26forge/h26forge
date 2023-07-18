//! Slice header and data syntax element randomization.

use crate::common::data_structures::H264DecodedStream;
use crate::common::data_structures::MacroBlock;
use crate::common::data_structures::MbPartPredMode;
use crate::common::data_structures::MbType;
use crate::common::data_structures::PicParameterSet;
use crate::common::data_structures::SeqParameterSet;
use crate::common::data_structures::SubMbType;
use crate::common::data_structures::VideoParameters;
use crate::common::helper::is_slice_type;
use crate::vidgen::film::FilmState;
use crate::vidgen::generate_configurations::RandomMBRange;
use crate::vidgen::generate_configurations::RandomSliceHeaderRange;
use crate::vidgen::generate_configurations::RandomizeConfig;
use crate::vidgen::macroblock::random_b_mbtype;
use crate::vidgen::macroblock::random_i_mbtype;
use crate::vidgen::macroblock::random_p_mbtype;
use crate::vidgen::macroblock::random_si_mbtype;
use crate::vidgen::macroblock::randomize_mb_pred;
use crate::vidgen::macroblock::randomize_residual;
use crate::vidgen::macroblock::randomize_sub_mb_pred;

fn next_mb_addr(n: usize) -> usize {
    // TODO: calculate the next_mb_addr based off FMO
    n + 1
}

/// Generate random slice data syntax elements.
fn random_slice_data(
    slice_idx: usize,
    pps: &PicParameterSet,
    sps: &SeqParameterSet,
    vp: &VideoParameters,
    ignore_intra_pred: bool,
    ignore_edge_intra_pred: bool,
    ignore_ipcm: bool,
    empty_slice_data: bool,
    rconfig: &RandomMBRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    let mut prev_mb_skipped = false;
    let mut curr_mb_addr = (ds.slices[slice_idx].sh.first_mb_in_slice
        * (1 + match ds.slices[slice_idx].sh.mbaff_frame_flag {
            true => 1,
            _ => 0,
        })) as usize;

    for i in 0..ds.slices[slice_idx].sd.macroblock_vec.len() {
        // resets all the macroblock data to ensure that any future dependencies are encoded correctly
        ds.slices[slice_idx].sd.macroblock_vec[i] = MacroBlock::new();
        ds.slices[slice_idx].sd.macroblock_vec[i].mb_idx = i;
        ds.slices[slice_idx].sd.macroblock_vec[i].mb_addr = curr_mb_addr;
        ds.slices[slice_idx].sd.macroblock_vec[i].available = true;

        if vp.chroma_array_type == 0 {
            // monochrome
            ds.slices[slice_idx].sd.macroblock_vec[i].num_c8x8 = 0;
        } else {
            ds.slices[slice_idx].sd.macroblock_vec[i].num_c8x8 =
                (4 / (vp.sub_width_c * vp.sub_height_c)) as usize;
        }

        ds.slices[slice_idx].sd.macroblock_vec[i].mb_skip_flag = false;

        // I and SI slices don't read the mb_skip_flag; for other types we'll flip a biased coin to decide whether to skip or not
        if !is_slice_type(ds.slices[slice_idx].sh.slice_type, "I")
            && !is_slice_type(ds.slices[slice_idx].sh.slice_type, "SI")
        {
            if vp.entropy_coding_mode_flag {
                if empty_slice_data {
                    ds.slices[slice_idx].sd.macroblock_vec[i].mb_skip_flag = true;
                } else {
                    ds.slices[slice_idx].sd.macroblock_vec[i].mb_skip_flag =
                        rconfig.mb_skip_flag.sample(film);
                }
            } else {
                ds.slices[slice_idx]
                    .sd
                    .mb_skip_run
                    .push(rconfig.mb_skip_run.sample(film));
                // if the skip run is large enough we can stop generating macroblocks
                if ds.slices[slice_idx].sd.mb_skip_run[i] > 0 {
                    // TODO: insert filler data if the skip run doesn't go
                    //   to the end of the video. This requires setting the macroblock
                    //   types to be skipped and updating the MB address.
                    prev_mb_skipped = true;
                }
            }
        }

        if !ds.slices[slice_idx].sd.macroblock_vec[i].mb_skip_flag {
            if ds.slices[slice_idx].sh.mbaff_frame_flag
                && (ds.slices[slice_idx].sd.macroblock_vec[i].mb_addr % 2 == 0
                    || (ds.slices[slice_idx].sd.macroblock_vec[i].mb_addr % 2 == 1
                        && prev_mb_skipped))
            {
                let mb_field_decoding_flag = rconfig.mb_field_decoding_flag.sample(film);
                if i < ds.slices[slice_idx].sd.mb_field_decoding_flag.len() {
                    ds.slices[slice_idx].sd.mb_field_decoding_flag[i] = mb_field_decoding_flag;
                } else {
                    ds.slices[slice_idx]
                        .sd
                        .mb_field_decoding_flag
                        .push(mb_field_decoding_flag);
                }

                if ds.slices[slice_idx].sd.macroblock_vec[i].mb_addr % 2 == 1 && prev_mb_skipped {
                    ds.slices[slice_idx].sd.mb_field_decoding_flag[i - 1] = mb_field_decoding_flag;
                }
            } else if !ds.slices[slice_idx].sh.mbaff_frame_flag {
                let field_pic_flag = ds.slices[slice_idx].sh.field_pic_flag;
                ds.slices[slice_idx]
                    .sd
                    .mb_field_decoding_flag
                    .push(field_pic_flag);
            } else {
                if ds.slices[slice_idx].sd.macroblock_vec[i].mb_addr % 2 == 1 && i > 0 {
                    let prev_field_decode_flag =
                        ds.slices[slice_idx].sd.mb_field_decoding_flag[i - 1];
                    if i < ds.slices[slice_idx].sd.mb_field_decoding_flag.len() {
                        ds.slices[slice_idx].sd.mb_field_decoding_flag[i] = prev_field_decode_flag;
                    } else {
                        ds.slices[slice_idx]
                            .sd
                            .mb_field_decoding_flag
                            .push(prev_field_decode_flag);
                    }
                } else {
                    if i < ds.slices[slice_idx].sd.mb_field_decoding_flag.len() {
                        ds.slices[slice_idx].sd.mb_field_decoding_flag[i] = false;
                    } else {
                        ds.slices[slice_idx].sd.mb_field_decoding_flag.push(false);
                    }
                }
            }

            prev_mb_skipped = false;
            let mut ignore_intra_pred_flag = ignore_intra_pred;

            if ignore_edge_intra_pred {
                let x_d = (i as u32) % vp.pic_width_in_mbs;
                let y_d = (i as u32) / vp.pic_width_in_mbs;

                ignore_intra_pred_flag = x_d == 0 || y_d == 0;
            }

            if is_slice_type(ds.slices[slice_idx].sh.slice_type, "I") {
                if empty_slice_data {
                    ds.slices[slice_idx].sd.macroblock_vec[i].mb_type = MbType::INxN;
                } else {
                    ds.slices[slice_idx].sd.macroblock_vec[i].mb_type =
                        random_i_mbtype(ignore_intra_pred_flag, ignore_ipcm, rconfig, film);
                }
            } else if is_slice_type(ds.slices[slice_idx].sh.slice_type, "SI") {
                if empty_slice_data {
                    ds.slices[slice_idx].sd.macroblock_vec[i].mb_type = MbType::INxN;
                } else {
                    ds.slices[slice_idx].sd.macroblock_vec[i].mb_type =
                        random_si_mbtype(ignore_intra_pred_flag, ignore_ipcm, rconfig, film);
                }
            } else if is_slice_type(ds.slices[slice_idx].sh.slice_type, "B") {
                ds.slices[slice_idx].sd.macroblock_vec[i].mb_type =
                    random_b_mbtype(ignore_intra_pred_flag, ignore_ipcm, rconfig, film);
            } else {
                // P or SP slice
                ds.slices[slice_idx].sd.macroblock_vec[i].mb_type =
                    random_p_mbtype(ignore_intra_pred_flag, ignore_ipcm, rconfig, film);
            }

            if ds.slices[slice_idx].sd.macroblock_vec[i].mb_type == MbType::IPCM {
                ds.slices[slice_idx].sd.macroblock_vec[i].pcm_sample_luma = Vec::new(); // get rid of what was already there
                for _ in 0..256 {
                    // range is 0 to 2**luma_bit_depth
                    ds.slices[slice_idx].sd.macroblock_vec[i]
                        .pcm_sample_luma
                        .push(rconfig.pcm_sample_luma.sample(film));
                }

                ds.slices[slice_idx].sd.macroblock_vec[i].pcm_sample_chroma = Vec::new();

                for _ in 0..2 * vp.mb_width_c * vp.mb_height_c {
                    // range is 0 to 2**chroma_bit_depth
                    ds.slices[slice_idx].sd.macroblock_vec[i]
                        .pcm_sample_chroma
                        .push(rconfig.pcm_sample_chroma.sample(film));
                }
            } else {
                ds.slices[slice_idx].sd.macroblock_vec[i].no_sub_mb_part_size_less_than_8x8_flag =
                    true;

                if ds.slices[slice_idx].sd.macroblock_vec[i].mb_type != MbType::INxN
                    && ds.slices[slice_idx].sd.macroblock_vec[i].mb_part_pred_mode(0)
                        != MbPartPredMode::Intra16x16
                    && ds.slices[slice_idx].sd.macroblock_vec[i].num_mb_part() == 4
                {
                    // sub_mb_pred
                    randomize_sub_mb_pred(slice_idx, i, rconfig, ds, film);
                    for mb_part_idx in 0..4 {
                        if ds.slices[slice_idx].sd.macroblock_vec[i].sub_mb_type[mb_part_idx]
                            != SubMbType::BDirect8x8
                        {
                            if ds.slices[slice_idx].sd.macroblock_vec[i]
                                .num_sub_mb_part(mb_part_idx)
                                > 1
                            {
                                ds.slices[slice_idx].sd.macroblock_vec[i]
                                    .no_sub_mb_part_size_less_than_8x8_flag = false;
                            }
                        } else if !sps.direct_8x8_inference_flag {
                            ds.slices[slice_idx].sd.macroblock_vec[i]
                                .no_sub_mb_part_size_less_than_8x8_flag = false;
                        }
                    }
                } else {
                    if pps.transform_8x8_mode_flag
                        && ds.slices[slice_idx].sd.macroblock_vec[i].mb_type == MbType::INxN
                    {
                        ds.slices[slice_idx].sd.macroblock_vec[i].transform_size_8x8_flag =
                            rconfig.transform_size_8x8_flag.sample(film);
                    }

                    // mb_pred
                    // if we have empty_slice_data, then this code will only run for I slices, which means ignoring intra_pred
                    randomize_mb_pred(
                        slice_idx,
                        vp,
                        ignore_edge_intra_pred,
                        ignore_intra_pred || empty_slice_data,
                        i,
                        rconfig,
                        ds,
                        film,
                    );
                }

                if ds.slices[slice_idx].sd.macroblock_vec[i].mb_part_pred_mode(0)
                    != MbPartPredMode::Intra16x16
                {
                    // Bias B and P types to not have as much residue data
                    if is_slice_type(ds.slices[slice_idx].sh.slice_type, "B")
                        || is_slice_type(ds.slices[slice_idx].sh.slice_type, "P")
                    {
                        if rconfig.bias_b_p_no_residue.sample(film) {
                            ds.slices[slice_idx].sd.macroblock_vec[i].coded_block_pattern = 0;
                        } else {
                            ds.slices[slice_idx].sd.macroblock_vec[i].coded_block_pattern =
                                rconfig.coded_block_pattern.sample(film);
                        }
                    } else {
                        // I slices
                        if empty_slice_data {
                            ds.slices[slice_idx].sd.macroblock_vec[i].coded_block_pattern = 0;
                        } else {
                            ds.slices[slice_idx].sd.macroblock_vec[i].coded_block_pattern =
                                rconfig.coded_block_pattern.sample(film);
                        }
                    }

                    // for these chroma array types let's zero out the chroma part
                    if sps.chroma_format_idc == 0 || sps.chroma_format_idc == 3 {
                        // zero out the top two bits
                        ds.slices[slice_idx].sd.macroblock_vec[i].coded_block_pattern &= 0xF;
                    }

                    ds.slices[slice_idx].sd.macroblock_vec[i].coded_block_pattern_luma =
                        ds.slices[slice_idx].sd.macroblock_vec[i].coded_block_pattern % 16;
                    ds.slices[slice_idx].sd.macroblock_vec[i].coded_block_pattern_chroma =
                        ds.slices[slice_idx].sd.macroblock_vec[i].coded_block_pattern / 16;

                    if ds.slices[slice_idx].sd.macroblock_vec[i].coded_block_pattern_luma > 0
                        && pps.transform_8x8_mode_flag
                        && ds.slices[slice_idx].sd.macroblock_vec[i].mb_type != MbType::INxN
                        && ds.slices[slice_idx].sd.macroblock_vec[i]
                            .no_sub_mb_part_size_less_than_8x8_flag
                        && (ds.slices[slice_idx].sd.macroblock_vec[i].mb_type
                            != MbType::BDirect16x16
                            || sps.direct_8x8_inference_flag)
                    {
                        ds.slices[slice_idx].sd.macroblock_vec[i].transform_size_8x8_flag =
                            rconfig.transform_size_8x8_flag.sample(film);
                    }
                }
                // call this in case we're the appropriate type Intra16x16 type
                ds.slices[slice_idx].sd.macroblock_vec[i].set_cbp_chroma_and_luma();

                if ds.slices[slice_idx].sd.macroblock_vec[i].coded_block_pattern_luma > 0
                    || ds.slices[slice_idx].sd.macroblock_vec[i].coded_block_pattern_chroma > 0
                    || ds.slices[slice_idx].sd.macroblock_vec[i].mb_part_pred_mode(0)
                        == MbPartPredMode::Intra16x16
                {
                    ds.slices[slice_idx].sd.macroblock_vec[i].mb_qp_delta =
                        rconfig.mb_qp_delta.sample(film);
                    randomize_residual(slice_idx, i, vp, rconfig, ds, film);
                }
            }
        } else {
            prev_mb_skipped = true;
            if is_slice_type(ds.slices[slice_idx].sh.slice_type, "B") {
                ds.slices[slice_idx].sd.macroblock_vec[i].mb_type = MbType::BSkip;
            } else {
                // P or SP slice
                ds.slices[slice_idx].sd.macroblock_vec[i].mb_type = MbType::PSkip;
            }

            // if we haven't added to the list yet and are a bottom macroblock, then copy over
            if i > 0 && ds.slices[slice_idx].sd.macroblock_vec[i].mb_addr % 2 == 1 {
                if i >= ds.slices[slice_idx].sd.mb_field_decoding_flag.len() {
                    let prev_mb_field_decode_flag =
                        ds.slices[slice_idx].sd.mb_field_decoding_flag[i - 1];
                    ds.slices[slice_idx]
                        .sd
                        .mb_field_decoding_flag
                        .push(prev_mb_field_decode_flag);
                }
            } else {
                if i < ds.slices[slice_idx].sd.mb_field_decoding_flag.len() {
                    ds.slices[slice_idx].sd.mb_field_decoding_flag[i] = false;
                } else {
                    ds.slices[slice_idx].sd.mb_field_decoding_flag.push(false);
                }
            }
        }
        ds.slices[slice_idx].sd.end_of_slice_flag.push(false);

        curr_mb_addr = next_mb_addr(curr_mb_addr);
    }
    // set the last end_of_slice_flag to true
    let last_mb_idx = ds.slices[slice_idx].sd.macroblock_vec.len() - 1;
    ds.slices[slice_idx].sd.end_of_slice_flag[last_mb_idx] = true;
}

fn random_ref_pic_list_modification(
    slice_idx: usize,
    rconfig: &RandomSliceHeaderRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    if ds.slices[slice_idx].sh.slice_type % 5 != 2 && ds.slices[slice_idx].sh.slice_type % 5 != 4 {
        // ref_pic_list_modification_flag_l0
        ds.slices[slice_idx].sh.ref_pic_list_modification_flag_l0 =
            rconfig.ref_pic_list_modification_flag_l0.sample(film);

        if ds.slices[slice_idx].sh.ref_pic_list_modification_flag_l0 {
            let number_of_modifications_l0 = rconfig.number_of_modifications_l0.sample(film);

            // reset any previous lingering modifications
            ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l0 = Vec::new();
            ds.slices[slice_idx].sh.long_term_pic_num_l0 = Vec::new();
            ds.slices[slice_idx].sh.abs_diff_pic_num_minus1_l0 = Vec::new();

            for i in 0..(number_of_modifications_l0 as usize) {
                // The sample is from [0, 5] so we mod 3 to focus on [0, 2]
                ds.slices[slice_idx]
                    .sh
                    .modification_of_pic_nums_idc_l0
                    .push(rconfig.modification_of_pic_nums_idc_l0.sample(film) % 3);

                // abs_diff_pic_num_minus1
                if ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l0[i] == 0
                    || ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l0[i] == 1
                {
                    ds.slices[slice_idx]
                        .sh
                        .abs_diff_pic_num_minus1_l0
                        .push(rconfig.abs_diff_pic_num_minus1_l0.sample(film));
                } else {
                    ds.slices[slice_idx].sh.abs_diff_pic_num_minus1_l0.push(0);
                }

                if ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l0[i] == 2 {
                    ds.slices[slice_idx]
                        .sh
                        .long_term_pic_num_l0
                        .push(rconfig.long_term_pic_num_l0.sample(film));
                } else {
                    ds.slices[slice_idx].sh.long_term_pic_num_l0.push(0);
                }
            }
            ds.slices[slice_idx]
                .sh
                .modification_of_pic_nums_idc_l0
                .push(3); // no more operations
        }
    }

    if ds.slices[slice_idx].sh.slice_type % 5 == 1 {
        // ref_pic_list_modification_flag_l1
        ds.slices[slice_idx].sh.ref_pic_list_modification_flag_l1 =
            rconfig.ref_pic_list_modification_flag_l1.sample(film);

        if ds.slices[slice_idx].sh.ref_pic_list_modification_flag_l1 {
            let number_of_modifications_l1 = rconfig.number_of_modifications_l1.sample(film);

            for i in 0..(number_of_modifications_l1 as usize) {
                // The sample is from [0, 5] so we mod 3 to focus on [0, 2]
                ds.slices[slice_idx]
                    .sh
                    .modification_of_pic_nums_idc_l1
                    .push(rconfig.modification_of_pic_nums_idc_l1.sample(film) % 3);

                // abs_diff_pic_num_minus1
                if ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l1[i] == 0
                    || ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l1[i] == 1
                {
                    ds.slices[slice_idx]
                        .sh
                        .abs_diff_pic_num_minus1_l1
                        .push(rconfig.abs_diff_pic_num_minus1_l1.sample(film));
                } else {
                    ds.slices[slice_idx].sh.abs_diff_pic_num_minus1_l1.push(0);
                }

                if ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l1[i] == 2 {
                    ds.slices[slice_idx]
                        .sh
                        .long_term_pic_num_l1
                        .push(rconfig.long_term_pic_num_l1.sample(film));
                } else {
                    ds.slices[slice_idx].sh.long_term_pic_num_l1.push(0);
                }
            }
            ds.slices[slice_idx]
                .sh
                .modification_of_pic_nums_idc_l1
                .push(3); // no more operations
        }
    }
}

fn random_ref_pic_list_mvc_modification(
    slice_idx: usize,
    rconfig: &RandomSliceHeaderRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    if ds.slices[slice_idx].sh.slice_type % 5 != 2 && ds.slices[slice_idx].sh.slice_type % 5 != 4 {
        // ref_pic_list_modification_flag_l0
        ds.slices[slice_idx].sh.ref_pic_list_modification_flag_l0 =
            rconfig.ref_pic_list_modification_flag_l0.sample(film);

        if ds.slices[slice_idx].sh.ref_pic_list_modification_flag_l0 {
            let number_of_modifications_l0 = rconfig.number_of_modifications_l0.sample(film);

            // reset any previous lingering modifications
            ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l0 = Vec::new();
            ds.slices[slice_idx].sh.long_term_pic_num_l0 = Vec::new();
            ds.slices[slice_idx].sh.abs_diff_pic_num_minus1_l0 = Vec::new();

            for i in 0..(number_of_modifications_l0 as usize) {
                ds.slices[slice_idx]
                    .sh
                    .modification_of_pic_nums_idc_l0
                    .push(rconfig.modification_of_pic_nums_idc_l0.sample(film));

                // 3 is the exit condition so we change it
                if ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l0[i] == 3 {
                    ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l0[i] = 0;
                }

                // abs_diff_pic_num_minus1
                if ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l0[i] == 0
                    || ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l0[i] == 1
                {
                    ds.slices[slice_idx]
                        .sh
                        .abs_diff_pic_num_minus1_l0
                        .push(rconfig.abs_diff_pic_num_minus1_l0.sample(film));
                } else {
                    ds.slices[slice_idx].sh.abs_diff_pic_num_minus1_l0.push(0);
                }

                if ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l0[i] == 2 {
                    ds.slices[slice_idx]
                        .sh
                        .long_term_pic_num_l0
                        .push(rconfig.long_term_pic_num_l0.sample(film));
                } else {
                    ds.slices[slice_idx].sh.long_term_pic_num_l0.push(0);
                }

                if ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l0[i] == 4
                    || ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l0[i] == 5
                {
                    ds.slices[slice_idx]
                        .sh
                        .abs_diff_view_idx_minus1_l0
                        .push(rconfig.abs_diff_view_idx_minus1_l0.sample(film));
                } else {
                    ds.slices[slice_idx].sh.abs_diff_view_idx_minus1_l0.push(0);
                }
            }
            ds.slices[slice_idx]
                .sh
                .modification_of_pic_nums_idc_l0
                .push(3); // no more operations
        }
    }

    if ds.slices[slice_idx].sh.slice_type % 5 == 1 {
        // ref_pic_list_modification_flag_l1
        ds.slices[slice_idx].sh.ref_pic_list_modification_flag_l1 =
            rconfig.ref_pic_list_modification_flag_l1.sample(film);

        if ds.slices[slice_idx].sh.ref_pic_list_modification_flag_l1 {
            let number_of_modifications_l1 = rconfig.number_of_modifications_l1.sample(film);

            for i in 0..(number_of_modifications_l1 as usize) {
                ds.slices[slice_idx]
                    .sh
                    .modification_of_pic_nums_idc_l1
                    .push(rconfig.modification_of_pic_nums_idc_l1.sample(film));

                // 3 is the exit condition so we change it
                if ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l1[i] == 3 {
                    ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l1[i] = 0;
                }

                // abs_diff_pic_num_minus1
                if ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l1[i] == 0
                    || ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l1[i] == 1
                {
                    ds.slices[slice_idx]
                        .sh
                        .abs_diff_pic_num_minus1_l1
                        .push(rconfig.abs_diff_pic_num_minus1_l1.sample(film));
                } else {
                    ds.slices[slice_idx].sh.abs_diff_pic_num_minus1_l1.push(0);
                }

                if ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l1[i] == 2 {
                    ds.slices[slice_idx]
                        .sh
                        .long_term_pic_num_l1
                        .push(rconfig.long_term_pic_num_l1.sample(film));
                } else {
                    ds.slices[slice_idx].sh.long_term_pic_num_l1.push(0);
                }

                if ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l1[i] == 4
                    || ds.slices[slice_idx].sh.modification_of_pic_nums_idc_l1[i] == 5
                {
                    ds.slices[slice_idx]
                        .sh
                        .abs_diff_view_idx_minus1_l1
                        .push(rconfig.abs_diff_view_idx_minus1_l1.sample(film));
                } else {
                    ds.slices[slice_idx].sh.abs_diff_view_idx_minus1_l1.push(0);
                }
            }
            ds.slices[slice_idx]
                .sh
                .modification_of_pic_nums_idc_l1
                .push(3); // no more operations
        }
    }
}

fn random_pred_weight_table(
    slice_idx: usize,
    vp: &VideoParameters,
    rconfig: &RandomSliceHeaderRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    // luma_log2_weight_denom
    ds.slices[slice_idx].sh.luma_log2_weight_denom = rconfig.luma_log2_weight_denom.sample(film);

    // chroma_log2_weight_denom
    if vp.chroma_array_type != 0 {
        ds.slices[slice_idx].sh.chroma_log2_weight_denom =
            rconfig.chroma_log2_weight_denom.sample(film);
    }

    // get weight tables
    for i in 0..(ds.slices[slice_idx].sh.num_ref_idx_l0_active_minus1 + 1) {
        ds.slices[slice_idx]
            .sh
            .luma_weight_l0_flag
            .push(rconfig.luma_weight_l0_flag.sample(film));

        if ds.slices[slice_idx].sh.luma_weight_l0_flag[i as usize] {
            ds.slices[slice_idx]
                .sh
                .luma_weight_l0
                .push(rconfig.luma_weight_l0.sample(film));
            ds.slices[slice_idx]
                .sh
                .luma_offset_l0
                .push(rconfig.luma_offset_l0.sample(film));
        } else {
            // this is to ensure our pushes are aligned with the index
            ds.slices[slice_idx].sh.luma_weight_l0.push(0);
            ds.slices[slice_idx].sh.luma_offset_l0.push(0);
        }

        if vp.chroma_array_type != 0 {
            ds.slices[slice_idx]
                .sh
                .chroma_weight_l0_flag
                .push(rconfig.chroma_weight_l0_flag.sample(film));

            if ds.slices[slice_idx].sh.chroma_weight_l0_flag[i as usize] {
                let cwl0 = rconfig.chroma_weight_l0.sample(film);
                let col0 = rconfig.chroma_offset_l0.sample(film);

                let cwl1 = rconfig.chroma_weight_l0.sample(film);
                let col1 = rconfig.chroma_offset_l0.sample(film);

                ds.slices[slice_idx]
                    .sh
                    .chroma_weight_l0
                    .push(vec![cwl0, cwl1]);
                ds.slices[slice_idx]
                    .sh
                    .chroma_offset_l0
                    .push(vec![col0, col1]);
            } else {
                // to ensure the indices are aligned
                ds.slices[slice_idx].sh.chroma_weight_l0.push(vec![0, 0]);
                ds.slices[slice_idx].sh.chroma_offset_l0.push(vec![0, 0]);
            }
        }
    }
    // collect the l1 values
    if ds.slices[slice_idx].sh.slice_type % 5 == 1 {
        for i in 0..(ds.slices[slice_idx].sh.num_ref_idx_l1_active_minus1 + 1) {
            ds.slices[slice_idx]
                .sh
                .luma_weight_l1_flag
                .push(rconfig.luma_weight_l1_flag.sample(film));

            if ds.slices[slice_idx].sh.luma_weight_l1_flag[i as usize] {
                ds.slices[slice_idx]
                    .sh
                    .luma_weight_l1
                    .push(rconfig.luma_weight_l1.sample(film));
                ds.slices[slice_idx]
                    .sh
                    .luma_offset_l1
                    .push(rconfig.luma_offset_l1.sample(film));
            } else {
                // to ensure the indices are aligned
                ds.slices[slice_idx].sh.luma_weight_l1.push(0);
                ds.slices[slice_idx].sh.luma_offset_l1.push(0);
            }

            if vp.chroma_array_type != 0 {
                ds.slices[slice_idx]
                    .sh
                    .chroma_weight_l1_flag
                    .push(rconfig.chroma_weight_l1_flag.sample(film));

                if ds.slices[slice_idx].sh.chroma_weight_l1_flag[i as usize] {
                    let cwl0 = rconfig.chroma_weight_l1.sample(film);
                    let col0 = rconfig.chroma_offset_l1.sample(film);
                    let cwl1 = rconfig.chroma_weight_l1.sample(film);
                    let col1 = rconfig.chroma_offset_l1.sample(film);

                    ds.slices[slice_idx]
                        .sh
                        .chroma_weight_l1
                        .push(vec![cwl0, cwl1]);
                    ds.slices[slice_idx]
                        .sh
                        .chroma_offset_l1
                        .push(vec![col0, col1]);
                } else {
                    // to ensure the indices are aligned
                    ds.slices[slice_idx].sh.chroma_weight_l1.push(vec![0, 0]);
                    ds.slices[slice_idx].sh.chroma_offset_l1.push(vec![0, 0]);
                }
            }
        }
    }
}

fn random_dec_ref_pic_marking(
    slice_idx: usize,
    vp: &VideoParameters,
    rconfig: &RandomSliceHeaderRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    if vp.idr_pic_flag {
        ds.slices[slice_idx].sh.no_output_of_prior_pics_flag =
            rconfig.no_output_of_prior_pics_flag.sample(film);

        ds.slices[slice_idx].sh.long_term_reference_flag =
            rconfig.long_term_reference_flag.sample(film);
    } else {
        ds.slices[slice_idx].sh.adaptive_ref_pic_marking_mode_flag =
            rconfig.adaptive_ref_pic_marking_mode_flag.sample(film);

        if ds.slices[slice_idx].sh.adaptive_ref_pic_marking_mode_flag {
            let number_of_mem_ops = rconfig.number_of_mem_ops.sample(film) as usize;

            ds.slices[slice_idx].sh.memory_management_control_operation = Vec::new();
            for i in 0..number_of_mem_ops {
                ds.slices[slice_idx]
                    .sh
                    .memory_management_control_operation
                    .push(rconfig.memory_management_control_operation.sample(film));

                if ds.slices[slice_idx].sh.memory_management_control_operation[i] == 1
                    || ds.slices[slice_idx].sh.memory_management_control_operation[i] == 3
                {
                    ds.slices[slice_idx]
                        .sh
                        .difference_of_pic_nums_minus1
                        .push(rconfig.difference_of_pic_nums_minus1.sample(film));
                } else {
                    ds.slices[slice_idx]
                        .sh
                        .difference_of_pic_nums_minus1
                        .push(0);
                }

                if ds.slices[slice_idx].sh.memory_management_control_operation[i] == 2 {
                    ds.slices[slice_idx]
                        .sh
                        .long_term_pic_num
                        .push(rconfig.long_term_pic_num.sample(film));
                } else {
                    ds.slices[slice_idx].sh.long_term_pic_num.push(0);
                }

                if ds.slices[slice_idx].sh.memory_management_control_operation[i] == 3
                    || ds.slices[slice_idx].sh.memory_management_control_operation[i] == 6
                {
                    ds.slices[slice_idx]
                        .sh
                        .long_term_frame_idx
                        .push(rconfig.long_term_frame_idx.sample(film));
                } else {
                    ds.slices[slice_idx].sh.long_term_frame_idx.push(0);
                }
                if ds.slices[slice_idx].sh.memory_management_control_operation[i] == 4 {
                    ds.slices[slice_idx]
                        .sh
                        .max_long_term_frame_idx_plus1
                        .push(rconfig.max_long_term_frame_idx_plus1.sample(film));
                } else {
                    ds.slices[slice_idx]
                        .sh
                        .max_long_term_frame_idx_plus1
                        .push(0);
                }
            }

            ds.slices[slice_idx]
                .sh
                .memory_management_control_operation
                .push(0);
        }
    }
}

/// Generate random slice header syntax elements.
///
/// Returns the associated pps_idx, and current video parameters
fn random_slice_header(
    nalu_idx: usize,
    slice_idx: usize,
    pps: &PicParameterSet,
    sps: &SeqParameterSet,
    vp: &VideoParameters,
    rconfig: &RandomSliceHeaderRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    // NOTE: we sample first_mb_in_slice after determining field/frame slice

    // bias the first slice towards an I slice or IDR NALU types to be I slices
    if slice_idx == 0 || vp.idr_pic_flag {
        if rconfig.bias_i_slice.sample(film) {
            ds.slices[slice_idx].sh.slice_type = 2; // I slice
        } else {
            ds.slices[slice_idx].sh.slice_type = rconfig.slice_type.sample(film) as u8;
        }
    } else {
        ds.slices[slice_idx].sh.slice_type = rconfig.slice_type.sample(film) as u8;
    }
    ds.slices[slice_idx].sh.pic_parameter_set_id = pps.pic_parameter_set_id;

    if sps.separate_colour_plane_flag {
        ds.slices[slice_idx].sh.colour_plane_id = rconfig.colour_plane_id.sample(film) as u8;
    }

    // every so often randomize frame_num
    if rconfig.bias_non_rand_frame_num.sample(film) {
        if vp.idr_pic_flag {
            // IDR slices are supposed to be frame_num == 0 so we bias towards
            if rconfig.bias_idr_zero_frame_num.sample(film) {
                ds.slices[slice_idx].sh.frame_num = 0;
            } else {
                ds.slices[slice_idx].sh.frame_num = slice_idx as u32;
            }
        } else {
            ds.slices[slice_idx].sh.frame_num = slice_idx as u32;
        }
    } else {
        let frame_num_length_in_bits = sps.log2_max_frame_num_minus4 + 4;
        let max_frame_num = 2u32.pow(frame_num_length_in_bits as u32) - 1;
        ds.slices[slice_idx].sh.frame_num = rconfig.frame_num.sample(0, max_frame_num, film);
    }

    if !sps.frame_mbs_only_flag {
        ds.slices[slice_idx].sh.field_pic_flag = rconfig.field_pic_flag.sample(film);
        if ds.slices[slice_idx].sh.field_pic_flag {
            ds.slices[slice_idx].sh.bottom_field_flag = rconfig.bottom_field_flag.sample(film);
        } else {
            // check the length of frame slice in field-allowable video
            let macroblock_amount = ((sps.pic_width_in_mbs_minus1 + 1)
                * (sps.pic_height_in_map_units_minus1 + 1))
                as usize;

            // Frame slices in videos where fields are allowed actually contain
            // double the amount of macroblocks. If this doesn't match then we
            // add some empty macroblocks to randomize later on.
            if ds.slices[slice_idx].sd.macroblock_vec.len() != 2 * macroblock_amount {
                ds.slices[slice_idx]
                    .sd
                    .macroblock_vec
                    .extend(vec![MacroBlock::new(); macroblock_amount]);
            }
        }
    }

    // We sample first_mb_in_slice after determining field/frame
    // make slices lean towards first_mb_in_slice to 0
    if rconfig.bias_zero_first_mb_in_slice.sample(film) {
        ds.slices[slice_idx].sh.first_mb_in_slice = 0;
    } else {
        let frame_size_in_mbs = vp.frame_height_in_mbs * vp.pic_width_in_mbs;
        let max_mb_addr: u32;
        if !sps.frame_mbs_only_flag && ds.slices[slice_idx].sh.field_pic_flag {
            max_mb_addr = (frame_size_in_mbs / 2) - 1;
        } else {
            max_mb_addr = frame_size_in_mbs - 1;
        }
        // the max mb addr is based on the total number of macroblocks
        ds.slices[slice_idx].sh.first_mb_in_slice =
            rconfig.first_mb_in_slice.sample(0, max_mb_addr, film);
    }

    if vp.idr_pic_flag {
        ds.slices[slice_idx].sh.idr_pic_id = rconfig.idr_pic_id.sample(film);
    }
    if sps.pic_order_cnt_type == 0 {
        // pic_order_cnt_lsb depends on the number of bits available from the SPS
        let max_pic_order_cnt = 1 << (sps.log2_max_pic_order_cnt_lsb_minus4 + 4);
        ds.slices[slice_idx].sh.pic_order_cnt_lsb =
            rconfig.pic_order_cnt_lsb.sample(film) % max_pic_order_cnt;
        if pps.bottom_field_pic_order_in_frame_present_flag
            && !ds.slices[slice_idx].sh.field_pic_flag
        {
            ds.slices[slice_idx].sh.delta_pic_order_cnt_bottom =
                rconfig.delta_pic_order_cnt_bottom.sample(film);
        }
    }
    if sps.pic_order_cnt_type == 1 && !sps.delta_pic_order_always_zero_flag {
        ds.slices[slice_idx].sh.delta_pic_order_cnt = Vec::new();
        ds.slices[slice_idx]
            .sh
            .delta_pic_order_cnt
            .push(rconfig.delta_pic_order_cnt.sample(film));
        if pps.bottom_field_pic_order_in_frame_present_flag
            && !ds.slices[slice_idx].sh.field_pic_flag
        {
            ds.slices[slice_idx]
                .sh
                .delta_pic_order_cnt
                .push(rconfig.delta_pic_order_cnt.sample(film));
        }
    }

    if pps.redundant_pic_cnt_present_flag {
        ds.slices[slice_idx].sh.redundant_pic_cnt = rconfig.redundant_pic_cnt.sample(film);
    }

    if is_slice_type(ds.slices[slice_idx].sh.slice_type, "B") {
        ds.slices[slice_idx].sh.direct_spatial_mv_pred_flag =
            rconfig.direct_spatial_mv_pred_flag.sample(film);
    }

    if is_slice_type(ds.slices[slice_idx].sh.slice_type, "B")
        || is_slice_type(ds.slices[slice_idx].sh.slice_type, "P")
        || is_slice_type(ds.slices[slice_idx].sh.slice_type, "SP")
    {
        ds.slices[slice_idx].sh.num_ref_idx_active_override_flag =
            rconfig.num_ref_idx_active_override_flag.sample(film);
        if ds.slices[slice_idx].sh.num_ref_idx_active_override_flag {
            ds.slices[slice_idx].sh.num_ref_idx_l0_active_minus1 = rconfig
                .num_ref_idx_l0_active_minus1
                .sample(0, pps.num_ref_idx_l0_default_active_minus1 + 1, film);
            if is_slice_type(ds.slices[slice_idx].sh.slice_type, "B") {
                ds.slices[slice_idx].sh.num_ref_idx_l1_active_minus1 = rconfig
                    .num_ref_idx_l1_active_minus1
                    .sample(0, pps.num_ref_idx_l1_default_active_minus1 + 1, film);
            } else {
                ds.slices[slice_idx].sh.num_ref_idx_l1_active_minus1 =
                    pps.num_ref_idx_l1_default_active_minus1;
            }
        } else {
            // set to the PPS default
            ds.slices[slice_idx].sh.num_ref_idx_l0_active_minus1 =
                pps.num_ref_idx_l0_default_active_minus1;
            ds.slices[slice_idx].sh.num_ref_idx_l1_active_minus1 =
                pps.num_ref_idx_l1_default_active_minus1;
        }
    }

    if ds.nalu_headers[nalu_idx].nal_unit_type == 20
        || ds.nalu_headers[nalu_idx].nal_unit_type == 21
    {
        random_ref_pic_list_mvc_modification(slice_idx, rconfig, ds, film);
    } else {
        random_ref_pic_list_modification(slice_idx, rconfig, ds, film);
    }

    if (pps.weighted_pred_flag
        && (is_slice_type(ds.slices[slice_idx].sh.slice_type, "P")
            || is_slice_type(ds.slices[slice_idx].sh.slice_type, "SP")))
        || (pps.weighted_bipred_idc == 1 && is_slice_type(ds.slices[slice_idx].sh.slice_type, "B"))
    {
        random_pred_weight_table(slice_idx, vp, rconfig, ds, film);
    }

    if ds.nalu_headers[nalu_idx].nal_ref_idc != 0 {
        random_dec_ref_pic_marking(slice_idx, vp, rconfig, ds, film);
    }

    if pps.entropy_coding_mode_flag
        && is_slice_type(ds.slices[slice_idx].sh.slice_type, "I")
        && is_slice_type(ds.slices[slice_idx].sh.slice_type, "SI")
    {
        ds.slices[slice_idx].sh.cabac_init_idc = rconfig.cabac_init_idc.sample(film);
    }

    ds.slices[slice_idx].sh.slice_qp_delta = rconfig.slice_qp_delta.non_dependent_sample(film);

    if is_slice_type(ds.slices[slice_idx].sh.slice_type, "SI")
        || is_slice_type(ds.slices[slice_idx].sh.slice_type, "SP")
    {
        if is_slice_type(ds.slices[slice_idx].sh.slice_type, "SP") {
            ds.slices[slice_idx].sh.sp_for_switch_flag = rconfig.sp_for_switch_flag.sample(film);
        }
        ds.slices[slice_idx].sh.slice_qs_delta = rconfig.slice_qs_delta.sample(film)
    }

    if pps.deblocking_filter_control_present_flag {
        ds.slices[slice_idx].sh.disable_deblocking_filter_idc =
            rconfig.disable_deblocking_filter_idc.sample(film);
        if ds.slices[slice_idx].sh.disable_deblocking_filter_idc != 1 {
            ds.slices[slice_idx].sh.slice_alpha_c0_offset_div2 =
                rconfig.slice_alpha_c0_offset_div2.sample(film);
            ds.slices[slice_idx].sh.slice_beta_offset_div2 =
                rconfig.slice_beta_offset_div2.sample(film);
        }
    }

    if pps.num_slice_groups_minus1 > 0
        && pps.slice_group_map_type >= 3
        && pps.slice_group_map_type <= 5
    {
        ds.slices[slice_idx].sh.slice_group_change_cycle =
            rconfig.slice_group_change_cycle.sample(film);
    }

    // Variables that are computed from the spec but not decoded from the stream (and also used in encoding)

    // derivation from equation 7-25 in Spec
    ds.slices[slice_idx].sh.mbaff_frame_flag =
        sps.mb_adaptive_frame_field_flag && !ds.slices[slice_idx].sh.field_pic_flag;

    // equation 7-30
    ds.slices[slice_idx].sh.slice_qp_y =
        26 + ds.slices[slice_idx].sh.slice_qp_delta + pps.pic_init_qp_minus26;

    // if it's greater than the max value (51), then randomly decide whether to keep it or not
    // biased towards not keeping it
    if ds.slices[slice_idx].sh.slice_qp_y > 51 {
        if rconfig.bias_slice_qp_y_top_bound.sample(film) {
            let max_allowed_value = 51 - 26 - pps.pic_init_qp_minus26;

            if max_allowed_value < rconfig.slice_qp_delta.min {
                // This happens because our pps.pic_init_qp_minus26 is already out-of-bounds
                // So we need to set it to a negative value
                ds.slices[slice_idx].sh.slice_qp_delta = rconfig.slice_qp_delta.sample(
                    rconfig.slice_qp_delta.min,
                    0,
                    film,
                );
            } else {
                ds.slices[slice_idx].sh.slice_qp_delta = rconfig.slice_qp_delta.sample(
                    rconfig.slice_qp_delta.min,
                    max_allowed_value + 1,
                    film,
                );
            }
            ds.slices[slice_idx].sh.slice_qp_y =
                26 + ds.slices[slice_idx].sh.slice_qp_delta + pps.pic_init_qp_minus26;
        } else {
            // keep it as is
            println!("[WARNING] Keeping slice_qp_y greater than 51 -- may get decoding issues");
        }
    }

    // if it's less than 0, then randomly decide whether to keep it or not
    if ds.slices[slice_idx].sh.slice_qp_y < 0 {
        if rconfig.bias_slice_qp_y_bottom_bound.sample(film) {
            let min_allowed_value = -(pps.pic_init_qp_minus26 + 26);

            // some ranges are wild
            if min_allowed_value >= rconfig.slice_qp_delta.max {
                println!("[WARNING] PPS pic_init_qp_mins26 causing slice_qp_y to be out of bounds regardless of slice_qp_delta value");
                println!(
                    "[WARNING] Setting to min_allowed_value {}",
                    min_allowed_value
                );
                ds.slices[slice_idx].sh.slice_qp_delta = min_allowed_value;
            } else {
                ds.slices[slice_idx].sh.slice_qp_delta = rconfig.slice_qp_delta.sample(
                    min_allowed_value + 1,
                    rconfig.slice_qp_delta.max,
                    film,
                );
            }
            ds.slices[slice_idx].sh.slice_qp_y =
                26 + ds.slices[slice_idx].sh.slice_qp_delta + pps.pic_init_qp_minus26;
        } else {
            // keep it as is
            println!("[WARNING] Keeping slice_qp_y less than 0 -- may get decoding issues");
        }
    }

    ds.slices[slice_idx].sh.qp_y_prev = ds.slices[slice_idx].sh.slice_qp_y;
}

/// Generate random slice syntax elements.
///
/// It does not change the header or the number of macroblocks in the slice.
///
/// slice_idx : the slice to randomize
/// ignore_intra_pred : if true, will not pick intra predicted macroblock types
/// manual_seed : user-chosen seed to reproduce videos
/// ds : the decoded video file
pub fn random_slice(
    nalu_idx: usize,
    slice_idx: usize,
    pps: &PicParameterSet,
    sps: &SeqParameterSet,
    ignore_intra_pred: bool,
    ignore_edge_intra_pred: bool,
    ignore_ipcm: bool,
    empty_slice_data: bool,
    randomize_header: bool,
    rconfig: &RandomizeConfig,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    if slice_idx >= ds.slices.len() {
        println!(
            "\t [WARNING] Slice index {} greater than number of slices: {} - Skipping",
            slice_idx,
            ds.slices.len()
        );
        return;
    }

    let mut vp = VideoParameters::new(&ds.nalu_headers[nalu_idx], pps, sps);
    if randomize_header {
        random_slice_header(
            nalu_idx,
            slice_idx,
            pps,
            sps,
            &vp,
            &rconfig.random_slice_header_range,
            ds,
            film,
        );
        // for neighbor processing
        vp.mbaff_frame_flag = ds.slices[slice_idx].sh.mbaff_frame_flag;
    }
    random_slice_data(
        slice_idx,
        pps,
        sps,
        &vp,
        ignore_intra_pred,
        ignore_edge_intra_pred,
        ignore_ipcm,
        empty_slice_data,
        &rconfig.random_mb_range,
        ds,
        film,
    );
}

/// Generate random slice layer extension syntax elements.
pub fn random_slice_layer_extension(
    nalu_idx: usize,
    slice_idx: usize,
    subset_pps_idx: usize,
    subset_sps_idx: usize,
    ignore_intra_pred: bool,
    ignore_edge_intra_pred: bool,
    ignore_ipcm: bool,
    empty_slice_data: bool,
    randomize_header: bool,
    rconfig: &RandomizeConfig,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    // TODO: this is not a valid video because slice headers and slice data
    //      are different in SVC, MVC, and 3D

    //if ds.nalu_headers[nalu_idx].svc_extension_flag {
    //    random_slice_header_in_scalable_extension();
    //    let slice_skip_flag = false;
    //    if slice_skip_flag {
    //        random_slice_data_in_scalable_extension();
    //    }
    //} else if ds.nalu_headers[nalu_idx].avc_3d_extension_flag {
    //    random_slice_header_in_3davc_extension();
    //    random_slice_data_in_3davc_extension();
    //} else {
    let cur_sps = &ds.subset_spses[subset_sps_idx].sps.clone();
    let cur_pps = &ds.subset_ppses[subset_pps_idx].clone();
    random_slice(
        nalu_idx,
        slice_idx,
        cur_pps,
        cur_sps,
        ignore_intra_pred,
        ignore_edge_intra_pred,
        ignore_ipcm,
        empty_slice_data,
        randomize_header,
        rconfig,
        ds,
        film,
    );
    //}
}

#[allow(dead_code)]
fn random_slice_header_in_scalable_extension() {
    // TODO: Annex G
    println!("random_slice_header_in_scalable_extension - not yet supported");
}

#[allow(dead_code)]
fn random_slice_data_in_scalable_extension() {
    // TODO: Annex G
    println!("random_slice_data_in_scalable_extension - not yet supported");
}

#[allow(dead_code)]
fn random_slice_header_in_3davc_extension() {
    // TODO: Annex J
    println!("random_slice_header_in_3davc_extension - not yet supported");
}

#[allow(dead_code)]
fn random_slice_data_in_3davc_extension() {
    // TODO: Annex J
    println!("random_slice_data_in_3davc_extension - not yet supported");
}
