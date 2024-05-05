//! H.265 syntax element decoding.

use crate::common::helper::ByteStream;
use crate::decoder::expgolomb::exp_golomb_decode_one_wrapper;
use crate::decoder::nalu::split_into_nalu;
use crate::experimental::h265_data_structures::H265DecodedStream;
use crate::experimental::h265_data_structures::H265NALUHeader;
use crate::experimental::h265_data_structures::H265SPS3DExtension;
use crate::experimental::h265_data_structures::H265SPSMultilayerExtension;
use crate::experimental::h265_data_structures::H265SPSRangeExtension;
use crate::experimental::h265_data_structures::H265SPSSCCExtension;
use crate::experimental::h265_data_structures::H265SeqParameterSet;
use crate::experimental::h265_data_structures::H265VideoParameterSet;
use crate::experimental::h265_data_structures::H265VuiParameters;
use crate::experimental::h265_data_structures::NalUnitType;
use crate::experimental::h265_data_structures::ProfileTierLevel;
use crate::experimental::h265_data_structures::ScalingListData;
use crate::experimental::h265_data_structures::ShortTermRefPic;
use log::debug;
use std::cmp::min;
use std::time::SystemTime;

fn h265_decode_nalu_header(long_start_code: bool, bs: &mut ByteStream) -> H265NALUHeader {
    let mut nh = H265NALUHeader::new();

    nh.forbidden_zero_bit = bs.read_bits(1) as u8;
    let nal_unit_type = bs.read_bits(6);

    match nal_unit_type {
        0 => nh.nal_unit_type = NalUnitType::NalUnitCodedSliceTrailN, // 0
        1 => nh.nal_unit_type = NalUnitType::NalUnitCodedSliceTrailR, // 1
        2 => nh.nal_unit_type = NalUnitType::NalUnitCodedSliceTsaN,   // 2
        3 => nh.nal_unit_type = NalUnitType::NalUnitCodedSliceTsaR,   // 3
        4 => nh.nal_unit_type = NalUnitType::NalUnitCodedSliceStsaN,  // 4
        5 => nh.nal_unit_type = NalUnitType::NalUnitCodedSliceStsaR,  // 5
        6 => nh.nal_unit_type = NalUnitType::NalUnitCodedSliceRadlN,  // 6
        7 => nh.nal_unit_type = NalUnitType::NalUnitCodedSliceRadlR,  // 7
        8 => nh.nal_unit_type = NalUnitType::NalUnitCodedSliceRaslN,  // 8
        9 => nh.nal_unit_type = NalUnitType::NalUnitCodedSliceRaslR,  // 9
        10 => nh.nal_unit_type = NalUnitType::NalUnitReservedVclN10,
        11 => nh.nal_unit_type = NalUnitType::NalUnitReservedVclR11,
        12 => nh.nal_unit_type = NalUnitType::NalUnitReservedVclN12,
        13 => nh.nal_unit_type = NalUnitType::NalUnitReservedVclR13,
        14 => nh.nal_unit_type = NalUnitType::NalUnitReservedVclN14,
        15 => nh.nal_unit_type = NalUnitType::NalUnitCodedSliceBlaWLp,
        16 => nh.nal_unit_type = NalUnitType::NalUnitReservedVclR15, // 16
        17 => nh.nal_unit_type = NalUnitType::NalUnitCodedSliceBlaWRadl, // 17
        18 => nh.nal_unit_type = NalUnitType::NalUnitCodedSliceBlaNLp, // 18
        19 => nh.nal_unit_type = NalUnitType::NalUnitCodedSliceIdrWRadl, // 19
        20 => nh.nal_unit_type = NalUnitType::NalUnitCodedSliceIdrNLp, // 20
        21 => nh.nal_unit_type = NalUnitType::NalUnitCodedSliceCra,  // 21
        22 => nh.nal_unit_type = NalUnitType::NalUnitReservedIrapVcl22,
        23 => nh.nal_unit_type = NalUnitType::NalUnitReservedIrapVcl23,
        24 => nh.nal_unit_type = NalUnitType::NalUnitReservedVcl24,
        25 => nh.nal_unit_type = NalUnitType::NalUnitReservedVcl25,
        26 => nh.nal_unit_type = NalUnitType::NalUnitReservedVcl26,
        27 => nh.nal_unit_type = NalUnitType::NalUnitReservedVcl27,
        28 => nh.nal_unit_type = NalUnitType::NalUnitReservedVcl28,
        29 => nh.nal_unit_type = NalUnitType::NalUnitReservedVcl29,
        30 => nh.nal_unit_type = NalUnitType::NalUnitReservedVcl30,
        31 => nh.nal_unit_type = NalUnitType::NalUnitReservedVcl31,
        32 => nh.nal_unit_type = NalUnitType::NalUnitVps, // 32
        33 => nh.nal_unit_type = NalUnitType::NalUnitSps, // 33
        34 => nh.nal_unit_type = NalUnitType::NalUnitPps, // 34
        35 => nh.nal_unit_type = NalUnitType::NalUnitAccessUnitDelimiter, // 35
        36 => nh.nal_unit_type = NalUnitType::NalUnitEos, // 36
        37 => nh.nal_unit_type = NalUnitType::NalUnitEob, // 37
        38 => nh.nal_unit_type = NalUnitType::NalUnitFillerData, // 38
        39 => nh.nal_unit_type = NalUnitType::NalUnitPrefixSei, // 39
        40 => nh.nal_unit_type = NalUnitType::NalUnitSuffixSei, // 40
        41 => nh.nal_unit_type = NalUnitType::NalUnitReservedNvcl41,
        42 => nh.nal_unit_type = NalUnitType::NalUnitReservedNvcl42,
        43 => nh.nal_unit_type = NalUnitType::NalUnitReservedNvcl43,
        44 => nh.nal_unit_type = NalUnitType::NalUnitReservedNvcl44,
        45 => nh.nal_unit_type = NalUnitType::NalUnitReservedNvcl45,
        46 => nh.nal_unit_type = NalUnitType::NalUnitReservedNvcl46,
        47 => nh.nal_unit_type = NalUnitType::NalUnitReservedNvcl47,
        48 => nh.nal_unit_type = NalUnitType::NalUnitUnspecified48,
        49 => nh.nal_unit_type = NalUnitType::NalUnitUnspecified49,
        50 => nh.nal_unit_type = NalUnitType::NalUnitUnspecified50,
        51 => nh.nal_unit_type = NalUnitType::NalUnitUnspecified51,
        52 => nh.nal_unit_type = NalUnitType::NalUnitUnspecified52,
        53 => nh.nal_unit_type = NalUnitType::NalUnitUnspecified53,
        54 => nh.nal_unit_type = NalUnitType::NalUnitUnspecified54,
        55 => nh.nal_unit_type = NalUnitType::NalUnitUnspecified55,
        56 => nh.nal_unit_type = NalUnitType::NalUnitUnspecified56,
        57 => nh.nal_unit_type = NalUnitType::NalUnitUnspecified57,
        58 => nh.nal_unit_type = NalUnitType::NalUnitUnspecified58,
        59 => nh.nal_unit_type = NalUnitType::NalUnitUnspecified59,
        60 => nh.nal_unit_type = NalUnitType::NalUnitUnspecified60,
        61 => nh.nal_unit_type = NalUnitType::NalUnitUnspecified61,
        62 => nh.nal_unit_type = NalUnitType::NalUnitUnspecified62,
        63 => nh.nal_unit_type = NalUnitType::NalUnitUnspecified63,
        _ => nh.nal_unit_type = NalUnitType::NalUnitInvalid,
    };

    nh.nuh_layer_id = bs.read_bits(6) as u8;
    nh.nuh_temporal_id_plus1 = bs.read_bits(3) as u8;

    debug!(target: "decode","");
    debug!(target: "decode","");
    debug!(target: "decode","Annex B NALU w/ {} startcode, len {}, forbidden_bit {}, nuh_layer_id {}, nuh_temporal_id_plus1 {}, nal_unit_type {:?}",
            { if long_start_code {"long" } else {"short"} }, bs.bytestream.len() + 1, nh.forbidden_zero_bit, nh.nuh_layer_id, nh.nuh_temporal_id_plus1, nh.nal_unit_type);
    debug!(target: "decode","");

    nh
}

