//! Parameter Set (SPS, PPS, VUI, extensions) syntax element randomization.

use crate::common::data_structures::H264DecodedStream;
use crate::common::data_structures::HRDParameters;
use crate::common::data_structures::SeqParameterSet;
use crate::common::data_structures::VUIParameters;
use crate::vidgen::film::FilmState;
use crate::vidgen::generate_configurations::RandomHRDRange;
use crate::vidgen::generate_configurations::RandomPPSRange;
use crate::vidgen::generate_configurations::RandomSPSMVCExtensionRange;
use crate::vidgen::generate_configurations::RandomSPSRange;
use crate::vidgen::generate_configurations::RandomSubsetSPSRange;
use crate::vidgen::generate_configurations::RandomVUIMVCParametersRange;
use crate::vidgen::generate_configurations::RandomVUIRange;
use crate::vidgen::generate_configurations::RandomSPSExtensionRange;
use crate::vidgen::generate_configurations::RandomSPSSVCExtensionRange;
use crate::vidgen::generate_configurations::RandomVUISVCParametersRange;
use std::cmp;

/// Randomly chooses a level and returns the associated max framesize and max decoded picture buffer
/// Levels also dictate how quickly something should be decoded.
fn random_level_idc(
    sps: &mut SeqParameterSet,
    rconfig: &RandomSPSRange,
    film: &mut FilmState,
) -> (u32, u32) {
    // Levels are detailed in Annex A
    // they range from 1 to 6, with sub levels from 1 to 3

    let max_framesize: u32;
    let max_dpb_size: u32;

    sps.level_idc = rconfig.level_idc.sample(film) as u8;

    match sps.level_idc {
        10 => {
            max_framesize = 99;
            max_dpb_size = 396;
        }
        11 => {
            if sps.constraint_set3_flag {
                max_framesize = 99;
                max_dpb_size = 396;
            } else {
                max_framesize = 396;
                max_dpb_size = 900;
            }
        }
        12 => {
            max_framesize = 396;
            max_dpb_size = 2376;
        }
        13 => {
            max_framesize = 396;
            max_dpb_size = 2376;
        }
        20 => {
            max_framesize = 396;
            max_dpb_size = 2376;
        }
        21 => {
            max_framesize = 792;
            max_dpb_size = 4752;
        }
        22 => {
            max_framesize = 1620;
            max_dpb_size = 8100;
        }
        30 => {
            max_framesize = 1620;
            max_dpb_size = 8100;
        }
        31 => {
            max_framesize = 3600;
            max_dpb_size = 18000;
        }
        32 => {
            max_framesize = 5120;
            max_dpb_size = 20480;
        }
        40 => {
            max_framesize = 8192;
            max_dpb_size = 32768;
        }
        41 => {
            max_framesize = 8192;
            max_dpb_size = 32768;
        }
        42 => {
            max_framesize = 8704;
            max_dpb_size = 34816;
        }
        50 => {
            max_framesize = 22080;
            max_dpb_size = 110400;
        }
        51 => {
            max_framesize = 36864;
            max_dpb_size = 184320;
        }
        52 => {
            max_framesize = 36864;
            max_dpb_size = 184320;
        }
        0 => {
            max_framesize = 99;
            max_dpb_size = 396;
        }
        9 => {
            // Level 1B
            max_framesize = 99;
            max_dpb_size = 396;
        }

        // Level 6 onwards doesn't seem to be much supported, but it is 4k video
        60 => {
            max_framesize = 139264;
            max_dpb_size = 696320;
        }
        61 => {
            max_framesize = 139264;
            max_dpb_size = 696320;
        }
        62 => {
            max_framesize = 139264;
            max_dpb_size = 696320;
        }

        _ => {
            max_framesize = 3600;
            max_dpb_size = 18000;
        }
    };

    (max_framesize, max_dpb_size)
}

fn random_pic_size_and_max_num_ref_frames(
    sps: &mut SeqParameterSet,
    small_video: bool,
    silent_mode: bool,
    max_fs_and_dpb_mbs: (u32, u32),
    rconfig: &RandomSPSRange,
    film: &mut FilmState,
) {
    let max_fs = max_fs_and_dpb_mbs.0;
    let max_dpb_mbs = max_fs_and_dpb_mbs.1;

    // first pic a random video width within the max_fs amount
    if !silent_mode {
        println!("\t\t profile_idc: {}", &sps.profile_idc);
        println!("\t\t level_idc: {}", &sps.level_idc);
    }

    // max width/height value borrowed from OpenH264
    let max_param: u32 = if small_video {
        7 // at most (7+1) * 16 = 128x128 pixels
    } else {
        (8f64 * max_fs as f64).sqrt() as u32
    };

    if rconfig.pic_width_in_mbs_minus1.max <= max_param {
        sps.pic_width_in_mbs_minus1 = rconfig.pic_width_in_mbs_minus1.sample(film);
    } else {
        if !silent_mode {
            println!(" [WARNING] rconfig.pic_width_in_mbs_minus1.max {} is greater than possible value for level {} : {}", rconfig.pic_width_in_mbs_minus1.max, sps.level_idc, max_param);
        }
        if rconfig.pic_width_in_mbs_minus1.min < max_param {
            sps.pic_width_in_mbs_minus1 = rconfig
                .pic_height_in_map_units_minus1
                .sample_custom_max(max_param, film);
        } else {
            if !silent_mode {
                println!(" [WARNING] rconfig.pic_width_in_mbs_minus1.min {} is greater than possible value for level {} : {}", rconfig.pic_width_in_mbs_minus1.min, sps.level_idc, max_param);
                println!(" [WARNING] Setting to max_param value {}", max_param);
            }
            sps.pic_width_in_mbs_minus1 = max_param;
        }
    }

    // to ensure we can store at least one complete frame, we choose a random height from the space we have left
    let max_height = cmp::min(
        match sps.pic_width_in_mbs_minus1 {
            0 => max_fs,
            _ => max_fs / sps.pic_width_in_mbs_minus1,
        },
        max_param,
    ); // avoid a divide by 0 error
    if rconfig.pic_height_in_map_units_minus1.max <= max_height {
        sps.pic_height_in_map_units_minus1 = rconfig.pic_height_in_map_units_minus1.sample(film);
    } else {
        if !silent_mode {
            println!(" [WARNING] rconfig.pic_height_in_map_units_minus1.max {} is greater than possible value for level {} : {}", rconfig.pic_height_in_map_units_minus1.max, sps.level_idc, max_height);
        }
        if rconfig.pic_height_in_map_units_minus1.min < max_height {
            sps.pic_height_in_map_units_minus1 = rconfig
                .pic_height_in_map_units_minus1
                .sample_custom_max(max_height, film);
        } else {
            if !silent_mode {
                println!(" [WARNING] rconfig.pic_height_in_map_units_minus1.min {} is greater than possible value for level {} : {}", rconfig.pic_height_in_map_units_minus1.min, sps.level_idc, max_height);
                println!(" [WARNING] Setting to max_heigh value {}", max_height);
            }
            sps.pic_height_in_map_units_minus1 = max_height;
        }
    }

    let pic_size = (sps.pic_width_in_mbs_minus1 + 1) * (sps.pic_height_in_map_units_minus1 + 1);

    // now we set the max number of reference frames based on how many times the frame size can go into the rest of the buffer
    sps.max_num_ref_frames = cmp::min(max_dpb_mbs / pic_size, 16);
}

