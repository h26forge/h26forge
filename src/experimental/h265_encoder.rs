//! H.265 syntax element encoding.

use crate::encoder::binarization_functions::generate_unsigned_binary;
use crate::encoder::encoder::insert_emulation_three_byte;
use crate::encoder::expgolomb::exp_golomb_encode_one;
use crate::experimental::h265_data_structures::{
    H265DecodedStream, H265NALUHeader, H265SeqParameterSet, ProfileTierLevel, ShortTermRefPic,
};
use crate::{
    common::helper::bitstream_to_bytestream, experimental::h265_data_structures::NalUnitType,
};
use log::debug;

fn h265_encode_nalu_header(nh: &H265NALUHeader) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    bitstream_array.push(nh.forbidden_zero_bit);

    match nh.nal_unit_type {
        NalUnitType::NalUnitCodedSliceTrailN => {
            bitstream_array.append(&mut generate_unsigned_binary(0, 6))
        }
        NalUnitType::NalUnitCodedSliceTrailR => {
            bitstream_array.append(&mut generate_unsigned_binary(1, 6))
        }
        NalUnitType::NalUnitCodedSliceTsaN => {
            bitstream_array.append(&mut generate_unsigned_binary(2, 6))
        }
        NalUnitType::NalUnitCodedSliceTsaR => {
            bitstream_array.append(&mut generate_unsigned_binary(3, 6))
        }
        NalUnitType::NalUnitCodedSliceStsaN => {
            bitstream_array.append(&mut generate_unsigned_binary(4, 6))
        }
        NalUnitType::NalUnitCodedSliceStsaR => {
            bitstream_array.append(&mut generate_unsigned_binary(5, 6))
        }
        NalUnitType::NalUnitCodedSliceRadlN => {
            bitstream_array.append(&mut generate_unsigned_binary(6, 6))
        }
        NalUnitType::NalUnitCodedSliceRadlR => {
            bitstream_array.append(&mut generate_unsigned_binary(7, 6))
        }
        NalUnitType::NalUnitCodedSliceRaslN => {
            bitstream_array.append(&mut generate_unsigned_binary(8, 6))
        }
        NalUnitType::NalUnitCodedSliceRaslR => {
            bitstream_array.append(&mut generate_unsigned_binary(9, 6))
        }
        NalUnitType::NalUnitReservedVclN10 => {
            bitstream_array.append(&mut generate_unsigned_binary(10, 6))
        }
        NalUnitType::NalUnitReservedVclR11 => {
            bitstream_array.append(&mut generate_unsigned_binary(11, 6))
        }
        NalUnitType::NalUnitReservedVclN12 => {
            bitstream_array.append(&mut generate_unsigned_binary(12, 6))
        }
        NalUnitType::NalUnitReservedVclR13 => {
            bitstream_array.append(&mut generate_unsigned_binary(13, 6))
        }
        NalUnitType::NalUnitReservedVclN14 => {
            bitstream_array.append(&mut generate_unsigned_binary(14, 6))
        }
        NalUnitType::NalUnitCodedSliceBlaWLp => {
            bitstream_array.append(&mut generate_unsigned_binary(15, 6))
        }
        NalUnitType::NalUnitReservedVclR15 => {
            bitstream_array.append(&mut generate_unsigned_binary(16, 6))
        }
        NalUnitType::NalUnitCodedSliceBlaWRadl => {
            bitstream_array.append(&mut generate_unsigned_binary(17, 6))
        }
        NalUnitType::NalUnitCodedSliceBlaNLp => {
            bitstream_array.append(&mut generate_unsigned_binary(18, 6))
        }
        NalUnitType::NalUnitCodedSliceIdrWRadl => {
            bitstream_array.append(&mut generate_unsigned_binary(19, 6))
        }
        NalUnitType::NalUnitCodedSliceIdrNLp => {
            bitstream_array.append(&mut generate_unsigned_binary(20, 6))
        }
        NalUnitType::NalUnitCodedSliceCra => {
            bitstream_array.append(&mut generate_unsigned_binary(21, 6))
        }
        NalUnitType::NalUnitReservedIrapVcl22 => {
            bitstream_array.append(&mut generate_unsigned_binary(22, 6))
        }
        NalUnitType::NalUnitReservedIrapVcl23 => {
            bitstream_array.append(&mut generate_unsigned_binary(23, 6))
        }
        NalUnitType::NalUnitReservedVcl24 => {
            bitstream_array.append(&mut generate_unsigned_binary(24, 6))
        }
        NalUnitType::NalUnitReservedVcl25 => {
            bitstream_array.append(&mut generate_unsigned_binary(25, 6))
        }
        NalUnitType::NalUnitReservedVcl26 => {
            bitstream_array.append(&mut generate_unsigned_binary(26, 6))
        }
        NalUnitType::NalUnitReservedVcl27 => {
            bitstream_array.append(&mut generate_unsigned_binary(27, 6))
        }
        NalUnitType::NalUnitReservedVcl28 => {
            bitstream_array.append(&mut generate_unsigned_binary(28, 6))
        }
        NalUnitType::NalUnitReservedVcl29 => {
            bitstream_array.append(&mut generate_unsigned_binary(29, 6))
        }
        NalUnitType::NalUnitReservedVcl30 => {
            bitstream_array.append(&mut generate_unsigned_binary(30, 6))
        }
        NalUnitType::NalUnitReservedVcl31 => {
            bitstream_array.append(&mut generate_unsigned_binary(31, 6))
        }
        NalUnitType::NalUnitVps => bitstream_array.append(&mut generate_unsigned_binary(32, 6)),
        NalUnitType::NalUnitSps => bitstream_array.append(&mut generate_unsigned_binary(33, 6)),
        NalUnitType::NalUnitPps => bitstream_array.append(&mut generate_unsigned_binary(34, 6)),
        NalUnitType::NalUnitAccessUnitDelimiter => {
            bitstream_array.append(&mut generate_unsigned_binary(35, 6))
        }
        NalUnitType::NalUnitEos => bitstream_array.append(&mut generate_unsigned_binary(36, 6)),
        NalUnitType::NalUnitEob => bitstream_array.append(&mut generate_unsigned_binary(37, 6)),
        NalUnitType::NalUnitFillerData => {
            bitstream_array.append(&mut generate_unsigned_binary(38, 6))
        }
        NalUnitType::NalUnitPrefixSei => {
            bitstream_array.append(&mut generate_unsigned_binary(39, 6))
        }
        NalUnitType::NalUnitSuffixSei => {
            bitstream_array.append(&mut generate_unsigned_binary(40, 6))
        }
        NalUnitType::NalUnitReservedNvcl41 => {
            bitstream_array.append(&mut generate_unsigned_binary(41, 6))
        }
        NalUnitType::NalUnitReservedNvcl42 => {
            bitstream_array.append(&mut generate_unsigned_binary(42, 6))
        }
        NalUnitType::NalUnitReservedNvcl43 => {
            bitstream_array.append(&mut generate_unsigned_binary(43, 6))
        }
        NalUnitType::NalUnitReservedNvcl44 => {
            bitstream_array.append(&mut generate_unsigned_binary(44, 6))
        }
        NalUnitType::NalUnitReservedNvcl45 => {
            bitstream_array.append(&mut generate_unsigned_binary(45, 6))
        }
        NalUnitType::NalUnitReservedNvcl46 => {
            bitstream_array.append(&mut generate_unsigned_binary(46, 6))
        }
        NalUnitType::NalUnitReservedNvcl47 => {
            bitstream_array.append(&mut generate_unsigned_binary(47, 6))
        }
        NalUnitType::NalUnitUnspecified48 => {
            bitstream_array.append(&mut generate_unsigned_binary(48, 6))
        }
        NalUnitType::NalUnitUnspecified49 => {
            bitstream_array.append(&mut generate_unsigned_binary(49, 6))
        }
        NalUnitType::NalUnitUnspecified50 => {
            bitstream_array.append(&mut generate_unsigned_binary(50, 6))
        }
        NalUnitType::NalUnitUnspecified51 => {
            bitstream_array.append(&mut generate_unsigned_binary(51, 6))
        }
        NalUnitType::NalUnitUnspecified52 => {
            bitstream_array.append(&mut generate_unsigned_binary(52, 6))
        }
        NalUnitType::NalUnitUnspecified53 => {
            bitstream_array.append(&mut generate_unsigned_binary(53, 6))
        }
        NalUnitType::NalUnitUnspecified54 => {
            bitstream_array.append(&mut generate_unsigned_binary(54, 6))
        }
        NalUnitType::NalUnitUnspecified55 => {
            bitstream_array.append(&mut generate_unsigned_binary(55, 6))
        }
        NalUnitType::NalUnitUnspecified56 => {
            bitstream_array.append(&mut generate_unsigned_binary(56, 6))
        }
        NalUnitType::NalUnitUnspecified57 => {
            bitstream_array.append(&mut generate_unsigned_binary(57, 6))
        }
        NalUnitType::NalUnitUnspecified58 => {
            bitstream_array.append(&mut generate_unsigned_binary(58, 6))
        }
        NalUnitType::NalUnitUnspecified59 => {
            bitstream_array.append(&mut generate_unsigned_binary(59, 6))
        }
        NalUnitType::NalUnitUnspecified60 => {
            bitstream_array.append(&mut generate_unsigned_binary(60, 6))
        }
        NalUnitType::NalUnitUnspecified61 => {
            bitstream_array.append(&mut generate_unsigned_binary(61, 6))
        }
        NalUnitType::NalUnitUnspecified62 => {
            bitstream_array.append(&mut generate_unsigned_binary(62, 6))
        }
        NalUnitType::NalUnitUnspecified63 => {
            bitstream_array.append(&mut generate_unsigned_binary(63, 6))
        }
        NalUnitType::NalUnitInvalid => panic!("[h265_encode_nalu_header] NAL UNIT TYPE NEVER SET"),
    };

    bitstream_array.append(&mut generate_unsigned_binary(nh.nuh_layer_id as u32, 6));
    bitstream_array.append(&mut generate_unsigned_binary(
        nh.nuh_temporal_id_plus1 as u32,
        3,
    ));

    bitstream_to_bytestream(bitstream_array, 0)
}

