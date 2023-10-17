//! SEI syntax element randomization.

use crate::common::data_structures::H264DecodedStream;
use crate::common::data_structures::SEIBufferingPeriod;
use crate::common::data_structures::SEIFilmGrainCharacteristics;
use crate::common::data_structures::SEIPayload;
use crate::common::data_structures::SEIPicTiming;
use crate::common::data_structures::SEIRecoveryPoint;
use crate::common::data_structures::SEIUserDataUnregistered;
use crate::common::data_structures::UUID_APPLE1;
use crate::common::data_structures::UUID_APPLE2;
use crate::common::data_structures::UUID_APPLE3;
use crate::vidgen::film::FilmState;
use crate::vidgen::generate_configurations::RandomSEIBufferingPeriodRange;
use crate::vidgen::generate_configurations::RandomSEIFilmGrainCharacteristicsRange;
use crate::vidgen::generate_configurations::RandomSEIPicTimingRange;
use crate::vidgen::generate_configurations::RandomSEIRange;
use crate::vidgen::generate_configurations::RandomSEIRecoveryPointRange;
use crate::vidgen::generate_configurations::RandomSEIUserDataUnregisteredRange;

/// Generate a random SEI NALU
pub fn random_sei(
    sei_idx: usize,
    rconfig: &RandomSEIRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    let num_seis = rconfig.num_seis.sample(film);
    for _ in 0..num_seis {
        let payload_type = rconfig.payload_type.sample(film);
        ds.seis[sei_idx].payload_type.push(payload_type);
        let sei_payload = random_sei_payload(payload_type, rconfig, ds, film);
        ds.seis[sei_idx].payload.push(sei_payload);
        ds.seis[sei_idx].payload_size.push(0); // the encoder will properly put the size
    }
}

