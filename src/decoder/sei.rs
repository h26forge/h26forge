//! SEI syntax element decoding.

use crate::common::data_structures::SEIBufferingPeriod;
use crate::common::data_structures::SEIFilmGrainCharacteristics;
use crate::common::data_structures::SEINalu;
use crate::common::data_structures::SEIPayload;
use crate::common::data_structures::SEIPicTiming;
use crate::common::data_structures::SEIRecoveryPoint;
use crate::common::data_structures::SEIUserDataUnregistered;
use crate::common::data_structures::SeqParameterSet;
use crate::common::data_structures::UUID_APPLE1;
use crate::common::data_structures::UUID_APPLE2;
use crate::common::helper::decoder_formatted_print;
use crate::common::helper::ByteStream;
use crate::decoder::expgolomb::exp_golomb_decode_one_wrapper;
use log::debug;
use std::collections::VecDeque;

/// Follows section 7.3.2.3
pub fn decode_sei_message(spses: &Vec<SeqParameterSet>, bs: &mut ByteStream) -> SEINalu {
    let mut res = SEINalu::new();

    if bs.bytestream.len() < 2 {
        println!("[WARNING] SEI NALU too short: {}", bs.bytestream.len());
        return res;
    }

    loop {
        let mut payload_type = 0;
        let mut next_byte = bs.read_bits(8);

        while next_byte == 0xff {
            next_byte = bs.read_bits(8);
            payload_type += 255;
        }
        payload_type += next_byte;

        let mut payload_size = 0;
        let mut next_byte = bs.read_bits(8);

        while next_byte == 0xff {
            next_byte = bs.read_bits(8);
            payload_size += 255;
        }
        // in number of bytes
        payload_size += next_byte;
        res.payload_type.push(payload_type);
        res.payload_size.push(payload_size);
        res.payload
            .push(decode_sei_payload(payload_type, payload_size, spses, bs));
        // if we don't have enough bytes for a new payload and size, then break
        if !bs.more_data() && bs.bytestream.len() < 2 {
            break;
        }
    }

    return res;
}

/// D.1.1 General SEI message syntax
fn decode_sei_payload(
    payload_type: u32,
    payload_size: u32,
    spses: &Vec<SeqParameterSet>,
    bs: &mut ByteStream,
) -> SEIPayload {
    let mut res = SEIPayload::new();
    println!("\t\tSEI: payload_type {}", payload_type);
    println!("\t\tSEI: payload_size {}", payload_size);

    decoder_formatted_print("SEI: payload_type", &payload_type, 63);
    decoder_formatted_print("SEI: payload_size", &payload_size, 63);
    // this currently only matches the types of a target decoder
    match payload_type {
        0 => {
            // buffering period
            res.buffering_period = decode_buffering_period(spses, bs);
            res.available = true;
        }
        1 => {
            // picture timing
            let sps: SeqParameterSet;
            if spses.len() > 0 {
                sps = spses[spses.len() - 1].clone();
            } else {
                sps = SeqParameterSet::new();
            }
            res.pic_timing = decode_pic_timing(&sps, bs);
            res.available = true;
        }
        2 => {
            // pan scan rect
            decode_pan_scan_rect(payload_size, bs);
        }
        3 => {
            // filler payload
            decode_filler_payload(payload_size, bs);
        }
        4 => {
            // user data registered ITU T T35
            decode_user_data_registered_itu_t_t35(payload_size, bs);
        }
        5 => {
            // Unregistered user data
            res.unregistered_user_data = decode_user_data_unregistered(payload_size, bs);
            res.available = true;
        }
        6 => {
            // recovery point
            res.recovery_point = decode_recovery_point(bs);
            res.available = true;
        }
        7 => {
            // pic marking repetition
            decode_ref_pic_marking_repetition(payload_size, bs);
        }
        8 => {
            // spare pic
            decode_spare_pic(payload_size, bs);
        }
        9 => {
            // scene info
            decode_scene_info(payload_size, bs);
        }
        10 => {
            // sub seq info
            decode_sub_seq_info(payload_size, bs);
        }
        11 => {
            // sub seq layer characteristics
            decode_sub_seq_layer_characteristics(payload_size, bs);
        }
        12 => {
            // sub seq characteristics
            decode_sub_seq_characteristics(payload_size, bs);
        }
        13 => {
            // full_frame_freeze
            decode_full_frame_freeze(payload_size, bs);
        }
        14 => {
            decode_full_frame_freeze_release(payload_size, bs);
        }
        15 => {
            decode_full_frame_snapshot(payload_size, bs);
        }
        16 => {
            decode_progressive_refinement_segment_start(payload_size, bs);
        }
        17 => {
            decode_progressive_refinement_segment_end(payload_size, bs);
        }
        18 => {
            decode_motion_constrained_slice_group_set(payload_size, bs);
        }
        19 => {
            res.film_grain_characteristics = decode_film_grain_characteristics(bs);
            res.available = true;
        }
        20 => {
            decode_deblocking_filter_display_preference(payload_size, bs);
        }
        21 => {
            decode_stereo_video_info(payload_size, bs);
        }
        22 => {
            decode_post_filter_hint(payload_size, bs);
        }
        23 => {
            decode_tone_mapping_info(payload_size, bs);
        }
        24 => {
            decode_scalability_info(payload_size, bs); // specified in Annex G
        }
        25 => {
            decode_sub_pic_scalable_layer(payload_size, bs); // specified in Annex G
        }
        26 => {
            decode_non_required_layer_rep(payload_size, bs); // specified in Annex G
        }
        27 => {
            decode_priority_layer_info(payload_size, bs); // specified in Annex G
        }
        28 => {
            decode_layers_not_present(payload_size, bs); // specified in Annex G
        }
        29 => {
            decode_layer_dependency_change(payload_size, bs); // specified in Annex G
        }
        30 => {
            decode_scalable_nesting(payload_size, bs); // specified in Annex G
        }
        31 => {
            decode_base_layer_temporal_hrd(payload_size, bs); // specified in Annex G
        }
        32 => {
            decode_quality_layer_integrity_check(payload_size, bs); // specified in Annex G
        }
        33 => {
            decode_redundant_pic_property(payload_size, bs); // specified in Annex G
        }
        34 => {
            decode_tl0_dep_rep_index(payload_size, bs); // specified in Annex G
        }
        35 => {
            decode_tl_switching_point(payload_size, bs); // specified in Annex G
        }
        36 => {
            decode_parallel_decoding_info(payload_size, bs); // specified in Annex H
        }
        37 => {
            decode_mvc_scalable_nesting(payload_size, bs); // specified in Annex H
        }
        38 => {
            decode_view_scalability_info(payload_size, bs); // specified in Annex H
        }
        39 => {
            decode_multiview_scene_info(payload_size, bs); // specified in Annex H
        }
        40 => {
            decode_multiview_acquisition_info(payload_size, bs); // specified in Annex H
        }
        41 => {
            decode_non_required_view_component(payload_size, bs); // specified in Annex H
        }
        42 => {
            decode_view_dependency_change(payload_size, bs); // specified in Annex H
        }
        43 => {
            decode_operation_points_not_present(payload_size, bs); // specified in Annex H
        }
        44 => {
            decode_base_view_temporal_hrd(payload_size, bs); // specified in Annex H
        }
        45 => {
            decode_frame_packing_arrangement(payload_size, bs);
        }
        46 => {
            decode_multiview_view_position(payload_size, bs); // specified in Annex H
        }
        47 => {
            decode_display_orientation(payload_size, bs);
        }
        48 => {
            decode_mvcd_scalable_nesting(payload_size, bs); // specified in Annex I
        }
        49 => {
            decode_mvcd_view_scalability_info(payload_size, bs); // specified in Annex I
        }
        50 => {
            decode_depth_representation_info(payload_size, bs); // specified in Annex I
        }
        51 => {
            decode_three_dimensional_reference_displays_info(payload_size, bs); // specified in Annex I
        }
        52 => {
            decode_depth_timing(payload_size, bs); // specified in Annex I
        }
        53 => {
            decode_depth_sampling_info(payload_size, bs); // specified in Annex I
        }
        54 => {
            decode_constrained_depth_parameter_set_identifier(payload_size, bs);
            // specified in Annex J
        }
        56 => {
            decode_green_metadata(payload_size, bs); // specified in ISO/IEC 23001-11
        }
        137 => {
            decode_mastering_display_colour_volume(payload_size, bs);
        }
        142 => {
            decode_colour_remapping_info(payload_size, bs);
        }
        144 => {
            decode_content_light_level_info(payload_size, bs);
        }
        147 => {
            decode_alternative_transfer_characteristics(payload_size, bs);
        }
        148 => {
            decode_ambient_viewing_environment(payload_size, bs);
        }
        149 => {
            decode_content_colour_volume(payload_size, bs);
        }
        150 => {
            decode_equirectangular_projection(payload_size, bs);
        }
        151 => {
            decode_cubemap_projection(payload_size, bs);
        }
        154 => {
            decode_sphere_rotation(payload_size, bs);
        }
        155 => {
            decode_regionwise_packing(payload_size, bs);
        }
        156 => {
            decode_omni_viewport(payload_size, bs);
        }
        181 => {
            decode_alternative_depth_info(payload_size, bs); // specified in Annex I
        }
        200 => {
            decode_sei_manifest(payload_size, bs);
        }
        201 => {
            decode_sei_prefix_indication(payload_size, bs);
        }
        202 => {
            decode_annotated_regions(payload_size, bs);
        }
        205 => {
            decode_shutter_interval_info(payload_size, bs);
        }
        _ => {
            debug!(target: "decode","decode_sei_payload - payload_type {} reserved_sei_message", payload_type);
            decode_reserved_sei_message(payload_size, bs);
        }
    }

    // if not parsed, then just consume the payload_size number of bytes
    if !res.available {
        if (payload_size as usize) < bs.bytestream.len() {
            bs.bytestream.drain(0..(payload_size as usize));
        } else {
            debug!(target: "decode","[WARNING] SEI Payload size {} larger than bytes available {} ", payload_size, bs.bytestream.len());
            bs.bytestream.clear();
            bs.byte_offset = 0;
        }
    }

    // if stream is not byte aligned then read bit equal to 1 then 0 bits until byte aligned
    if bs.byte_offset > 0 {
        if bs.bytestream.len() > 0 {
            bs.bytestream.pop_front();
            bs.byte_offset = 0;
        }
    }

    return res;
}