fn h265_encode_profile_tier_level(
    profile_present_flag: bool,
    max_sub_layers_minus1: usize,
    ptl: &ProfileTierLevel,
) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    if profile_present_flag {
        bitstream_array.extend(generate_unsigned_binary(
            ptl.general_profile_space as u32,
            2,
        ));
        bitstream_array.push(match ptl.general_tier_flag {
            true => 1,
            _ => 0,
        });
        bitstream_array.extend(generate_unsigned_binary(ptl.general_profile_idc as u32, 5));

        for i in 0..32 {
            bitstream_array.push(match ptl.general_profile_compatibility_flag[i] {
                true => 1,
                _ => 0,
            });
        }

        bitstream_array.push(match ptl.general_progressive_source_flag {
            true => 1,
            _ => 0,
        });
        bitstream_array.push(match ptl.general_interlaced_source_flag {
            true => 1,
            _ => 0,
        });
        bitstream_array.push(match ptl.general_non_packed_constraint_flag {
            true => 1,
            _ => 0,
        });
        bitstream_array.push(match ptl.general_frame_only_constraint_flag {
            true => 1,
            _ => 0,
        });

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

            bitstream_array.push(match ptl.general_max_12bit_constraint_flag {
                true => 1,
                _ => 0,
            });
            bitstream_array.push(match ptl.general_max_10bit_constraint_flag {
                true => 1,
                _ => 0,
            });
            bitstream_array.push(match ptl.general_max_8bit_constraint_flag {
                true => 1,
                _ => 0,
            });
            bitstream_array.push(match ptl.general_max_422chroma_constraint_flag {
                true => 1,
                _ => 0,
            });
            bitstream_array.push(match ptl.general_max_420chroma_constraint_flag {
                true => 1,
                _ => 0,
            });
            bitstream_array.push(match ptl.general_max_monochrome_constraint_flag {
                true => 1,
                _ => 0,
            });
            bitstream_array.push(match ptl.general_intra_constraint_flag {
                true => 1,
                _ => 0,
            });
            bitstream_array.push(match ptl.general_one_picture_only_constraint_flag {
                true => 1,
                _ => 0,
            });
            bitstream_array.push(match ptl.general_lower_bit_rate_constraint_flag {
                true => 1,
                _ => 0,
            });

            if ptl.general_profile_idc == 5
                || ptl.general_profile_compatibility_flag[5]
                || ptl.general_profile_idc == 9
                || ptl.general_profile_compatibility_flag[9]
                || ptl.general_profile_idc == 10
                || ptl.general_profile_compatibility_flag[10]
                || ptl.general_profile_idc == 11
                || ptl.general_profile_compatibility_flag[11]
            {
                bitstream_array.push(match ptl.general_max_14bit_constraint_flag {
                    true => 1,
                    _ => 0,
                });

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
            bitstream_array.extend(generate_unsigned_binary(
                ptl.general_reserved_zero_7bits as u32,
                7,
            ));

            bitstream_array.push(match ptl.general_one_picture_only_constraint_flag {
                true => 1,
                _ => 0,
            });

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
            // The number of bits in this syntax structure is not affected by this condition
            bitstream_array.push(match ptl.general_inbld_flag {
                true => 1,
                _ => 0,
            });
        } else {
            bitstream_array.push(ptl.general_reserved_zero_bit);
        }
    }

    bitstream_array.extend(generate_unsigned_binary(ptl.general_level_idc as u32, 8));

    for i in 0..max_sub_layers_minus1 {
        bitstream_array.push(match ptl.sub_layer_profile_present_flag[i] {
            true => 1,
            _ => 0,
        });
        bitstream_array.push(match ptl.sub_layer_level_present_flag[i] {
            true => 1,
            _ => 0,
        });
    }

    if max_sub_layers_minus1 > 0 {
        for i in max_sub_layers_minus1..8 {
            bitstream_array.extend(generate_unsigned_binary(
                ptl.reserved_zero_2bits[i] as u32,
                2,
            ));
        }
    }

    for i in 0..max_sub_layers_minus1 {
        if ptl.sub_layer_profile_present_flag[i] {
            bitstream_array.extend(generate_unsigned_binary(
                ptl.sub_layer_profile_space[i] as u32,
                2,
            ));
            bitstream_array.push(match ptl.sub_layer_tier_flag[i] {
                true => 1,
                _ => 0,
            });
            bitstream_array.extend(generate_unsigned_binary(
                ptl.sub_layer_profile_idc[i] as u32,
                5,
            ));

            for j in 0..32 {
                bitstream_array.push(match ptl.sub_layer_profile_compatibility_flag[i][j] {
                    true => 1,
                    _ => 0,
                });
            }

            bitstream_array.push(match ptl.sub_layer_progressive_source_flag[i] {
                true => 1,
                _ => 0,
            });
            bitstream_array.push(match ptl.sub_layer_interlaced_source_flag[i] {
                true => 1,
                _ => 0,
            });
            bitstream_array.push(match ptl.sub_layer_non_packed_constraint_flag[i] {
                true => 1,
                _ => 0,
            });
            bitstream_array.push(match ptl.sub_layer_frame_only_constraint_flag[i] {
                true => 1,
                _ => 0,
            });

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

                bitstream_array.push(match ptl.sub_layer_max_12bit_constraint_flag[i] {
                    true => 1,
                    _ => 0,
                });
                bitstream_array.push(match ptl.sub_layer_max_10bit_constraint_flag[i] {
                    true => 1,
                    _ => 0,
                });
                bitstream_array.push(match ptl.sub_layer_max_8bit_constraint_flag[i] {
                    true => 1,
                    _ => 0,
                });
                bitstream_array.push(match ptl.sub_layer_max_422chroma_constraint_flag[i] {
                    true => 1,
                    _ => 0,
                });
                bitstream_array.push(match ptl.sub_layer_max_420chroma_constraint_flag[i] {
                    true => 1,
                    _ => 0,
                });
                bitstream_array.push(match ptl.sub_layer_max_monochrome_constraint_flag[i] {
                    true => 1,
                    _ => 0,
                });
                bitstream_array.push(match ptl.sub_layer_intra_constraint_flag[i] {
                    true => 1,
                    _ => 0,
                });
                bitstream_array.push(match ptl.sub_layer_one_picture_only_constraint_flag[i] {
                    true => 1,
                    _ => 0,
                });
                bitstream_array.push(match ptl.sub_layer_lower_bit_rate_constraint_flag[i] {
                    true => 1,
                    _ => 0,
                });

                if ptl.sub_layer_profile_idc[i] == 5
                    || ptl.sub_layer_profile_compatibility_flag[i][5]
                    || ptl.sub_layer_profile_idc[i] == 9
                    || ptl.sub_layer_profile_compatibility_flag[i][9]
                    || ptl.sub_layer_profile_idc[i] == 10
                    || ptl.sub_layer_profile_compatibility_flag[i][10]
                    || ptl.sub_layer_profile_idc[i] == 11
                    || ptl.sub_layer_profile_compatibility_flag[i][11]
                {
                    bitstream_array.push(match ptl.sub_layer_max_14bit_constraint_flag[i] {
                        true => 1,
                        _ => 0,
                    });

                    // store as an unsigned binary encoding
                    for i in (0..33).rev() {
                        bitstream_array
                            .push(((ptl.sub_layer_reserved_zero_33bits[i] >> i) & 1) as u8);
                    }
                } else {
                    // store as an unsigned binary encoding
                    for i in (0..34).rev() {
                        bitstream_array
                            .push(((ptl.sub_layer_reserved_zero_34bits[i] >> i) & 1) as u8);
                    }
                }
            } else if ptl.sub_layer_profile_idc[i] == 2
                || ptl.sub_layer_profile_compatibility_flag[i][2]
            {
                bitstream_array.extend(generate_unsigned_binary(
                    ptl.sub_layer_reserved_zero_7bits[i] as u32,
                    7,
                ));
                bitstream_array.push(match ptl.sub_layer_one_picture_only_constraint_flag[i] {
                    true => 1,
                    _ => 0,
                });

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
                // The number of bits in this syntax structure is not affected by this condition
                bitstream_array.push(match ptl.sub_layer_inbld_flag[i] {
                    true => 1,
                    _ => 0,
                });
            } else {
                bitstream_array.push(ptl.sub_layer_reserved_zero_bit[i]);
            }
        }

        if ptl.sub_layer_level_present_flag[i] {
            bitstream_array.extend(generate_unsigned_binary(
                ptl.sub_layer_level_idc[i] as u32,
                8,
            ));
        }
    }

    bitstream_array
}