/// SPS Scaling list randomization - only delta scale is user-generated
fn randomize_sps_scaling_list(
    delta_scaling_list: &mut Vec<i32>,
    size_of_scaling_list: usize,
    rconfig: &RandomSPSRange,
    film: &mut FilmState,
) {
    // not generating video, so we can just add random delta_scaling list values
    for _ in 0..size_of_scaling_list {
        delta_scaling_list.push(rconfig.delta_scale.sample(film));
    }
}

/// Pic Scaling list randomization - only delta scale is user-generated
fn randomize_pps_scaling_list(
    delta_scaling_list: &mut Vec<i32>,
    size_of_scaling_list: usize,
    rconfig: RandomPPSRange,
    film: &mut FilmState,
) {
    // not generating video, so we can just add random delta_scaling list values
    for _ in 0..size_of_scaling_list {
        delta_scaling_list.push(rconfig.delta_scale.sample(film));
    }
}

/// Generate a random VUI
fn random_vui(sps: &mut SeqParameterSet, rconfig: RandomVUIRange, film: &mut FilmState) {
    sps.vui_parameters.aspect_ratio_info_present_flag =
        rconfig.aspect_ratio_info_present_flag.sample(film);

    if sps.vui_parameters.aspect_ratio_info_present_flag {
        sps.vui_parameters.aspect_ratio_idc = rconfig.aspect_ratio_idc.sample(film) as u8;

        // see table E-1 for parsing aspect_ratio_idc
        if sps.vui_parameters.aspect_ratio_idc == 255 {
            // Extended_SAR
            sps.vui_parameters.sar_width = rconfig.sar_width.sample(film) as u16;
            sps.vui_parameters.sar_height = rconfig.sar_height.sample(film) as u16;
        }
    }

    sps.vui_parameters.overscan_info_present_flag = rconfig.overscan_info_present_flag.sample(film);

    if sps.vui_parameters.overscan_info_present_flag {
        sps.vui_parameters.overscan_appropriate_flag =
            rconfig.overscan_appropriate_flag.sample(film);
    }

    sps.vui_parameters.video_signal_type_present_flag =
        rconfig.video_signal_type_present_flag.sample(film);

    if sps.vui_parameters.video_signal_type_present_flag {
        sps.vui_parameters.video_format = rconfig.video_format.sample(film) as u8;
        sps.vui_parameters.video_full_range_flag = rconfig.video_full_range_flag.sample(film);
        sps.vui_parameters.colour_description_present_flag =
            rconfig.colour_description_present_flag.sample(film);

        if sps.vui_parameters.colour_description_present_flag {
            sps.vui_parameters.colour_primaries = rconfig.colour_primaries.sample(film) as u8;
            sps.vui_parameters.transfer_characteristics =
                rconfig.transfer_characteristics.sample(film) as u8;
            sps.vui_parameters.matrix_coefficients = rconfig.matrix_coefficients.sample(film) as u8;
        }
    }

    sps.vui_parameters.chroma_loc_info_present_flag =
        rconfig.chroma_loc_info_present_flag.sample(film);

    if sps.vui_parameters.chroma_loc_info_present_flag {
        sps.vui_parameters.chroma_sample_loc_type_top_field =
            rconfig.chroma_sample_loc_type_top_field.sample(film);
        sps.vui_parameters.chroma_sample_loc_type_bottom_field =
            rconfig.chroma_sample_loc_type_bottom_field.sample(film);
    }

    sps.vui_parameters.timing_info_present_flag = rconfig.timing_info_present_flag.sample(film);

    if sps.vui_parameters.timing_info_present_flag {
        sps.vui_parameters.num_units_in_tick = rconfig.num_units_in_tick.sample(film);
        sps.vui_parameters.time_scale = rconfig.time_scale.sample(film);
        sps.vui_parameters.fixed_frame_rate_flag = rconfig.fixed_frame_rate_flag.sample(film);
    }

    sps.vui_parameters.nal_hrd_parameters_present_flag =
        rconfig.nal_hrd_parameters_present_flag.sample(film);

    if sps.vui_parameters.nal_hrd_parameters_present_flag {
        sps.vui_parameters.nal_hrd_parameters =
            random_hrd_parameters(rconfig.random_hrd_range, film);
    }

    sps.vui_parameters.vcl_hrd_parameters_present_flag =
        rconfig.vcl_hrd_parameters_present_flag.sample(film);

    if sps.vui_parameters.vcl_hrd_parameters_present_flag {
        sps.vui_parameters.vcl_hrd_parameters =
            random_hrd_parameters(rconfig.random_hrd_range, film);
    }

    if sps.vui_parameters.nal_hrd_parameters_present_flag
        || sps.vui_parameters.vcl_hrd_parameters_present_flag
    {
        sps.vui_parameters.low_delay_hrd_flag = rconfig.low_delay_hrd_flag.sample(film);
    }

    sps.vui_parameters.pic_struct_present_flag = rconfig.pic_struct_present_flag.sample(film);
    sps.vui_parameters.bitstream_restriction_flag = rconfig.bitstream_restriction_flag.sample(film);

    if sps.vui_parameters.bitstream_restriction_flag {
        sps.vui_parameters.motion_vectors_over_pic_boundaries_flag =
            rconfig.motion_vectors_over_pic_boundaries_flag.sample(film);
        sps.vui_parameters.max_bytes_per_pic_denom = rconfig.max_bytes_per_pic_denom.sample(film);
        sps.vui_parameters.max_bits_per_mb_denom = rconfig.max_bits_per_mb_denom.sample(film);
        sps.vui_parameters.log2_max_mv_length_horizontal =
            rconfig.log2_max_mv_length_horizontal.sample(film);
        sps.vui_parameters.log2_max_mv_length_vertical =
            rconfig.log2_max_mv_length_vertical.sample(film);
        sps.vui_parameters.max_num_reorder_frames = rconfig.max_num_reorder_frames.sample(film);
        // the below cannot be more than the max already defined within SPS
        sps.vui_parameters.max_dec_frame_buffering =
            rconfig
                .max_dec_frame_buffering
                .sample(0, sps.max_num_ref_frames, film);
    }
}

