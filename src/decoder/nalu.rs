//! NALU header and extensions syntax element decoding.

use crate::common::data_structures::AccessUnitDelim;
use crate::common::data_structures::NALUHeader3DAVCExtension;
use crate::common::data_structures::NALUHeaderMVCExtension;
use crate::common::data_structures::NALUHeaderSVCExtension;
use crate::common::data_structures::NALUheader;
use crate::common::data_structures::PrefixNALU;
use crate::common::data_structures::NALU;
use crate::common::data_structures::RefBasePicMarking;
use crate::common::helper::decoder_formatted_print;
use crate::common::helper::ByteStream;
use crate::decoder::expgolomb::exp_golomb_decode_one_wrapper;
use log::debug;
use std::fs::File;
use std::io::Read;

/// Split a bytestream into NALUs
pub fn split_into_nalu(filename: &str) -> Vec<NALU> {
    let f = match File::open(filename) {
        Err(_) => {
            println!("ERROR - couldn't open {}", filename);
            std::process::exit(1);
        }
        Ok(file) => file,
    };

    let mut results: Vec<NALU> = Vec::new();

    // state machine approach to find matching NALU start codes
    let mut zerocount = 0;
    let mut longstart = false;
    let mut curnalu: Vec<u8> = Vec::new();
    let mut firststore = false;

    for (_, byte) in f.bytes().enumerate() {
        let curbyte = byte.unwrap();

        if firststore {
            curnalu.push(curbyte);
        }

        if curbyte == 0 {
            zerocount += 1;
        } else if curbyte == 1 {
            // two or more, followed by 0x01, counts as a start code
            if zerocount > 1 {
                // if is the first NALU, it's time to start
                // storing contents
                if !firststore {
                    firststore = true;

                // if it's not, then let's remove the start codes that were added,
                // save the curnalu, and reset it
                } else {
                    curnalu.pop(); // 1
                                   // remove zerocount amount
                    let final_length = curnalu.len().saturating_sub(zerocount);
                    curnalu.truncate(final_length);

                    let cur: NALU = NALU {
                        longstartcode: longstart,
                        content: remove_emulation_prevention_three_byte(&curnalu),
                    };
                    results.push(cur);
                    curnalu.truncate(0); // reset it
                }

                longstart = zerocount == 3;
            }

            // reset the zero count
            zerocount = 0;
        } else {
            // go back to the start
            zerocount = 0;
        }
    }

    // push the last NALU if start code was found
    if firststore {
        let cur: NALU = NALU {
            longstartcode: longstart,
            content: remove_emulation_prevention_three_byte(&curnalu),
        };
        results.push(cur);
    }
    debug!(target: "decode","Found {} NALUs", results.len());
    debug!(target: "decode","Done splitting");

    results
}

/// The emulation prevention three byte (0x00 0x00 0x03) is inserted into a stream
/// whenever the encoded contents are:
///  - 0x00 0x00 0x00
///  - 0x00 0x00 0x01
///  - 0x00 0x00 0x02
///  - 0x00 0x00 0x03
///
/// This is to prevent confusion with a potential start code
fn remove_emulation_prevention_three_byte(stream: &[u8]) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();

    let mut zero1: bool = false;
    let mut zero2: bool = false;

    let mut i = 0;
    while i < stream.len() {
        if zero1 {
            if zero2 {
                if stream[i] != 3 {
                    // our emulation prevention 3 byte gets skipped
                    res.push(stream[i]);
                }
                zero1 = false;
                zero2 = false;
            } else {
                if stream[i] == 0 {
                    zero2 = true;
                } else {
                    zero1 = false;
                }
                res.push(stream[i]);
            }
        } else {
            if stream[i] == 0 {
                zero1 = true;
            }
            res.push(stream[i]);
        }

        i += 1;
    }
    res
}

