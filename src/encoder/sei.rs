//! SEI syntax element encoding.

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
use crate::common::helper::bitstream_to_bytestream;
use crate::common::helper::encoder_formatted_print;
use crate::encoder::binarization_functions::generate_unsigned_binary;
use crate::encoder::expgolomb::exp_golomb_encode_one;
use log::debug;

/// Follows section 7.3.2.3
pub fn encode_sei_message(sei: &SEINalu, spses: &[SeqParameterSet]) -> Vec<u8> {
    let mut bytestream_array = Vec::new();
    for i in 0..sei.payload.len() {
        println!("\t\tSEI({}): payload_type {}", i, sei.payload_type[i]);

        encoder_formatted_print("SEI: payload_type", &sei.payload_type[i], 63);
        encoder_formatted_print("SEI: payload_size", &sei.payload_size[i], 63);

        let mut encoded_payload_bitstream =
            encode_sei_payload(sei.payload_type[i], &sei.payload[i], spses);

        // if not byte aligned, then we append a 1 bit then all 0s until byte aligned
        if encoded_payload_bitstream.len() % 8 != 0 {
            encoded_payload_bitstream.push(1);
            while encoded_payload_bitstream.len() % 8 != 0 {
                encoded_payload_bitstream.push(0);
            }
        }
        let encoded_payload = bitstream_to_bytestream(encoded_payload_bitstream, 0);

        if encoded_payload.len() == 0 && sei.payload_size[i] > 0 {
            debug!(target: "encode","[WARNING] SEI Encoded Payload is empty, likely not yet implemented");
            return Vec::new();
        }

        let mut payload_type = sei.payload_type[i];
        while payload_type > 0xff {
            bytestream_array.push(0xff);
            payload_type -= 0xff;
        }
        bytestream_array.push(payload_type as u8);

        let mut payload_size = sei.payload_size[i];

        // TODO: set a flag to decide whether to go with the actual size or specified size.
        //       For now, if payload_size is 0, then we'll write it in the encoded size
        if (encoded_payload.len() as u32) != sei.payload_size[i] && sei.payload_size[i] == 0 {
            println!(
                "[WARNING] SEI Payload Size {} doesn't match actual encoded size {}",
                sei.payload_size[i],
                encoded_payload.len()
            );
            debug!(target: "encode","[WARNING] SEI Payload Size {} doesn't match actual encoded size {}", sei.payload_size[i], encoded_payload.len());

            // use actual encoded size
            payload_size = encoded_payload.len() as u32;
        }

        while payload_size > 0xff {
            bytestream_array.push(0xff);
            payload_size -= 0xff;
        }
        bytestream_array.push(payload_size as u8);

        bytestream_array.append(&mut encoded_payload.clone());
    }

    // add the RBSP trailing 1 bit
    bytestream_array.push(128);

    bytestream_array
}