/// Generate a random Sequence Parameter Set
pub fn random_sps(
    sps: &mut SeqParameterSet,
    enable_extensions: bool,
    rconfig: &RandomSPSRange,
    small_video: bool,
    silent_mode: bool,
    film: &mut FilmState,
) {
    sps.available = true;

    sps.profile_idc = if enable_extensions {
        rconfig.profile_idc_extension.sample(film) as u8
    } else {
        rconfig.profile_idc.sample(film) as u8
    };
    sps.constraint_set0_flag = rconfig.constraint_set0_flag.sample(film);
    sps.constraint_set1_flag = rconfig.constraint_set1_flag.sample(film);
    sps.constraint_set2_flag = rconfig.constraint_set2_flag.sample(film);
    sps.constraint_set3_flag = rconfig.constraint_set3_flag.sample(film);
    sps.constraint_set4_flag = rconfig.constraint_set4_flag.sample(film);
    sps.constraint_set5_flag = rconfig.constraint_set5_flag.sample(film);
    sps.reserved_zero_2bits = rconfig.reserved_zero_2bits.sample(film) as u8;
    let max_fs_and_dpb_mbs = random_level_idc(sps, rconfig, film);
    sps.seq_parameter_set_id = rconfig.seq_parameter_set_id.sample(film);

    if !silent_mode {
        println!(
            "\t\t SPS seq_parameter_set_id: {}",
            sps.seq_parameter_set_id
        );
    }

    if sps.profile_idc == 100
        || sps.profile_idc == 110
        || sps.profile_idc == 122
        || sps.profile_idc == 244
        || sps.profile_idc == 44
        || sps.profile_idc == 83
        || sps.profile_idc == 86
        || sps.profile_idc == 118
        || sps.profile_idc == 128
        || sps.profile_idc == 138
        || sps.profile_idc == 139
        || sps.profile_idc == 134
        || sps.profile_idc == 135
    {
        sps.chroma_format_idc = rconfig.chroma_format_idc.sample(film) as u8;

        if sps.chroma_format_idc == 3 {
            sps.separate_colour_plane_flag = rconfig.separate_colour_plane_flag.sample(film);
        }

        // Many decoders do not like differing bit depths so we bias towards them being the same
        if rconfig.bias_same_bit_depth.sample(film) {
            let bit_depth = rconfig.bit_depth_luma_minus8.sample(film) as u8;
            sps.bit_depth_luma_minus8 = bit_depth;
            sps.bit_depth_chroma_minus8 = bit_depth;
        } else {
            if !silent_mode {
                println!("\t\t SPS with different bit_depth values");
            }
            sps.bit_depth_luma_minus8 = rconfig.bit_depth_luma_minus8.sample(film) as u8;
            sps.bit_depth_chroma_minus8 = rconfig.bit_depth_chroma_minus8.sample(film) as u8;
        }

        sps.qpprime_y_zero_transform_bypass_flag =
            rconfig.qpprime_y_zero_transform_bypass_flag.sample(film);
        sps.seq_scaling_matrix_present_flag = rconfig.seq_scaling_matrix_present_flag.sample(film);

        if sps.seq_scaling_matrix_present_flag {
            let cur_max = match sps.chroma_format_idc != 3 {
                true => 8,
                _ => 12,
            };

            for i in 0..cur_max {
                sps.seq_scaling_list_present_flag
                    .push(rconfig.seq_scaling_list_present_flag.sample(film));

                // ensure that each i has a value
                sps.delta_scale_4x4.push(Vec::new());
                sps.scaling_list_4x4.push(Vec::new());
                sps.use_default_scaling_matrix_4x4.push(false);

                sps.delta_scale_8x8.push(Vec::new());
                sps.scaling_list_8x8.push(Vec::new());
                sps.use_default_scaling_matrix_8x8.push(false);

                if sps.seq_scaling_list_present_flag[i] {
                    if i < 6 {
                        randomize_sps_scaling_list(&mut sps.delta_scale_4x4[i], 16, rconfig, film);
                    } else {
                        randomize_sps_scaling_list(&mut sps.delta_scale_8x8[i], 64, rconfig, film);
                    }
                }
            }
        }
    }

    sps.log2_max_frame_num_minus4 = rconfig.log2_max_frame_num_minus4.sample(film);
    sps.pic_order_cnt_type = rconfig.pic_order_cnt_type.sample(film);

    if sps.pic_order_cnt_type == 0 {
        sps.log2_max_pic_order_cnt_lsb_minus4 =
            rconfig.log2_max_pic_order_cnt_lsb_minus4.sample(film) as u8;
    } else if sps.pic_order_cnt_type == 1 {
        sps.delta_pic_order_always_zero_flag =
            rconfig.delta_pic_order_always_zero_flag.sample(film);
        sps.offset_for_non_ref_pic = rconfig.offset_for_non_ref_pic.sample(film);
        sps.offset_for_top_to_bottom_field = rconfig.offset_for_top_to_bottom_field.sample(film);
        sps.num_ref_frames_in_pic_order_cnt_cycle =
            rconfig.num_ref_frames_in_pic_order_cnt_cycle.sample(film);
        for _ in 0..sps.num_ref_frames_in_pic_order_cnt_cycle {
            sps.offset_for_ref_frame
                .push(rconfig.offset_for_ref_frame.sample(film));
        }
    }

    //sps.max_num_ref_frames = rconfig.max_num_ref_frames.sample(film);
    sps.gaps_in_frame_num_value_allowed_flag =
        rconfig.gaps_in_frame_num_value_allowed_flag.sample(film);
    //sps.pic_width_in_mbs_minus1 = rconfig.pic_width_in_mbs_minus1.sample(film);
    //sps.pic_height_in_map_units_minus1 = rconfig.pic_height_in_map_units_minus1.sample(film);
    random_pic_size_and_max_num_ref_frames(
        sps,
        small_video,
        silent_mode,
        max_fs_and_dpb_mbs,
        rconfig,
        film,
    );
    sps.frame_mbs_only_flag = rconfig.frame_mbs_only_flag.sample(film);

    if !sps.frame_mbs_only_flag {
        sps.mb_adaptive_frame_field_flag = rconfig.mb_adaptive_frame_field_flag.sample(film);
    }

    sps.direct_8x8_inference_flag = rconfig.direct_8x8_inference_flag.sample(film);
    sps.frame_cropping_flag = rconfig.frame_cropping_flag.sample(film);
    if sps.frame_cropping_flag {
        // set these to be in line with the current macroblock values
        let width = sps.pic_width_in_mbs_minus1 + 1;
        let height = sps.pic_height_in_map_units_minus1 + 1;
        sps.frame_crop_left_offset = rconfig.frame_crop_left_offset.sample(0, width, film);
        sps.frame_crop_right_offset = rconfig.frame_crop_right_offset.sample(0, width, film);
        sps.frame_crop_top_offset = rconfig.frame_crop_top_offset.sample(0, height, film);
        sps.frame_crop_bottom_offset = rconfig.frame_crop_bottom_offset.sample(0, height, film);
    }

    let width_in_pixels;
    let height_in_pixels;

    // check for underflow
    if ((sps.pic_width_in_mbs_minus1 as i64 + 1) * 16)
        - 2 * (sps.frame_crop_left_offset as i64 + sps.frame_crop_right_offset as i64)
        < 0
    {
        if !silent_mode {
            println!("\t\t [WARNING] Underflowing horizontal frame cropping - ignoring cropping in picture size calculation");
        }
        width_in_pixels = (sps.pic_width_in_mbs_minus1 + 1) * 16;
    } else {
        // check for overflow
        if 2 * (sps.frame_crop_left_offset as i64 + sps.frame_crop_right_offset as i64)
            > (std::i32::MAX as i64)
        {
            if !silent_mode {
                println!("\t\t [WARNING] Overflowing horizontal frame cropping - ignoring cropping in picture size calculation");
            }
            width_in_pixels = (sps.pic_width_in_mbs_minus1 + 1) * 16;
        } else {
            width_in_pixels = ((sps.pic_width_in_mbs_minus1 + 1) * 16)
                - 2 * (sps.frame_crop_left_offset + sps.frame_crop_right_offset);
        }
    }

    if ((sps.pic_height_in_map_units_minus1 as i64 + 1) * 16)
        - 2 * (sps.frame_crop_top_offset as i64 + sps.frame_crop_bottom_offset as i64)
        < 0
    {
        if !silent_mode {
            println!("\t\t [WARNING] Underflowing vertical frame cropping - ignoring cropping in picture size calculation");
        }
        height_in_pixels = (sps.pic_height_in_map_units_minus1 + 1) * 16;
    } else {
        if 2 * (sps.frame_crop_top_offset as i64 + sps.frame_crop_bottom_offset as i64)
            > (std::i32::MAX as i64)
        {
            if !silent_mode {
                println!("\t\t [WARNING] Overflowing vertical frame cropping - ignoring cropping in picture size calculation");
            }
            height_in_pixels = (sps.pic_height_in_map_units_minus1 + 1) * 16;
        } else {
            height_in_pixels = ((sps.pic_height_in_map_units_minus1 + 1) * 16)
                - 2 * (sps.frame_crop_top_offset + sps.frame_crop_bottom_offset);
        }
    }

    if !silent_mode {
        println!(
            "\t\t Total size : {}x{} = {} ",
            width_in_pixels,
            height_in_pixels,
            width_in_pixels * height_in_pixels
        );
    }
    sps.vui_parameters_present_flag = rconfig.vui_parameters_present_flag.sample(film);

    if sps.vui_parameters_present_flag {
        sps.vui_parameters = VUIParameters::new();
        random_vui(sps, rconfig.random_vui_range, film);
    }
}