fn h265_decode_profile_tier_level(
    profile_present_flag: bool,
    max_sub_layers_minus1: usize,
    bs: &mut ByteStream,
) -> ProfileTierLevel {
    let mut ptl = ProfileTierLevel::new();

    if profile_present_flag {
        ptl.general_profile_space = bs.read_bits(2) as u8; // u(2)
        ptl.general_tier_flag = 1 == bs.read_bits(1); // u(1)
        ptl.general_profile_idc = bs.read_bits(5) as u8; // u(5)
        for i in 0..32 {
            ptl.general_profile_compatibility_flag[i] = 1 == bs.read_bits(1);
        }
        ptl.general_progressive_source_flag = 1 == bs.read_bits(1);
        ptl.general_interlaced_source_flag = 1 == bs.read_bits(1);
        ptl.general_non_packed_constraint_flag = 1 == bs.read_bits(1);
        ptl.general_frame_only_constraint_flag = 1 == bs.read_bits(1);
        if ptl.general_profile_idc == 4
            || ptl.general_profile_compatibility_flag[4]
            || ptl.general_profile_idc == 5
            || ptl.general_profile_compatibility_flag[5]
            || ptl.general_profile_idc == 6
            || ptl.general_profile_compatibility_flag[6]
            || ptl.general_profile_idc == 7
            || ptl.general_profile_compatibility_flag[7]
            || ptl.general_profile_idc == 8
            || ptl.general_profile_compatibility_flag[8]
            || ptl.general_profile_idc == 9
            || ptl.general_profile_compatibility_flag[9]
            || ptl.general_profile_idc == 10
            || ptl.general_profile_compatibility_flag[10]
            || ptl.general_profile_idc == 11
            || ptl.general_profile_compatibility_flag[11]
        {
            // The number of bits in this syntax structure is not affected by this condition
            ptl.general_max_12bit_constraint_flag = 1 == bs.read_bits(1);
            ptl.general_max_10bit_constraint_flag = 1 == bs.read_bits(1);
            ptl.general_max_8bit_constraint_flag = 1 == bs.read_bits(1);
            ptl.general_max_422chroma_constraint_flag = 1 == bs.read_bits(1);
            ptl.general_max_420chroma_constraint_flag = 1 == bs.read_bits(1);
            ptl.general_max_monochrome_constraint_flag = 1 == bs.read_bits(1);
            ptl.general_intra_constraint_flag = 1 == bs.read_bits(1);
            ptl.general_one_picture_only_constraint_flag = 1 == bs.read_bits(1);
            ptl.general_lower_bit_rate_constraint_flag = 1 == bs.read_bits(1);

            if ptl.general_profile_idc == 5
                || ptl.general_profile_compatibility_flag[5]
                || ptl.general_profile_idc == 9
                || ptl.general_profile_compatibility_flag[9]
                || ptl.general_profile_idc == 10
                || ptl.general_profile_compatibility_flag[10]
                || ptl.general_profile_idc == 11
                || ptl.general_profile_compatibility_flag[11]
            {
                ptl.general_max_14bit_constraint_flag = 1 == bs.read_bits(1);
                ptl.general_reserved_zero_33bits = bs.read_bits64(33);
            } else {
                ptl.general_reserved_zero_34bits = bs.read_bits64(34); // u(34)
            }
        } else if ptl.general_profile_idc == 2 || ptl.general_profile_compatibility_flag[2] {
            ptl.general_reserved_zero_7bits = bs.read_bits(7) as u8; // u(7)
            ptl.general_one_picture_only_constraint_flag = 1 == bs.read_bits(1); // u(1)
            ptl.general_reserved_zero_35bits = bs.read_bits64(35); // u(35)
        } else {
            ptl.general_reserved_zero_43bits = bs.read_bits64(43);
        }
        if ptl.general_profile_idc == 1
            || ptl.general_profile_compatibility_flag[1]
            || ptl.general_profile_idc == 2
            || ptl.general_profile_compatibility_flag[2]
            || ptl.general_profile_idc == 3
            || ptl.general_profile_compatibility_flag[3]
            || ptl.general_profile_idc == 4
            || ptl.general_profile_compatibility_flag[4]
            || ptl.general_profile_idc == 5
            || ptl.general_profile_compatibility_flag[5]
            || ptl.general_profile_idc == 9
            || ptl.general_profile_compatibility_flag[9]
            || ptl.general_profile_idc == 11
            || ptl.general_profile_compatibility_flag[11]
        {
            // The number of bits in this syntax structure is not affected by this condition
            ptl.general_inbld_flag = 1 == bs.read_bits(1);
        } else {
            ptl.general_reserved_zero_bit = bs.read_bits(1) as u8;
        }
    }

    ptl.general_level_idc = bs.read_bits(8) as u8; // u(8)

    for _ in 0..max_sub_layers_minus1 {
        ptl.sub_layer_profile_present_flag
            .push(1 == bs.read_bits(1));
        ptl.sub_layer_level_present_flag.push(1 == bs.read_bits(1));
    }

    if max_sub_layers_minus1 > 0 {
        ptl.reserved_zero_2bits = vec![0; 8];
        for i in max_sub_layers_minus1..8 {
            ptl.reserved_zero_2bits[i] = bs.read_bits(2) as u8;
        }
    }

    for i in 0..max_sub_layers_minus1 {
        // Initialize loop parameters for proper alignment
        ptl.sub_layer_profile_space.push(0);
        ptl.sub_layer_tier_flag.push(false);
        ptl.sub_layer_profile_idc.push(0);
        ptl.sub_layer_profile_compatibility_flag.push(Vec::new());
        ptl.sub_layer_progressive_source_flag.push(false);
        ptl.sub_layer_interlaced_source_flag.push(false);
        ptl.sub_layer_non_packed_constraint_flag.push(false);
        ptl.sub_layer_frame_only_constraint_flag.push(false);
        ptl.sub_layer_max_12bit_constraint_flag.push(false);
        ptl.sub_layer_max_10bit_constraint_flag.push(false);
        ptl.sub_layer_max_8bit_constraint_flag.push(false);
        ptl.sub_layer_max_422chroma_constraint_flag.push(false);
        ptl.sub_layer_max_420chroma_constraint_flag.push(false);
        ptl.sub_layer_max_monochrome_constraint_flag.push(false);
        ptl.sub_layer_intra_constraint_flag.push(false);
        ptl.sub_layer_one_picture_only_constraint_flag.push(false);
        ptl.sub_layer_lower_bit_rate_constraint_flag.push(false);
        ptl.sub_layer_max_14bit_constraint_flag.push(false);
        ptl.sub_layer_reserved_zero_33bits.push(0);
        ptl.sub_layer_reserved_zero_34bits.push(0);
        ptl.sub_layer_reserved_zero_7bits.push(0);
        ptl.sub_layer_one_picture_only_constraint_flag.push(false);
        ptl.sub_layer_reserved_zero_35bits.push(0);
        ptl.sub_layer_reserved_zero_43bits.push(0);
        ptl.sub_layer_inbld_flag.push(false);
        ptl.sub_layer_reserved_zero_bit.push(0);
        ptl.sub_layer_level_idc.push(0);

        if ptl.sub_layer_profile_present_flag[i] {
            ptl.sub_layer_profile_space[i] = bs.read_bits(2) as u8;
            ptl.sub_layer_tier_flag[i] = 1 == bs.read_bits(1);
            ptl.sub_layer_profile_idc[i] = bs.read_bits(5) as u8;

            for _ in 0..32 {
                ptl.sub_layer_profile_compatibility_flag[i].push(1 == bs.read_bits(1));
            }

            ptl.sub_layer_progressive_source_flag[i] = 1 == bs.read_bits(1);
            ptl.sub_layer_interlaced_source_flag[i] = 1 == bs.read_bits(1);
            ptl.sub_layer_non_packed_constraint_flag[i] = 1 == bs.read_bits(1);
            ptl.sub_layer_frame_only_constraint_flag[i] = 1 == bs.read_bits(1);

            if ptl.sub_layer_profile_idc[i] == 4
                || ptl.sub_layer_profile_compatibility_flag[i][4]
                || ptl.sub_layer_profile_idc[i] == 5
                || ptl.sub_layer_profile_compatibility_flag[i][5]
                || ptl.sub_layer_profile_idc[i] == 6
                || ptl.sub_layer_profile_compatibility_flag[i][6]
                || ptl.sub_layer_profile_idc[i] == 7
                || ptl.sub_layer_profile_compatibility_flag[i][7]
                || ptl.sub_layer_profile_idc[i] == 8
                || ptl.sub_layer_profile_compatibility_flag[i][8]
                || ptl.sub_layer_profile_idc[i] == 9
                || ptl.sub_layer_profile_compatibility_flag[i][9]
                || ptl.sub_layer_profile_idc[i] == 10
                || ptl.sub_layer_profile_compatibility_flag[i][10]
                || ptl.sub_layer_profile_idc[i] == 11
                || ptl.sub_layer_profile_compatibility_flag[i][11]
            {
                // The number of bits in this syntax structure is not affected by this condition
                ptl.sub_layer_max_12bit_constraint_flag[i] = 1 == bs.read_bits(1);
                ptl.sub_layer_max_10bit_constraint_flag[i] = 1 == bs.read_bits(1);
                ptl.sub_layer_max_8bit_constraint_flag[i] = 1 == bs.read_bits(1);
                ptl.sub_layer_max_422chroma_constraint_flag[i] = 1 == bs.read_bits(1);
                ptl.sub_layer_max_420chroma_constraint_flag[i] = 1 == bs.read_bits(1);
                ptl.sub_layer_max_monochrome_constraint_flag[i] = 1 == bs.read_bits(1);
                ptl.sub_layer_intra_constraint_flag[i] = 1 == bs.read_bits(1);
                ptl.sub_layer_one_picture_only_constraint_flag[i] = 1 == bs.read_bits(1);
                ptl.sub_layer_lower_bit_rate_constraint_flag[i] = 1 == bs.read_bits(1);

                if ptl.sub_layer_profile_idc[i] == 5
                    || ptl.sub_layer_profile_compatibility_flag[i][5]
                    || ptl.sub_layer_profile_idc[i] == 9
                    || ptl.sub_layer_profile_compatibility_flag[i][9]
                    || ptl.sub_layer_profile_idc[i] == 10
                    || ptl.sub_layer_profile_compatibility_flag[i][10]
                    || ptl.sub_layer_profile_idc[i] == 11
                    || ptl.sub_layer_profile_compatibility_flag[i][11]
                {
                    ptl.sub_layer_max_14bit_constraint_flag[i] = 1 == bs.read_bits(1);
                    ptl.sub_layer_reserved_zero_33bits[i] = bs.read_bits64(33);
                } else {
                    ptl.sub_layer_reserved_zero_34bits[i] = bs.read_bits64(34);
                }
            } else if ptl.sub_layer_profile_idc[i] == 2
                || ptl.sub_layer_profile_compatibility_flag[i][2]
            {
                ptl.sub_layer_reserved_zero_7bits[i] = bs.read_bits(7) as u8;
                ptl.sub_layer_one_picture_only_constraint_flag[i] = 1 == bs.read_bits(1);
                ptl.sub_layer_reserved_zero_35bits[i] = bs.read_bits64(35);
            } else {
                ptl.sub_layer_reserved_zero_43bits[i] = bs.read_bits64(43);
            }
            if ptl.sub_layer_profile_idc[i] == 1
                || ptl.sub_layer_profile_compatibility_flag[i][1]
                || ptl.sub_layer_profile_idc[i] == 2
                || ptl.sub_layer_profile_compatibility_flag[i][2]
                || ptl.sub_layer_profile_idc[i] == 3
                || ptl.sub_layer_profile_compatibility_flag[i][3]
                || ptl.sub_layer_profile_idc[i] == 4
                || ptl.sub_layer_profile_compatibility_flag[i][4]
                || ptl.sub_layer_profile_idc[i] == 5
                || ptl.sub_layer_profile_compatibility_flag[i][5]
                || ptl.sub_layer_profile_idc[i] == 9
                || ptl.sub_layer_profile_compatibility_flag[i][9]
                || ptl.sub_layer_profile_idc[i] == 11
                || ptl.sub_layer_profile_compatibility_flag[i][11]
            {
                // The number of bits in this syntax structure is not affected by this condition
                ptl.sub_layer_inbld_flag[i] = 1 == bs.read_bits(1);
            } else {
                ptl.sub_layer_reserved_zero_bit[i] = bs.read_bits(1) as u8;
            }
        }

        if ptl.sub_layer_level_present_flag[i] {
            ptl.sub_layer_level_idc[i] = bs.read_bits(8) as u8;
        }
    }

    ptl
}