// described in D.1.1 General SEI message syntax
fn encode_sei_payload(
    payload_type: u32,
    payload: &SEIPayload,
    spses: &[SeqParameterSet],
) -> Vec<u8> {
    let mut res = Vec::new();

    match payload_type {
        0 => {
            // buffering period
            res.append(&mut encode_buffering_period(
                &payload.buffering_period,
                spses,
            ));
        }
        1 => {
            // picture timing
            let sps: SeqParameterSet;
            if spses.len() > 0 {
                sps = spses[spses.len() - 1].clone();
            } else {
                sps = SeqParameterSet::new();
            }
            res.append(&mut encode_pic_timing(&payload.pic_timing, &sps));
        }
        2 => {
            // pan scan rect
            res.append(&mut encode_pan_scan_rect());
        }
        3 => {
            // filler payload
            res.append(&mut encode_filler_payload());
        }
        4 => {
            // user data registered ITU T T35
            res.append(&mut encode_user_data_registered_itu_t_t35());
        }
        5 => {
            res.append(&mut encode_user_data_unregistered(
                &payload.unregistered_user_data,
            ));
        }
        6 => {
            // recovery point
            res.append(&mut encode_recovery_point(&payload.recovery_point));
        }
        7 => {
            // pic marking repetition
            res.append(&mut encode_ref_pic_marking_repetition());
        }
        8 => {
            // spare pic
            res.append(&mut encode_spare_pic());
        }
        9 => {
            // scene info
            res.append(&mut encode_scene_info());
        }
        10 => {
            // sub seq info
            res.append(&mut encode_sub_seq_info());
        }
        11 => {
            // sub seq layer characteristics
            res.append(&mut encode_sub_seq_layer_characteristics());
        }
        12 => {
            // sub seq characteristics
            res.append(&mut encode_sub_seq_characteristics());
        }
        13 => {
            // full_frame_freeze
            res.append(&mut encode_full_frame_freeze());
        }
        14 => {
            res.append(&mut encode_full_frame_freeze_release());
        }
        15 => {
            res.append(&mut encode_full_frame_snapshot());
        }
        16 => {
            res.append(&mut encode_progressive_refinement_segment_start());
        }
        17 => {
            res.append(&mut encode_progressive_refinement_segment_end());
        }
        18 => {
            res.append(&mut encode_motion_constrained_slice_group_set());
        }
        19 => {
            res.append(&mut encode_film_grain_characteristics(
                &payload.film_grain_characteristics,
            ));
        }
        20 => {
            res.append(&mut encode_deblocking_filter_display_preference());
        }
        21 => {
            res.append(&mut encode_stereo_video_info());
        }
        22 => {
            res.append(&mut encode_post_filter_hint());
        }
        23 => {
            res.append(&mut encode_tone_mapping_info());
        }
        24 => {
            res.append(&mut encode_scalability_info()); // specified in Annex G
        }
        25 => {
            res.append(&mut encode_sub_pic_scalable_layer()); // specified in Annex G
        }
        26 => {
            res.append(&mut encode_non_required_layer_rep()); // specified in Annex G
        }
        27 => {
            res.append(&mut encode_priority_layer_info()); // specified in Annex G
        }
        28 => {
            res.append(&mut encode_layers_not_present()); // specified in Annex G
        }
        29 => {
            res.append(&mut encode_layer_dependency_change()); // specified in Annex G
        }
        30 => {
            res.append(&mut encode_scalable_nesting()); // specified in Annex G
        }
        31 => {
            res.append(&mut encode_base_layer_temporal_hrd()); // specified in Annex G
        }
        32 => {
            res.append(&mut encode_quality_layer_integrity_check()); // specified in Annex G
        }
        33 => {
            res.append(&mut encode_redundant_pic_property()); // specified in Annex G
        }
        34 => {
            res.append(&mut encode_tl0_dep_rep_index()); // specified in Annex G
        }
        35 => {
            res.append(&mut encode_tl_switching_point()); // specified in Annex G
        }
        36 => {
            res.append(&mut encode_parallel_decoding_info()); // specified in Annex H
        }
        37 => {
            res.append(&mut encode_mvc_scalable_nesting()); // specified in Annex H
        }
        38 => {
            res.append(&mut encode_view_scalability_info()); // specified in Annex H
        }
        39 => {
            res.append(&mut encode_multiview_scene_info()); // specified in Annex H
        }
        40 => {
            res.append(&mut encode_multiview_acquisition_info()); // specified in Annex H
        }
        41 => {
            res.append(&mut encode_non_required_view_component()); // specified in Annex H
        }
        42 => {
            res.append(&mut encode_view_dependency_change()); // specified in Annex H
        }
        43 => {
            res.append(&mut encode_operation_points_not_present()); // specified in Annex H
        }
        44 => {
            res.append(&mut encode_base_view_temporal_hrd()); // specified in Annex H
        }
        45 => {
            res.append(&mut encode_frame_packing_arrangement());
        }
        46 => {
            res.append(&mut encode_multiview_view_position()); // specified in Annex H
        }
        47 => {
            res.append(&mut encode_display_orientation());
        }
        48 => {
            res.append(&mut encode_mvcd_scalable_nesting()); // specified in Annex I
        }
        49 => {
            res.append(&mut encode_mvcd_view_scalability_info()); // specified in Annex I
        }
        50 => {
            res.append(&mut encode_depth_representation_info()); // specified in Annex I
        }
        51 => {
            res.append(&mut encode_three_dimensional_reference_displays_info());
            // specified in Annex I
        }
        52 => {
            res.append(&mut encode_depth_timing()); // specified in Annex I
        }
        53 => {
            res.append(&mut encode_depth_sampling_info()); // specified in Annex I
        }
        54 => {
            res.append(&mut encode_constrained_depth_parameter_set_identifier());
            // specified in Annex J
        }
        56 => {
            res.append(&mut encode_green_metadata()); // specified in ISO/IEC 23001-11
        }
        137 => {
            res.append(&mut encode_mastering_display_colour_volume());
        }
        142 => {
            res.append(&mut encode_colour_remapping_info());
        }
        144 => {
            res.append(&mut encode_content_light_level_info());
        }
        147 => {
            res.append(&mut encode_alternative_transfer_characteristics());
        }
        148 => {
            res.append(&mut encode_ambient_viewing_environment());
        }
        149 => {
            res.append(&mut encode_content_colour_volume());
        }
        150 => {
            res.append(&mut encode_equirectangular_projection());
        }
        151 => {
            res.append(&mut encode_cubemap_projection());
        }
        154 => {
            res.append(&mut encode_sphere_rotation());
        }
        155 => {
            res.append(&mut encode_regionwise_packing());
        }
        156 => {
            res.append(&mut encode_omni_viewport());
        }
        181 => {
            res.append(&mut encode_alternative_depth_info()); // specified in Annex I
        }
        200 => {
            res.append(&mut encode_sei_manifest());
        }
        201 => {
            res.append(&mut encode_sei_prefix_indication());
        }
        202 => {
            res.append(&mut encode_annotated_regions());
        }
        205 => {
            res.append(&mut encode_shutter_interval_info());
        }
        _ => {
            debug!(target: "encode","encode_sei_payload - payload_type {} reserved_sei_message", payload_type);
            res.append(&mut encode_reserved_sei_message());
        }
    }

    return res;
}