/// Generate a random Picture Parameter Set
pub fn random_pps(
    pps_idx: usize,
    sps: &SeqParameterSet,
    rconfig: RandomPPSRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    ds.ppses[pps_idx].available = true;

    ds.ppses[pps_idx].pic_parameter_set_id = rconfig.pic_parameter_set_id.sample(film);
    //println!(
    //    "\t\t PPS pic_parameter_set_id: {}",
    //    ds.ppses[pps_idx].pic_parameter_set_id
    //);
    ds.ppses[pps_idx].seq_parameter_set_id = sps.seq_parameter_set_id;

    ds.ppses[pps_idx].entropy_coding_mode_flag = rconfig.entropy_coding_mode_flag.sample(film);
    ds.ppses[pps_idx].bottom_field_pic_order_in_frame_present_flag = rconfig
        .bottom_field_pic_order_in_frame_present_flag
        .sample(film);

    // ignore slice groups 66% of the time
    if rconfig.bias_ignore_slice_groups.sample(film) {
        ds.ppses[pps_idx].num_slice_groups_minus1 = 0;
    } else {
        ds.ppses[pps_idx].num_slice_groups_minus1 = rconfig.num_slice_groups_minus1.sample(film);
    }

    if ds.ppses[pps_idx].num_slice_groups_minus1 > 0 {
        ds.ppses[pps_idx].slice_group_map_type = rconfig.slice_group_map_type.sample(film);

        if ds.ppses[pps_idx].slice_group_map_type == 0 {
            for _ in 0..=ds.ppses[pps_idx].num_slice_groups_minus1 {
                ds.ppses[pps_idx]
                    .run_length_minus1
                    .push(rconfig.run_length_minus1.sample(film));
            }
        } else if ds.ppses[pps_idx].slice_group_map_type == 2 {
            for _ in 0..ds.ppses[pps_idx].num_slice_groups_minus1 {
                ds.ppses[pps_idx]
                    .top_left
                    .push(rconfig.top_left.sample(film));
                ds.ppses[pps_idx]
                    .bottom_right
                    .push(rconfig.bottom_right.sample(film));
            }
        } else if ds.ppses[pps_idx].slice_group_map_type == 3
            || ds.ppses[pps_idx].slice_group_map_type == 4
            || ds.ppses[pps_idx].slice_group_map_type == 5
        {
            ds.ppses[pps_idx].slice_group_change_direction_flag =
                rconfig.slice_group_change_direction_flag.sample(film);
            ds.ppses[pps_idx].slice_group_change_rate_minus1 =
                rconfig.slice_group_change_rate_minus1.sample(film);
        } else if ds.ppses[pps_idx].slice_group_map_type == 6 {
            ds.ppses[pps_idx].pic_size_in_map_units_minus1 =
                rconfig.pic_size_in_map_units_minus1.sample(film);

            for _ in 0..=ds.ppses[pps_idx].pic_size_in_map_units_minus1 {
                ds.ppses[pps_idx]
                    .slice_group_id
                    .push(rconfig.slice_group_id.sample(film));
            }
        }
    }

    ds.ppses[pps_idx].num_ref_idx_l0_default_active_minus1 =
        rconfig.num_ref_idx_l0_default_active_minus1.sample(film);
    ds.ppses[pps_idx].num_ref_idx_l1_default_active_minus1 =
        rconfig.num_ref_idx_l1_default_active_minus1.sample(film);
    ds.ppses[pps_idx].weighted_pred_flag = rconfig.weighted_pred_flag.sample(film);
    ds.ppses[pps_idx].weighted_bipred_idc = rconfig.weighted_bipred_idc.sample(film) as u8;
    ds.ppses[pps_idx].pic_init_qp_minus26 = rconfig.pic_init_qp_minus26.sample(film);
    ds.ppses[pps_idx].pic_init_qs_minus26 = rconfig.pic_init_qs_minus26.sample(film);
    ds.ppses[pps_idx].chroma_qp_index_offset = rconfig.chroma_qp_index_offset.sample(film);
    ds.ppses[pps_idx].deblocking_filter_control_present_flag =
        rconfig.deblocking_filter_control_present_flag.sample(film);
    ds.ppses[pps_idx].constrained_intra_pred_flag =
        rconfig.constrained_intra_pred_flag.sample(film);
    ds.ppses[pps_idx].redundant_pic_cnt_present_flag =
        rconfig.redundant_pic_cnt_present_flag.sample(film);

    ds.ppses[pps_idx].more_data_flag = rconfig.include_more_data.sample(film);

    // rbsp_more_data()
    if ds.ppses[pps_idx].more_data_flag {
        ds.ppses[pps_idx].transform_8x8_mode_flag = rconfig.transform_8x8_mode_flag.sample(film);
        ds.ppses[pps_idx].pic_scaling_matrix_present_flag =
            rconfig.pic_scaling_matrix_present_flag.sample(film);

        if ds.ppses[pps_idx].pic_scaling_matrix_present_flag {
            let max_val = 6
                + ((match sps.chroma_format_idc != 3 {
                    true => 2,
                    false => 6,
                }) * (match ds.ppses[pps_idx].transform_8x8_mode_flag {
                    true => 1,
                    false => 0,
                }));
            for i in 0..max_val {
                ds.ppses[pps_idx]
                    .pic_scaling_list_present_flag
                    .push(rconfig.pic_scaling_list_present_flag.sample(film));

                // ensure that each i has a value
                ds.ppses[pps_idx].delta_scale_4x4.push(Vec::new());
                ds.ppses[pps_idx].scaling_list_4x4.push(Vec::new());
                ds.ppses[pps_idx].use_default_scaling_matrix_4x4.push(false);
                ds.ppses[pps_idx].delta_scale_8x8.push(Vec::new());
                ds.ppses[pps_idx].scaling_list_8x8.push(Vec::new());
                ds.ppses[pps_idx].use_default_scaling_matrix_8x8.push(false);

                if ds.ppses[pps_idx].pic_scaling_list_present_flag[i] {
                    if i < 6 {
                        randomize_pps_scaling_list(
                            &mut ds.ppses[pps_idx].delta_scale_4x4[i],
                            16,
                            rconfig,
                            film,
                        )
                    } else {
                        randomize_pps_scaling_list(
                            &mut ds.ppses[pps_idx].delta_scale_8x8[i],
                            64,
                            rconfig,
                            film,
                        )
                    }
                }
            }
        }

        ds.ppses[pps_idx].second_chroma_qp_index_offset =
            rconfig.second_chroma_qp_index_offset.sample(film);
    }
}