/// Generate the payload for the chosen SEI Type
fn random_sei_payload(
    payload_type: u32,
    rconfig: &RandomSEIRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) -> SEIPayload {
    let mut sei_payload = SEIPayload::new();
    match payload_type {
        0 => {
            // buffering period
            sei_payload.buffering_period =
                random_buffering_period(rconfig.random_buffering_period_range, ds, film);
        }

        1 => {
            sei_payload.pic_timing = random_pic_timing(rconfig.random_pic_timing_range, ds, film);
        } /*
        2 => {
        // pan scan rect
        random_pan_scan_rect();
        },
        3 => {
        // filler payload
        random_filler_payload();
        },
        4 => {
        // user data registered ITU T T35
        random_user_data_registered_itu_t_t35();
        }, */
        5 => {
            sei_payload.unregistered_user_data =
                random_user_data_unregistered(rconfig.random_user_data_unregistered_range, film);
        }
        6 => {
            // recovery point
            sei_payload.recovery_point =
                random_recovery_point(rconfig.random_recovery_point_range, film);
        } /*
        7 => {
        // pic marking repetition
        random_ref_pic_marking_repetition();
        },
        8 => {
        // spare pic
        random_spare_pic();
        },
        9 => {
        // scene info
        random_scene_info();
        },
        10 => {
        // sub seq info
        random_sub_seq_info();
        },
        11 => {
        // sub seq layer characteristics
        random_sub_seq_layer_characteristics();
        },
        12 => {
        // sub seq characteristics
        random_sub_seq_characteristics();
        },
        13 => {
        // full_frame_freeze
        random_full_frame_freeze();
        },
        14 => {
        random_full_frame_freeze_release();
        },
        15 => {
        random_full_frame_snapshot();
        },
        16 => {
        random_progressive_refinement_segment_start();
        },
        17 => {
        random_progressive_refinement_segment_end();
        },
        18 => {
        random_motion_constrained_slice_group_set();
        }, */
        19 => {
            sei_payload.film_grain_characteristics =
                random_film_grain_characteristics(rconfig.random_film_grain_char_range, film);
        } /*
        20 => {
        random_deblocking_filter_display_preference();
        },
        21 => {
        random_stereo_video_info();
        },
        22 => {
        random_post_filter_hint();
        },
        23 => {
        random_tone_mapping_info();
        },
        24 => {
        random_scalability_info(); // specified in Annex G
        },
        25 => {
        random_sub_pic_scalable_layer(); // specified in Annex G
        },
        26 => {
        random_non_required_layer_rep(); // specified in Annex G
        },
        27 => {
        random_priority_layer_info(); // specified in Annex G
        },
        28 => {
        random_layers_not_present(); // specified in Annex G
        },
        29 => {
        random_layer_dependency_change(); // specified in Annex G
        },
        30 => {
        random_scalable_nesting(); // specified in Annex G
        },
        31 => {
        random_base_layer_temporal_hrd(); // specified in Annex G
        },
        32 => {
        random_quality_layer_integrity_check(); // specified in Annex G
        },
        33 => {
        random_redundant_pic_property(); // specified in Annex G
        },
        34 => {
        random_tl0_dep_rep_index(); // specified in Annex G
        },
        35 => {
        random_tl_switching_point(); // specified in Annex G
        },
        36 => {
        random_parallel_decoding_info(); // specified in Annex H
        },
        37 => {
        random_mvc_scalable_nesting(); // specified in Annex H
        },
        38 => {
        random_view_scalability_info(); // specified in Annex H
        },
        39 => {
        random_multiview_scene_info(); // specified in Annex H
        },
        40 => {
        random_multiview_acquisition_info(); // specified in Annex H
        },
        41 => {
        random_non_required_view_component(); // specified in Annex H
        },
        42 => {
        random_view_dependency_change(); // specified in Annex H
        },
        43 => {
        random_operation_points_not_present(); // specified in Annex H
        },
        44 => {
        random_base_view_temporal_hrd(); // specified in Annex H
        },
        45 => {
        random_frame_packing_arrangement();
        },
        46 => {
        random_multiview_view_position(); // specified in Annex H
        },
        47 => {
        random_display_orientation();
        },
        48 => {
        random_mvcd_scalable_nesting(); // specified in Annex I
        }
        49 => {
        random_mvcd_view_scalability_info(); // specified in Annex I
        },
        50 => {
        random_depth_representation_info(); // specified in Annex I
        },
        51 => {
        random_three_dimensional_reference_displays_info(); // specified in Annex I
        },
        52 => {
        random_depth_timing(); // specified in Annex I
        },
        53 => {
        random_depth_sampling_info(); // specified in Annex I
        },
        54 => {
        random_constrained_depth_parameter_set_identifier(); // specified in Annex J
        },
        56 => {
        random_green_metadata(); // specified in ISO/IEC 23001-11
        },
        137 => {
        random_mastering_display_colour_volume();
        },
        142 => {
        random_colour_remapping_info();
        },
        144 => {
        random_content_light_level_info();
        },
        147 => {
        random_alternative_transfer_characteristics();
        },
        148 => {
        random_ambient_viewing_environment();
        },
        149 => {
        random_content_colour_volume();
        },
        150 => {
        random_equirectangular_projection();
        },
        151 => {
        random_cubemap_projection();
        },
        154 => {
        random_sphere_rotation();
        },
        155 => {
        random_regionwise_packing();
        },
        156 => {
        random_omni_viewport();
        },
        181 => {
        random_alternative_depth_info(); // specified in Annex I
        },
        200 => {
        random_sei_manifest();
        },
        201 => {
        random_sei_prefix_indication();
        },
        202 => {
        random_annotated_regions();
        },
        205 => {
        random_shutter_interval_info();
        },*/
        _ => {
            sei_payload.buffering_period =
                random_buffering_period(rconfig.random_buffering_period_range, ds, film);
        }
    }

    sei_payload
}

/// Generate an SEI Payload of type 0 - Buffering period
fn random_buffering_period(
    rconfig: RandomSEIBufferingPeriodRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) -> SEIBufferingPeriod {
    let mut bp = SEIBufferingPeriod::new();

    // TODO: use any available SPS index.
    // We currently use the last generated SPS as we ran into issues with
    // multiple SPSes with the same ID
    //let sps_idx = rconfig.seq_parameter_set_id.sample(0, ds.spses.len()-1, film);
    let sps_idx = ds.spses.len() - 1;
    bp.seq_parameter_set_id = ds.spses[sps_idx].seq_parameter_set_id;

    if ds.spses[sps_idx]
        .vui_parameters
        .nal_hrd_parameters_present_flag
    {
        for _ in 0..=ds.spses[sps_idx]
            .vui_parameters
            .nal_hrd_parameters
            .cpb_cnt_minus1
        {
            bp.nal_initial_cpb_removal_delay
                .push(rconfig.initial_cpb_removal_delay.sample(film));
            bp.nal_initial_cpb_removal_delay_offset
                .push(rconfig.initial_cpb_removal_delay_offset.sample(film));
        }
    }

    if ds.spses[sps_idx]
        .vui_parameters
        .vcl_hrd_parameters_present_flag
    {
        for _ in 0..=ds.spses[sps_idx]
            .vui_parameters
            .vcl_hrd_parameters
            .cpb_cnt_minus1
        {
            bp.vcl_initial_cpb_removal_delay
                .push(rconfig.initial_cpb_removal_delay.sample(film));
            bp.vcl_initial_cpb_removal_delay_offset
                .push(rconfig.initial_cpb_removal_delay_offset.sample(film));
        }
    }

    bp
}