fn encode_buffering_period(bp: &SEIBufferingPeriod, spses: &[SeqParameterSet]) -> Vec<u8> {
    let mut res = Vec::new();
    res.append(&mut exp_golomb_encode_one(
        bp.seq_parameter_set_id as i32,
        false,
        0,
        false,
    ));

    encoder_formatted_print(
        "SEI (Buffering Period): seq_parameter_set_id",
        &bp.seq_parameter_set_id,
        63,
    );

    let mut cur_sps_wrapper: Option<&SeqParameterSet> = None;
    for i in (0..spses.len()).rev() {
        if spses[i].seq_parameter_set_id == bp.seq_parameter_set_id {
            cur_sps_wrapper = Some(&spses[i]);
            break;
        }
    }

    let cur_sps: &SeqParameterSet;
    match cur_sps_wrapper {
        Some(x) => cur_sps = x,
        _ => panic!(
            "encode_buffering_period - SPS with id {} not found",
            bp.seq_parameter_set_id
        ),
    }

    if cur_sps.vui_parameters_present_flag {
        if cur_sps.vui_parameters.nal_hrd_parameters_present_flag {
            let bit_length = cur_sps
                .vui_parameters
                .nal_hrd_parameters
                .initial_cpb_removal_delay_length_minus1 as usize
                + 1;
            encoder_formatted_print(
                "SEI (Buffering Period): initial_cpb_removal_delay_length_minus1+1",
                &bit_length,
                63,
            );
            encoder_formatted_print(
                "SEI (Buffering Period): cpb_cnt_minus1",
                &cur_sps.vui_parameters.nal_hrd_parameters.cpb_cnt_minus1,
                63,
            );
            for sched_sel_idx in 0..=cur_sps.vui_parameters.nal_hrd_parameters.cpb_cnt_minus1 {
                res.append(&mut generate_unsigned_binary(
                    bp.nal_initial_cpb_removal_delay[sched_sel_idx as usize],
                    bit_length,
                ));
                encoder_formatted_print(
                    "SEI (Buffering Period): NAL initial_cpb_removal_delay[]",
                    &bp.nal_initial_cpb_removal_delay[sched_sel_idx as usize],
                    63,
                );
                res.append(&mut generate_unsigned_binary(
                    bp.nal_initial_cpb_removal_delay_offset[sched_sel_idx as usize],
                    bit_length,
                ));
                encoder_formatted_print(
                    "SEI (Buffering Period): NAL initial_cpb_removal_delay_offset[]",
                    &bp.nal_initial_cpb_removal_delay_offset[sched_sel_idx as usize],
                    63,
                );
            }
        }

        if cur_sps.vui_parameters.vcl_hrd_parameters_present_flag {
            let bit_length = cur_sps
                .vui_parameters
                .vcl_hrd_parameters
                .initial_cpb_removal_delay_length_minus1 as usize
                + 1;
            encoder_formatted_print(
                "SEI (Buffering Period): initial_cpb_removal_delay_length_minus1+1",
                &bit_length,
                63,
            );
            encoder_formatted_print(
                "SEI (Buffering Period): cpb_cnt_minus1",
                &cur_sps.vui_parameters.vcl_hrd_parameters.cpb_cnt_minus1,
                63,
            );
            for sched_sel_idx in 0..=cur_sps.vui_parameters.vcl_hrd_parameters.cpb_cnt_minus1 {
                res.append(&mut generate_unsigned_binary(
                    bp.vcl_initial_cpb_removal_delay[sched_sel_idx as usize],
                    bit_length,
                ));
                encoder_formatted_print(
                    "SEI (Buffering Period): VCL initial_cpb_removal_delay[]",
                    &bp.vcl_initial_cpb_removal_delay[sched_sel_idx as usize],
                    63,
                );
                res.append(&mut generate_unsigned_binary(
                    bp.vcl_initial_cpb_removal_delay_offset[sched_sel_idx as usize],
                    bit_length,
                ));
                encoder_formatted_print(
                    "SEI (Buffering Period): VCL initial_cpb_removal_delay_offset[]",
                    &bp.vcl_initial_cpb_removal_delay_offset[sched_sel_idx as usize],
                    63,
                );
            }
        }
    }

    return res;
}