/// D.1.2 Buffering period SEI message syntax
fn decode_buffering_period(
    spses: &Vec<SeqParameterSet>,
    bs: &mut ByteStream,
) -> SEIBufferingPeriod {
    let mut res = SEIBufferingPeriod::new();
    res.seq_parameter_set_id = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print(
        "SEI (Buffering Period): seq_parameter_set_id",
        &res.seq_parameter_set_id,
        63,
    );

    let mut cur_sps_wrapper: Option<&SeqParameterSet> = None;

    for i in (0..spses.len()).rev() {
        if spses[i].seq_parameter_set_id == res.seq_parameter_set_id {
            cur_sps_wrapper = Some(&spses[i]);
            break;
        }
    }

    let cur_sps: &SeqParameterSet;
    match cur_sps_wrapper {
        Some(x) => cur_sps = x,
        _ => panic!(
            "decode_buffering_period - SPS with id {} not found",
            res.seq_parameter_set_id
        ),
    }

    // The variable NalHrdBpPresentFlag is derived as follows:
    //  - If any of the following is true, the value of NalHrdBpPresentFlag shall be set equal to 1:
    //  - nal_hrd_parameters_present_flag is present in the bitstream and is equal to 1,
    //  - the need for presence of buffering periods for NAL HRD operation to be present in the bitstream in buffering
    //  period SEI messages is determined by the application, by some means not specified in this Recommendation |
    //  International Standard.
    //  - Otherwise, the value of NalHrdBpPresentFlag shall be set equal to 0.
    let nal_hrd_bp_present_flag = cur_sps.vui_parameters.nal_hrd_parameters_present_flag;

    if nal_hrd_bp_present_flag {
        res.nal_initial_cpb_removal_delay =
            vec![0; cur_sps.vui_parameters.nal_hrd_parameters.cpb_cnt_minus1 as usize + 1];
        res.nal_initial_cpb_removal_delay_offset =
            vec![0; cur_sps.vui_parameters.nal_hrd_parameters.cpb_cnt_minus1 as usize + 1];

        let bits_to_read = cur_sps
            .vui_parameters
            .nal_hrd_parameters
            .initial_cpb_removal_delay_length_minus1
            + 1;
        decoder_formatted_print(
            "SEI (Buffering Period): initial_cpb_removal_delay_length_minus1+1",
            &bits_to_read,
            63,
        );
        decoder_formatted_print(
            "SEI (Buffering Period): cpb_cnt_minus1",
            &cur_sps.vui_parameters.nal_hrd_parameters.cpb_cnt_minus1,
            63,
        );
        for sched_sel_idx in 0..=cur_sps.vui_parameters.nal_hrd_parameters.cpb_cnt_minus1 {
            res.nal_initial_cpb_removal_delay[sched_sel_idx as usize] = bs.read_bits(bits_to_read);
            decoder_formatted_print(
                "SEI (Buffering Period): NAL initial_cpb_removal_delay[]",
                &res.nal_initial_cpb_removal_delay[sched_sel_idx as usize],
                63,
            );
            res.nal_initial_cpb_removal_delay_offset[sched_sel_idx as usize] =
                bs.read_bits(bits_to_read);
            decoder_formatted_print(
                "SEI (Buffering Period): NAL initial_cpb_removal_delay_offset[]",
                &res.nal_initial_cpb_removal_delay_offset[sched_sel_idx as usize],
                63,
            );
        }
    }

    let vcl_hrd_bp_present_flag = cur_sps.vui_parameters.vcl_hrd_parameters_present_flag;

    if vcl_hrd_bp_present_flag {
        res.vcl_initial_cpb_removal_delay =
            vec![0; cur_sps.vui_parameters.vcl_hrd_parameters.cpb_cnt_minus1 as usize + 1];
        res.vcl_initial_cpb_removal_delay_offset =
            vec![0; cur_sps.vui_parameters.vcl_hrd_parameters.cpb_cnt_minus1 as usize + 1];

        let bits_to_read = cur_sps
            .vui_parameters
            .vcl_hrd_parameters
            .initial_cpb_removal_delay_length_minus1
            + 1;
        decoder_formatted_print(
            "SEI (Buffering Period): initial_cpb_removal_delay_length_minus1+1",
            &bits_to_read,
            63,
        );
        decoder_formatted_print(
            "SEI (Buffering Period): cpb_cnt_minus1",
            &cur_sps.vui_parameters.vcl_hrd_parameters.cpb_cnt_minus1,
            63,
        );
        for sched_sel_idx in 0..=cur_sps.vui_parameters.vcl_hrd_parameters.cpb_cnt_minus1 {
            res.vcl_initial_cpb_removal_delay[sched_sel_idx as usize] = bs.read_bits(bits_to_read);
            decoder_formatted_print(
                "SEI (Buffering Period): VCL initial_cpb_removal_delay[]",
                &res.vcl_initial_cpb_removal_delay[sched_sel_idx as usize],
                63,
            );
            res.vcl_initial_cpb_removal_delay_offset[sched_sel_idx as usize] =
                bs.read_bits(bits_to_read);
            decoder_formatted_print(
                "SEI (Buffering Period): VCL initial_cpb_removal_delay_offset[]",
                &res.vcl_initial_cpb_removal_delay_offset[sched_sel_idx as usize],
                63,
            );
        }
    }

    return res;
}