/// Generate an SEI Payload of type 1 - Pic Timing
fn random_pic_timing(
    rconfig: RandomSEIPicTimingRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) -> SEIPicTiming {
    let mut pt = SEIPicTiming::new();

    let sps = &ds.spses[ds.spses.len() - 1];
    let cpb_dpb_delays_present_flag = sps.vui_parameters.nal_hrd_parameters_present_flag
        || sps.vui_parameters.vcl_hrd_parameters_present_flag;

    if cpb_dpb_delays_present_flag {
        // bits to read comes from
        let cpb_max: u64;
        let dpb_max: u64;

        if sps.vui_parameters.nal_hrd_parameters_present_flag {
            cpb_max = 1
                << (sps
                    .vui_parameters
                    .nal_hrd_parameters
                    .cpb_removal_delay_length_minus1
                    + 1);
            dpb_max = 1
                << (sps
                    .vui_parameters
                    .nal_hrd_parameters
                    .dpb_output_delay_length_minus1
                    + 1);
        } else {
            // sps.vui_parameters.vcl_nal_hrd_parameters_present_flag is true
            cpb_max = 1
                << (sps
                    .vui_parameters
                    .vcl_hrd_parameters
                    .cpb_removal_delay_length_minus1
                    + 1);
            dpb_max = 1
                << (sps
                    .vui_parameters
                    .vcl_hrd_parameters
                    .dpb_output_delay_length_minus1
                    + 1);
        }

        pt.cpb_removal_delay = rconfig
            .cpb_removal_delay
            .sample(0, (cpb_max - 1) as u32, film);
        pt.dpb_output_delay = rconfig
            .dpb_output_delay
            .sample(0, (dpb_max - 1) as u32, film);
    }

    let pic_struct_present_flag = sps.vui_parameters.pic_struct_present_flag;

    if pic_struct_present_flag {
        let num_clock_ts: usize;

        pt.pic_struct = rconfig.pic_struct.sample(film);
        // Table D-1 - Interpretation of pic_struct
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
                //println!(
                //    "[WARNING] SEI (Pic timing): Undefined value : {}",
                //    pt.pic_struct
                //);
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
            pt.clock_timestamp_flag
                .push(rconfig.clock_timestamp_flag.sample(film));

            if pt.clock_timestamp_flag[i] {
                pt.ct_type.push(rconfig.ct_type.sample(film));
                pt.nuit_field_based_flag
                    .push(rconfig.nuit_field_based_flag.sample(film));
                pt.counting_type.push(rconfig.counting_type.sample(film));
                pt.full_timestamp_flag
                    .push(rconfig.full_timestamp_flag.sample(film));
                pt.discontinuity_flag
                    .push(rconfig.discontinuity_flag.sample(film));
                pt.cnt_dropped_flag
                    .push(rconfig.cnt_dropped_flag.sample(film));
                pt.n_frames.push(rconfig.n_frames.sample(film));

                if pt.full_timestamp_flag[i] {
                    pt.seconds_value.push(rconfig.seconds_value.sample(film));
                    pt.minutes_value.push(rconfig.minutes_value.sample(film));
                    pt.hours_value.push(rconfig.hours_value.sample(film));

                    // push the flag values
                    pt.seconds_flag.push(true);
                    pt.minutes_flag.push(true);
                    pt.hours_flag.push(true);
                } else {
                    pt.seconds_flag.push(rconfig.seconds_flag.sample(film));
                    if pt.seconds_flag[i] {
                        pt.seconds_value.push(rconfig.seconds_value.sample(film));
                        pt.minutes_flag.push(rconfig.minutes_flag.sample(film));
                        if pt.minutes_flag[i] {
                            pt.minutes_value.push(rconfig.minutes_value.sample(film));
                            pt.hours_flag.push(rconfig.hours_flag.sample(film));
                            if pt.hours_flag[i] {
                                pt.hours_value.push(rconfig.hours_value.sample(film));
                            } else {
                                pt.hours_value.push(0);
                            }
                        } else {
                            pt.minutes_value.push(0);
                            pt.hours_flag.push(false);
                            pt.hours_value.push(0);
                        }
                    } else {
                        pt.seconds_value.push(0);
                        pt.minutes_flag.push(false);
                        pt.minutes_value.push(0);
                        pt.hours_flag.push(false);
                        pt.hours_value.push(0);
                    }
                }

                let time_offset_length;

                if sps.vui_parameters.nal_hrd_parameters_present_flag {
                    time_offset_length = sps.vui_parameters.nal_hrd_parameters.time_offset_length;
                } else {
                    // sps.vui_parameters.vcl_nal_hrd_parameters_present_flag is true
                    time_offset_length = sps.vui_parameters.vcl_hrd_parameters.time_offset_length;
                }

                let time_offset_max = (1 << time_offset_length) - 1;

                if time_offset_length > 0 {
                    pt.time_offset
                        .push(rconfig.time_offset.sample(0, time_offset_max, film));
                // i(v)
                } else {
                    pt.time_offset.push(0);
                }
            } else {
                pt.ct_type.push(0);
                pt.nuit_field_based_flag.push(false);
                pt.counting_type.push(0);
                pt.full_timestamp_flag.push(false);
                pt.discontinuity_flag.push(false);
                pt.cnt_dropped_flag.push(false);
                pt.n_frames.push(0);
                pt.seconds_value.push(0);
                pt.minutes_value.push(0);
                pt.hours_value.push(0);
                pt.seconds_flag.push(false);
                pt.minutes_flag.push(false);
                pt.hours_flag.push(false);
                pt.time_offset.push(0);
            }
        }
    }

    pt
}