fn encode_pic_timing(pt: &SEIPicTiming, sps: &SeqParameterSet) -> Vec<u8> {
    let mut bitstream_array = Vec::new();

    let cpb_dpb_delays_present_flag = sps.vui_parameters.nal_hrd_parameters_present_flag
        || sps.vui_parameters.vcl_hrd_parameters_present_flag;
    encoder_formatted_print(
        "SEI (Pic timing): cpb_dpb_delays_present_flag",
        &cpb_dpb_delays_present_flag,
        63,
    );
    if cpb_dpb_delays_present_flag {
        let cpb_bits_to_write;
        let dpb_bits_to_write;

        if sps.vui_parameters.nal_hrd_parameters_present_flag {
            cpb_bits_to_write = sps
                .vui_parameters
                .nal_hrd_parameters
                .cpb_removal_delay_length_minus1
                + 1;
            dpb_bits_to_write = sps
                .vui_parameters
                .nal_hrd_parameters
                .dpb_output_delay_length_minus1
                + 1;
        } else {
            // sps.vui_parameters.vcl_nal_hrd_parameters_present_flag is true
            cpb_bits_to_write = sps
                .vui_parameters
                .vcl_hrd_parameters
                .cpb_removal_delay_length_minus1
                + 1;
            dpb_bits_to_write = sps
                .vui_parameters
                .vcl_hrd_parameters
                .dpb_output_delay_length_minus1
                + 1;
        }

        bitstream_array.append(&mut generate_unsigned_binary(
            pt.cpb_removal_delay,
            cpb_bits_to_write as usize,
        ));
        encoder_formatted_print(
            "SEI (Pic timing): cpb_removal_delay",
            &pt.cpb_removal_delay,
            63,
        );
        bitstream_array.append(&mut generate_unsigned_binary(
            pt.dpb_output_delay,
            dpb_bits_to_write as usize,
        ));
        encoder_formatted_print(
            "SEI (Pic timing): dpb_output_delay",
            &pt.dpb_output_delay,
            63,
        );
    }

    let pic_struct_present_flag = sps.vui_parameters.pic_struct_present_flag;
    encoder_formatted_print(
        "SEI (Pic timing): pic_struct_present_flag",
        &pic_struct_present_flag,
        63,
    );
    if pic_struct_present_flag {
        bitstream_array.append(&mut generate_unsigned_binary(pt.pic_struct, 4));
        encoder_formatted_print("SEI (Pic timing): pic_struct", &pt.pic_struct, 63);

        let num_clock_ts;
        match pt.pic_struct {
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
                println!(
                    "[WARNING] SEI (Pic timing): Reserved value for pic_struct : {}",
                    pt.pic_struct
                );
                num_clock_ts = 0;
            }
            _ => {
                panic!(
                    "SEI (Pic timing): Out of bounds Pic_struct : {}",
                    pt.pic_struct
                );
            }
        }

        for i in 0..num_clock_ts {
            bitstream_array.push(match pt.clock_timestamp_flag[i] {
                true => 1,
                false => 0,
            });
            encoder_formatted_print(
                "SEI (Pic timing): clock_timestamp_flag[]",
                &pt.clock_timestamp_flag[i],
                63,
            );

            let time_offset_length: usize;
            if sps.vui_parameters.nal_hrd_parameters_present_flag {
                time_offset_length =
                    sps.vui_parameters.nal_hrd_parameters.time_offset_length as usize;
            } else {
                // sps.vui_parameters.vcl_nal_hrd_parameters_present_flag is true
                time_offset_length =
                    sps.vui_parameters.vcl_hrd_parameters.time_offset_length as usize;
            }

            if pt.clock_timestamp_flag[i] {
                bitstream_array.append(&mut generate_unsigned_binary(pt.ct_type[i], 2));
                encoder_formatted_print("SEI (Pic timing): ct_type[]", &pt.ct_type[i], 63);
                bitstream_array.push(match pt.nuit_field_based_flag[i] {
                    true => 1,
                    false => 0,
                });
                encoder_formatted_print(
                    "SEI (Pic timing): nuit_field_based_flag[]",
                    &pt.nuit_field_based_flag[i],
                    63,
                );
                bitstream_array.append(&mut generate_unsigned_binary(pt.counting_type[i], 5));
                encoder_formatted_print(
                    "SEI (Pic timing): counting_type[]",
                    &pt.counting_type[i],
                    63,
                );
                bitstream_array.push(match pt.full_timestamp_flag[i] {
                    true => 1,
                    false => 0,
                });
                encoder_formatted_print(
                    "SEI (Pic timing): full_timestamp_flag[]",
                    &pt.full_timestamp_flag[i],
                    63,
                );
                bitstream_array.push(match pt.discontinuity_flag[i] {
                    true => 1,
                    false => 0,
                });
                encoder_formatted_print(
                    "SEI (Pic timing): discontinuity_flag[]",
                    &pt.discontinuity_flag[i],
                    63,
                );
                bitstream_array.push(match pt.cnt_dropped_flag[i] {
                    true => 1,
                    false => 0,
                });
                encoder_formatted_print(
                    "SEI (Pic timing): cnt_dropped_flag[]",
                    &pt.cnt_dropped_flag[i],
                    63,
                );
                bitstream_array.append(&mut generate_unsigned_binary(pt.n_frames[i], 8));
                encoder_formatted_print("SEI (Pic timing): n_frames[]", &pt.n_frames[i], 63);
                if pt.full_timestamp_flag[i] {
                    bitstream_array.append(&mut generate_unsigned_binary(pt.seconds_value[i], 6));
                    encoder_formatted_print(
                        "SEI (Pic timing): seconds_value[]",
                        &pt.seconds_value[i],
                        63,
                    );
                    bitstream_array.append(&mut generate_unsigned_binary(pt.minutes_value[i], 6));
                    encoder_formatted_print(
                        "SEI (Pic timing): minutes_value[]",
                        &pt.minutes_value[i],
                        63,
                    );
                    bitstream_array.append(&mut generate_unsigned_binary(pt.hours_value[i], 5));
                    encoder_formatted_print(
                        "SEI (Pic timing): hours_value[]",
                        &pt.hours_value[i],
                        63,
                    );
                } else {
                    bitstream_array.push(match pt.seconds_flag[i] {
                        true => 1,
                        false => 0,
                    });
                    encoder_formatted_print(
                        "SEI (Pic timing): seconds_flag[]",
                        &pt.seconds_flag[i],
                        63,
                    );
                    if pt.seconds_flag[i] {
                        bitstream_array
                            .append(&mut generate_unsigned_binary(pt.seconds_value[i], 6));
                        encoder_formatted_print(
                            "SEI (Pic timing): seconds_value[]",
                            &pt.seconds_value[i],
                            63,
                        );
                        bitstream_array.push(match pt.minutes_flag[i] {
                            true => 1,
                            false => 0,
                        });
                        encoder_formatted_print(
                            "SEI (Pic timing): minutes_flag[]",
                            &pt.minutes_flag[i],
                            63,
                        );
                        if pt.minutes_flag[i] {
                            bitstream_array
                                .append(&mut generate_unsigned_binary(pt.minutes_value[i], 6));
                            encoder_formatted_print(
                                "SEI (Pic timing): minutes_value[]",
                                &pt.minutes_value[i],
                                63,
                            );
                            bitstream_array.push(match pt.hours_flag[i] {
                                true => 1,
                                false => 0,
                            });
                            encoder_formatted_print(
                                "SEI (Pic timing): hours_flag[]",
                                &pt.hours_flag[i],
                                63,
                            );
                            if pt.hours_flag[i] {
                                bitstream_array
                                    .append(&mut generate_unsigned_binary(pt.hours_value[i], 5));
                                encoder_formatted_print(
                                    "SEI (Pic timing): hours_value[]",
                                    &pt.hours_value[i],
                                    63,
                                );
                            }
                        }
                    }
                }

                if time_offset_length > 0 {
                    bitstream_array.append(&mut generate_unsigned_binary(
                        pt.time_offset[i],
                        time_offset_length,
                    ));
                    encoder_formatted_print(
                        "SEI (Pic timing): time_offset[]",
                        &pt.time_offset[i],
                        63,
                    );
                }
            }
        }
    }

    return bitstream_array;
}