/// D.1.3 Picture timing SEI message syntax
fn decode_pic_timing(sps: &SeqParameterSet, bs: &mut ByteStream) -> SEIPicTiming {
    let mut result = SEIPicTiming::new();
    // From Page 460 of the H.264 Spec:
    // The variable CpbDpbDelaysPresentFlag is derived as follows:
    // - If any of the following is true, the value of CpbDpbDelaysPresentFlag shall be set equal to 1:
    // - nal_hrd_parameters_present_flag is present in the bitstream and is equal to 1,
    // - vcl_hrd_parameters_present_flag is present in the bitstream and is equal to 1,
    // - the need for presence of CPB and DPB output delays in the bitstream in picture timing SEI messages is
    // determined by the application, by some means not specified in this Recommendation | International Standard.

    let cpb_dpb_delays_present_flag = sps.vui_parameters.nal_hrd_parameters_present_flag
        || sps.vui_parameters.vcl_hrd_parameters_present_flag;
    decoder_formatted_print(
        "SEI (Pic timing): cpb_dpb_delays_present_flag",
        &cpb_dpb_delays_present_flag,
        63,
    );
    if cpb_dpb_delays_present_flag {
        // number of bits:The syntax element is a
        // fixed length code having a length in bits given by cpb_removal_delay_length_minus1 + 1.
        // The cpb_removal_delay is the
        // remainder of a modulo 2^(cpb_removal_delay_length_minus1 + 1) counter.

        // bits to read comes from
        let cpb_bits_to_read;
        let dpb_bits_to_read;

        if sps.vui_parameters.nal_hrd_parameters_present_flag {
            cpb_bits_to_read = sps
                .vui_parameters
                .nal_hrd_parameters
                .cpb_removal_delay_length_minus1
                + 1;
            dpb_bits_to_read = sps
                .vui_parameters
                .nal_hrd_parameters
                .dpb_output_delay_length_minus1
                + 1;
        } else {
            // sps.vui_parameters.vcl_nal_hrd_parameters_present_flag is true
            cpb_bits_to_read = sps
                .vui_parameters
                .vcl_hrd_parameters
                .cpb_removal_delay_length_minus1
                + 1;
            dpb_bits_to_read = sps
                .vui_parameters
                .vcl_hrd_parameters
                .dpb_output_delay_length_minus1
                + 1;
        }

        result.cpb_removal_delay = bs.read_bits(cpb_bits_to_read);
        // From section D.2.3 of the Spec:
        // When vui_parameters.max_dec_frame_buffering is equal to 0, dpb_output_delay shall be equal to 0
        result.dpb_output_delay = bs.read_bits(dpb_bits_to_read);
        decoder_formatted_print(
            "SEI (Pic timing): cpb_removal_delay",
            &result.cpb_removal_delay,
            63,
        );
        decoder_formatted_print(
            "SEI (Pic timing): dpb_output_delay",
            &result.dpb_output_delay,
            63,
        );
    }

    let pic_struct_present_flag = sps.vui_parameters.pic_struct_present_flag;
    decoder_formatted_print(
        "SEI (Pic timing): pic_struct_present_flag",
        &pic_struct_present_flag,
        63,
    );
    if pic_struct_present_flag {
        result.pic_struct = bs.read_bits(4);
        decoder_formatted_print("SEI (Pic timing): pic_struct", &result.pic_struct, 63);
        let num_clock_ts;

        // Table D-1 - Interpretation of pic_struct
        match result.pic_struct {
            0 => {
                // (progressive) frame
                // field_pic_flag shall be 0
                // TopFieldOrderCnt == BottomFieldOrderCnt
                num_clock_ts = 1;
            }
            1 => {
                // top field
                // field_pic_flag shall be 1
                // bottom_field_flag shall be 0
                num_clock_ts = 1;
            }
            2 => {
                // bottom field
                // field_pic_flag shall be 1
                // bottom_field_flag shall be 1
                num_clock_ts = 1;
            }
            3 => {
                // top field, bottom field, in that order
                // field_pic_flag shall be 0
                // TopFieldOrderCnt <= BottomFieldOrderCnt
                num_clock_ts = 2;
            }
            4 => {
                // bottom field, top field, in that order
                // field_pic_flag shall be 0
                // TopFieldOrderCnt >= BottomFieldOrderCnt
                num_clock_ts = 2;
            }
            5 => {
                // top field, bottom field, top field repeated, in that order
                // field_pic_flag shall be 0
                // TopFieldOrderCnt <= BottomFieldOrderCnt
                num_clock_ts = 3;
            }
            6 => {
                // bottom field, top field, bottom field repeated, in that order
                // field_pic_flag shall be 0
                // TopFieldOrderCnt >= BottomFieldOrderCnt
                num_clock_ts = 3;
            }
            7 => {
                // frame doubling
                // field_pic_flag shall be 0
                // fixed_frame_rate_flag shall be 1
                // TopFieldOrderCnt == BottomFieldOrderCnt
                num_clock_ts = 2;
            }
            8 => {
                // frame tripling
                // field_pic_flag shall be 0
                // fixed_frame_rate_flag shall be 1
                // TopFieldOrderCnt == BottomFieldOrderCnt
                num_clock_ts = 3;
            }
            9..=15 => {
                panic!(
                    "SEI (Pic timing): Reserved value for pic_struct : {}",
                    result.pic_struct
                );
            }
            _ => {
                panic!(
                    "SEI (Pic timing): Out of bounds Pic_struct : {}",
                    result.pic_struct
                );
            }
        }
        for i in 0..num_clock_ts {
            result.clock_timestamp_flag.push(1 == bs.read_bits(1));
            decoder_formatted_print(
                "SEI (Pic timing): clock_timestamp_flag[]",
                &result.clock_timestamp_flag[i],
                63,
            );

            let time_offset_length;
            if sps.vui_parameters.nal_hrd_parameters_present_flag {
                time_offset_length = sps.vui_parameters.nal_hrd_parameters.time_offset_length;
            } else {
                // sps.vui_parameters.vcl_nal_hrd_parameters_present_flag is true
                time_offset_length = sps.vui_parameters.vcl_hrd_parameters.time_offset_length;
            }

            if result.clock_timestamp_flag[i] {
                result.ct_type.push(bs.read_bits(2));
                decoder_formatted_print("SEI (Pic timing): ct_type[]", &result.ct_type[i], 63);

                result.nuit_field_based_flag.push(1 == bs.read_bits(1));
                decoder_formatted_print(
                    "SEI (Pic timing): nuit_field_based_flag[]",
                    &result.nuit_field_based_flag[i],
                    63,
                );

                result.counting_type.push(bs.read_bits(5));
                decoder_formatted_print(
                    "SEI (Pic timing): counting_type[]",
                    &result.counting_type[i],
                    63,
                );

                result.full_timestamp_flag.push(1 == bs.read_bits(1));
                decoder_formatted_print(
                    "SEI (Pic timing): full_timestamp_flag[]",
                    &result.full_timestamp_flag[i],
                    63,
                );

                result.discontinuity_flag.push(1 == bs.read_bits(1));
                decoder_formatted_print(
                    "SEI (Pic timing): discontinuity_flag[]",
                    &result.discontinuity_flag[i],
                    63,
                );

                result.cnt_dropped_flag.push(1 == bs.read_bits(1));
                decoder_formatted_print(
                    "SEI (Pic timing): cnt_dropped_flag[]",
                    &result.cnt_dropped_flag[i],
                    63,
                );

                result.n_frames.push(bs.read_bits(8));
                decoder_formatted_print("SEI (Pic timing): n_frames[]", &result.n_frames[i], 63);

                if result.full_timestamp_flag[i] {
                    result.seconds_value.push(bs.read_bits(6));
                    decoder_formatted_print(
                        "SEI (Pic timing): seconds_value[]",
                        &result.seconds_value[i],
                        63,
                    );

                    result.minutes_value.push(bs.read_bits(6));
                    decoder_formatted_print(
                        "SEI (Pic timing): minutes_value[]",
                        &result.minutes_value[i],
                        63,
                    );

                    result.hours_value.push(bs.read_bits(5));
                    decoder_formatted_print(
                        "SEI (Pic timing): hours_value[]",
                        &result.hours_value[i],
                        63,
                    );

                    // push the flag values
                    result.seconds_flag.push(true);
                    result.minutes_flag.push(true);
                    result.hours_flag.push(true);
                } else {
                    result.seconds_flag.push(1 == bs.read_bits(1));
                    decoder_formatted_print(
                        "SEI (Pic timing): seconds_flag[]",
                        &result.seconds_flag[i],
                        63,
                    );
                    if result.seconds_flag[i] {
                        result.seconds_value.push(bs.read_bits(6));
                        decoder_formatted_print(
                            "SEI (Pic timing): seconds_value[]",
                            &result.seconds_value[i],
                            63,
                        );
                        result.minutes_flag.push(1 == bs.read_bits(1));
                        decoder_formatted_print(
                            "SEI (Pic timing): minutes_flag[]",
                            &result.minutes_flag[i],
                            63,
                        );
                        if result.minutes_flag[i] {
                            result.minutes_value.push(bs.read_bits(6));
                            decoder_formatted_print(
                                "SEI (Pic timing): minutes_value[]",
                                &result.minutes_value[i],
                                63,
                            );
                            result.hours_flag.push(1 == bs.read_bits(1));
                            decoder_formatted_print(
                                "SEI (Pic timing): hours_flag[]",
                                &result.hours_flag[i],
                                63,
                            );
                            if result.hours_flag[i] {
                                result.hours_value.push(bs.read_bits(5));
                                decoder_formatted_print(
                                    "SEI (Pic timing): hours_value[]",
                                    &result.hours_value[i],
                                    63,
                                );
                            } else {
                                result.hours_value.push(0);
                            }
                        } else {
                            result.minutes_value.push(0);
                            result.hours_flag.push(false);
                            result.hours_value.push(0);
                        }
                    } else {
                        result.seconds_value.push(0);
                        result.minutes_flag.push(false);
                        result.minutes_value.push(0);
                        result.hours_flag.push(false);
                        result.hours_value.push(0);
                    }
                }
                if time_offset_length > 0 {
                    result.time_offset.push(bs.read_bits(time_offset_length)); // i(v)
                } else {
                    result.time_offset.push(0);
                }
            } else {
                result.ct_type.push(0);
                result.nuit_field_based_flag.push(false);
                result.counting_type.push(0);
                result.full_timestamp_flag.push(false);
                result.discontinuity_flag.push(false);
                result.cnt_dropped_flag.push(false);
                result.n_frames.push(0);
                result.seconds_value.push(0);
                result.minutes_value.push(0);
                result.hours_value.push(0);
                result.seconds_flag.push(false);
                result.minutes_flag.push(false);
                result.hours_flag.push(false);
                result.time_offset.push(0);
            }
        }
    } else {
        // TODO: Infer default values
        // NOTE 6 - When pic_struct_present_flag is equal to 0, then in many cases default values may be inferred. In the absence of other
        // indications of the intended display type of a picture, the decoder should infer the value of pic_struct as follows:
        // - If field_pic_flag is equal to 1, pic_struct should be inferred to be equal to (1 + bottom_field_flag).
        // - Otherwise, if TopFieldOrderCnt is equal to BottomFieldOrderCnt, pic_struct should be inferred to be equal to 0.
        // - Otherwise, if TopFieldOrderCnt is less than BottomFieldOrderCnt, pic_struct should be inferred to be equal to 3.
        // - Otherwise (field_pic_flag is equal to 0 and TopFieldOrderCnt is greater than BottomFieldOrderCnt), pic_struct should be
        // inferred to be equal to 4.
        // pic_struct is only a hint as to how the decoded video should be displayed on an assumed display type (e.g., interlaced or progressive)
        // at an assumed display rate. When another display type or display rate is used by the decoder, then pic_struct does not indicate the
        // display method, but may aid in processing the decoded video for the alternative display. When it is desired for pic_struct to have an
        // effective value in the range of 5 to 8, inclusive, pic_struct_present_flag should be equal to 1, as the above inference rule will not
        // produce these values.
    }

    return result;
}