/// Generate an SEI Payload of type 5 - Unregistered user data
fn random_user_data_unregistered(
    rconfig: RandomSEIUserDataUnregisteredRange,
    film: &mut FilmState,
) -> SEIUserDataUnregistered {
    let mut udu = SEIUserDataUnregistered::new();

    udu.uuid_iso_iec_11578 = match rconfig.uuid_iso_iec_11578.sample(film) {
        0 => UUID_APPLE1,
        1 => UUID_APPLE2,
        2 => UUID_APPLE3,
        _ => {
            //println!("[WARNING] random_user_data_unregistered - uuid_iso_iec_115678 sampling range too large, choosing default");
            UUID_APPLE1
        }
    };

    match udu.uuid_iso_iec_11578 {
        UUID_APPLE1 => {
            udu.user_data_apple1.mystery_param1 =
                rconfig.user_data_apple1.mystery_param1.sample(film);
        }
        UUID_APPLE2 => {
            udu.user_data_apple2.mystery_param1 =
                rconfig.user_data_apple2.mystery_param1.sample(film);
            udu.user_data_apple2.mystery_param2 =
                rconfig.user_data_apple2.mystery_param2.sample(film);
            udu.user_data_apple2.mystery_param3 =
                rconfig.user_data_apple2.mystery_param3.sample(film);
            udu.user_data_apple2.mystery_param4 =
                rconfig.user_data_apple2.mystery_param4.sample(film);
            udu.user_data_apple2.mystery_param5 =
                rconfig.user_data_apple2.mystery_param5.sample(film);
            udu.user_data_apple2.mystery_param6 =
                rconfig.user_data_apple2.mystery_param6.sample(film);
            udu.user_data_apple2.mystery_param7 =
                rconfig.user_data_apple2.mystery_param7.sample(film);
            udu.user_data_apple2.mystery_param8 =
                rconfig.user_data_apple2.mystery_param8.sample(film);
        }
        UUID_APPLE3 => {
            udu.user_data_payload_byte = vec![b'm', b'e', b't', b'a']; // checks for "meta" at the start

            // TODO: this claims to get parsed as XML, so we need to see what the parameters are
            let length = rconfig.user_data_payload_length.sample(film);
            udu.user_data_payload_byte
                .extend(&film.read_film_bytes(length));
        }
        _ => {
            //println!(
            //    "[WARNING] random_user_data_unregistered - unknown uuid_iso_iec_11578 {:?}",
            //    udu.uuid_iso_iec_11578
            //);
            udu.user_data_payload_byte = vec![0xf, 0xe, 0xe, 0xd, 0xf, 0x0, 0x0, 0xd];
        }
    }

    udu
}