fn encode_pan_scan_rect() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_filler_payload() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_user_data_registered_itu_t_t35() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_user_data_unregistered(uud: &SEIUserDataUnregistered) -> Vec<u8> {
    let mut res = Vec::new();

    for uuid_byte in uud.uuid_iso_iec_11578 {
        res.append(&mut generate_unsigned_binary(uuid_byte as u32, 8));
    }

    match uud.uuid_iso_iec_11578 {
        UUID_APPLE1 => {
            res.append(&mut generate_unsigned_binary(
                uud.user_data_apple1.mystery_param1,
                8,
            ));
        }
        UUID_APPLE2 => {
            res.append(&mut generate_unsigned_binary(
                uud.user_data_apple2.mystery_param1,
                8,
            ));
            res.append(&mut generate_unsigned_binary(
                uud.user_data_apple2.mystery_param2,
                8,
            ));
            res.append(&mut generate_unsigned_binary(
                uud.user_data_apple2.mystery_param3,
                8,
            ));
            res.append(&mut generate_unsigned_binary(
                uud.user_data_apple2.mystery_param4,
                8,
            ));
            res.append(&mut generate_unsigned_binary(
                uud.user_data_apple2.mystery_param5,
                8,
            ));
            res.append(&mut generate_unsigned_binary(
                uud.user_data_apple2.mystery_param6,
                8,
            ));
            res.append(&mut generate_unsigned_binary(
                uud.user_data_apple2.mystery_param7,
                8,
            ));
            res.append(&mut generate_unsigned_binary(
                uud.user_data_apple2.mystery_param8,
                8,
            ));
        }
        _ => {
            for b in uud.user_data_payload_byte.iter() {
                res.append(&mut generate_unsigned_binary(*b as u32, 8));
            }
        }
    }

    res
}

