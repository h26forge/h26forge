//! H.265 syntax element encoding.

use std::vec;

use crate::common::data_structures::NALU;
use crate::encoder::binarization_functions::generate_unsigned_binary;
use crate::encoder::encoder::insert_emulation_three_byte;
use crate::encoder::expgolomb::exp_golomb_encode_one;
use crate::experimental::h265_data_structures::{
    H265DecodedStream, H265NALUHeader, H265SeqParameterSet, ProfileTierLevel, ShortTermRefPic, H265HRDParameters, H265VideoParameterSet, H265PicParameterSet
};
use crate::{
    common::helper::bitstream_to_bytestream, experimental::h265_data_structures::NalUnitType,
};
use crate::vidgen::film::FilmState;
use log::debug;

use super::h265_data_structures::H265SubLayerHRDParameters;

// Macros to (1) generate a value and (2) encode the bitstream
macro_rules! r_ue {
    ( $se:expr, $min:expr, $max:expr, $film:ident, $ba:ident ) => {
        {
            $se = $film.read_film_u32($min, $max);
            $ba.extend(exp_golomb_encode_one($se as i32, false, 0, false));
        }
    };
}

macro_rules! r_se {
    ( $se:expr, $min:expr, $max:expr, $film:ident, $ba:ident ) => {
        {
            $se = $film.read_film_i32($min, $max);
            $ba.extend(exp_golomb_encode_one($se, true, 0, false));
        }
    };
}

macro_rules! r_u {
    ( $se:expr, $min:expr, $max:expr, $length:expr, $film:ident, $ba:ident ) => {
        {
            $se = $film.read_film_u32($min, $max);
            $ba.extend(generate_unsigned_binary($se, $length));
        }
    };
}

macro_rules! r_u8 {
    ( $se:expr, $min:expr, $max:expr, $length:expr, $film:ident, $ba:ident ) => {
        {
            $se = $film.read_film_u32($min, $max) as u8;
            $ba.extend(generate_unsigned_binary($se as u32, $length));
        }
    };
}

macro_rules! r_bool {
    ( $se:expr, $film:ident, $ba:ident ) => {
        {
            $se = $film.read_film_bool(0, 2, 1);
            $ba.push(match $se {
                true => 1,
                _ => 0,
            });
        }
    };
    
    ( $se:expr, $min:expr, $max:expr, $threshold:expr, $film:ident, $ba:ident ) => {
        {
            $se = $film.read_film_bool($min, $max, $threshold);
            $ba.push(match $se {
                true => 1,
                _ => 0,
            });
        }
    };
}

macro_rules! r_bool_false {
    ( $se:expr, $film:ident, $ba:ident ) => {
        {
            $se = false;
            $ba.push(match $se {
                true => 1,
                _ => 0,
            });
        }
    };
}

fn h265_randcode_nalu_header(nh: &mut H265NALUHeader, cur_nal_idx : usize, film : &mut FilmState) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    nh.forbidden_zero_bit = 0; // TODO: randomly sample later
    bitstream_array.push(nh.forbidden_zero_bit);

    let nal_unit_type : u32;
    // Ensure proper required ordering
    if cur_nal_idx == 0 {
        r_u!(nal_unit_type, 32, 32, 6, film, bitstream_array);
        nh.nal_unit_type = NalUnitType::NalUnitVps;
    } else if cur_nal_idx == 1 {
        r_u!(nal_unit_type, 33, 33, 6, film, bitstream_array);
        nh.nal_unit_type = NalUnitType::NalUnitSps;
    } else if cur_nal_idx == 2 {
        r_u!(nal_unit_type, 34, 34, 6, film, bitstream_array);
        nh.nal_unit_type = NalUnitType::NalUnitPps;
    } else {
        r_u!(nal_unit_type, 32, 34, 6, film, bitstream_array);
        nh.nal_unit_type = NalUnitType::from(nal_unit_type);
    }

    r_u8!(nh.nuh_layer_id, 0, 63, 6, film, bitstream_array);
    r_u8!(nh.nuh_temporal_id_plus1, 0, 7, 3, film, bitstream_array);

    bitstream_to_bytestream(bitstream_array, 0)
}