/// Parse NALU header contents
pub fn decode_nalu_header(long_start_code: bool, nalu_data: &mut ByteStream) -> NALUheader {
    // each NALU has a 1 byte header; the rest is control information
    // or coded video data

    let forbidden_zero_bit = nalu_data.read_bits(1) as u8; // f(1)
    let nal_ref_idc = nalu_data.read_bits(2) as u8; // u(2)
    let nal_unit_type = nalu_data.read_bits(5) as u8; // u(5)
    debug!(target: "decode","");
    debug!(target: "decode","");
    debug!(target: "decode","Annex B NALU w/ {} startcode, len {}, forbidden_bit {}, nal_reference_idc {}, nal_unit_type {}",
            { if long_start_code {"long" } else {"short"} }, nalu_data.bytestream.len() + 1, forbidden_zero_bit, nal_ref_idc, nal_unit_type);
    debug!(target: "decode","");

    let mut svc_extension_flag = false;
    let mut avc_3d_extension_flag = false;
    let mut svc_extension = NALUHeaderSVCExtension::new();
    let mut avc_3d_extension = NALUHeader3DAVCExtension::new();
    let mut mvc_extension = NALUHeaderMVCExtension::new();

    if nal_unit_type == 14 || nal_unit_type == 20 || nal_unit_type == 21 {
        if nal_unit_type != 21 {
            svc_extension_flag = 1 == nalu_data.read_bits(1); // u(1)
            decoder_formatted_print(
                "NALU Extension: svc_extension_flag",
                &svc_extension_flag,
                63,
            );
        } else {
            avc_3d_extension_flag = 1 == nalu_data.read_bits(1); // u(1)
            decoder_formatted_print(
                "NALU Extension: avc_3d_extension_flag",
                &avc_3d_extension_flag,
                63,
            );
        }

        if svc_extension_flag {
            // specified in Annex G
            svc_extension = decode_nal_unit_header_svc_extension(nalu_data);
        } else if avc_3d_extension_flag {
            // specified in Annex J
            avc_3d_extension = decode_nal_unit_header_3davc_extension(nalu_data);
        } else {
            // specified in Annex H
            mvc_extension = decode_nal_unit_header_mvc_extension(nalu_data);
        }
    }

    NALUheader {
        forbidden_zero_bit,
        nal_ref_idc,
        nal_unit_type,
        svc_extension_flag,
        svc_extension,
        avc_3d_extension_flag,
        avc_3d_extension,
        mvc_extension,
    }
}

fn decode_nal_unit_header_svc_extension(bs: &mut ByteStream) -> NALUHeaderSVCExtension {
    let idr_flag = 1 == bs.read_bits(1);
    decoder_formatted_print("NALU SVC Extension: idr_flag", &idr_flag, 63);
    let priority_id = bs.read_bits(6) as u8;
    decoder_formatted_print("NALU SVC Extension: priority_id", &priority_id, 63);
    let no_inter_layer_pred_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "NALU SVC Extension: no_inter_layer_pred_flag",
        &no_inter_layer_pred_flag,
        63,
    );
    let dependency_id = bs.read_bits(3) as u8;
    decoder_formatted_print("NALU SVC Extension: dependency_id", &dependency_id, 63);
    let quality_id = bs.read_bits(4) as u8;
    decoder_formatted_print("NALU SVC Extension: quality_id", &quality_id, 63);
    let temporal_id = bs.read_bits(3) as u8;
    decoder_formatted_print("NALU SVC Extension: temporal_id", &temporal_id, 63);
    let use_ref_base_pic_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "NALU SVC Extension: use_ref_base_pic_flag",
        &use_ref_base_pic_flag,
        63,
    );
    let discardable_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "NALU SVC Extension: discardable_flag",
        &discardable_flag,
        63,
    );
    let output_flag = 1 == bs.read_bits(1);
    decoder_formatted_print("NALU SVC Extension: output_flag", &output_flag, 63);
    let reserved_three_2bits = bs.read_bits(2) as u8;
    decoder_formatted_print(
        "NALU SVC Extension: reserved_three_2bits",
        &reserved_three_2bits,
        63,
    );

    NALUHeaderSVCExtension {
        idr_flag,
        priority_id,
        no_inter_layer_pred_flag,
        dependency_id,
        quality_id,
        temporal_id,
        use_ref_base_pic_flag,
        discardable_flag,
        output_flag,
        reserved_three_2bits,
    }
}

fn decode_nal_unit_header_3davc_extension(bs: &mut ByteStream) -> NALUHeader3DAVCExtension {
    let view_idx = bs.read_bits(8) as u8;
    decoder_formatted_print("NALU 3DAVC Extension: view_idx", &view_idx, 63);
    let depth_flag = 1 == bs.read_bits(1);
    decoder_formatted_print("NALU 3DAVC Extension: depth_flag", &depth_flag, 63);
    let non_idr_flag = 1 == bs.read_bits(1);
    decoder_formatted_print("NALU 3DAVC Extension: non_idr_flag", &non_idr_flag, 63);
    let temporal_id = bs.read_bits(3) as u8;
    decoder_formatted_print("NALU 3DAVC Extension: temporal_id", &temporal_id, 63);
    let anchor_pic_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "NALU 3DAVC Extension: anchor_pic_flag",
        &anchor_pic_flag,
        63,
    );
    let inter_view_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "NALU 3DAVC Extension: inter_view_flag",
        &inter_view_flag,
        63,
    );

    NALUHeader3DAVCExtension {
        view_idx,
        depth_flag,
        non_idr_flag,
        temporal_id,
        anchor_pic_flag,
        inter_view_flag,
    }
}