fn encode_recovery_point(rp: &SEIRecoveryPoint) -> Vec<u8> {
    let mut res = Vec::new();

    res.append(&mut exp_golomb_encode_one(
        rp.recovery_frame_cnt as i32,
        false,
        0,
        false,
    ));
    res.push(match rp.exact_match_flag {
        true => 1,
        false => 0,
    });
    res.push(match rp.broken_link_flag {
        true => 1,
        false => 0,
    });
    res.append(&mut generate_unsigned_binary(
        rp.changing_slice_group_idc as u32,
        2,
    ));

    rp.encoder_pretty_print();

    res
}

fn encode_ref_pic_marking_repetition() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_spare_pic() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_scene_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_sub_seq_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_sub_seq_layer_characteristics() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_sub_seq_characteristics() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_full_frame_freeze() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_full_frame_freeze_release() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_full_frame_snapshot() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_progressive_refinement_segment_start() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_progressive_refinement_segment_end() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_motion_constrained_slice_group_set() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_film_grain_characteristics(fgc: &SEIFilmGrainCharacteristics) -> Vec<u8> {
    let mut res = Vec::new();

    res.push(match fgc.film_grain_characteristics_cancel_flag {
        true => 1,
        _ => 0,
    });

    if !fgc.film_grain_characteristics_cancel_flag {
        res.append(&mut generate_unsigned_binary(
            fgc.film_grain_model_id as u32,
            2,
        ));
        res.push(match fgc.separate_colour_description_present_flag {
            true => 1,
            _ => 0,
        });

        if fgc.separate_colour_description_present_flag {
            res.append(&mut generate_unsigned_binary(
                fgc.film_grain_bit_depth_luma_minus8 as u32,
                3,
            ));
            res.append(&mut generate_unsigned_binary(
                fgc.film_grain_bit_depth_chroma_minus8 as u32,
                3,
            ));
            res.push(match fgc.film_grain_full_range_flag {
                true => 1,
                _ => 0,
            });
            res.append(&mut generate_unsigned_binary(
                fgc.film_grain_colour_primaries as u32,
                8,
            ));
            res.append(&mut generate_unsigned_binary(
                fgc.film_grain_transfer_characteristics as u32,
                8,
            ));
            res.append(&mut generate_unsigned_binary(
                fgc.film_grain_matrix_coefficients as u32,
                8,
            ));
        }

        res.append(&mut generate_unsigned_binary(
            fgc.blending_mode_id as u32,
            2,
        ));
        res.append(&mut generate_unsigned_binary(
            fgc.log2_scale_factor as u32,
            4,
        ));

        for c in 0..3 {
            res.push(match fgc.comp_model_present_flag[c] {
                true => 1,
                _ => 0,
            });
        }

        for c in 0..3 {
            if fgc.comp_model_present_flag[c] {
                res.append(&mut generate_unsigned_binary(
                    fgc.num_intensity_intervals_minus1[c] as u32,
                    8,
                ));
                res.append(&mut generate_unsigned_binary(
                    fgc.num_model_values_minus1[c] as u32,
                    3,
                ));

                for i in 0..fgc.num_intensity_intervals_minus1[c] as usize {
                    res.append(&mut generate_unsigned_binary(
                        fgc.intensity_interval_lower_bound[c][i] as u32,
                        8,
                    ));
                    res.append(&mut generate_unsigned_binary(
                        fgc.intensity_interval_upper_bound[c][i] as u32,
                        8,
                    ));
                    for j in 0..fgc.num_model_values_minus1[c] as usize {
                        exp_golomb_encode_one(fgc.comp_model_value[c][i][j], true, 0, false);
                    }
                }
            }
        }
        exp_golomb_encode_one(
            fgc.film_grain_characteristics_repetition_period as i32,
            false,
            0,
            false,
        );
    }

    fgc.encoder_pretty_print();

    res
}