fn h265_randcode_profile_tier_level(
    profile_present_flag: bool,
    max_sub_layers_minus1: usize,
    ptl: &mut ProfileTierLevel,
    film : &mut FilmState
) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    // TODO: generate all the different reserved zero bits as well

    if profile_present_flag {
        r_u8!(ptl.general_profile_space, 0, 3, 2, film, bitstream_array);
        r_bool!(ptl.general_tier_flag, film, bitstream_array);
        r_u8!(ptl.general_profile_idc, 0, 31, 5, film, bitstream_array);

        for i in 0..32 {
            r_bool!(ptl.general_profile_compatibility_flag[i], film, bitstream_array);
        }
        r_bool!(ptl.general_progressive_source_flag, film, bitstream_array);
        r_bool!(ptl.general_interlaced_source_flag, film, bitstream_array);
        r_bool!(ptl.general_non_packed_constraint_flag, film, bitstream_array);
        r_bool!(ptl.general_frame_only_constraint_flag, film, bitstream_array);

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
            r_bool!(ptl.general_max_12bit_constraint_flag, film, bitstream_array);
            r_bool!(ptl.general_max_10bit_constraint_flag, film, bitstream_array);
            r_bool!(ptl.general_max_8bit_constraint_flag, film, bitstream_array);
            r_bool!(ptl.general_max_422chroma_constraint_flag, film, bitstream_array);
            r_bool!(ptl.general_max_420chroma_constraint_flag, film, bitstream_array);
            r_bool!(ptl.general_max_monochrome_constraint_flag, film, bitstream_array);
            r_bool!(ptl.general_intra_constraint_flag, film, bitstream_array);
            r_bool!(ptl.general_one_picture_only_constraint_flag, film, bitstream_array);
            r_bool!(ptl.general_lower_bit_rate_constraint_flag, film, bitstream_array);

            if ptl.general_profile_idc == 5
                || ptl.general_profile_compatibility_flag[5]
                || ptl.general_profile_idc == 9
                || ptl.general_profile_compatibility_flag[9]
                || ptl.general_profile_idc == 10
                || ptl.general_profile_compatibility_flag[10]
                || ptl.general_profile_idc == 11
                || ptl.general_profile_compatibility_flag[11]
            {
                r_bool!(ptl.general_max_14bit_constraint_flag, film, bitstream_array);

                // store as an unsigned binary encoding
                for i in (0..33).rev() {
                    bitstream_array.push(((ptl.general_reserved_zero_33bits >> i) & 1) as u8);
                }
            } else {
                // store as an unsigned binary encoding
                for i in (0..34).rev() {
                    bitstream_array.push(((ptl.general_reserved_zero_34bits >> i) & 1) as u8);
                }
            }
        } else if ptl.general_profile_idc == 2 || ptl.general_profile_compatibility_flag[2] {
            r_u8!(ptl.general_reserved_zero_7bits, 0, 0, 7, film, bitstream_array);
            r_bool!(ptl.general_one_picture_only_constraint_flag, film, bitstream_array);

            // store as an unsigned binary encoding
            for i in (0..35).rev() {
                bitstream_array.push(((ptl.general_reserved_zero_35bits >> i) & 1) as u8);
            }
        } else {
            // store as an unsigned binary encoding
            for i in (0..43).rev() {
                bitstream_array.push(((ptl.general_reserved_zero_43bits >> i) & 1) as u8);
            }
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
            r_bool!(ptl.general_inbld_flag, film, bitstream_array);
        } else {
            bitstream_array.push(ptl.general_reserved_zero_bit);
        }
    }
    
    r_u8!(ptl.general_level_idc, 0, 255, 8, film, bitstream_array);

    ptl.sub_layer_profile_present_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_level_present_flag = vec![false; max_sub_layers_minus1];

    for i in 0..max_sub_layers_minus1 {
        r_bool!(ptl.sub_layer_profile_present_flag[i], film, bitstream_array);
        r_bool!(ptl.sub_layer_level_present_flag[i], film, bitstream_array);
    }

    if max_sub_layers_minus1 > 0 {
        ptl.reserved_zero_2bits = vec![0; 8];
        for i in max_sub_layers_minus1..8 {
            r_u8!(ptl.reserved_zero_2bits[i], 0, 3, 2, film, bitstream_array);
        }
    }

    ptl.sub_layer_profile_space = vec![0; max_sub_layers_minus1];
    ptl.sub_layer_tier_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_profile_idc = vec![0; max_sub_layers_minus1];
    ptl.sub_layer_reserved_zero_33bits = vec![0; 33];
    ptl.sub_layer_reserved_zero_34bits = vec![0; 34];
    ptl.sub_layer_profile_compatibility_flag = vec![vec![false; 32]; max_sub_layers_minus1];
    ptl.sub_layer_progressive_source_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_interlaced_source_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_non_packed_constraint_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_frame_only_constraint_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_max_12bit_constraint_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_max_10bit_constraint_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_max_8bit_constraint_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_max_422chroma_constraint_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_max_420chroma_constraint_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_max_monochrome_constraint_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_intra_constraint_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_one_picture_only_constraint_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_lower_bit_rate_constraint_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_max_14bit_constraint_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_inbld_flag = vec![false; max_sub_layers_minus1];
    ptl.sub_layer_level_idc = vec![0; max_sub_layers_minus1];


    for i in 0..max_sub_layers_minus1 {
        if ptl.sub_layer_profile_present_flag[i] {
            r_u8!(ptl.sub_layer_profile_space[i], 0, 3, 2, film, bitstream_array);
            r_bool!(ptl.sub_layer_tier_flag[i], film, bitstream_array);
            r_u8!(ptl.sub_layer_profile_idc[i], 0, 31, 5, film, bitstream_array);

            for j in 0..32 {
                r_bool!(ptl.sub_layer_profile_compatibility_flag[i][j], film, bitstream_array);
            }

            r_bool!(ptl.sub_layer_progressive_source_flag[i], film, bitstream_array);
            r_bool!(ptl.sub_layer_interlaced_source_flag[i], film, bitstream_array);
            r_bool!(ptl.sub_layer_non_packed_constraint_flag[i], film, bitstream_array);
            r_bool!(ptl.sub_layer_frame_only_constraint_flag[i], film, bitstream_array);


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
                r_bool!(ptl.sub_layer_max_12bit_constraint_flag[i], film, bitstream_array);
                r_bool!(ptl.sub_layer_max_10bit_constraint_flag[i], film, bitstream_array);
                r_bool!(ptl.sub_layer_max_8bit_constraint_flag[i], film, bitstream_array);
                r_bool!(ptl.sub_layer_max_422chroma_constraint_flag[i], film, bitstream_array);
                r_bool!(ptl.sub_layer_max_420chroma_constraint_flag[i], film, bitstream_array);
                r_bool!(ptl.sub_layer_max_monochrome_constraint_flag[i], film, bitstream_array);
                r_bool!(ptl.sub_layer_intra_constraint_flag[i], film, bitstream_array);
                r_bool!(ptl.sub_layer_one_picture_only_constraint_flag[i], film, bitstream_array);
                r_bool!(ptl.sub_layer_lower_bit_rate_constraint_flag[i], film, bitstream_array);

                if ptl.sub_layer_profile_idc[i] == 5
                    || ptl.sub_layer_profile_compatibility_flag[i][5]
                    || ptl.sub_layer_profile_idc[i] == 9
                    || ptl.sub_layer_profile_compatibility_flag[i][9]
                    || ptl.sub_layer_profile_idc[i] == 10
                    || ptl.sub_layer_profile_compatibility_flag[i][10]
                    || ptl.sub_layer_profile_idc[i] == 11
                    || ptl.sub_layer_profile_compatibility_flag[i][11]
                {
                    r_bool!(ptl.sub_layer_max_14bit_constraint_flag[i], film, bitstream_array);

                    // store as an unsigned binary encoding
                    for i in (0..33).rev() {
                        ptl.sub_layer_reserved_zero_33bits[i] = film.read_film_u32(0, 1) as u64;
                        bitstream_array
                            .push(((ptl.sub_layer_reserved_zero_33bits[i] >> i) & 1) as u8);
                    }
                } else {
                    // store as an unsigned binary encoding
                    for i in (0..34).rev() {
                        ptl.sub_layer_reserved_zero_34bits[i] = film.read_film_u32(0, 1) as u64;
                        bitstream_array
                            .push(((ptl.sub_layer_reserved_zero_34bits[i] >> i) & 1) as u8);
                    }
                }
            } else if ptl.sub_layer_profile_idc[i] == 2
                || ptl.sub_layer_profile_compatibility_flag[i][2]
            {
                r_u8!(ptl.sub_layer_reserved_zero_7bits[i], 0, 0, 7, film, bitstream_array);
                r_bool!(ptl.sub_layer_one_picture_only_constraint_flag[i], film, bitstream_array);

                // store as an unsigned binary encoding
                for i in (0..35).rev() {
                    bitstream_array.push(((ptl.sub_layer_reserved_zero_35bits[i] >> i) & 1) as u8);
                }
            } else {
                // store as an unsigned binary encoding
                for i in (0..43).rev() {
                    bitstream_array.push(((ptl.sub_layer_reserved_zero_43bits[i] >> i) & 1) as u8);
                }
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
                r_bool!(ptl.sub_layer_inbld_flag[i], film, bitstream_array);
            } else {
                bitstream_array.push(ptl.sub_layer_reserved_zero_bit[i]);
            }
        }

        if ptl.sub_layer_level_present_flag[i] {
            r_u8!(ptl.sub_layer_level_idc[i], 0, 255, 8, film, bitstream_array);
        }
    }

    bitstream_array
}