fn h265_encode_st_ref_pic_set(
    st_rps_idx: u32,
    num_short_term_ref_pic_sets: u32,
    ref_pics: &Vec<ShortTermRefPic>,
    strp: &ShortTermRefPic,
) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    if st_rps_idx != 0 {
        bitstream_array.push(match strp.inter_ref_pic_set_prediction_flag {
            true => 1,
            _ => 0,
        });
    }

    if strp.inter_ref_pic_set_prediction_flag {
        if st_rps_idx == num_short_term_ref_pic_sets {
            bitstream_array.extend(exp_golomb_encode_one(
                strp.delta_idx_minus1 as i32,
                false,
                0,
                false,
            ));

            if (strp.delta_idx_minus1 + 1) >= st_rps_idx {
                println!("[WARNING] StRefPic delta_idx_minus1 is greater than st_rps_idx. May encounter encoding errors");
                debug!(target: "encode","[WARNING] StRefPic delta_idx_minus1 is greater than st_rps_idx. May encounter encoding errors");
            }
        }

        bitstream_array.push(match strp.delta_rps_sign {
            true => 1,
            _ => 0,
        });
        bitstream_array.extend(exp_golomb_encode_one(
            strp.abs_delta_rps_minus1 as i32,
            false,
            0,
            false,
        ));
        // NumDeltaPics is an array of stored delta values
        // RefRpsIdx is st_rps_idx - (delta_idx_minus1 + 1). When not present, it's inferred to be 0
        let ref_rps_idx = (st_rps_idx - (strp.delta_idx_minus1 + 1)) as usize % ref_pics.len();
        for j in 0..=(ref_pics[ref_rps_idx].num_delta_pics as usize) {
            bitstream_array.push(match strp.used_by_curr_pic_flag[j] {
                true => 1,
                _ => 0,
            });
            if !strp.used_by_curr_pic_flag[j] {
                bitstream_array.push(match strp.use_delta_flag[j] {
                    true => 1,
                    _ => 0,
                });
            }
        }
    } else {
        bitstream_array.extend(exp_golomb_encode_one(
            strp.num_negative_pics as i32,
            false,
            0,
            false,
        ));
        bitstream_array.extend(exp_golomb_encode_one(
            strp.num_positive_pics as i32,
            false,
            0,
            false,
        ));

        for i in 0..(strp.num_negative_pics as usize) {
            bitstream_array.extend(exp_golomb_encode_one(
                strp.delta_poc_s0_minus1[i] as i32,
                false,
                0,
                false,
            ));

            bitstream_array.push(match strp.used_by_curr_pic_s0_flag[i] {
                true => 1,
                _ => 0,
            });
        }
        for i in 0..(strp.num_positive_pics as usize) {
            bitstream_array.extend(exp_golomb_encode_one(
                strp.delta_poc_s1_minus1[i] as i32,
                false,
                0,
                false,
            ));
            bitstream_array.push(match strp.used_by_curr_pic_s1_flag[i] {
                true => 1,
                _ => 0,
            });
        }
    }

    bitstream_array
}