/// Described in H.7.3.1.1 NAL unit header MVC extension syntax
fn decode_nal_unit_header_mvc_extension(bs: &mut ByteStream) -> NALUHeaderMVCExtension {
    let non_idr_flag = 1 == bs.read_bits(1);
    decoder_formatted_print("NALU MVC Extension: non_idr_flag", &non_idr_flag, 63);
    let priority_id = bs.read_bits(6) as u8;
    decoder_formatted_print("NALU MVC Extension: priority_id", &priority_id, 63);
    let view_id = bs.read_bits(10);
    decoder_formatted_print("NALU MVC Extension: view_id", &view_id, 63);
    let temporal_id = bs.read_bits(3) as u8;
    decoder_formatted_print("NALU MVC Extension: temporal_id", &temporal_id, 63);
    let anchor_pic_flag = 1 == bs.read_bits(1);
    decoder_formatted_print("NALU MVC Extension: anchor_pic_flag", &anchor_pic_flag, 63);
    let inter_view_flag = 1 == bs.read_bits(1);
    decoder_formatted_print("NALU MVC Extension: inter_view_flag", &inter_view_flag, 63);
    let reserved_one_bit = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "NALU MVC Extension: reserved_one_bit",
        &reserved_one_bit,
        63,
    );

    NALUHeaderMVCExtension {
        non_idr_flag,
        priority_id,
        view_id,
        temporal_id,
        anchor_pic_flag,
        inter_view_flag,
        reserved_one_bit,
    }
}

/// Described in G.7.3.2.12.1 Prefix NAL unit SVC syntax
pub fn decode_prefix_nal_unit_svc(nh: NALUheader, bs: &mut ByteStream) -> PrefixNALU {
    let mut res = PrefixNALU::new();

    if nh.nal_ref_idc != 0 {
        res.store_ref_base_pic_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "Prefix NALU: store_ref_base_pic_flag",
            &res.store_ref_base_pic_flag,
            63,
        );
        if (res.store_ref_base_pic_flag || nh.svc_extension.use_ref_base_pic_flag)
            && !nh.svc_extension.idr_flag
        {
            dec_ref_base_pic_marking(&mut res.ref_base_pic_marking, bs);
        }
        res.additional_prefix_nal_unit_extension_flag = 1 == bs.read_bits(1);
        decoder_formatted_print(
            "Prefix NALU: additional_prefix_nal_unit_extension_flag",
            &res.additional_prefix_nal_unit_extension_flag,
            63,
        );
        if res.additional_prefix_nal_unit_extension_flag {
            let i = 0;
            while bs.more_data() {
                res.additional_prefix_nal_unit_extension_data_flag
                    .push(1 == bs.read_bits(1));
                decoder_formatted_print(
                    "Prefix NALU: additional_prefix_nal_unit_extension_data_flag",
                    &res.additional_prefix_nal_unit_extension_data_flag[i],
                    63,
                );
            }
        }
    } else if bs.more_data() {
        let i = 0;
        while bs.more_data() {
            res.additional_prefix_nal_unit_extension_data_flag
                .push(1 == bs.read_bits(1));
            decoder_formatted_print(
                "Prefix NALU: additional_prefix_nal_unit_extension_data_flag",
                &res.additional_prefix_nal_unit_extension_data_flag[i],
                63,
            );
        }
    }

    res
}

/// Described in G.7.3.3.5 Decoded reference base picture marking syntax
pub fn dec_ref_base_pic_marking(res: &mut RefBasePicMarking, bs: &mut ByteStream) {
    res.adaptive_ref_base_pic_marking_mode_flag = 1 == bs.read_bits(1);
    decoder_formatted_print(
        "RefBasePicMarking: adaptive_ref_base_pic_marking_mode_flag",
        &res.adaptive_ref_base_pic_marking_mode_flag,
        63,
    );
    if res.adaptive_ref_base_pic_marking_mode_flag {
        let mut i = 0;
        loop {
            res.memory_management_base_control_operation
                .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
            decoder_formatted_print(
                "RefBasePicMarking: memory_management_base_control_operation",
                &res.memory_management_base_control_operation[i],
                63,
            );

            if res.memory_management_base_control_operation[i] == 1 {
                res.difference_of_base_pic_nums_minus1
                    .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                decoder_formatted_print(
                    "RefBasePicMarking: difference_of_base_pic_nums_minus1",
                    &res.difference_of_base_pic_nums_minus1[i],
                    63,
                );
            } else {
                res.difference_of_base_pic_nums_minus1.push(0);
            }

            if res.memory_management_base_control_operation[i] == 2 {
                res.long_term_base_pic_num
                    .push(exp_golomb_decode_one_wrapper(bs, false, 0) as u32);
                decoder_formatted_print(
                    "RefBasePicMarking: long_term_base_pic_num",
                    &res.long_term_base_pic_num[i],
                    63,
                );
            } else {
                res.long_term_base_pic_num.push(0);
            }

            if res.memory_management_base_control_operation[i] == 0 {
                break;
            }

            i += 1;
        }
    }
}

/// Described in 7.3.2.4 Access unit delimiter syntax
pub fn decode_access_unit_delimiter(bs: &mut ByteStream) -> AccessUnitDelim {
    let mut aud = AccessUnitDelim::new();

    aud.primary_pic_type = bs.read_bits(3) as u8;
    decoder_formatted_print("AUD: primary_pic_type", &aud.primary_pic_type, 63);

    aud
}