fn h265_randcode_sub_layer_hrd_parameters(subhrd: &mut H265SubLayerHRDParameters, cpb_count : usize, sub_pic_hrd_params_present_flag: bool, film: &mut FilmState) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    subhrd.bit_rate_value_minus1 = vec![0; cpb_count];
    subhrd.cpb_size_value_minus1 = vec![0; cpb_count];
    subhrd.cpb_size_du_value_minus1 = vec![0; cpb_count];
    subhrd.bit_rate_du_value_minus1 = vec![0; cpb_count];
    subhrd.cbr_flag = vec![false; cpb_count];

    for i in 0..cpb_count {
        r_ue!(subhrd.bit_rate_value_minus1[i], 0, std::u32::MAX, film, bitstream_array);
        r_ue!(subhrd.cpb_size_value_minus1[i], 0, std::u32::MAX, film, bitstream_array);
        if sub_pic_hrd_params_present_flag {
            r_ue!(subhrd.cpb_size_du_value_minus1[i], 0, std::u32::MAX, film, bitstream_array);
            r_ue!(subhrd.bit_rate_du_value_minus1[i], 0, std::u32::MAX, film, bitstream_array);
        }
        r_bool!(subhrd.cbr_flag[i], film, bitstream_array);
    }
    bitstream_array
}