/// Generate a random Subset SPS
pub fn random_subset_sps(
    subset_sps_idx: usize,
    enable_extensions: bool,
    rconfig: &RandomSubsetSPSRange,
    small_video: bool,
    silent_mode: bool,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    random_sps(
        &mut ds.subset_spses[subset_sps_idx].sps,
        enable_extensions,
        &rconfig.random_sps_range,
        small_video,
        silent_mode,
        film,
    );

    // TODO: SVC/MVC/3DAVC extensions
    if ds.subset_spses[subset_sps_idx].sps.profile_idc == 83
        || ds.subset_spses[subset_sps_idx].sps.profile_idc == 86
    {
        let chroma_array_type = match ds.subset_spses[subset_sps_idx].sps.separate_colour_plane_flag {
            true => ds.subset_spses[subset_sps_idx].sps.chroma_format_idc,
            false => 0,
        };
        random_sps_svc_extension(chroma_array_type, subset_sps_idx, rconfig.random_sps_svc_range, ds, film); // specified in Annex G
        ds.subset_spses[subset_sps_idx].svc_vui_parameters_present_flag = rconfig.svc_vui_parameters_present_flag.sample(film);
        if ds.subset_spses[subset_sps_idx].svc_vui_parameters_present_flag {
            random_vui_svc_parameters(subset_sps_idx, rconfig.random_svc_vui_range, ds, film); // specified in Annex G
        }
    } else if ds.subset_spses[subset_sps_idx].sps.profile_idc == 118
        || ds.subset_spses[subset_sps_idx].sps.profile_idc == 128
        || ds.subset_spses[subset_sps_idx].sps.profile_idc == 134
    {
        ds.subset_spses[subset_sps_idx].bit_equal_to_one =
            rconfig.bit_equal_to_one.sample(film) as u8;
        random_sps_mvc_extension(subset_sps_idx, rconfig.random_sps_mvc_range, ds, film); // specified in Annex H
        ds.subset_spses[subset_sps_idx].mvc_vui_parameters_present_flag =
            rconfig.mvc_vui_parameters_present_flag.sample(film);

        if ds.subset_spses[subset_sps_idx].mvc_vui_parameters_present_flag {
            random_vui_mvc_parameters(subset_sps_idx, rconfig.random_mvc_vui_range, ds, film);
            // specified in Annex H
        }
    } else if ds.subset_spses[subset_sps_idx].sps.profile_idc == 138 || ds.subset_spses[subset_sps_idx].sps.profile_idc == 135
    {
        //ds.subset_spses[subset_sps_idx].bit_equal_to_one = rconfig.bit_equal_to_one.sample(film);
        //random_sps_mvcd_extension(&ds.subset_spses[subset_sps_idx].sps_mvcd); // specified in Annex I
        println!("TODO: random_sps_mvcd_extension");
    } else if ds.subset_spses[subset_sps_idx].sps.profile_idc == 139 {
        //ds.subset_spses[subset_sps_idx].bit_equal_to_one = rconfig.bit_equal_to_one.sample(film);
        //random_sps_mvcd_extension(&ds.subset_spses[subset_sps_idx].sps_mvcd); // specified in Annex I
        //random_sps_3davc_extension(&ds.subset_spses[subset_sps_idx].sps_3davc); // specified in Annex J
        println!("TODO: random_sps_mvcd_extension and random_sps_3davc_extension");
    }

    let num_additional_extension2_flag = rconfig.num_additional_extension2_flag.sample(film);
    for _ in 0..num_additional_extension2_flag {
        ds.subset_spses[subset_sps_idx]
            .additional_extension2_flag
            .push(rconfig.additional_extension2_flag.sample(film));
    }
}