/// Generate an SEI Payload of type 6 - Recovery point
fn random_recovery_point(
    rconfig: RandomSEIRecoveryPointRange,
    film: &mut FilmState,
) -> SEIRecoveryPoint {
    let mut rp = SEIRecoveryPoint::new();

    rp.recovery_frame_cnt = rconfig.recovery_frame_cnt.sample(film);
    rp.exact_match_flag = rconfig.exact_match_flag.sample(film);
    rp.broken_link_flag = rconfig.broken_link_flag.sample(film);
    rp.changing_slice_group_idc = rconfig.changing_slice_group_idc.sample(film) as u8;

    rp
}

/// Generate an SEI Payload of type 19 - Film Grain Characteristics
fn random_film_grain_characteristics(
    rconfig: RandomSEIFilmGrainCharacteristicsRange,
    film: &mut FilmState,
) -> SEIFilmGrainCharacteristics {
    let mut fgc = SEIFilmGrainCharacteristics::new();

    fgc.film_grain_characteristics_cancel_flag =
        rconfig.film_grain_characteristics_cancel_flag.sample(film);

    if !fgc.film_grain_characteristics_cancel_flag {
        fgc.film_grain_model_id = rconfig.film_grain_model_id.sample(film) as u8;
        fgc.separate_colour_description_present_flag = rconfig
            .separate_colour_description_present_flag
            .sample(film);

        if fgc.separate_colour_description_present_flag {
            fgc.film_grain_bit_depth_luma_minus8 =
                rconfig.film_grain_bit_depth_luma_minus8.sample(film) as u8;
            fgc.film_grain_bit_depth_chroma_minus8 =
                rconfig.film_grain_bit_depth_chroma_minus8.sample(film) as u8;
            fgc.film_grain_full_range_flag = rconfig.film_grain_full_range_flag.sample(film);
            fgc.film_grain_colour_primaries =
                rconfig.film_grain_colour_primaries.sample(film) as u8;
            fgc.film_grain_transfer_characteristics =
                rconfig.film_grain_transfer_characteristics.sample(film) as u8;
            fgc.film_grain_matrix_coefficients =
                rconfig.film_grain_matrix_coefficients.sample(film) as u8;
        }
        fgc.blending_mode_id = rconfig.blending_mode_id.sample(film) as u8;
        fgc.log2_scale_factor = rconfig.log2_scale_factor.sample(film) as u8;

        for _ in 0..3 {
            fgc.comp_model_present_flag
                .push(rconfig.comp_model_present_flag.sample(film));
        }

        for c in 0..3 {
            if fgc.comp_model_present_flag[c] {
                fgc.num_intensity_intervals_minus1
                    .push(rconfig.num_intensity_intervals_minus1.sample(film) as u8);
                fgc.num_model_values_minus1
                    .push(rconfig.num_model_values_minus1.sample(film) as u8);

                fgc.intensity_interval_lower_bound.push(Vec::new());
                fgc.intensity_interval_upper_bound.push(Vec::new());
                fgc.comp_model_value.push(Vec::new());
                for i in 0..fgc.num_intensity_intervals_minus1[c] as usize {
                    fgc.intensity_interval_lower_bound[c]
                        .push(rconfig.intensity_interval_lower_bound.sample(film) as u8);
                    fgc.intensity_interval_upper_bound[c]
                        .push(rconfig.intensity_interval_upper_bound.sample(film) as u8);

                    fgc.comp_model_value[c].push(Vec::new());
                    for _ in 0..fgc.num_model_values_minus1[c] as usize {
                        fgc.comp_model_value[c][i].push(rconfig.comp_model_value.sample(film));
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
        fgc.film_grain_characteristics_repetition_period = rconfig
            .film_grain_characteristics_repetition_period
            .sample(film);
    }
    fgc
}