fn h265_randcode_hrd_parameters(hrd: &mut H265HRDParameters, common_inf_present_flag : bool, max_num_sub_layers_minus1 : usize, film: &mut FilmState) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();
    if common_inf_present_flag {
        r_bool!(hrd.nal_hrd_parameters_present_flag, film, bitstream_array);
        r_bool!(hrd.vcl_hrd_parameters_present_flag, film, bitstream_array);
        if hrd.nal_hrd_parameters_present_flag || hrd.vcl_hrd_parameters_present_flag {
            r_bool!(hrd.sub_pic_hrd_params_present_flag, film, bitstream_array);
            if hrd.sub_pic_hrd_params_present_flag {
                r_u8!(hrd.tick_divisor_minus2, 0, 255, 8, film, bitstream_array);
                r_u8!(hrd.du_cpb_removal_delay_increment_length_minus1, 0, 31, 5, film, bitstream_array);
                r_bool!(hrd.sub_pic_cpb_params_in_pic_timing_sei_flag, film, bitstream_array);
                r_u8!(hrd.dpb_output_delay_du_length_minus1, 0, 31, 5, film, bitstream_array);
            }
            r_u8!(hrd.bit_rate_scale, 0, 15, 4, film, bitstream_array);
            r_u8!(hrd.cpb_size_scale, 0, 15, 4, film, bitstream_array);
            if hrd.sub_pic_hrd_params_present_flag {
                r_u8!(hrd.cpb_size_du_scale, 0, 15, 4, film, bitstream_array);
            }
            r_u8!(hrd.initial_cpb_removal_delay_length_minus1, 0, 31, 5, film, bitstream_array);
            r_u8!(hrd.au_cpb_removal_delay_length_minus1, 0, 31, 5, film, bitstream_array);
            r_u8!(hrd.dpb_output_delay_length_minus1, 0, 31, 5, film, bitstream_array);
        }
    }
    hrd.fixed_pic_rate_general_flag = vec![false; max_num_sub_layers_minus1 + 1];
    hrd.fixed_pic_rate_within_cvs_flag = vec![false; max_num_sub_layers_minus1 + 1];
    hrd.elemental_duration_in_tc_minus1 = vec![0; max_num_sub_layers_minus1 + 1];
    hrd.low_delay_hrd_flag = vec![false; max_num_sub_layers_minus1 + 1];
    hrd.cpb_cnt_minus1 = vec![0; max_num_sub_layers_minus1 + 1];
    hrd.nal_sub_layer_hrd_parameters = vec![H265SubLayerHRDParameters::new(); max_num_sub_layers_minus1 + 1];
    hrd.vcl_sub_layer_hrd_parameters = vec![H265SubLayerHRDParameters::new(); max_num_sub_layers_minus1 + 1];
    for i in 0..=max_num_sub_layers_minus1 {
        r_bool!(hrd.fixed_pic_rate_general_flag[i], film, bitstream_array);
        if !hrd.fixed_pic_rate_general_flag[i] {
            r_bool!(hrd.fixed_pic_rate_within_cvs_flag[i], film, bitstream_array);
        }
        if hrd.fixed_pic_rate_within_cvs_flag[i] {
            r_ue!(hrd.elemental_duration_in_tc_minus1[i], 0, 65535, film, bitstream_array);
        } else {
            r_bool!(hrd.low_delay_hrd_flag[i], film, bitstream_array);
        }
        if !hrd.low_delay_hrd_flag[i] {
            r_ue!(hrd.cpb_cnt_minus1[i], 0, 128, film, bitstream_array);
        }
        
        if hrd.nal_hrd_parameters_present_flag {
            bitstream_array.extend(h265_randcode_sub_layer_hrd_parameters(
                &mut hrd.nal_sub_layer_hrd_parameters[i],
                hrd.cpb_cnt_minus1[i] as usize + 1,
                hrd.sub_pic_hrd_params_present_flag,
                film,
            ));
        } 
        if hrd.vcl_hrd_parameters_present_flag {
            bitstream_array.extend(h265_randcode_sub_layer_hrd_parameters(
                &mut hrd.vcl_sub_layer_hrd_parameters[i],
                hrd.cpb_cnt_minus1[i] as usize + 1,
                hrd.sub_pic_hrd_params_present_flag,
                film,
            ));
        }
        
    }
    bitstream_array
}

fn h265_randcode_video_parameter_set(vps: &mut H265VideoParameterSet, film: &mut FilmState) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();
    r_u8!(vps.vps_video_parameter_set_id, 0, 15, 4, film, bitstream_array);
    r_bool!(vps.vps_base_layer_internal_flag, film, bitstream_array);
    r_bool!(vps.vps_base_layer_available_flag, film, bitstream_array);
    r_u8!(vps.vps_max_layers_minus1, 0, 63, 6, film, bitstream_array);
    r_u8!(vps.vps_max_sub_layers_minus1, 0, 7, 3, film, bitstream_array);
    r_bool!(vps.vps_temporal_id_nesting_flag, film, bitstream_array);
    r_u!(vps.vps_reserved_0xffff_16bits, 0xffff, 0xffff, 16, film, bitstream_array);

    bitstream_array.extend(h265_randcode_profile_tier_level(
        true,
        vps.vps_max_sub_layers_minus1 as usize,
        &mut vps.ptl,
        film,
    ));

    r_bool!(vps.vps_sub_layer_ordering_info_present_flag, film, bitstream_array);

    vps.vps_max_dec_pic_buffering_minus1 = vec![0; vps.vps_max_sub_layers_minus1 as usize + 1];
    vps.vps_max_num_reorder_pics = vec![0; vps.vps_max_sub_layers_minus1 as usize + 1];
    vps.vps_max_latency_increase_plus1 = vec![0; vps.vps_max_sub_layers_minus1 as usize + 1];

    for i in match vps.vps_sub_layer_ordering_info_present_flag {false => 0, true => vps.vps_max_sub_layers_minus1}..=vps.vps_max_sub_layers_minus1 {
        r_ue!(vps.vps_max_dec_pic_buffering_minus1[i as usize], 0, 65535, film, bitstream_array);
        r_ue!(vps.vps_max_num_reorder_pics[i as usize], 0, 65535, film, bitstream_array);
        r_ue!(vps.vps_max_latency_increase_plus1[i as usize], 0, 65535, film, bitstream_array);
    }
    r_u8!(vps.vps_max_layer_id, 0, 63, 6, film, bitstream_array);
    // Expected range is [0, 1023]
    r_ue!(vps.vps_num_layer_sets_minus1, 0, 100, film, bitstream_array);

    vps.layer_id_included_flag = vec![vec![false; vps.vps_max_layer_id as usize + 1]; vps.vps_num_layer_sets_minus1 as usize + 1];
    for i in 1..=vps.vps_num_layer_sets_minus1 {
        for j in 0..= vps.vps_max_layer_id {
            r_bool!(vps.layer_id_included_flag[i as usize][j as usize], film, bitstream_array);
        }
    }
    
    r_bool!(vps.vps_timing_info_present_flag, film, bitstream_array);
    if vps.vps_timing_info_present_flag {
        r_u!(vps.vps_num_units_in_tick, 0, std::u32::MAX, 32, film, bitstream_array);
        r_u!(vps.vps_time_scale, 0, std::u32::MAX, 32, film, bitstream_array);
        r_bool!(vps.vps_poc_proportional_to_timing_flag, film, bitstream_array);
        if vps.vps_poc_proportional_to_timing_flag {
            r_ue!(vps.vps_num_ticks_poc_diff_one_minus1, 0, std::u32::MAX, film, bitstream_array);
        }
        // Used as a loop bound - Slows down generation a lot. Expected range is [0, vps_num_layer_sets_minus1+1]
        r_ue!(vps.vps_num_hrd_parameters, 0, 100, film, bitstream_array);

        // Initialize
        vps.hrd_layer_set_idx = vec![0; vps.vps_num_hrd_parameters as usize];
        vps.cprms_present_flag = vec![false; vps.vps_num_hrd_parameters as usize];
        vps.hrd_parameters = vec![H265HRDParameters::new(); vps.vps_num_hrd_parameters as usize];

        for i in 0..vps.vps_num_hrd_parameters as usize {
            r_ue!(vps.hrd_layer_set_idx[i], 0, 10000, film, bitstream_array);
            if i > 0 {
                r_bool!(vps.cprms_present_flag[i], film, bitstream_array);
            }
            bitstream_array.extend(h265_randcode_hrd_parameters(
                &mut vps.hrd_parameters[i],
                vps.cprms_present_flag[i],
                vps.vps_max_sub_layers_minus1 as usize,
                film,
            ));
        }
    }
    r_bool_false!(vps.vps_extension_flag, film, bitstream_array);

    
    bitstream_to_bytestream(bitstream_array, 0)
}