/// Generate a random SPS Extension
pub fn random_sps_extension(
    sps_ext_idx: usize,
    sps_idx: usize,
    rconfig: &RandomSPSExtensionRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    ds.sps_extensions[sps_ext_idx].seq_parameter_set_id = ds.spses[sps_idx].seq_parameter_set_id;
    ds.sps_extensions[sps_ext_idx].aux_format_idc = rconfig.aux_format_idc.sample(film);

    if ds.sps_extensions[sps_ext_idx].aux_format_idc != 0 {
        ds.sps_extensions[sps_ext_idx].bit_depth_aux_minus8 = rconfig.bit_depth_aux_minus8.sample(film);
        ds.sps_extensions[sps_ext_idx].alpha_incr_flag = rconfig.alpha_incr_flag.sample(film);
        ds.sps_extensions[sps_ext_idx].alpha_opaque_value = rconfig.alpha_opaque_value.sample(film);
        ds.sps_extensions[sps_ext_idx].alpha_transparent_value = rconfig.alpha_transparent_value.sample(film);
    }

    ds.sps_extensions[sps_ext_idx].additional_extension_flag = rconfig.additional_extension_flag.sample(film);
}

fn random_sps_svc_extension(
    chroma_array_type: u8,
    subset_sps_idx: usize,
    rconfig: RandomSPSSVCExtensionRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    ds.subset_spses[subset_sps_idx].sps_svc.inter_layer_deblocking_filter_control_present_flag = rconfig.inter_layer_deblocking_filter_control_present_flag.sample(film);
    ds.subset_spses[subset_sps_idx].sps_svc.extended_spatial_scalability_idc = rconfig.extended_spatial_scalability_idc.sample(film) as u8;
    if chroma_array_type == 1 || chroma_array_type == 2 {
        ds.subset_spses[subset_sps_idx].sps_svc.chroma_phase_x_plus1_flag = rconfig.chroma_phase_x_plus1_flag.sample(film);
    }

    if chroma_array_type == 1 {
        ds.subset_spses[subset_sps_idx].sps_svc.chroma_phase_y_plus1 = rconfig.chroma_phase_y_plus1.sample(film) as u8;
    }

    if ds.subset_spses[subset_sps_idx].sps_svc.extended_spatial_scalability_idc == 1 {
        if chroma_array_type > 0 {
            ds.subset_spses[subset_sps_idx].sps_svc.seq_ref_layer_chroma_phase_x_plus1_flag = rconfig.seq_ref_layer_chroma_phase_x_plus1_flag.sample(film);
            ds.subset_spses[subset_sps_idx].sps_svc.seq_ref_layer_chroma_phase_y_plus1 = rconfig.seq_ref_layer_chroma_phase_y_plus1.sample(film) as u8;
        }
        ds.subset_spses[subset_sps_idx].sps_svc.seq_scaled_ref_layer_left_offset = rconfig.seq_scaled_ref_layer_left_offset.sample(film);
        ds.subset_spses[subset_sps_idx].sps_svc.seq_scaled_ref_layer_top_offset = rconfig.seq_scaled_ref_layer_top_offset.sample(film);
        ds.subset_spses[subset_sps_idx].sps_svc.seq_scaled_ref_layer_right_offset = rconfig.seq_scaled_ref_layer_right_offset.sample(film);
        ds.subset_spses[subset_sps_idx].sps_svc.seq_scaled_ref_layer_bottom_offset = rconfig.seq_scaled_ref_layer_bottom_offset.sample(film);
    }
    ds.subset_spses[subset_sps_idx].sps_svc.seq_tcoeff_level_prediction_flag = rconfig.seq_tcoeff_level_prediction_flag.sample(film);
    if ds.subset_spses[subset_sps_idx].sps_svc.seq_tcoeff_level_prediction_flag {
        ds.subset_spses[subset_sps_idx].sps_svc.adaptive_tcoeff_level_prediction_flag = rconfig.adaptive_tcoeff_level_prediction_flag.sample(film);
    }
    ds.subset_spses[subset_sps_idx].sps_svc.slice_header_restriction_flag = rconfig.slice_header_restriction_flag.sample(film);
}

fn random_vui_svc_parameters(
    subset_sps_idx: usize,
    rconfig: RandomVUISVCParametersRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_num_entries_minus1 = rconfig.vui_ext_num_entries_minus1.sample(film);

    for i in 0..=(ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_num_entries_minus1 as usize) {
        ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_dependency_id.push(rconfig.vui_ext_dependency_id.sample(film) as u8);
        ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_quality_id.push(rconfig.vui_ext_quality_id.sample(film) as u8);
        ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_temporal_id.push(rconfig.vui_ext_temporal_id.sample(film) as u8);
        ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_timing_info_present_flag.push(rconfig.vui_ext_timing_info_present_flag.sample(film));

        if ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_timing_info_present_flag[i] {
            ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_num_units_in_tick.push(rconfig.vui_ext_num_units_in_tick.sample(film));
            ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_time_scale.push(rconfig.vui_ext_time_scale.sample(film));
            ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_fixed_frame_rate_flag.push(rconfig.vui_ext_fixed_frame_rate_flag.sample(film));
        }

        ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_nal_hrd_parameters_present_flag.push(rconfig.vui_ext_nal_hrd_parameters_present_flag.sample(film));
        if ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_nal_hrd_parameters_present_flag[i] {
            ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_nal_hrd_parameters.push(random_hrd_parameters(rconfig.vui_ext_nal_hrd_parameters, film));
        }

        ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_vcl_hrd_parameters_present_flag.push(rconfig.vui_ext_vcl_hrd_parameters_present_flag.sample(film));
        if ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_vcl_hrd_parameters_present_flag[i] {
            ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_vcl_hrd_parameters.push(random_hrd_parameters(rconfig.vui_ext_vcl_hrd_parameters, film));
        }

        if ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_nal_hrd_parameters_present_flag[i] || ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_vcl_hrd_parameters_present_flag[i]
        {
            ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_low_delay_hrd_flag.push(rconfig.vui_ext_low_delay_hrd_flag.sample(film));
        }
        ds.subset_spses[subset_sps_idx].svc_vui.vui_ext_pic_struct_present_flag.push(rconfig.vui_ext_pic_struct_present_flag.sample(film));
    }

}