fn h265_decode_scaling_list(bs: &mut ByteStream) -> ScalingListData {
    let mut sl = ScalingListData::new();

    for size_id in 0..4 {
        sl.scaling_list_pred_mode_flag.push(Vec::new());
        sl.scaling_list_pred_matrix_id_delta.push(Vec::new());
        sl.scaling_list_dc_coef_minus8.push(Vec::new());
        sl.scaling_list_delta_coef.push(Vec::new());
        sl.scaling_list.push(Vec::new());

        for matrix_id in (0..6).step_by(if size_id == 3 { 3 } else { 1 }) {
            sl.scaling_list_pred_mode_flag[size_id].push(1 == bs.read_bits(1));

            sl.scaling_list_delta_coef[size_id].push(Vec::new());
            sl.scaling_list[size_id].push(Vec::new());
            if !sl.scaling_list_pred_mode_flag[size_id][matrix_id] {
                sl.scaling_list_pred_matrix_id_delta[size_id]
                    .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                if size_id > 1 {
                    sl.scaling_list_dc_coef_minus8[size_id - 2].push(0);
                }
            } else {
                // for balance
                sl.scaling_list_pred_matrix_id_delta[size_id].push(0);

                let mut next_coef = 8;
                let coef_num = min(64, 1 << (4 + (size_id << 1)));

                if size_id > 1 {
                    sl.scaling_list_dc_coef_minus8[size_id - 2]
                        .push(exp_golomb_decode_one_wrapper(bs, true, 0));
                    next_coef = sl.scaling_list_dc_coef_minus8[size_id - 2][matrix_id] + 8;
                }

                for i in 0..coef_num {
                    sl.scaling_list_delta_coef[size_id][matrix_id]
                        .push(exp_golomb_decode_one_wrapper(bs, true, 0));
                    next_coef =
                        (next_coef + sl.scaling_list_delta_coef[size_id][matrix_id][i] + 256) % 256;
                    sl.scaling_list[size_id][matrix_id].push(next_coef as u32);
                }
            }
        }
    }

    sl
}