fn h265_randcode_st_ref_pic_set(
    st_rps_idx: u32,
    num_short_term_ref_pic_sets: u32,
    ref_pics: &Vec<ShortTermRefPic>,
    strp: &mut ShortTermRefPic,
    film: &mut FilmState
) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    if st_rps_idx != 0 {
        r_bool!(strp.inter_ref_pic_set_prediction_flag, film, bitstream_array);
    }

    // Will never go here when st_rps_idx is 0
    if strp.inter_ref_pic_set_prediction_flag {
        if st_rps_idx == num_short_term_ref_pic_sets {
            r_ue!(strp.delta_idx_minus1, 0, 65535, film, bitstream_array);

            if (strp.delta_idx_minus1 + 1) >= st_rps_idx {
                println!("[WARNING] StRefPic delta_idx_minus1 is greater than st_rps_idx. May encounter encoding errors");
                debug!(target: "encode","[WARNING] StRefPic delta_idx_minus1 is greater than st_rps_idx. May encounter encoding errors");
            }
        }

        r_bool!(strp.delta_rps_sign, film, bitstream_array);
        r_ue!(strp.abs_delta_rps_minus1, 0, 65535, film, bitstream_array);
        // NumDeltaPics is an array of stored delta values
        // RefRpsIdx is st_rps_idx - (delta_idx_minus1 + 1). When not present, it's inferred to be 0
        let ref_rps_idx: usize = (st_rps_idx - (strp.delta_idx_minus1 + 1)) as usize % ref_pics.len();
        strp.used_by_curr_pic_flag = vec![false; ref_pics[ref_rps_idx].num_delta_pics as usize + 1];
        strp.use_delta_flag = vec![false; ref_pics[ref_rps_idx].num_delta_pics as usize + 1];

        for j in 0..=(ref_pics[ref_rps_idx].num_delta_pics as usize) {
            r_bool!(strp.used_by_curr_pic_flag[j], film, bitstream_array);
            if !strp.used_by_curr_pic_flag[j] {
                r_bool!(strp.use_delta_flag[j], film, bitstream_array);
            }
        }
    } else {
        r_ue!(strp.num_negative_pics, 0, 127, film, bitstream_array);
        r_ue!(strp.num_positive_pics, 0, 127, film, bitstream_array);

        strp.delta_poc_s0_minus1 = vec![0; strp.num_negative_pics as usize];
        strp.used_by_curr_pic_s0_flag = vec![false; strp.num_negative_pics as usize];
        strp.delta_poc_s1_minus1 = vec![0; strp.num_positive_pics as usize];
        strp.used_by_curr_pic_s1_flag = vec![false; strp.num_positive_pics as usize];


        for i in 0..(strp.num_negative_pics as usize) {
            r_ue!(strp.delta_poc_s0_minus1[i], 0, 65535, film, bitstream_array);
            r_bool!(strp.used_by_curr_pic_s0_flag[i], film, bitstream_array);
        }
        for i in 0..(strp.num_positive_pics as usize) {
            r_ue!(strp.delta_poc_s1_minus1[i], 0, 65535, film, bitstream_array);
            r_bool!(strp.used_by_curr_pic_s1_flag[i], film, bitstream_array);
        }
    }

    bitstream_array
}