/// Generate a random SPS MVC Extension
fn random_sps_mvc_extension(
    subset_sps_idx: usize,
    rconfig: RandomSPSMVCExtensionRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    ds.subset_spses[subset_sps_idx].sps_mvc.num_views_minus1 =
        rconfig.num_views_minus1.sample(film) as usize;

    for _ in 0..=ds.subset_spses[subset_sps_idx].sps_mvc.num_views_minus1 {
        ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .view_id
            .push(rconfig.view_id.sample(film));
    }

    // 0th index
    ds.subset_spses[subset_sps_idx]
        .sps_mvc
        .num_anchor_refs_l0
        .push(0);
    ds.subset_spses[subset_sps_idx]
        .sps_mvc
        .anchor_refs_l0
        .push(Vec::new());
    ds.subset_spses[subset_sps_idx]
        .sps_mvc
        .num_anchor_refs_l1
        .push(0);
    ds.subset_spses[subset_sps_idx]
        .sps_mvc
        .anchor_refs_l1
        .push(Vec::new());

    // there are 1 to ds.subset_spses[subset_sps_idx].sps_mvc.num_views_minus1 values
    for i in 1..=ds.subset_spses[subset_sps_idx].sps_mvc.num_views_minus1 {
        ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .num_anchor_refs_l0
            .push(rconfig.num_anchor_refs_l0.sample(film));
        ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .anchor_refs_l0
            .push(Vec::new());
        for _ in 0..ds.subset_spses[subset_sps_idx].sps_mvc.num_anchor_refs_l0[i] {
            ds.subset_spses[subset_sps_idx].sps_mvc.anchor_refs_l0[i]
                .push(rconfig.anchor_refs_l0.sample(film));
        }
        ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .num_anchor_refs_l1
            .push(rconfig.num_anchor_refs_l1.sample(film));
        ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .anchor_refs_l1
            .push(Vec::new());
        for _ in 0..ds.subset_spses[subset_sps_idx].sps_mvc.num_anchor_refs_l1[i] {
            ds.subset_spses[subset_sps_idx].sps_mvc.anchor_refs_l1[i]
                .push(rconfig.anchor_refs_l1.sample(film));
        }
    }

    //0th index
    ds.subset_spses[subset_sps_idx]
        .sps_mvc
        .num_non_anchor_refs_l0
        .push(0);
    ds.subset_spses[subset_sps_idx]
        .sps_mvc
        .non_anchor_refs_l0
        .push(Vec::new());
    ds.subset_spses[subset_sps_idx]
        .sps_mvc
        .num_non_anchor_refs_l1
        .push(0);
    ds.subset_spses[subset_sps_idx]
        .sps_mvc
        .non_anchor_refs_l1
        .push(Vec::new());

    for i in 1..=ds.subset_spses[subset_sps_idx].sps_mvc.num_views_minus1 {
        ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .num_non_anchor_refs_l0
            .push(rconfig.num_non_anchor_refs_l0.sample(film));
        ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .non_anchor_refs_l0
            .push(Vec::new());
        for _ in 0..ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .num_non_anchor_refs_l0[i]
        {
            ds.subset_spses[subset_sps_idx].sps_mvc.non_anchor_refs_l0[i]
                .push(rconfig.non_anchor_refs_l0.sample(film));
        }
        ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .num_non_anchor_refs_l1
            .push(rconfig.num_non_anchor_refs_l1.sample(film));
        ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .non_anchor_refs_l1
            .push(Vec::new());
        for _ in 0..ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .num_non_anchor_refs_l1[i]
        {
            ds.subset_spses[subset_sps_idx].sps_mvc.non_anchor_refs_l1[i]
                .push(rconfig.non_anchor_refs_l1.sample(film));
        }
    }

    ds.subset_spses[subset_sps_idx]
        .sps_mvc
        .num_level_values_signalled_minus1 =
        rconfig.num_level_values_signalled_minus1.sample(film) as usize;
    for i in 0..=ds.subset_spses[subset_sps_idx]
        .sps_mvc
        .num_level_values_signalled_minus1
    {
        ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .level_idc
            .push(rconfig.level_idc.sample(film) as u8);
        ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .num_applicable_ops_minus1
            .push(rconfig.num_applicable_ops_minus1.sample(film) as usize);

        ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .applicable_op_temporal_id
            .push(Vec::new());
        ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .applicable_op_num_target_views_minus1
            .push(Vec::new());
        ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .applicable_op_target_view_id
            .push(Vec::new());
        ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .applicable_op_num_views_minus1
            .push(Vec::new());
        for j in 0..=ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .num_applicable_ops_minus1[i]
        {
            ds.subset_spses[subset_sps_idx]
                .sps_mvc
                .applicable_op_temporal_id[i]
                .push(rconfig.applicable_op_temporal_id.sample(film) as u8);
            ds.subset_spses[subset_sps_idx]
                .sps_mvc
                .applicable_op_num_target_views_minus1[i]
                .push(rconfig.applicable_op_num_target_views_minus1.sample(film));

            ds.subset_spses[subset_sps_idx]
                .sps_mvc
                .applicable_op_target_view_id[i]
                .push(Vec::new());
            //insert new
            for _ in 0..=ds.subset_spses[subset_sps_idx]
                .sps_mvc
                .applicable_op_num_target_views_minus1[i][j]
            {
                ds.subset_spses[subset_sps_idx]
                    .sps_mvc
                    .applicable_op_target_view_id[i][j as usize]
                    .push(rconfig.applicable_op_target_view_id.sample(film));
            }
            ds.subset_spses[subset_sps_idx]
                .sps_mvc
                .applicable_op_num_views_minus1[i]
                .push(rconfig.applicable_op_num_views_minus1.sample(film));
        }
    }

    if ds.subset_spses[subset_sps_idx].sps.profile_idc == 134 {
        ds.subset_spses[subset_sps_idx].sps_mvc.mfc_format_idc =
            rconfig.mfc_format_idc.sample(film) as u8;
        if ds.subset_spses[subset_sps_idx].sps_mvc.mfc_format_idc == 0
            || ds.subset_spses[subset_sps_idx].sps_mvc.mfc_format_idc == 1
        {
            ds.subset_spses[subset_sps_idx]
                .sps_mvc
                .default_grid_position_flag = rconfig.default_grid_position_flag.sample(film);
            if !ds.subset_spses[subset_sps_idx]
                .sps_mvc
                .default_grid_position_flag
            {
                ds.subset_spses[subset_sps_idx]
                    .sps_mvc
                    .view0_grid_position_x = rconfig.view0_grid_position_x.sample(film) as u8;
                ds.subset_spses[subset_sps_idx]
                    .sps_mvc
                    .view0_grid_position_y = rconfig.view0_grid_position_y.sample(film) as u8;
                ds.subset_spses[subset_sps_idx]
                    .sps_mvc
                    .view1_grid_position_x = rconfig.view1_grid_position_x.sample(film) as u8;
                ds.subset_spses[subset_sps_idx]
                    .sps_mvc
                    .view1_grid_position_y = rconfig.view1_grid_position_y.sample(film) as u8;
            }
        }
        ds.subset_spses[subset_sps_idx]
            .sps_mvc
            .rpu_filter_enabled_flag = rconfig.rpu_filter_enabled_flag.sample(film);
        if !ds.subset_spses[subset_sps_idx].sps.frame_mbs_only_flag {
            ds.subset_spses[subset_sps_idx]
                .sps_mvc
                .rpu_field_processing_flag = rconfig.rpu_field_processing_flag.sample(film);
        }
    }
}