/// D.1.4 Pan-scan rectangle SEI message syntax
fn decode_pan_scan_rect(_payload_size: u32, bs: &mut ByteStream) {
    let pan_scan_rect_id = exp_golomb_decode_one_wrapper(bs, false, 0);
    decoder_formatted_print(
        "SEI (Pan-scan rectangle): pan_scan_rect_id",
        &pan_scan_rect_id,
        63,
    );
    let pan_scan_rect_cancel_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "SEI (Pan-scan rectangle): pan_scan_rect_cancel_flag",
        &pan_scan_rect_cancel_flag,
        63,
    );
    if !pan_scan_rect_cancel_flag {
        let pan_scan_cnt_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0);
        decoder_formatted_print(
            "SEI (Pan-scan rectangle): pan_scan_cnt_minus1",
            &pan_scan_cnt_minus1,
            63,
        );
        // prepare
        let mut pan_scan_rect_left_offset = Vec::new();
        let mut pan_scan_rect_right_offset = Vec::new();
        let mut pan_scan_rect_top_offset = Vec::new();
        let mut pan_scan_rect_bottom_offset = Vec::new();
        for _ in 0..=pan_scan_cnt_minus1 {
            pan_scan_rect_left_offset.push(exp_golomb_decode_one_wrapper(bs, true, 0));
            decoder_formatted_print(
                "SEI (Pan-scan rectangle): pan_scan_rect_left_offset",
                &pan_scan_rect_left_offset,
                63,
            );
            pan_scan_rect_right_offset.push(exp_golomb_decode_one_wrapper(bs, true, 0));
            decoder_formatted_print(
                "SEI (Pan-scan rectangle): pan_scan_rect_right_offset",
                &pan_scan_rect_right_offset,
                63,
            );
            pan_scan_rect_top_offset.push(exp_golomb_decode_one_wrapper(bs, true, 0));
            decoder_formatted_print(
                "SEI (Pan-scan rectangle): pan_scan_rect_top_offset",
                &pan_scan_rect_top_offset,
                63,
            );
            pan_scan_rect_bottom_offset.push(exp_golomb_decode_one_wrapper(bs, true, 0));
            decoder_formatted_print(
                "SEI (Pan-scan rectangle): pan_scan_rect_bottom_offset",
                &pan_scan_rect_bottom_offset,
                63,
            );
        }
        let _pan_scan_rect_repetition_period = exp_golomb_decode_one_wrapper(bs, false, 0);
    }
}