fn h265_randcode_seq_parameter_set(sps: &mut H265SeqParameterSet, film : &mut FilmState) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    r_u8!(sps.sps_video_parameter_set_id, 0, 15, 4, film, bitstream_array);
    r_u8!(sps.sps_max_sub_layers_minus1, 0, 7, 3, film, bitstream_array);
    r_bool!(sps.sps_temporal_id_nesting_flag, film, bitstream_array);

    bitstream_array.extend(h265_randcode_profile_tier_level(
        true,
        sps.sps_max_sub_layers_minus1 as usize,
        &mut sps.profile_tier_level,
        film,
    ));

    r_ue!(sps.sps_seq_parameter_set_id, 0, 31, film, bitstream_array);
    r_ue!(sps.chroma_format_idc, 0, 31, film, bitstream_array);

    if sps.chroma_format_idc == 3 {
        r_bool!(sps.separate_colour_plane_flag, film, bitstream_array);
    }
    r_ue!(sps.pic_width_in_luma_samples, 0, 65535, film, bitstream_array);
    r_ue!(sps.pic_height_in_luma_samples, 0, 65535, film, bitstream_array);
    r_bool!(sps.conformance_window_flag, film, bitstream_array);

    if sps.conformance_window_flag {
        r_ue!(sps.conf_win_left_offset, 0, 65535, film, bitstream_array);
        r_ue!(sps.conf_win_right_offset, 0, 65535, film, bitstream_array);
        r_ue!(sps.conf_win_top_offset, 0, 65535, film, bitstream_array);
        r_ue!(sps.conf_win_bottom_offset, 0, 65535, film, bitstream_array);
    }

    r_ue!(sps.bit_depth_luma_minus8, 0, 32, film, bitstream_array);
    r_ue!(sps.bit_depth_chroma_minus8, 0, 32, film, bitstream_array);
    r_ue!(sps.log2_max_pic_order_cnt_lsb_minus4, 0, 32, film, bitstream_array);
    r_bool!(sps.sps_sub_layer_ordering_info_present_flag, film, bitstream_array);

    let min = if sps.sps_sub_layer_ordering_info_present_flag {
        0
    } else {
        sps.sps_max_sub_layers_minus1 as usize
    };

    sps.sps_max_dec_pic_buffering_minus1 = vec![0; sps.sps_max_sub_layers_minus1 as usize + 1];
    sps.sps_max_num_reorder_pics = vec![0; sps.sps_max_sub_layers_minus1 as usize + 1];
    sps.sps_max_latency_increase_plus1 = vec![0; sps.sps_max_sub_layers_minus1 as usize + 1];

    for i in min..=(sps.sps_max_sub_layers_minus1 as usize) {
        r_ue!(sps.sps_max_dec_pic_buffering_minus1[i], 0, 65535, film, bitstream_array);
        r_ue!(sps.sps_max_num_reorder_pics[i], 0, 65535, film, bitstream_array);
        r_ue!(sps.sps_max_latency_increase_plus1[i], 0, 65535, film, bitstream_array);
    }
    r_ue!(sps.log2_min_luma_coding_block_size_minus3, 0, 32, film, bitstream_array);
    r_ue!(sps.log2_diff_max_min_luma_coding_block_size, 0, 32, film, bitstream_array);
    r_ue!(sps.log2_min_luma_transform_block_size_minus2, 0, 32, film, bitstream_array);
    r_ue!(sps.log2_diff_max_min_luma_transform_block_size, 0, 32, film, bitstream_array);
    r_ue!(sps.max_transform_hierarchy_depth_inter, 0, 65535, film, bitstream_array);
    r_ue!(sps.max_transform_hierarchy_depth_intra, 0, 65535, film, bitstream_array);


    // TODO: scaling list
    r_bool_false!(sps.scaling_list_enabled_flag, film, bitstream_array);

    // TODO: H265 scaling list
    //if sps.scaling_list_enabled_flag  {
    //    bitstream_array.push(match sps.sps_scaling_list_data_present_flag {true => 1, _ => 0});
    //
    //    if sps.sps_scaling_list_data_present_flag {
    //        sps.scaling_list_data = h265_encode_scaling_list(bs);
    //    }
    //
    //}
    r_bool!(sps.amp_enabled_flag, film, bitstream_array);
    r_bool!(sps.sample_adaptive_offset_enabled_flag, film, bitstream_array);
    r_bool!(sps.pcm_enabled_flag, film, bitstream_array);

    if sps.pcm_enabled_flag {
        r_u8!(sps.pcm_sample_bit_depth_luma_minus1, 0, 15, 4, film, bitstream_array);
        r_u8!(sps.pcm_sample_bit_depth_chroma_minus1, 0, 15, 4, film, bitstream_array);
        r_ue!(sps.log2_min_pcm_luma_coding_block_size_minus3, 0, 32, film, bitstream_array);
        r_ue!(sps.log2_diff_max_min_pcm_luma_coding_block_size, 0, 32, film, bitstream_array);
        r_bool!(sps.pcm_loop_filter_disabled_flag, film, bitstream_array);
    }
    r_ue!(sps.num_short_term_ref_pic_sets, 0, 128, film, bitstream_array);

    for i in 0..sps.num_short_term_ref_pic_sets {
        let mut srps = ShortTermRefPic::new();
        bitstream_array.extend(h265_randcode_st_ref_pic_set(
            i,
            sps.num_short_term_ref_pic_sets,
            &sps.st_ref_pic_set,
            &mut srps,
            film
        ));
        sps.st_ref_pic_set.push(srps);
    }

    r_bool!(sps.long_term_ref_pics_present_flag, film, bitstream_array);

    if sps.long_term_ref_pics_present_flag {
        r_ue!(sps.num_long_term_ref_pics_sps, 0, 128, film, bitstream_array);

        sps.lt_ref_pic_poc_lsb_sps = vec![0; sps.num_long_term_ref_pics_sps as usize + 1];
        sps.used_by_curr_pic_lt_sps_flag = vec![false; sps.num_long_term_ref_pics_sps as usize + 1];

        let bit_size = sps.log2_max_pic_order_cnt_lsb_minus4 + 4;
        for i in 0..(sps.num_long_term_ref_pics_sps as usize) {
            r_u!(sps.lt_ref_pic_poc_lsb_sps[i], 0, 65535, bit_size as usize, film, bitstream_array);
            r_bool!(sps.used_by_curr_pic_lt_sps_flag[i], film, bitstream_array);
        }
    }

    r_bool!(sps.sps_temporal_mvp_enabled_flag, film, bitstream_array);
    r_bool!(sps.strong_intra_smoothing_enabled_flag, film, bitstream_array);
    r_bool_false!(sps.vui_parameters_present_flag, film, bitstream_array);

    // TODO: H.265 VUI Parameters
    //if sps.vui_parameters_present_flag {
    //    sps.vui_parameters = h265_encode_vui_parameters();
    //}
    
    r_bool!(sps.sps_extension_present_flag, film, bitstream_array);

    if sps.sps_extension_present_flag {
        r_bool_false!(sps.sps_range_extension_flag, film, bitstream_array);
        r_bool_false!(sps.sps_multilayer_extension_flag, film, bitstream_array);
        r_bool_false!(sps.sps_3d_extension_flag, film, bitstream_array);
        r_bool_false!(sps.sps_scc_extension_flag, film, bitstream_array);
        r_u8!(sps.sps_extension_4bits, 0, 0, 4, film, bitstream_array);
    }

    // TODO: SPS extensions
    //if sps.sps_range_extension_flag {
    //    sps.sps_range_extension = h265_encode_sps_range_extension( );
    //}
    //if sps.sps_multilayer_extension_flag {
    //    sps.sps_multilayer_extension = h265_encode_sps_multilayer_extension( ); // specified in Annex F
    //}
    //if sps.sps_3d_extension_flag {
    //    sps.sps_3d_extension = h265_encode_sps_3d_extension( ); // specified in Annex I
    //}
    //if sps.sps_scc_extension_flag  {
    //    sps.sps_scc_extension = h265_encode_sps_scc_extension( );
    //}
    //if sps.sps_extension_4bits > 0 {
    //    // TODO: keep track of amount of data pushed to the end
    //    r_bool!(sps.sps_extension_data_flag, film, bitstream_array);
    //}

    // insert rbsp_stop_one_bit
    bitstream_array.push(1);

    sps.encoder_pretty_print();

    bitstream_to_bytestream(bitstream_array, 0)
}