/// Generate a random SPS MVC VUI Extension
fn random_vui_mvc_parameters(
    subset_sps_idx: usize,
    rconfig: RandomVUIMVCParametersRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    ds.subset_spses[subset_sps_idx]
        .mvc_vui
        .vui_mvc_num_ops_minus1 = rconfig.vui_mvc_num_ops_minus1.sample(film);

    for i in 0..=(ds.subset_spses[subset_sps_idx]
        .mvc_vui
        .vui_mvc_num_ops_minus1 as usize)
    {
        ds.subset_spses[subset_sps_idx]
            .mvc_vui
            .vui_mvc_temporal_id
            .push(rconfig.vui_mvc_temporal_id.sample(film) as u8);

        ds.subset_spses[subset_sps_idx]
            .mvc_vui
            .vui_mvc_num_target_output_views_minus1
            .push(rconfig.vui_mvc_num_target_output_views_minus1.sample(film));

        ds.subset_spses[subset_sps_idx]
            .mvc_vui
            .vui_mvc_view_id
            .push(Vec::new());
        for _ in 0..=(ds.subset_spses[subset_sps_idx]
            .mvc_vui
            .vui_mvc_num_target_output_views_minus1[i] as usize)
        {
            ds.subset_spses[subset_sps_idx].mvc_vui.vui_mvc_view_id[i]
                .push(rconfig.vui_mvc_view_id.sample(film));
        }

        ds.subset_spses[subset_sps_idx]
            .mvc_vui
            .vui_mvc_timing_info_present_flag
            .push(rconfig.vui_mvc_timing_info_present_flag.sample(film));

        if ds.subset_spses[subset_sps_idx]
            .mvc_vui
            .vui_mvc_timing_info_present_flag[i]
        {
            ds.subset_spses[subset_sps_idx]
                .mvc_vui
                .vui_mvc_num_units_in_tick
                .push(rconfig.vui_mvc_num_units_in_tick.sample(film));
            ds.subset_spses[subset_sps_idx]
                .mvc_vui
                .vui_mvc_time_scale
                .push(rconfig.vui_mvc_time_scale.sample(film));
            ds.subset_spses[subset_sps_idx]
                .mvc_vui
                .vui_mvc_fixed_frame_rate_flag
                .push(rconfig.vui_mvc_fixed_frame_rate_flag.sample(film));
        } else {
            ds.subset_spses[subset_sps_idx]
                .mvc_vui
                .vui_mvc_num_units_in_tick
                .push(0);
            ds.subset_spses[subset_sps_idx]
                .mvc_vui
                .vui_mvc_time_scale
                .push(0);
            ds.subset_spses[subset_sps_idx]
                .mvc_vui
                .vui_mvc_fixed_frame_rate_flag
                .push(false);
        }

        ds.subset_spses[subset_sps_idx]
            .mvc_vui
            .vui_mvc_nal_hrd_parameters_present_flag
            .push(rconfig.vui_mvc_nal_hrd_parameters_present_flag.sample(film));
        if ds.subset_spses[subset_sps_idx]
            .mvc_vui
            .vui_mvc_nal_hrd_parameters_present_flag[i]
        {
            ds.subset_spses[subset_sps_idx]
                .mvc_vui
                .vui_mvc_nal_hrd_parameters
                .push(random_hrd_parameters(
                    rconfig.vui_mvc_nal_hrd_parameters,
                    film,
                ));
        } else {
            // push empty HRD_parameters
            ds.subset_spses[subset_sps_idx]
                .mvc_vui
                .vui_mvc_nal_hrd_parameters
                .push(HRDParameters::new());
        }

        ds.subset_spses[subset_sps_idx]
            .mvc_vui
            .vui_mvc_vcl_hrd_parameters_present_flag
            .push(rconfig.vui_mvc_vcl_hrd_parameters_present_flag.sample(film));
        if ds.subset_spses[subset_sps_idx]
            .mvc_vui
            .vui_mvc_vcl_hrd_parameters_present_flag[i]
        {
            ds.subset_spses[subset_sps_idx]
                .mvc_vui
                .vui_mvc_vcl_hrd_parameters
                .push(random_hrd_parameters(
                    rconfig.vui_mvc_vcl_hrd_parameters,
                    film,
                ));
        } else {
            // push empty HRD_parameters
            ds.subset_spses[subset_sps_idx]
                .mvc_vui
                .vui_mvc_vcl_hrd_parameters
                .push(HRDParameters::new());
        }

        if ds.subset_spses[subset_sps_idx]
            .mvc_vui
            .vui_mvc_nal_hrd_parameters_present_flag[i]
            || ds.subset_spses[subset_sps_idx]
                .mvc_vui
                .vui_mvc_vcl_hrd_parameters_present_flag[i]
        {
            ds.subset_spses[subset_sps_idx]
                .mvc_vui
                .vui_mvc_low_delay_hrd_flag
                .push(rconfig.vui_mvc_low_delay_hrd_flag.sample(film));
        } else {
            // push false
            ds.subset_spses[subset_sps_idx]
                .mvc_vui
                .vui_mvc_low_delay_hrd_flag
                .push(false);
        }

        ds.subset_spses[subset_sps_idx]
            .mvc_vui
            .vui_mvc_pic_struct_present_flag
            .push(rconfig.vui_mvc_pic_struct_present_flag.sample(film));
    }
}

/// Generate a random HRD parameters
fn random_hrd_parameters(rconfig: RandomHRDRange, film: &mut FilmState) -> HRDParameters {
    let mut hrd_param = HRDParameters::new();

    hrd_param.cpb_cnt_minus1 = rconfig.cpb_cnt_minus1.sample(film);
    hrd_param.bit_rate_scale = rconfig.bit_rate_scale.sample(film) as u8;
    hrd_param.cpb_size_scale = rconfig.cpb_size_scale.sample(film) as u8;

    for _ in 0..=hrd_param.cpb_cnt_minus1 {
        hrd_param
            .bit_rate_value_minus1
            .push(rconfig.bit_rate_value_minus1.sample(film));
        hrd_param
            .cpb_size_values_minus1
            .push(rconfig.cpb_size_value_minus1.sample(film));
        hrd_param.cbr_flag.push(rconfig.cbr_flag.sample(film));
    }

    hrd_param.initial_cpb_removal_delay_length_minus1 =
        rconfig.initial_cpb_removal_delay_length_minus1.sample(film) as u8;
    hrd_param.cpb_removal_delay_length_minus1 =
        rconfig.cpb_removal_delay_length_minus1.sample(film) as u8;
    hrd_param.dpb_output_delay_length_minus1 =
        rconfig.dpb_output_delay_length_minus1.sample(film) as u8;
    hrd_param.time_offset_length = rconfig.time_offset_length.sample(film) as u8;

    hrd_param
}
