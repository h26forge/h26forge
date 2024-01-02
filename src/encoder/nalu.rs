//! NALU header and extensions syntax element encoding.

use crate::common::data_structures::AccessUnitDelim;
use crate::common::data_structures::NALUHeader;
use crate::common::data_structures::PrefixNALU;
use crate::common::data_structures::RefBasePicMarking;
use crate::common::helper::bitstream_to_bytestream;
use crate::encoder::binarization_functions::generate_unsigned_binary;
use crate::encoder::expgolomb::exp_golomb_encode_one;

/// Encode the NALU header into one byte (or two bytes if an extension)
pub fn encode_nalu_header(nh: &NALUHeader) -> Vec<u8> {
    let mut bytestream_array: Vec<u8> = Vec::new();

    bytestream_array.push(nh.forbidden_zero_bit << 7 | nh.nal_ref_idc << 5 | nh.nal_unit_type); // these should all fit one byte

    let mut bitstream_array: Vec<u8> = Vec::new();
    if nh.nal_unit_type == 14 || nh.nal_unit_type == 20 || nh.nal_unit_type == 21 {
        if nh.nal_unit_type != 21 {
            bitstream_array.push(match nh.svc_extension_flag {
                false => 0,
                true => 1,
            });
        } else {
            bitstream_array.push(match nh.avc_3d_extension_flag {
                false => 0,
                true => 1,
            });
        }

        if nh.svc_extension_flag {
            bitstream_array.append(&mut encode_nal_unit_header_svc_extension(nh));
        } else if nh.avc_3d_extension_flag {
            bitstream_array.append(&mut encode_nal_unit_header_avc3d_extension(nh));
        } else {
            bitstream_array.append(&mut encode_nal_unit_header_mvc_extension(nh));
        }

        bytestream_array.append(&mut bitstream_to_bytestream(bitstream_array, 0));
    }
    bytestream_array
}

fn encode_nal_unit_header_svc_extension(nh: &NALUHeader) -> Vec<u8> {
    let mut res = Vec::new();

    res.push(match nh.svc_extension.idr_flag {
        false => 0,
        true => 1,
    });
    res.append(&mut generate_unsigned_binary(
        nh.svc_extension.priority_id as u32,
        6,
    ));
    res.push(match nh.svc_extension.no_inter_layer_pred_flag {
        false => 0,
        true => 1,
    });
    res.append(&mut generate_unsigned_binary(
        nh.svc_extension.dependency_id as u32,
        3,
    ));
    res.append(&mut generate_unsigned_binary(
        nh.svc_extension.quality_id as u32,
        4,
    ));
    res.append(&mut generate_unsigned_binary(
        nh.svc_extension.temporal_id as u32,
        3,
    ));
    res.push(match nh.svc_extension.use_ref_base_pic_flag {
        false => 0,
        true => 1,
    });
    res.push(match nh.svc_extension.discardable_flag {
        false => 0,
        true => 1,
    });
    res.push(match nh.svc_extension.output_flag {
        false => 0,
        true => 1,
    });
    res.append(&mut generate_unsigned_binary(
        nh.svc_extension.reserved_three_2bits as u32,
        2,
    ));

    res
}

fn encode_nal_unit_header_avc3d_extension(nh: &NALUHeader) -> Vec<u8> {
    let mut res = Vec::new();

    res.append(&mut generate_unsigned_binary(
        nh.avc_3d_extension.view_idx as u32,
        8,
    ));
    res.push(match nh.avc_3d_extension.depth_flag {
        false => 0,
        true => 1,
    });
    res.push(match nh.avc_3d_extension.non_idr_flag {
        false => 0,
        true => 1,
    });
    res.append(&mut generate_unsigned_binary(
        nh.avc_3d_extension.temporal_id as u32,
        3,
    ));
    res.push(match nh.avc_3d_extension.anchor_pic_flag {
        false => 0,
        true => 1,
    });
    res.push(match nh.avc_3d_extension.inter_view_flag {
        false => 0,
        true => 1,
    });

    res
}