fn h265_randcode_pic_parameter_set(pps: &mut H265PicParameterSet, sps_id : u32, film: &mut FilmState) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    r_ue!(pps.pps_pic_parameter_set_id, 0, 120, film, bitstream_array); // [0, 63]
    pps.pps_seq_parameter_set_id = sps_id;
    r_bool!(pps.dependent_slice_segments_enabled_flag, film, bitstream_array);
    r_bool!(pps.output_flag_present_flag, film, bitstream_array);
    r_u8!(pps.num_extra_slice_header_bits, 0, 7, 3, film, bitstream_array); // [0, 2]
    r_bool!(pps.sign_data_hiding_enabled_flag, film, bitstream_array);
    r_bool!(pps.cabac_init_present_flag, film, bitstream_array);
    r_ue!(pps.num_ref_idx_l0_default_active_minus1, 0, 32, film, bitstream_array); // [0, 14]
    r_ue!(pps.num_ref_idx_l1_default_active_minus1, 0, 32, film, bitstream_array); // [0, 14]
    r_se!(pps.init_qp_minus26, -26, 25, film, bitstream_array); // Depends on QpBdOffset_gamma
    r_bool!(pps.constrained_intra_pred_flag, film, bitstream_array);
    r_bool!(pps.transform_skip_enabled_flag, film, bitstream_array);
    r_bool!(pps.cu_qp_delta_enabled_flag, film, bitstream_array);
    if pps.cu_qp_delta_enabled_flag {
        r_ue!(pps.diff_cu_qp_delta_depth, 0, 128, film, bitstream_array); // range is [0, log2_diff_max_min_luma_coding_block_size]
    }
    // Values should be ignored if ChromaArrayType is 0
    r_se!(pps.pps_cb_qp_offset, -128, 128, film, bitstream_array); // [-12, 12]
    r_se!(pps.pps_cr_qp_offset, -128, 128, film, bitstream_array); // [-12, 12]
    r_bool!(pps.pps_slice_chroma_qp_offsets_present_flag, film, bitstream_array);
    r_bool!(pps.weighted_pred_flag, film, bitstream_array);
    r_bool!(pps.weighted_bipred_flag, film, bitstream_array);
    r_bool!(pps.transquant_bypass_enabled_flag, film, bitstream_array);
    r_bool!(pps.tiles_enabled_flag, film, bitstream_array);
    r_bool!(pps.entropy_coding_sync_enabled_flag, film, bitstream_array);
    if pps.tiles_enabled_flag {
        r_ue!(pps.num_tile_columns_minus1, 0, 128, film, bitstream_array); // [0, PicWidthInCtbsY -1]
        r_ue!(pps.num_tile_rows_minus1, 0, 128, film, bitstream_array);    // [0, PicHeightInCtbsY  -1]
        r_bool!(pps.uniform_spacing_flag, film, bitstream_array);
        if !pps.uniform_spacing_flag {
            pps.column_width_minus1 = vec![0; pps.num_tile_columns_minus1 as usize];
            pps.row_height_minus1 = vec![0; pps.num_tile_rows_minus1 as usize];
            for i in  0..pps.num_tile_columns_minus1 as usize {
                r_ue!(pps.column_width_minus1[i], 0, 1000, film, bitstream_array);
            }
            for i in 0..pps.num_tile_rows_minus1 as usize {
                r_ue!(pps.row_height_minus1[i], 0, 1000, film, bitstream_array);
            }
        }
        r_bool!(pps.loop_filter_across_tiles_enabled_flag, film, bitstream_array);
    }
    r_bool!(pps.pps_loop_filter_across_slices_enabled_flag, film, bitstream_array);
    r_bool!(pps.deblocking_filter_control_present_flag, film, bitstream_array);
    if pps.deblocking_filter_control_present_flag {
        r_bool!(pps.deblocking_filter_override_enabled_flag, film, bitstream_array);
        r_bool!(pps.pps_deblocking_filter_disabled_flag, film, bitstream_array);
        if !pps.pps_deblocking_filter_disabled_flag {
            r_se!(pps.pps_beta_offset_div2, -128, 128, film, bitstream_array);   // [-6, 6]
            r_se!(pps.pps_tc_offset_div2, -128, 128, film, bitstream_array);     // [-6, 6]
        }
    }
    r_bool_false!(pps.pps_scaling_list_data_present_flag, film, bitstream_array);
    /*
    if pps.pps_scaling_list_data_present_flag {
        pps.scaling_list_data = h265_randcode_scaling_list_data();
    }
    */
    r_bool!(pps.lists_modification_present_flag, film, bitstream_array);
    r_ue!(pps.log2_parallel_merge_level_minus2, 0, 128, film, bitstream_array);   // [0, CtbLog2SizeY − 2]
    r_bool!(pps.slice_segment_header_extension_present_flag, film, bitstream_array);
    r_bool!(pps.pps_extension_present_flag, film, bitstream_array);
    if pps.pps_extension_present_flag {
        r_bool_false!(pps.pps_range_extension_flag, film, bitstream_array);
        r_bool_false!(pps.pps_multilayer_extension_flag, film, bitstream_array);
        r_bool_false!(pps.pps_3d_extension_flag, film, bitstream_array);
        r_bool_false!(pps.pps_scc_extension_flag, film, bitstream_array);
        r_u8!(pps.pps_extension_4bits, 0, 0, 4, film, bitstream_array);
    }

    /*
    if( pps_range_extension_flag )
        pps_range_extension( )
    if( pps_multilayer_extension_flag )
        pps_multilayer_extension( ) // specified in Annex F
    if( pps_3d_extension_flag )
        pps_3d_extension( ) // specified in Annex I
    if( pps_scc_extension_flag )
        pps_scc_extension( )
    if( pps_extension_4bits )
        while( more_rbsp_data( ) )
            pps_extension_data_flag, film, bitstream_array);
    */

    // insert rbsp_stop_one_bit
    bitstream_array.push(1);

    bitstream_to_bytestream(bitstream_array, 0)
}