fn h265_decode_st_ref_pic_set(
    st_rps_idx: u32,
    num_short_term_ref_pic_sets: u32,
    ref_pics: &Vec<ShortTermRefPic>,
    bs: &mut ByteStream,
) -> ShortTermRefPic {
    let mut strp = ShortTermRefPic::new();

    if st_rps_idx != 0 {
        strp.inter_ref_pic_set_prediction_flag = 1 == bs.read_bits(1);
    }

    if strp.inter_ref_pic_set_prediction_flag {
        if st_rps_idx == num_short_term_ref_pic_sets {
            strp.delta_idx_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;

            if (strp.delta_idx_minus1 + 1) >= st_rps_idx {
                println!("[WARNING] StRefPic delta_idx_minus1 is greater than st_rps_idx. May encounter decoding errors");
            }
        }

        strp.delta_rps_sign = 1 == bs.read_bits(1);
        strp.abs_delta_rps_minus1 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        // NumDeltaPics is an array of stored delta values
        // RefRpsIdx is st_rps_idx - (delta_idx_minus1 + 1). When not present, it's inferred to be 0
        let ref_rps_idx = (st_rps_idx - (strp.delta_idx_minus1 + 1)) as usize % ref_pics.len();
        for j in 0..=ref_pics[ref_rps_idx].num_delta_pics {
            strp.used_by_curr_pic_flag.push(1 == bs.read_bits(1));
            if !strp.used_by_curr_pic_flag[j as usize] {
                strp.use_delta_flag.push(1 == bs.read_bits(1));
            } else {
                strp.use_delta_flag.push(false);
            }
        }
    } else {
        strp.num_negative_pics = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        strp.num_positive_pics = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        strp.num_delta_pics = strp.num_negative_pics + strp.num_positive_pics;

        for _i in 0..strp.num_negative_pics {
            strp.delta_poc_s0_minus1
                .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
            strp.used_by_curr_pic_s0_flag.push(1 == bs.read_bits(1));
        }
        for _i in 0..strp.num_positive_pics {
            strp.delta_poc_s1_minus1
                .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
            strp.used_by_curr_pic_s1_flag.push(1 == bs.read_bits(1));
        }
    }

    strp
}