/// D.1.5 Filler payload SEI message syntax
fn decode_filler_payload(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.6 User data registered by Rec. ITU-T T.35 SEI message syntax
fn decode_user_data_registered_itu_t_t35(payload_size: u32, bs: &mut ByteStream) {
    let itu_t_t35_country_code = bs.read_bits(8);
    decoder_formatted_print(
        "SEI (ITU-T T.35 User Data): itu_t_t35_country_code",
        &itu_t_t35_country_code,
        63,
    );
    let mut i: u32 = if itu_t_t35_country_code == 0xff {
        1
    } else {
        let itu_t_t35_country_code_extension_byte = bs.read_bits(8);
        decoder_formatted_print(
            "SEI (ITU-T T.35 User Data): itu_t_t35_country_code_extension_byte",
            &itu_t_t35_country_code_extension_byte,
            63,
        );
        2
    };

    loop {
        let itu_t_t35_payload_byte = bs.read_bits(8);
        decoder_formatted_print(
            "SEI (ITU-T T.35 User Data): itu_t_t35_payload_byte",
            &itu_t_t35_payload_byte,
            63,
        );
        i += 1;
        if i < payload_size {
            break;
        }
    }
}

/// D.1.7 User data unregistered SEI message syntax
fn decode_user_data_unregistered(
    mut payload_size: u32,
    bs: &mut ByteStream,
) -> SEIUserDataUnregistered {
    let mut res = SEIUserDataUnregistered::new();

    if bs.bytestream.len() < (payload_size as usize) {
        println!("[WARNING] decode_user_data_unregistered - Malformed SEI unit - payload_size {} doesn't match NALU content length {}", payload_size, bs.bytestream.len());
        decoder_formatted_print(
            "SEI (Unregistered User Data): bytestream contents",
            &bs.bytestream.clone(),
            63,
        );
    }

    if bs.bytestream.len() < 16 {
        println!("[WARNING] decode_user_data_unregistered - Not enough bytes for UUID");
    }

    for i in 0..16 {
        res.uuid_iso_iec_11578[i] = bs.read_bits(8) as u8;
    }
    decoder_formatted_print(
        "SEI (Unregistered User Data): uuid_iso_iec_11578",
        &res.uuid_iso_iec_11578,
        63,
    );
    // adjust the size for UUID
    payload_size -= 16;
    // switch to see what UUID value we're dealing with
    match res.uuid_iso_iec_11578 {
        UUID_APPLE1 => {
            res.user_data_apple1.mystery_param1 = bs.read_bits(8);
            // TODO: there is a derived parameter on if the above is less than 4, but we hold off for now
        }
        UUID_APPLE2 => {
            res.user_data_apple2.mystery_param1 = bs.read_bits(8);
            res.user_data_apple2.mystery_param2 = bs.read_bits(8);
            res.user_data_apple2.mystery_param3 = bs.read_bits(8);
            res.user_data_apple2.mystery_param4 = bs.read_bits(8);
            res.user_data_apple2.mystery_param5 = bs.read_bits(8);
            res.user_data_apple2.mystery_param6 = bs.read_bits(8);
            res.user_data_apple2.mystery_param7 = bs.read_bits(8);
            res.user_data_apple2.mystery_param8 = bs.read_bits(8);
        }
        _ => {
            if (payload_size as usize) > bs.bytestream.len() {
                debug!(target: "decode","SEI - payload_size {} larger than bytestream length {}: copying the rest", payload_size, bs.bytestream.len());
                // just copy the rest of the data
                res.user_data_payload_byte.extend(&bs.bytestream.clone());

                // empty it out since everything is consumed
                bs.bytestream = VecDeque::new();
                bs.byte_offset = 0;
            } else {
                for _ in 0..payload_size {
                    let cur_byte = bs.read_bits(8) as u8;
                    res.user_data_payload_byte.push(cur_byte);
                }
                decoder_formatted_print(
                    "SEI (Unregistered User Data): user_data_payload_byte",
                    &res.user_data_payload_byte,
                    63,
                );
            }
        }
    }

    res
}

/// D.1.8 Recovery point SEI message syntax
fn decode_recovery_point(bs: &mut ByteStream) -> SEIRecoveryPoint {
    let mut res = SEIRecoveryPoint::new();

    res.recovery_frame_cnt = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    decoder_formatted_print(
        "SEI (Recovery point): recovery_frame_cnt",
        &res.recovery_frame_cnt,
        63,
    );
    res.exact_match_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "SEI (Recovery point): exact_match_flag",
        &res.exact_match_flag,
        63,
    );
    res.broken_link_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "SEI (Recovery point): broken_link_flag",
        &res.broken_link_flag,
        63,
    );
    res.changing_slice_group_idc = bs.read_bits(2) as u8;
    decoder_formatted_print(
        "SEI (Recovery point): changing_slice_group_idc",
        &res.changing_slice_group_idc,
        63,
    );

    res
}