fn h265_encode_seq_parameter_set(sps: &H265SeqParameterSet) -> Vec<u8> {
    let mut bitstream_array: Vec<u8> = Vec::new();

    bitstream_array.extend(generate_unsigned_binary(
        sps.sps_video_parameter_set_id as u32,
        4,
    ));
    bitstream_array.extend(generate_unsigned_binary(
        sps.sps_max_sub_layers_minus1 as u32,
        3,
    ));
    bitstream_array.push(match sps.sps_temporal_id_nesting_flag {
        true => 1,
        _ => 0,
    });

    bitstream_array.extend(h265_encode_profile_tier_level(
        true,
        sps.sps_max_sub_layers_minus1 as usize,
        &sps.profile_tier_level,
    ));

    bitstream_array.extend(exp_golomb_encode_one(
        sps.sps_seq_parameter_set_id as i32,
        false,
        0,
        false,
    ));
    bitstream_array.extend(exp_golomb_encode_one(
        sps.chroma_format_idc as i32,
        false,
        0,
        false,
    ));

    if sps.chroma_format_idc == 3 {
        bitstream_array.push(match sps.separate_colour_plane_flag {
            true => 1,
            _ => 0,
        });
    }

    bitstream_array.extend(exp_golomb_encode_one(
        sps.pic_width_in_luma_samples as i32,
        false,
        0,
        false,
    ));
    bitstream_array.extend(exp_golomb_encode_one(
        sps.pic_height_in_luma_samples as i32,
        false,
        0,
        false,
    ));
    bitstream_array.push(match sps.conformance_window_flag {
        true => 1,
        _ => 0,
    });

    if sps.conformance_window_flag {
        bitstream_array.extend(exp_golomb_encode_one(
            sps.conf_win_left_offset as i32,
            false,
            0,
            false,
        ));
        bitstream_array.extend(exp_golomb_encode_one(
            sps.conf_win_right_offset as i32,
            false,
            0,
            false,
        ));
        bitstream_array.extend(exp_golomb_encode_one(
            sps.conf_win_top_offset as i32,
            false,
            0,
            false,
        ));
        bitstream_array.extend(exp_golomb_encode_one(
            sps.conf_win_bottom_offset as i32,
            false,
            0,
            false,
        ));
    }
    bitstream_array.extend(exp_golomb_encode_one(
        sps.bit_depth_luma_minus8 as i32,
        false,
        0,
        false,
    ));
    bitstream_array.extend(exp_golomb_encode_one(
        sps.bit_depth_chroma_minus8 as i32,
        false,
        0,
        false,
    ));
    bitstream_array.extend(exp_golomb_encode_one(
        sps.log2_max_pic_order_cnt_lsb_minus4 as i32,
        false,
        0,
        false,
    ));
    bitstream_array.push(match sps.sps_sub_layer_ordering_info_present_flag {
        true => 1,
        _ => 0,
    });

    let min = if sps.sps_sub_layer_ordering_info_present_flag {
        0
    } else {
        sps.sps_max_sub_layers_minus1 as usize
    };

    for i in min..=(sps.sps_max_sub_layers_minus1 as usize) {
        bitstream_array.extend(exp_golomb_encode_one(
            sps.sps_max_dec_pic_buffering_minus1[i] as i32,
            false,
            0,
            false,
        ));
        bitstream_array.extend(exp_golomb_encode_one(
            sps.sps_max_num_reorder_pics[i] as i32,
            false,
            0,
            false,
        ));
        bitstream_array.extend(exp_golomb_encode_one(
            sps.sps_max_latency_increase_plus1[i] as i32,
            false,
            0,
            false,
        ));
    }
    bitstream_array.extend(exp_golomb_encode_one(
        sps.log2_min_luma_coding_block_size_minus3 as i32,
        false,
        0,
        false,
    ));
    bitstream_array.extend(exp_golomb_encode_one(
        sps.log2_diff_max_min_luma_coding_block_size as i32,
        false,
        0,
        false,
    ));
    bitstream_array.extend(exp_golomb_encode_one(
        sps.log2_min_luma_transform_block_size_minus2 as i32,
        false,
        0,
        false,
    ));
    bitstream_array.extend(exp_golomb_encode_one(
        sps.log2_diff_max_min_luma_transform_block_size as i32,
        false,
        0,
        false,
    ));
    bitstream_array.extend(exp_golomb_encode_one(
        sps.max_transform_hierarchy_depth_inter as i32,
        false,
        0,
        false,
    ));
    bitstream_array.extend(exp_golomb_encode_one(
        sps.max_transform_hierarchy_depth_intra as i32,
        false,
        0,
        false,
    ));
    bitstream_array.push(match sps.scaling_list_enabled_flag {
        true => 1,
        _ => 0,
    });

    // TODO: H265 scaling list
    //if sps.scaling_list_enabled_flag  {
    //    bitstream_array.push(match sps.sps_scaling_list_data_present_flag {true => 1, _ => 0});
    //
    //    if sps.sps_scaling_list_data_present_flag {
    //        sps.scaling_list_data = h265_encode_scaling_list(bs);
    //    }
    //
    //}

    bitstream_array.push(match sps.amp_enabled_flag {
        true => 1,
        _ => 0,
    });
    bitstream_array.push(match sps.sample_adaptive_offset_enabled_flag {
        true => 1,
        _ => 0,
    });
    bitstream_array.push(match sps.pcm_enabled_flag {
        true => 1,
        _ => 0,
    });

    if sps.pcm_enabled_flag {
        bitstream_array.extend(generate_unsigned_binary(
            sps.pcm_sample_bit_depth_luma_minus1 as u32,
            4,
        ));
        bitstream_array.extend(generate_unsigned_binary(
            sps.pcm_sample_bit_depth_chroma_minus1 as u32,
            4,
        ));

        bitstream_array.extend(exp_golomb_encode_one(
            sps.log2_min_pcm_luma_coding_block_size_minus3 as i32,
            false,
            0,
            false,
        ));
        bitstream_array.extend(exp_golomb_encode_one(
            sps.log2_diff_max_min_pcm_luma_coding_block_size as i32,
            false,
            0,
            false,
        ));
        bitstream_array.push(match sps.pcm_loop_filter_disabled_flag {
            true => 1,
            _ => 0,
        });
    }

    bitstream_array.extend(exp_golomb_encode_one(
        sps.num_short_term_ref_pic_sets as i32,
        false,
        0,
        false,
    ));

    for i in 0..sps.num_short_term_ref_pic_sets {
        bitstream_array.extend(h265_encode_st_ref_pic_set(
            i,
            sps.num_short_term_ref_pic_sets,
            &sps.st_ref_pic_set,
            &sps.st_ref_pic_set[i as usize],
        ));
    }

    bitstream_array.push(match sps.long_term_ref_pics_present_flag {
        true => 1,
        _ => 0,
    });

    if sps.long_term_ref_pics_present_flag {
        bitstream_array.extend(exp_golomb_encode_one(
            sps.num_long_term_ref_pics_sps as i32,
            false,
            0,
            false,
        ));

        let bit_size = sps.log2_max_pic_order_cnt_lsb_minus4 + 4;
        for i in 0..(sps.num_long_term_ref_pics_sps as usize) {
            bitstream_array.extend(generate_unsigned_binary(
                sps.lt_ref_pic_poc_lsb_sps[i] as u32,
                bit_size as usize,
            ));
            bitstream_array.push(match sps.used_by_curr_pic_lt_sps_flag[i] {
                true => 1,
                _ => 0,
            });
        }
    }

    bitstream_array.push(match sps.sps_temporal_mvp_enabled_flag {
        true => 1,
        _ => 0,
    });
    bitstream_array.push(match sps.strong_intra_smoothing_enabled_flag {
        true => 1,
        _ => 0,
    });
    bitstream_array.push(match sps.vui_parameters_present_flag {
        true => 1,
        _ => 0,
    });

    // TODO: H.265 VUI Parameters
    //if sps.vui_parameters_present_flag {
    //    sps.vui_parameters = h265_encode_vui_parameters();
    //}

    bitstream_array.push(match sps.sps_extension_present_flag {
        true => 1,
        _ => 0,
    });
    if sps.sps_extension_present_flag {
        bitstream_array.push(match sps.sps_range_extension_flag {
            true => 1,
            _ => 0,
        });
        bitstream_array.push(match sps.sps_multilayer_extension_flag {
            true => 1,
            _ => 0,
        });
        bitstream_array.push(match sps.sps_3d_extension_flag {
            true => 1,
            _ => 0,
        });
        bitstream_array.push(match sps.sps_scc_extension_flag {
            true => 1,
            _ => 0,
        });
        bitstream_array.extend(generate_unsigned_binary(sps.sps_extension_4bits as u32, 4));
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
    //    bitstream_array.push(match sps.sps_extension_data_flag {true => 1, _ => 0});
    //}

    // insert rbsp_stop_one_bit
    bitstream_array.push(1);

    sps.encoder_pretty_print();

    bitstream_to_bytestream(bitstream_array, 0)
}

/// Given a H265 Decoded Stream object, reencode it and write it to a file
pub fn encode_bitstream(ds: &mut H265DecodedStream) -> Vec<u8> {
    let mut encoded_str: Vec<u8> = Vec::new();

    let mut sps_idx = 0;

    // AVCC encoding elements

    debug!(target: "encode","Encoding {} NALUs", ds.nalu_elements.len());

    for i in 0..ds.nalu_elements.len() {
        debug!(target: "encode","");
        debug!(target: "encode","Annex B NALU w/ {} startcode, len 0, forbidden_bit {}, nuh_layer_id {}, nuh_temporal_id_plus1 {}, nal_unit_type {:?}",
            { if ds.nalu_elements[i].longstartcode {"long" } else {"short"} }, ds.nalu_headers[i].forbidden_zero_bit,
                ds.nalu_headers[i].nuh_layer_id, ds.nalu_headers[i].nuh_temporal_id_plus1, ds.nalu_headers[i].nal_unit_type);

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

        let encoded_header = h265_encode_nalu_header(&ds.nalu_headers[i]);
        encoded_str.extend(encoded_header.iter());

        match ds.nalu_headers[i].nal_unit_type {
            NalUnitType::NalUnitSps => {
                println!("\t encode_bitstream - NALU {} - {:?} - Encoding Sequence Parameter Set (SPS)", i,  ds.nalu_headers[i].nal_unit_type);

                encoded_str.extend(insert_emulation_three_byte(&h265_encode_seq_parameter_set(
                    &ds.spses[sps_idx],
                )));
                sps_idx += 1;
            }
            _ => {
                println!(
                    "\t encode_bitstream - TODO: {:?} parsing",
                    ds.nalu_headers[i].nal_unit_type
                );
                // start at 2 because the header is 2 bytes
                encoded_str.extend(insert_emulation_three_byte(
                    &ds.nalu_elements[i].content[2..],
                ));
            }
        };
    }

    encoded_str
}