fn h265_decode_vui_parameters() -> H265VuiParameters {
    let vui = H265VuiParameters::new();

    vui
}

fn h265_decode_sps_range_extension() -> H265SPSRangeExtension {
    let sps_range = H265SPSRangeExtension::new();

    sps_range
}

fn h265_decode_sps_multilayer_extension() -> H265SPSMultilayerExtension {
    let sps_multi = H265SPSMultilayerExtension::new();

    sps_multi
}

fn h265_decode_sps_3d_extension() -> H265SPS3DExtension {
    let sps_3d = H265SPS3DExtension::new();

    sps_3d
}

fn h265_decode_sps_scc_extension() -> H265SPSSCCExtension {
    let sps_scc = H265SPSSCCExtension::new();

    sps_scc
}

fn h265_decode_seq_parameter_set(bs: &mut ByteStream) -> H265SeqParameterSet {
    let mut sps = H265SeqParameterSet::new();

    sps.sps_video_parameter_set_id = bs.read_bits(4) as u8; // u(4)
    sps.sps_max_sub_layers_minus1 = bs.read_bits(3) as u8; // u(3)
    sps.sps_temporal_id_nesting_flag = 1 == bs.read_bits(1);

    sps.profile_tier_level =
        h265_decode_profile_tier_level(true, sps.sps_max_sub_layers_minus1 as usize, bs);

    sps.sps_seq_parameter_set_id = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    sps.chroma_format_idc = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    if sps.chroma_format_idc == 3 {
        sps.separate_colour_plane_flag = 1 == bs.read_bits(1);
    }

    sps.pic_width_in_luma_samples = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    sps.pic_height_in_luma_samples = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    sps.conformance_window_flag = 1 == bs.read_bits(1);

    if sps.conformance_window_flag {
        sps.conf_win_left_offset = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        sps.conf_win_right_offset = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        sps.conf_win_top_offset = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        sps.conf_win_bottom_offset = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    }

    sps.bit_depth_luma_minus8 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    sps.bit_depth_chroma_minus8 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    sps.log2_max_pic_order_cnt_lsb_minus4 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    sps.sps_sub_layer_ordering_info_present_flag = 1 == bs.read_bits(1);

    let min = if sps.sps_sub_layer_ordering_info_present_flag {
        0
    } else {
        sps.sps_max_sub_layers_minus1
    };

    for _i in min..=sps.sps_max_sub_layers_minus1 {
        sps.sps_max_dec_pic_buffering_minus1
            .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        sps.sps_max_num_reorder_pics
            .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
        sps.sps_max_latency_increase_plus1
            .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
    }

    sps.log2_min_luma_coding_block_size_minus3 = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    sps.log2_diff_max_min_luma_coding_block_size =
        exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    sps.log2_min_luma_transform_block_size_minus2 =
        exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    sps.log2_diff_max_min_luma_transform_block_size =
        exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    sps.max_transform_hierarchy_depth_inter = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    sps.max_transform_hierarchy_depth_intra = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
    sps.scaling_list_enabled_flag = 1 == bs.read_bits(1);

    if sps.scaling_list_enabled_flag {
        sps.sps_scaling_list_data_present_flag = 1 == bs.read_bits(1);

        if sps.sps_scaling_list_data_present_flag {
            sps.scaling_list_data = h265_decode_scaling_list(bs);
        }
    }
    sps.amp_enabled_flag = 1 == bs.read_bits(1);
    sps.sample_adaptive_offset_enabled_flag = 1 == bs.read_bits(1);
    sps.pcm_enabled_flag = 1 == bs.read_bits(1);

    if sps.pcm_enabled_flag {
        sps.pcm_sample_bit_depth_luma_minus1 = bs.read_bits(4) as u8;
        sps.pcm_sample_bit_depth_chroma_minus1 = bs.read_bits(4) as u8;
        sps.log2_min_pcm_luma_coding_block_size_minus3 =
            exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        sps.log2_diff_max_min_pcm_luma_coding_block_size =
            exp_golomb_decode_one_wrapper(bs, false, 0) as u32;
        sps.pcm_loop_filter_disabled_flag = 1 == bs.read_bits(1);
    }

    sps.num_short_term_ref_pic_sets = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;

    for i in 0..sps.num_short_term_ref_pic_sets {
        let st_ref_pic =
            h265_decode_st_ref_pic_set(i, sps.num_short_term_ref_pic_sets, &sps.st_ref_pic_set, bs);
        sps.st_ref_pic_set.push(st_ref_pic);
    }

    sps.long_term_ref_pics_present_flag = 1 == bs.read_bits(1);

    if sps.long_term_ref_pics_present_flag {
        sps.num_long_term_ref_pics_sps = exp_golomb_decode_one_wrapper(bs, false, 0) as u32;

        for _i in 0..sps.num_long_term_ref_pics_sps {
            sps.lt_ref_pic_poc_lsb_sps
                .push(bs.read_bits((sps.log2_max_pic_order_cnt_lsb_minus4 as u8) + 4));
            sps.used_by_curr_pic_lt_sps_flag.push(1 == bs.read_bits(1));
        }
    }
    sps.sps_temporal_mvp_enabled_flag = 1 == bs.read_bits(1);
    sps.strong_intra_smoothing_enabled_flag = 1 == bs.read_bits(1);
    sps.vui_parameters_present_flag = 1 == bs.read_bits(1);

    if sps.vui_parameters_present_flag {
        sps.vui_parameters = h265_decode_vui_parameters();
    }

    sps.sps_extension_present_flag = 1 == bs.read_bits(1);
    if sps.sps_extension_present_flag {
        sps.sps_range_extension_flag = 1 == bs.read_bits(1);
        sps.sps_multilayer_extension_flag = 1 == bs.read_bits(1);
        sps.sps_3d_extension_flag = 1 == bs.read_bits(1);
        sps.sps_scc_extension_flag = 1 == bs.read_bits(1);
        sps.sps_extension_4bits = bs.read_bits(4) as u8;
    }

    if sps.sps_range_extension_flag {
        sps.sps_range_extension = h265_decode_sps_range_extension();
    }
    if sps.sps_multilayer_extension_flag {
        sps.sps_multilayer_extension = h265_decode_sps_multilayer_extension(); // specified in Annex F
    }
    if sps.sps_3d_extension_flag {
        sps.sps_3d_extension = h265_decode_sps_3d_extension(); // specified in Annex I
    }
    if sps.sps_scc_extension_flag {
        sps.sps_scc_extension = h265_decode_sps_scc_extension();
    }
    if sps.sps_extension_4bits > 0 {
        while bs.more_data() {
            sps.sps_extension_data_flag = 1 == bs.read_bits(1);
        }
    }

    sps.debug_print();
    sps
}