fn encode_nal_unit_header_mvc_extension(nh: &NALUHeader) -> Vec<u8> {
    let mut res = Vec::new();

    res.push(match nh.mvc_extension.non_idr_flag {
        false => 0,
        true => 1,
    });
    res.append(&mut generate_unsigned_binary(
        nh.mvc_extension.priority_id as u32,
        6,
    ));
    res.append(&mut generate_unsigned_binary(nh.mvc_extension.view_id, 10));
    res.append(&mut generate_unsigned_binary(
        nh.mvc_extension.temporal_id as u32,
        3,
    ));
    res.push(match nh.mvc_extension.anchor_pic_flag {
        false => 0,
        true => 1,
    });
    res.push(match nh.mvc_extension.inter_view_flag {
        false => 0,
        true => 1,
    });
    res.push(match nh.mvc_extension.reserved_one_bit {
        false => 0,
        true => 1,
    });

    res
}

/// Described in G.7.3.2.12.1 -- Prefix NAL unit SVC syntax
pub fn encode_prefix_nal_unit_svc(nh: &NALUHeader, pn: &PrefixNALU) -> Vec<u8> {
    let mut bitstream_array = Vec::new();

    if nh.nal_ref_idc != 0 {
        bitstream_array.push(match pn.store_ref_base_pic_flag {
            false => 0,
            true => 1,
        });
        if (pn.store_ref_base_pic_flag || nh.svc_extension.use_ref_base_pic_flag)
            && !nh.svc_extension.idr_flag
        {
            bitstream_array.append(&mut encode_ref_base_pic_marking(&pn.ref_base_pic_marking));
        }
        bitstream_array.push(match pn.additional_prefix_nal_unit_extension_flag {
            false => 0,
            true => 1,
        });
        if pn.additional_prefix_nal_unit_extension_flag {
            for i in 0..pn.additional_prefix_nal_unit_extension_data_flag.len() {
                bitstream_array.push(match pn.additional_prefix_nal_unit_extension_data_flag[i] {
                    false => 0,
                    true => 1,
                });
            }
        }
    }

    for i in 0..pn.additional_prefix_nal_unit_extension_data_flag.len() {
        bitstream_array.push(match pn.additional_prefix_nal_unit_extension_data_flag[i] {
            false => 0,
            true => 1,
        });
    }

    pn.encoder_pretty_print();

    // RBSP trailing bits
    bitstream_array.push(1);

    bitstream_to_bytestream(bitstream_array, 0)
}

fn encode_ref_base_pic_marking(pn: &RefBasePicMarking) -> Vec<u8> {
    let mut bitstream_array = Vec::new();

    bitstream_array.push(match pn.adaptive_ref_base_pic_marking_mode_flag {
        false => 0,
        true => 1,
    });

    if pn.adaptive_ref_base_pic_marking_mode_flag {
        for i in 0..pn.memory_management_base_control_operation.len() {
            bitstream_array.append(&mut exp_golomb_encode_one(
                pn.memory_management_base_control_operation[i] as i32,
                false,
                0,
                false,
            ));
            if pn.memory_management_base_control_operation[i] == 1 {
                bitstream_array.append(&mut exp_golomb_encode_one(
                    pn.difference_of_base_pic_nums_minus1[i] as i32,
                    false,
                    0,
                    false,
                ));
            } else if pn.memory_management_base_control_operation[i] == 2 {
                bitstream_array.append(&mut exp_golomb_encode_one(
                    pn.long_term_base_pic_num[i] as i32,
                    false,
                    0,
                    false,
                ));
            }
        }
    }

    bitstream_array
}

/// Described in 7.3.2.4 -- Access Unit Delimiter
pub fn encode_access_unit_delimiter(aud: &AccessUnitDelim) -> Vec<u8> {
    let mut bitstream_array = Vec::new();

    bitstream_array.append(&mut generate_unsigned_binary(
        aud.primary_pic_type as u32,
        3,
    ));
    aud.encoder_pretty_print();

    // RBSP trailing bits
    bitstream_array.push(1);

    bitstream_to_bytestream(bitstream_array, 0)
}