/// D.1.9 Decoded reference picture marking repetition SEI message syntax
fn decode_ref_pic_marking_repetition(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.10 Spare picture SEI message syntax
fn decode_spare_pic(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.11 Scene information SEI message syntax
fn decode_scene_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.12 Sub-sequence information SEI message syntax
fn decode_sub_seq_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.13 Sub-sequence layer characteristics SEI message syntax
fn decode_sub_seq_layer_characteristics(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.14 Sub-sequence characteristics SEI message syntax
fn decode_sub_seq_characteristics(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.15 Full-frame freeze SEI message syntax
fn decode_full_frame_freeze(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.16 Full-frame freeze release SEI message syntax
fn decode_full_frame_freeze_release(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.17 Full-frame snapshot SEI message syntax
fn decode_full_frame_snapshot(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.18 Progressive refinement segment start SEI message syntax
fn decode_progressive_refinement_segment_start(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.19 Progressive refinement segment end SEI message syntax
fn decode_progressive_refinement_segment_end(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.20 Motion-constrained slice group set SEI message syntax
fn decode_motion_constrained_slice_group_set(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.21 Film grain characteristics SEI message syntax
fn decode_film_grain_characteristics(bs: &mut ByteStream) -> SEIFilmGrainCharacteristics {
    let mut fgc = SEIFilmGrainCharacteristics::new();

    fgc.film_grain_characteristics_cancel_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "SEI (Film Grain Characteristics): film_grain_characteristics_cancel_flag",
        &fgc.film_grain_characteristics_cancel_flag,
        63,
    );

    if !fgc.film_grain_characteristics_cancel_flag {
        fgc.film_grain_model_id = bs.read_bits(2) as u8;
        decoder_formatted_print(
            "SEI (Film Grain Characteristics): film_grain_model_id",
            &fgc.film_grain_model_id,
            63,
        );

        fgc.separate_colour_description_present_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "SEI (Film Grain Characteristics): separate_colour_description_present_flag",
            &fgc.separate_colour_description_present_flag,
            63,
        );

        if fgc.separate_colour_description_present_flag {
            fgc.film_grain_bit_depth_luma_minus8 = bs.read_bits(3) as u8;
            decoder_formatted_print(
                "SEI (Film Grain Characteristics): film_grain_bit_depth_luma_minus8",
                &fgc.film_grain_bit_depth_luma_minus8,
                63,
            );

            fgc.film_grain_bit_depth_chroma_minus8 = bs.read_bits(3) as u8;
            decoder_formatted_print(
                "SEI (Film Grain Characteristics): film_grain_bit_depth_chroma_minus8",
                &fgc.film_grain_bit_depth_chroma_minus8,
                63,
            );

            fgc.film_grain_full_range_flag = 1 == bs.read_bits(1);
            decoder_formatted_print(
                "SEI (Film Grain Characteristics): film_grain_full_range_flag",
                &fgc.film_grain_full_range_flag,
                63,
            );

            fgc.film_grain_colour_primaries = bs.read_bits(8) as u8;
            decoder_formatted_print(
                "SEI (Film Grain Characteristics): film_grain_colour_primaries",
                &fgc.film_grain_colour_primaries,
                63,
            );

            fgc.film_grain_transfer_characteristics = bs.read_bits(8) as u8;
            decoder_formatted_print(
                "SEI (Film Grain Characteristics): film_grain_transfer_characteristics",
                &fgc.film_grain_transfer_characteristics,
                63,
            );

            fgc.film_grain_matrix_coefficients = bs.read_bits(8) as u8;
            decoder_formatted_print(
                "SEI (Film Grain Characteristics): film_grain_matrix_coefficients",
                &fgc.film_grain_matrix_coefficients,
                63,
            );
        }
        fgc.blending_mode_id = bs.read_bits(2) as u8;
        decoder_formatted_print(
            "SEI (Film Grain Characteristics): blending_mode_id",
            &fgc.blending_mode_id,
            63,
        );

        fgc.log2_scale_factor = bs.read_bits(4) as u8;
        decoder_formatted_print(
            "SEI (Film Grain Characteristics): log2_scale_factor",
            &fgc.log2_scale_factor,
            63,
        );

        for c in 0..3 {
            fgc.comp_model_present_flag.push(1 == bs.read_bits(1));
            decoder_formatted_print(
                "SEI (Film Grain Characteristics): comp_model_present_flag",
                &fgc.comp_model_present_flag[c],
                63,
            );
        }

        for c in 0..3 {
            if fgc.comp_model_present_flag[c] {
                fgc.num_intensity_intervals_minus1
                    .push(bs.read_bits(8) as u8);
                decoder_formatted_print(
                    "SEI (Film Grain Characteristics): num_intensity_intervals_minus1",
                    &fgc.num_intensity_intervals_minus1[c],
                    63,
                );
                fgc.num_model_values_minus1.push(bs.read_bits(3) as u8);
                decoder_formatted_print(
                    "SEI (Film Grain Characteristics): num_model_values_minus1",
                    &fgc.num_model_values_minus1[c],
                    63,
                );

                fgc.intensity_interval_lower_bound.push(Vec::new());
                fgc.intensity_interval_upper_bound.push(Vec::new());
                fgc.comp_model_value.push(Vec::new());
                for i in 0..fgc.num_intensity_intervals_minus1[c] as usize {
                    fgc.intensity_interval_lower_bound[c].push(bs.read_bits(8) as u8);
                    decoder_formatted_print(
                        "SEI (Film Grain Characteristics): intensity_interval_lower_bound",
                        &fgc.intensity_interval_lower_bound[c][i],
                        63,
                    );

                    fgc.intensity_interval_upper_bound[c].push(bs.read_bits(8) as u8);
                    decoder_formatted_print(
                        "SEI (Film Grain Characteristics): intensity_interval_upper_bound",
                        &fgc.intensity_interval_upper_bound[c][i],
                        63,
                    );

                    fgc.comp_model_value[c].push(Vec::new());
                    for j in 0..fgc.num_model_values_minus1[c] as usize {
                        fgc.comp_model_value[c][i].push(exp_golomb_decode_one_wrapper(bs, true, 0));
                        decoder_formatted_print(
                            "SEI (Film Grain Characteristics): comp_model_value",
                            &fgc.comp_model_value[c][i][j],
                            63,
                        );
                    }
                }
            } else {
                fgc.num_intensity_intervals_minus1.push(0);
                fgc.num_model_values_minus1.push(0);
                fgc.intensity_interval_lower_bound.push(Vec::new());
                fgc.intensity_interval_upper_bound.push(Vec::new());
                fgc.comp_model_value.push(Vec::new());
            }
        }
        fgc.film_grain_characteristics_repetition_period =
            exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        decoder_formatted_print(
            "SEI (Film Grain Characteristics): film_grain_characteristics_repetition_period",
            &fgc.film_grain_characteristics_repetition_period,
            63,
        );
    }

    fgc
}

/// D.1.22 Deblocking filter display preference SEI message syntax
fn decode_deblocking_filter_display_preference(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.23 Stereo video information SEI message syntax
fn decode_stereo_video_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.24 Post-filter hint SEI message syntax
fn decode_post_filter_hint(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.25 Tone mapping information SEI message syntax
fn decode_tone_mapping_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// G.13.1.1 Scalability information SEI message syntax
fn decode_scalability_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// G.13.1.2Sub-picture scalable layer SEI message syntax
fn decode_sub_pic_scalable_layer(_payload_size: u32, _bs: &mut ByteStream) {}

/// G.13.1.3Non-required layer representation SEI message syntax
fn decode_non_required_layer_rep(_payload_size: u32, _bs: &mut ByteStream) {}

/// G.13.1.4Priority layer information SEI message syntax
fn decode_priority_layer_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// G.13.1.5Layers not present SEI message syntax
fn decode_layers_not_present(_payload_size: u32, _bs: &mut ByteStream) {}

/// G.13.1.6Layer dependency change SEI message syntax
fn decode_layer_dependency_change(_payload_size: u32, _bs: &mut ByteStream) {}

/// G.13.1.7Scalable nesting SEI message syntax
fn decode_scalable_nesting(_payload_size: u32, _bs: &mut ByteStream) {}

/// G.13.1.8Base layer temporal HRD SEI message syntax
fn decode_base_layer_temporal_hrd(_payload_size: u32, _bs: &mut ByteStream) {}

/// G.13.1.9Quality layer integrity check SEI message syntax
fn decode_quality_layer_integrity_check(_payload_size: u32, _bs: &mut ByteStream) {}

/// G.13.1.10 Redundant picture property SEI message syntax
fn decode_redundant_pic_property(_payload_size: u32, _bs: &mut ByteStream) {}

/// G.13.1.11 Temporal level zero dependency representation index SEI message syntax
fn decode_tl0_dep_rep_index(_payload_size: u32, _bs: &mut ByteStream) {}

/// G.13.1.12 Temporal level switching point SEI message syntax
fn decode_tl_switching_point(_payload_size: u32, _bs: &mut ByteStream) {}

/// H.13.1.1Parallel decoding information SEI message syntax
fn decode_parallel_decoding_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// H.13.1.2MVC scalable nesting SEI message syntax
fn decode_mvc_scalable_nesting(_payload_size: u32, _bs: &mut ByteStream) {}

/// H.13.1.3 View scalability information SEI message syntax
fn decode_view_scalability_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// H.13.1.4 Multiview scene information SEI message syntax
fn decode_multiview_scene_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// H.13.1.5 Multiview acquisition information SEI message syntax
fn decode_multiview_acquisition_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// H.13.1.6Non-required view component SEI message syntax
fn decode_non_required_view_component(_payload_size: u32, _bs: &mut ByteStream) {}

/// H.13.1.7View dependency change SEI message syntax
fn decode_view_dependency_change(_payload_size: u32, _bs: &mut ByteStream) {}

/// H.13.1.8Operation point not present SEI message syntax
fn decode_operation_points_not_present(_payload_size: u32, _bs: &mut ByteStream) {}

/// H.13.1.9Base view temporal HRD SEI message syntax
fn decode_base_view_temporal_hrd(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.26 Frame packing arrangement SEI message syntax
fn decode_frame_packing_arrangement(_payload_size: u32, bs: &mut ByteStream) {
    let frame_packing_arrangement_id = exp_golomb_decode_one_wrapper(bs, false, 0);
    decoder_formatted_print(
        "SEI (Frame Packing): frame_packing_arrangement_id",
        &frame_packing_arrangement_id,
        63,
    );
    let frame_packing_arrangement_cancel_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "SEI (Frame Packing): frame_packing_arrangement_cancel_flag",
        &frame_packing_arrangement_cancel_flag,
        63,
    );
    if !frame_packing_arrangement_cancel_flag {
        let frame_packing_arrangement_type = bs.read_bits(7);
        decoder_formatted_print(
            "SEI (Frame Packing): frame_packing_arrangement_type",
            &frame_packing_arrangement_type,
            63,
        );
        let quincunx_sampling_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "SEI (Frame Packing): quincunx_sampling_flag",
            &quincunx_sampling_flag,
            63,
        );
        let content_interpretation_type = bs.read_bits(6);
        decoder_formatted_print(
            "SEI (Frame Packing): content_interpretation_type",
            &content_interpretation_type,
            63,
        );
        let spatial_flipping_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "SEI (Frame Packing): spatial_flipping_flag",
            &spatial_flipping_flag,
            63,
        );
        let frame0_flipped_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "SEI (Frame Packing): frame0_flipped_flag",
            &frame0_flipped_flag,
            63,
        );
        let field_views_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "SEI (Frame Packing): field_views_flag",
            &field_views_flag,
            63,
        );
        let current_frame_is_frame0_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "SEI (Frame Packing): current_frame_is_frame0_flag",
            &current_frame_is_frame0_flag,
            63,
        );
        let frame0_self_contained_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "SEI (Frame Packing): frame0_self_contained_flag",
            &frame0_self_contained_flag,
            63,
        );
        let frame1_self_contained_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "SEI (Frame Packing): frame1_self_contained_flag",
            &frame1_self_contained_flag,
            63,
        );
        if !quincunx_sampling_flag && frame_packing_arrangement_type != 5 {
            let frame0_grid_position_x = bs.read_bits(4);
            decoder_formatted_print(
                "SEI (Frame Packing): frame0_grid_position_x",
                &frame0_grid_position_x,
                63,
            );
            let frame0_grid_position_y = bs.read_bits(4);
            decoder_formatted_print(
                "SEI (Frame Packing): frame0_grid_position_y",
                &frame0_grid_position_y,
                63,
            );
            let frame1_grid_position_x = bs.read_bits(4);
            decoder_formatted_print(
                "SEI (Frame Packing): frame1_grid_position_x",
                &frame1_grid_position_x,
                63,
            );
            let frame1_grid_position_y = bs.read_bits(4);
            decoder_formatted_print(
                "SEI (Frame Packing): frame1_grid_position_y",
                &frame1_grid_position_y,
                63,
            );
        }
        let frame_packing_arrangement_reserved_byte = bs.read_bits(8);
        decoder_formatted_print(
            "SEI (Frame Packing): frame_packing_arrangement_reserved_byte",
            &frame_packing_arrangement_reserved_byte,
            63,
        );
        let frame_packing_arrangement_repetition_period =
            exp_golomb_decode_one_wrapper(bs, false, 0);
        decoder_formatted_print(
            "SEI (Frame Packing): frame_packing_arrangement_repetition_period",
            &frame_packing_arrangement_repetition_period,
            63,
        );
    }
    let frame_packing_arrangement_extension_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "SEI (Frame Packing): frame_packing_arrangement_extension_flag",
        &frame_packing_arrangement_extension_flag,
        63,
    );
}

/// H.13.1.10 Multiview view position SEI message syntax
fn decode_multiview_view_position(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.27 Display orientation SEI message syntax
fn decode_display_orientation(_payload_size: u32, _bs: &mut ByteStream) {}

/// I.13.1.2 MVCD scalable nesting SEI message syntax
fn decode_mvcd_scalable_nesting(_payload_size: u32, _bs: &mut ByteStream) {}

/// I.13.1.1 MVCD view scalability information SEI message syntax
fn decode_mvcd_view_scalability_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// I.13.1.3 Depth representation information SEI message syntax
fn decode_depth_representation_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// I.13.1.4 3D reference displays information SEI message syntax
fn decode_three_dimensional_reference_displays_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// I.13.1.5 Depth timing SEI message syntax
fn decode_depth_timing(_payload_size: u32, _bs: &mut ByteStream) {}

/// I.13.1.7 Depth sampling information SEI message syntax
fn decode_depth_sampling_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// J.13.1.1 Constrained depth parameter set identifier SEI message syntax
fn decode_constrained_depth_parameter_set_identifier(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.28 Green metadata SEI message syntax
/// The syntax for this SEI message is specified in ISO/IEC 23001-11 (Green metadata), which facilitates reduced power
/// consumption in decoders, encoders, displays, and in media selection
fn decode_green_metadata(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.29 Mastering display colour volume SEI message syntax
fn decode_mastering_display_colour_volume(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.30 Colour remapping information SEI message syntax
fn decode_colour_remapping_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.31 Content light level information SEI message syntax
fn decode_content_light_level_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.32 Alternative transfer characteristics SEI message syntax
fn decode_alternative_transfer_characteristics(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.33 Content colour volume SEI message syntax
fn decode_content_colour_volume(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.34 Ambient viewing environment SEI message syntax
fn decode_ambient_viewing_environment(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.35.1 Equirectangular projection SEI message syntax
fn decode_equirectangular_projection(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.35.2 Cubemap projection SEI message syntax
fn decode_cubemap_projection(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.35.3 Sphere rotation SEI message syntax
fn decode_sphere_rotation(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.35.4 Region-wise packing SEI message syntax
fn decode_regionwise_packing(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.35.5 Omnidirectional viewport SEI message syntax
fn decode_omni_viewport(_payload_size: u32, _bs: &mut ByteStream) {}

/// I.13.1.6 Alternative depth information SEI message syntax
fn decode_alternative_depth_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.36 SEI manifest SEI message syntax
fn decode_sei_manifest(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.37 SEI prefix indication SEI message syntax
fn decode_sei_prefix_indication(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.38 Annotated regions SEI message syntax
fn decode_annotated_regions(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.39 Shutter interval information SEI message syntax
fn decode_shutter_interval_info(_payload_size: u32, _bs: &mut ByteStream) {}

/// D.1.40 Reserved SEI message syntax
fn decode_reserved_sei_message(_payload_size: u32, _bs: &mut ByteStream) {}