pub fn decode_bitstream(filename: &str, perf_output: bool) -> H265DecodedStream {
    let start_time = SystemTime::now();
    let nalu_elements = split_into_nalu(filename);

    if perf_output {
        let duration = start_time.elapsed();
        match duration {
            Ok(elapsed) => {
                println!(
                    "[PERF] decode_bitstream - split_into_nalu - duration: {} ns",
                    elapsed.as_nanos()
                );
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    // Currently only
    let mut nalu_headers: Vec<H265NALUHeader> = Vec::new();
    let vpses: Vec<H265VideoParameterSet> = Vec::new();
    let mut spses: Vec<H265SeqParameterSet> = Vec::new();

    println!("\tFound {:?} NALUs", nalu_elements.len());

    for (i, n) in nalu_elements.iter().enumerate() {
        let mut nalu_data = ByteStream::new(n.content.clone());

        let header = h265_decode_nalu_header(n.longstartcode, &mut nalu_data);
        nalu_headers.push(header.clone());

        match header.nal_unit_type {
            NalUnitType::NalUnitSps => {
                println!(
                    "\t decode_bitstream - NALU {} - {:?} - Decoding Sequence Parameter Set (SPS)",
                    i, header.nal_unit_type
                );
                let start_time = SystemTime::now();
                let sps = h265_decode_seq_parameter_set(&mut nalu_data);

                if perf_output {
                    let duration = start_time.elapsed();
                    match duration {
                        Ok(elapsed) => {
                            println!("[PERF] decode_bitstream - decode_seq_parameter_set - duration: {} ns", elapsed.as_nanos());
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                }
                spses.push(sps);
            }
            _ => {
                println!(
                    "\t decode_bitstream - TODO: {:?} parsing",
                    header.nal_unit_type
                );
            }
        };
    }

    println!(
        "\t decode_bitstream - Decoded a total of {} slices",
        nalu_headers.len()
    );

    H265DecodedStream {
        nalu_elements,
        nalu_headers,
        vpses,
        spses,
    }
}