/// Take in a range of syntax elements and produce
pub fn randcode_syntax_elements(seed : u64) -> Vec<u8> {
    let mut ds: H265DecodedStream = H265DecodedStream::new();
    let mut encoded_str: Vec<u8> = Vec::new();
    let mut film : FilmState;
    if seed == 0 {
        film = FilmState::setup_film();
    } else {
        film = FilmState::setup_film_from_seed(seed);
    }

    let mut vps_idx = 0;
    let mut sps_idx = 0;
    let mut pps_idx = 0;

    let num_nalus : usize = film.read_film_u32(10, 30) as usize;
    debug!(target: "encode","Generating and encoding {} NALUs", num_nalus);
    debug!(target: "encode","Using seed: {}", film.seed);

    println!("Generating and encoding {} nalus with seed value {}", num_nalus, film.seed);

    for i in 0..num_nalus {
        ds.nalu_elements.push(NALU::new());

        ds.nalu_elements[i].longstartcode = film.read_film_bool(0, 2, 1);

        if ds.nalu_elements[i].longstartcode {
            encoded_str.push(0);
            encoded_str.push(0);
            encoded_str.push(0);
            encoded_str.push(1);
        } else {
            encoded_str.push(0);
            encoded_str.push(0);
            encoded_str.push(1);
        }

        ds.nalu_headers.push(H265NALUHeader::new());

        // randcode_nalu_header guarantees VPS, SPS, and PPS are the first items
        let encoded_header = h265_randcode_nalu_header(&mut ds.nalu_headers[i], i, &mut film);
        encoded_str.extend(encoded_header.iter());

        match ds.nalu_headers[i].nal_unit_type {
            NalUnitType::NalUnitVps => {
                println!("\t randcode_syntax_elements - NALU {} - {:?} - Randomly Encoding Video Parameter Set (VPS)", i,  ds.nalu_headers[i].nal_unit_type);
                
                ds.vpses.push(H265VideoParameterSet::new());

                encoded_str.extend(insert_emulation_three_byte(&h265_randcode_video_parameter_set(
                    &mut ds.vpses[vps_idx], &mut film
                )));
                vps_idx += 1;
            },
            NalUnitType::NalUnitSps => {
                println!("\t randcode_syntax_elements - NALU {} - {:?} - Randomly Encoding Sequence Parameter Set (SPS)", i,  ds.nalu_headers[i].nal_unit_type);
                
                ds.spses.push(H265SeqParameterSet::new());

                encoded_str.extend(insert_emulation_three_byte(&h265_randcode_seq_parameter_set(
                    &mut ds.spses[sps_idx], &mut film
                )));
                sps_idx += 1;
            },
            NalUnitType::NalUnitPps => {
                println!("\t randcode_syntax_elements - NALU {} - {:?} - Randomly Encoding Picture Parameter Set (PPS)", i,  ds.nalu_headers[i].nal_unit_type);

                ds.ppses.push(H265PicParameterSet::new());

                encoded_str.extend(insert_emulation_three_byte(&h265_randcode_pic_parameter_set(
                    &mut ds.ppses[pps_idx], ds.spses[sps_idx-1].sps_seq_parameter_set_id, &mut film
                )));
                pps_idx += 1;
            }
            _ => {
                println!(
                    "\t randcode_syntax_elements - TODO: {:?} parsing",
                    ds.nalu_headers[i].nal_unit_type
                );
                ds.nalu_elements[i].content = film.read_film_bytes(128);
                // start at 2 because the header is 2 bytes
                encoded_str.extend(insert_emulation_three_byte(
                    &ds.nalu_elements[i].content[2..],
                ));
            }
        };
    }

    encoded_str
}