fn encode_deblocking_filter_display_preference() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_stereo_video_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_post_filter_hint() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_tone_mapping_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex G
fn encode_scalability_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex G
fn encode_sub_pic_scalable_layer() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex G
fn encode_non_required_layer_rep() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex G
fn encode_priority_layer_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex G
fn encode_layers_not_present() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex G
fn encode_layer_dependency_change() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex G
fn encode_scalable_nesting() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex G
fn encode_base_layer_temporal_hrd() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex G
fn encode_quality_layer_integrity_check() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex G
fn encode_redundant_pic_property() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex G
fn encode_tl0_dep_rep_index() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex G
fn encode_tl_switching_point() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex H
fn encode_parallel_decoding_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex H
fn encode_mvc_scalable_nesting() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex H
fn encode_view_scalability_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex H
fn encode_multiview_scene_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex H
fn encode_multiview_acquisition_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex H
fn encode_non_required_view_component() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex H
fn encode_view_dependency_change() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex H
fn encode_operation_points_not_present() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex H
fn encode_base_view_temporal_hrd() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_frame_packing_arrangement() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex H
fn encode_multiview_view_position() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_display_orientation() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex I
fn encode_mvcd_scalable_nesting() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex I
fn encode_mvcd_view_scalability_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex I
fn encode_depth_representation_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex I
fn encode_three_dimensional_reference_displays_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex I
fn encode_depth_timing() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex I
fn encode_depth_sampling_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex J
fn encode_constrained_depth_parameter_set_identifier() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in ISO/IEC 23001-11
fn encode_green_metadata() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_mastering_display_colour_volume() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_colour_remapping_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_content_light_level_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_alternative_transfer_characteristics() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_ambient_viewing_environment() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_content_colour_volume() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_equirectangular_projection() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_cubemap_projection() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_sphere_rotation() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_regionwise_packing() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_omni_viewport() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

// specified in Annex I
fn encode_alternative_depth_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_sei_manifest() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_sei_prefix_indication() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_annotated_regions() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_shutter_interval_info() -> Vec<u8> {
    let res = Vec::new();

    return res;
}

fn encode_reserved_sei_message() -> Vec<u8> {
    let res = Vec::new();

    return res;
}
