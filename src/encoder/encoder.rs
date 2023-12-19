//! Encoder entropy point.

use crate::common::data_structures::AVCCFormat;
use crate::common::data_structures::H264DecodedStream;
use crate::common::data_structures::PicParameterSet;
use crate::common::data_structures::SeqParameterSet;
use crate::common::data_structures::SubsetSPS;
use crate::common::data_structures::VideoParameters;
use crate::encoder::avcc::AvccMode;
use crate::encoder::avcc::save_avcc_file;
use crate::encoder::mp4::save_mp4_file;
use crate::encoder::nalu::encode_access_unit_delimiter;
use crate::encoder::nalu::encode_nalu_header;
use crate::encoder::nalu::encode_prefix_nal_unit_svc;
use crate::encoder::parameter_sets::encode_pps;
use crate::encoder::parameter_sets::encode_sps;
use crate::encoder::parameter_sets::encode_sps_extension;
use crate::encoder::parameter_sets::encode_subset_sps;
use crate::encoder::rtp::encapsulate_rtp_nalu;
use crate::encoder::rtp::save_rtp_file;
use crate::encoder::safestart::prepend_safe_video;
use crate::encoder::safestart::SAFESTART_VIDEO_HEIGHT;
use crate::encoder::safestart::SAFESTART_VIDEO_WIDTH;
use crate::encoder::sei::encode_sei_message;
use crate::encoder::slice::encode_slice;
use crate::encoder::slice::encode_slice_layer_extension_rbsp;
use log::debug;
use std::fs::File;
use std::io::prelude::*;

/// Insert the emulation three byte to an encoded stream
pub fn insert_emulation_three_byte(stream: &[u8]) -> Vec<u8> {
    let mut res: Vec<u8> = Vec::new();

    //let mut epb_count = 0;
    //let mut count = 0;
    //let mut offset_overwrite = 0;

    let mut zero1 = false;
    let mut zero2 = false;
    for &cur_byte in stream {
        //count += 1;
        if zero1 {
            if zero2 {
                if cur_byte == 0 || cur_byte == 1 || cur_byte == 2 || cur_byte == 3 {
                    //epb_count+=1;

                    /*
                    // This snippet is used for targeting with CVE-2022-32939
                    if epb_count == 257 {
                        offset_overwrite = count*8 + 1;
                        // The location of the 257th emulation prevention byte overwrites
                        // the array index. This value is subsequently incremented.
                        println!("[X] Index Overwrite:  0x{:x}", offset_overwrite);
                        debug!(target: "encode","[X] Index Overwrite: 0x{:x}", offset_overwrite);
                    } else if epb_count == 258 {
                        // The location of the 258th emulation prevention byte is
                        // used to calculate the value we write
                        let value_to_write = 8*(count-offset_overwrite+1) + 2048;
                        println!("[X] Write value: {} (0x{:x})", value_to_write, value_to_write);
                        debug!(target: "encode","[X] Write value: {} (0x{:x})", value_to_write, value_to_write);
                    }
                    */
                    res.push(3); // insert emulation 3 byte
                }
                zero1 = false;
                zero2 = false;
            } else if cur_byte == 0 {
                zero2 = true;
            } else {
                zero1 = false;
            }
        }
        // we get rid of the else because of the following case: 0x00 0x00 0x00 0x00 0x00 0x00
        // it should be set to 0x00 0x00 0x03 0x00 0x00 0x03 0x00 0x00
        if cur_byte == 0 {
            zero1 = true;
        }

        res.push(cur_byte);
    }

    //println!("[X] Added {} emulation prevention bytes", epb_count);

    res
}

/// Save the encoded stream as .264, AVCC, MP4 or RTP
pub fn save_encoded_stream(
    annex_b_video: Vec<u8>,
    avcc_video: AVCCFormat,
    filename: &str,
    width: i32,
    height: i32,
    mp4_out: bool,
    is_mp4_fragment: bool,
    is_hevc: bool,
    avcc_out: bool,
    enable_safestart: bool,
    rtp_video: Vec<Vec<u8>>,
) {
    println!("   Writing to {}", filename);
    let mut f = match File::create(filename) {
        Err(_) => panic!("couldn't open {}", filename),
        Ok(file) => file,
    };

    match f.write_all(annex_b_video.as_slice()) {
        Err(_) => panic!("couldn't write to file {}", filename),
        Ok(()) => (),
    };

    if enable_safestart {
        println!("   Adding safestart video at the start of the video");
        let mut safestart_filename: String = filename.to_owned();
        safestart_filename.push_str(".safestart.264");

        let safestart_encoded_str = prepend_safe_video(&annex_b_video);

        println!("   Writing to {}", safestart_filename);
        let mut f = match File::create(&safestart_filename) {
            Err(_) => panic!("couldn't open {}", safestart_filename),
            Ok(file) => file,
        };

        match f.write_all(safestart_encoded_str.as_slice()) {
            Err(_) => panic!("couldn't write to file {}", safestart_filename),
            Ok(()) => (),
        };

        // save the safestart MP4
        if mp4_out {
            let mut mp4_filename: String = filename.to_owned();
            mp4_filename.push_str(".safestart.mp4");
            save_mp4_file(
                mp4_filename,
                SAFESTART_VIDEO_WIDTH,
                SAFESTART_VIDEO_HEIGHT,
                is_mp4_fragment,
                false,
                &safestart_encoded_str,
            );
        }
    }

    if avcc_out {
        save_avcc_file(avcc_video, filename);
    }

    if mp4_out {
        let mut mp4_filename: String = filename.to_owned();
        mp4_filename.push_str(".mp4");
        save_mp4_file(
            mp4_filename,
            width,
            height,
            is_mp4_fragment,
            is_hevc,
            &annex_b_video,
        );
    }

    // save RTP dump
    if rtp_video.len() > 0 {
        println!("Writing RTP dump");
        let mut rtp_filename: String = filename.to_owned();
        rtp_filename.push_str(".rtpdump");
        save_rtp_file(rtp_filename, &rtp_video, enable_safestart);
    }
}

/// Given a H.264 Decoded Stream object, output a correct, emulation prevented, encoded bitstream
pub fn encode_bitstream(
    ds: &mut H264DecodedStream,
    cut_nalu: i32,
    avcc_out: bool,
    silent_mode: bool,
    rtp_out: bool,
) -> (Vec<u8>, AVCCFormat, Vec<Vec<u8>>) {
    let mut annex_b_video: Vec<u8> = Vec::new();
    let mut avcc_video = AVCCFormat::new();
    let mut rtp_video: Vec<Vec<u8>> = Vec::new();

    // NALU type indices
    let mut sps_idx = 0;
    let mut subset_sps_idx = 0;
    let mut sps_extension_idx = 0;
    let mut pps_idx = 0;
    let mut prefix_nalu_idx = 0;
    let mut slice_idx = 0;
    let mut sei_idx = 0;
    let mut aud_idx = 0;

    if avcc_out {
        avcc_video.initial_sps = ds.spses[0].clone();
    }

    debug!(target: "encode","Encoding {} NALUs", ds.nalu_elements.len());

    for i in 0..ds.nalu_elements.len() {
        let mut cur_annex_b_nal: Vec<u8> = Vec::new();
        let mut cur_avcc_nal: Vec<u8> = Vec::new();
        let mut cur_rtp_nal: Vec<u8> = Vec::new();

        // The mode
        let mut avcc_mode = AvccMode::AvccNalu;

        debug!(target: "encode","");
        debug!(target: "encode","Annex B NALU w/ {} startcode, len {}, forbidden_bit {}, nal_reference_idc {}, nal_unit_type {}",
            { if ds.nalu_elements[i].longstartcode {"long" } else {"short"} }, ds.nalu_elements[i].content.len(), ds.nalu_headers[i].forbidden_zero_bit, ds.nalu_headers[i].nal_ref_idc, ds.nalu_headers[i].nal_unit_type);

        if ds.nalu_elements[i].longstartcode {
            cur_annex_b_nal.extend(vec![0, 0, 0, 1]);
        } else {
            cur_annex_b_nal.extend(vec![0, 0, 1]);
        }

        let encoded_header = encode_nalu_header(&ds.nalu_headers[i]);
        cur_annex_b_nal.extend(encoded_header.iter());

        if rtp_out {
            // RTP vid doesn't require start codes
            cur_rtp_nal.extend(encoded_header.iter());
        }
        if avcc_out {
            cur_avcc_nal.extend(encoded_header.iter());
        }
        match ds.nalu_headers[i].nal_unit_type {
            0 => {
                if !silent_mode {
                    println!("\t encode_bitstream - NALU {} - Unknown nal_unit_type of 0 - not affecting encoding process", i);
                }
                cur_annex_b_nal.extend(insert_emulation_three_byte(&ds.nalu_elements[i].content[1..]));

                if rtp_out {
                    cur_rtp_nal.extend(insert_emulation_three_byte(&ds.nalu_elements[i].content[1..]));
                }
                if avcc_out {
                    cur_avcc_nal.extend(insert_emulation_three_byte(&ds.nalu_elements[i].content[1..]));
                }
            }
            1 => {
                if !silent_mode {
                    println!(
                        "\t encode_bitstream - NALU {} - Coded slice of a non-IDR picture",
                        i
                    );
                }

                let associated_pps_id = ds.slices[slice_idx].sh.pic_parameter_set_id;

                let mut cur_pps_wrapper: Option<&PicParameterSet> = None;
                // retrieve the corresponding PPS
                // we search in reverse to get the most recent; ID collision is possible with random video generation
                // we use pps_idx to ensure only already encoded PPS's are used
                for i in (0..pps_idx).rev() {
                    if ds.ppses[i].pic_parameter_set_id == associated_pps_id {
                        cur_pps_wrapper = Some(&ds.ppses[i]);
                        break;
                    }
                }

                let cur_pps: &PicParameterSet;
                match cur_pps_wrapper {
                    Some(x) => cur_pps = x,
                    _ => panic!("encode_bitstream - Associated SPS not found for PPS - associated_sps_idx : {}", associated_pps_id),
                }

                let associated_sps_id = cur_pps.seq_parameter_set_id;
                let mut cur_sps_wrapper: Option<&SeqParameterSet> = None;

                // we search in reverse to get the most recent; ID collision is possible with random video generation
                // we use sps_idx to ensure only already encoded SPS's are used
                if cur_pps.is_subset_pps {
                    // try subset sps
                    for i in (0..subset_sps_idx).rev() {
                        if ds.subset_spses[i].sps.seq_parameter_set_id == associated_sps_id {
                            cur_sps_wrapper = Some(&ds.subset_spses[i].sps);
                            break;
                        }
                    }
                } else {
                    for i in (0..sps_idx).rev() {
                        if ds.spses[i].seq_parameter_set_id == associated_sps_id {
                            cur_sps_wrapper = Some(&ds.spses[i]);
                            break;
                        }
                    }
                }

                let cur_sps: &SeqParameterSet;
                match cur_sps_wrapper {
                    Some(x) => {
                        cur_sps = x;
                    },
                    None => panic!("encode_bitstream - Associated SPS not found for PPS - associated_sps_idx : {}", associated_sps_id),
                }
                let mut vp = VideoParameters::new(&ds.nalu_headers[i], cur_pps, cur_sps);
                // for neighbor macroblock processing
                vp.mbaff_frame_flag = ds.slices[slice_idx].sh.mbaff_frame_flag;

                let res = insert_emulation_three_byte(&encode_slice(
                    &ds.nalu_headers[i],
                    &ds.slices[slice_idx],
                    cur_sps,
                    cur_pps,
                    &vp,
                    silent_mode,
                ));

                cur_annex_b_nal.extend(res.iter());

                if rtp_out {
                    cur_rtp_nal.extend(res.iter());
                }
                if avcc_out {
                    cur_avcc_nal.extend(res.iter());
                }
                slice_idx += 1;
            }
            2 => {
                if !silent_mode {
                    println!(
                        "\t encode_bitstream - NALU {} - Coded slice data partition A",
                        i
                    );
                }
                // TODO: Coded slice data partition A encoding. For now, just append nalu elements
                cur_annex_b_nal.extend(insert_emulation_three_byte(&ds.nalu_elements[i].content[1..]));

                if rtp_out {
                    cur_rtp_nal.extend(insert_emulation_three_byte(&ds.nalu_elements[i].content[1..]));
                }
                if avcc_out {
                    cur_avcc_nal.extend(insert_emulation_three_byte(&ds.nalu_elements[i].content[1..]));
                }
            }
            3 => {
                if !silent_mode {
                    println!(
                        "\t encode_bitstream - NALU {} - Coded slice data partition B",
                        i
                    );
                }
                // TODO: Coded slice data partition B encoding. For now, just append nalu elements
                cur_annex_b_nal.extend(insert_emulation_three_byte(&ds.nalu_elements[i].content[1..]));

                if rtp_out {
                    cur_rtp_nal.extend(insert_emulation_three_byte(&ds.nalu_elements[i].content[1..]));
                }
                if avcc_out {
                    cur_avcc_nal.extend(insert_emulation_three_byte(&ds.nalu_elements[i].content[1..]));
                }
            }
            4 => {
                if !silent_mode {
                    println!(
                        "\t encode_bitstream - NALU {} - Coded slice data partition C",
                        i
                    );
                }
                // TODO: Coded slice data partition C encoding. For now, just append nalu elements
                cur_annex_b_nal.extend(insert_emulation_three_byte(&ds.nalu_elements[i].content[1..]));

                if rtp_out {
                    cur_rtp_nal.extend(insert_emulation_three_byte(&ds.nalu_elements[i].content[1..]));
                }
                if avcc_out {
                    cur_avcc_nal.extend(insert_emulation_three_byte(&ds.nalu_elements[i].content[1..]));
                }
            }
            5 => {
                if !silent_mode {
                    println!(
                        "\t encode_bitstream - NALU {} - Coded slice of an IDR picture",
                        i
                    );
                }

                let associated_pps_id = ds.slices[slice_idx].sh.pic_parameter_set_id;

                let mut cur_pps_wrapper: Option<&PicParameterSet> = None;
                // retrieve the corresponding PPS
                // we search in reverse to get the most recent; ID collision is possible with random video generation
                // we use pps_idx to ensure only already encoded PPS's are used
                for i in (0..pps_idx).rev() {
                    if ds.ppses[i].pic_parameter_set_id == associated_pps_id {
                        cur_pps_wrapper = Some(&ds.ppses[i]);
                        break;
                    }
                }

                let cur_pps: &PicParameterSet;
                match cur_pps_wrapper {
                    Some(x) => cur_pps = x,
                    _ => panic!("encode_bitstream - Associated SPS not found for PPS - associated_sps_idx : {}", associated_pps_id),
                }

                let associated_sps_id = cur_pps.seq_parameter_set_id;
                let mut cur_sps_wrapper: Option<&SeqParameterSet> = None;

                // we search in reverse to get the most recent; ID collision is possible with random video generation
                // we use sps_idx to ensure only already encoded SPS's are used
                if cur_pps.is_subset_pps {
                    // try subset sps
                    for i in (0..subset_sps_idx).rev() {
                        if ds.subset_spses[i].sps.seq_parameter_set_id == associated_sps_id {
                            cur_sps_wrapper = Some(&ds.subset_spses[i].sps);
                            break;
                        }
                    }
                } else {
                    for i in (0..sps_idx).rev() {
                        if ds.spses[i].seq_parameter_set_id == associated_sps_id {
                            cur_sps_wrapper = Some(&ds.spses[i]);
                            break;
                        }
                    }
                }

                let cur_sps: &SeqParameterSet;
                match cur_sps_wrapper {
                    Some(x) => {
                        cur_sps = x;
                    },
                    None => panic!("encode_bitstream - Associated SPS not found for PPS - associated_sps_idx : {}", associated_sps_id),
                }
                let mut vp = VideoParameters::new(&ds.nalu_headers[i], cur_pps, cur_sps);
                vp.mbaff_frame_flag = ds.slices[slice_idx].sh.mbaff_frame_flag;

                let res = insert_emulation_three_byte(&encode_slice(
                    &ds.nalu_headers[i],
                    &ds.slices[slice_idx],
                    cur_sps,
                    cur_pps,
                    &vp,
                    silent_mode,
                ));

                cur_annex_b_nal.extend(res.iter());

                if rtp_out {
                    cur_rtp_nal.extend(res.iter());
                }
                if avcc_out {
                    cur_avcc_nal.extend(res.iter());
                }

                slice_idx += 1;
            }
            6 => {
                if !silent_mode {
                    println!("\t encode_bitstream - NALU {} - Supplemental enhancement information", i);
                }
                // only pass in already encoded SPSes
                let res = encode_sei_message(&ds.seis[sei_idx], &ds.spses[0..sps_idx], silent_mode);

                if res.len() == 0 {
                    debug!(target: "encode","[WARNING] SEI Encoded Payload is empty - copying over NALU bytes");
                    cur_annex_b_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));

                    if rtp_out {
                        cur_rtp_nal.extend(insert_emulation_three_byte(
                            &ds.nalu_elements[i].content[1..],
                        ));
                    }
                    if avcc_out {
                        cur_avcc_nal.extend(insert_emulation_three_byte(
                            &ds.nalu_elements[i].content[1..],
                        ));
                    }
                } else {
                    cur_annex_b_nal.extend(insert_emulation_three_byte(&res));

                    if rtp_out {
                        cur_rtp_nal.extend(insert_emulation_three_byte(&res));
                    }
                    if avcc_out {
                        cur_avcc_nal.extend(insert_emulation_three_byte(&res));
                    }
                }

                sei_idx += 1;
            }
            7 => {
                if !silent_mode {
                    println!(
                        "\t encode_bitstream - NALU {} - Encoding Sequence Parameter Set",
                        i
                    );
                }
                let res = insert_emulation_three_byte(&encode_sps(&ds.spses[sps_idx], false));

                cur_annex_b_nal.extend(res.iter());

                if rtp_out {
                    cur_rtp_nal.extend(res.iter());
                }
                if avcc_out {
                    cur_avcc_nal.extend(res.iter());
                    avcc_mode = AvccMode::AvccSps;
                }

                sps_idx += 1;
            }
            8 => {
                if !silent_mode {
                    println!(
                        "\t encode_bitstream - NALU {} - Encoding Picture Parameter Set",
                        i
                    );
                }

                let mut cur_sps_wrapper: Option<&SeqParameterSet> = None;
                if pps_idx < ds.ppses.len() {
                    let associated_sps_id = ds.ppses[pps_idx].seq_parameter_set_id;
                    if ds.ppses[pps_idx].is_subset_pps {
                        // try subset sps
                        for i in (0..subset_sps_idx).rev() {
                            if ds.subset_spses[i].sps.seq_parameter_set_id == associated_sps_id {
                                cur_sps_wrapper = Some(&ds.subset_spses[i].sps);
                                break;
                            }
                        }
                    } else {
                        // we search in reverse to get the most recent; ID collision is possible with random video generation
                        for i in (0..sps_idx).rev() {
                            if ds.spses[i].seq_parameter_set_id == associated_sps_id {
                                cur_sps_wrapper = Some(&ds.spses[i]);
                                break;
                            }
                        }
                    }

                    let cur_sps: &SeqParameterSet;
                    match cur_sps_wrapper {
                        Some(x) => {
                            cur_sps = x;
                            let res = insert_emulation_three_byte(&encode_pps(
                                &ds.ppses[pps_idx],
                                cur_sps,
                            ));

                            cur_annex_b_nal.extend(res.iter());
                            if rtp_out {
                                cur_rtp_nal.extend(res.iter());
                            }
                            if avcc_out {
                                cur_avcc_nal.extend(res.iter());
                                avcc_mode = AvccMode::AvccPps;
                            }
                            pps_idx += 1;
                        }
                        _ => {
                            // TODO: We could consider not panicking and continuing with a default SPS (e.g., LRU)
                            panic!(
                                "encode_bitstream - SPS or SubsetSPS with id {} not found",
                                associated_sps_id
                            )
                        }
                    }
                }
            }
            9 => {
                if !silent_mode {
                    println!(
                        "\t encode_bitstream - NALU {} - Access unit delimiter",
                        i
                    );
                }

                let res =
                    insert_emulation_three_byte(&encode_access_unit_delimiter(&ds.auds[aud_idx]));

                cur_annex_b_nal.extend(res.iter());

                if rtp_out {
                    cur_rtp_nal.extend(res.iter());
                }
                if avcc_out {
                    cur_avcc_nal.extend(res.iter());
                }

                aud_idx += 1;
            }
            10 => {
                if !silent_mode {
                    println!("\t encode_bitstream - NALU {} - End of Sequence", i);
                }
                // According to 7.3.2.5 there is nothing to parse
                // According to 7.4.2.5 this signals that the next NALU shall be an IDR
                if ds.nalu_elements[i].content.len() > 1 {
                    cur_annex_b_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));

                    if rtp_out {
                        cur_rtp_nal.extend(insert_emulation_three_byte(
                            &ds.nalu_elements[i].content[1..],
                        ));
                    }
                    if avcc_out {
                        cur_avcc_nal.extend(insert_emulation_three_byte(
                            &ds.nalu_elements[i].content[1..],
                        ));
                    }
                }
            }
            11 => {
                if !silent_mode {
                    println!("\t encode_bitstream - NALU {} - End of Stream", i);
                }
                // According to 7.3.2.6 there is nothing to parse
                // According to 7.4.2.6 this signals that there is nothing else to decode, so we could just `break;`
                if ds.nalu_elements[i].content.len() > 1 {
                    cur_annex_b_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));

                    if rtp_out {
                        cur_rtp_nal.extend(insert_emulation_three_byte(
                            &ds.nalu_elements[i].content[1..],
                        ));
                    }
                    if avcc_out {
                        cur_avcc_nal.extend(insert_emulation_three_byte(
                            &ds.nalu_elements[i].content[1..],
                        ));
                    }
                }
            }
            12 => {
                if !silent_mode {
                    println!("\t encode_bitstream - NALU {} - Filler Data", i);
                }
                // According to 7.3.2.7 and 7.4.2.7 this is, as the name describes, filler data
                // that should be all 0xff bytes
                // TODO: implement 7.3.2.7
                //filler_data_rbsp();
                cur_annex_b_nal.extend(insert_emulation_three_byte(
                    &ds.nalu_elements[i].content[1..],
                ));

                if rtp_out {
                    cur_rtp_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));
                }
                if avcc_out {
                    cur_avcc_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));
                }
            }
            13 => {
                if !silent_mode {
                    println!(
                        "\t encode_bitstream - NALU {} - Sequence parameter set extension",
                        i
                    );
                }
                let res = insert_emulation_three_byte(&encode_sps_extension(
                    &ds.sps_extensions[sps_extension_idx],
                ));
                cur_annex_b_nal.extend(res.iter());

                if rtp_out {
                    cur_rtp_nal.extend(res.iter());
                }
                if avcc_out {
                    cur_avcc_nal.extend(res.iter());
                    avcc_mode = AvccMode::AvccSps;
                }

                sps_extension_idx += 1;
            }
            14 => {
                if !silent_mode {
                    println!("\t encode_bitstream - NALU {} - Prefix NAL unit", i);
                }

                if ds.nalu_headers[i].svc_extension_flag {
                    let res = insert_emulation_three_byte(&encode_prefix_nal_unit_svc(
                        &ds.nalu_headers[i],
                        &ds.prefix_nalus[prefix_nalu_idx],
                    ));

                    cur_annex_b_nal.extend(res.iter());

                    if rtp_out {
                        cur_rtp_nal.extend(res.iter());
                    }
                    if avcc_out {
                        cur_avcc_nal.extend(res.iter());
                    }

                    prefix_nalu_idx += 1;
                }
            }
            15 => {
                if !silent_mode {
                    println!(
                        "\t encode_bitstream - NALU {} - Subset sequence parameter set",
                        i
                    );
                }
                let res = insert_emulation_three_byte(&encode_subset_sps(
                    &ds.subset_spses[subset_sps_idx],
                ));
                cur_annex_b_nal.extend(res.iter());

                if rtp_out {
                    cur_rtp_nal.extend(res.iter());
                }
                if avcc_out {
                    cur_avcc_nal.extend(res.iter());
                    avcc_mode = AvccMode::AvccSps;
                }

                subset_sps_idx += 1;
            }
            16 => {
                if !silent_mode {
                    println!(
                        "\t encode_bitstream - NALU {} - Depth parameter set",
                        i
                    );
                }
                // TODO: depth_parameter_set_rbsp();
                cur_annex_b_nal.extend(insert_emulation_three_byte(
                    &ds.nalu_elements[i].content[1..],
                ));

                if rtp_out {
                    cur_rtp_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));
                }
                if avcc_out {
                    cur_avcc_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));
                }
            }
            17..=18 => {
                if !silent_mode {
                    println!("\t encode_bitstream - NALU {} - RESERVED nal_unit_type of {} - Copying Bytes", i, ds.nalu_headers[i].nal_unit_type);
                }
                // Ignore for now
                cur_annex_b_nal.extend(insert_emulation_three_byte(
                    &ds.nalu_elements[i].content[1..],
                ));

                if rtp_out {
                    cur_rtp_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));
                }
                if avcc_out {
                    cur_avcc_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));
                }
            }
            19 => {
                if !silent_mode {
                    println!("\t encode_bitstream - NALU {} - Coded slice of an auxiliary coded picture without partitioning", i);
                }
                // TODO: slice_layer_without_partitioning_rbsp(); // but non-VCL
                cur_annex_b_nal.extend(insert_emulation_three_byte(
                    &ds.nalu_elements[i].content[1..],
                ));

                if rtp_out {
                    cur_rtp_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));
                }
                if avcc_out {
                    cur_avcc_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));
                }
            }
            20 => {
                if !silent_mode {
                    println!(
                        "\t encode_bitstream - NALU {} - Coded slice extension",
                        i
                    );
                }
                let associated_pps_id = ds.slices[slice_idx].sh.pic_parameter_set_id;

                let mut cur_pps_wrapper: Option<&PicParameterSet> = None;
                // retrieve the corresponding PPS
                // we search in reverse to get the most recent; ID collision is possible with random video generation
                for i in (0..pps_idx).rev() {
                    if ds.ppses[i].pic_parameter_set_id == associated_pps_id
                        && ds.ppses[i].is_subset_pps
                    {
                        cur_pps_wrapper = Some(&ds.ppses[i]);
                        break;
                    }
                }

                let cur_pps: &PicParameterSet;
                match cur_pps_wrapper {
                    Some(x) => cur_pps = x,
                    _ =>  panic!("encode_bitstream - Associated PPS not found for coded slice extension - associated_pps_idx : {}", associated_pps_id),
                }

                let associated_sps_id = cur_pps.seq_parameter_set_id;
                let mut cur_subset_sps_wrapper: Option<&SubsetSPS> = None;

                // we search in reverse to get the most recent; ID collision is possible with random video generation
                // we use sps_idx to ensure only already encoded SPS's are used
                for i in (0..subset_sps_idx).rev() {
                    if ds.subset_spses[i].sps.seq_parameter_set_id == associated_sps_id {
                        cur_subset_sps_wrapper = Some(&ds.subset_spses[i]);
                        break;
                    }
                }

                let cur_subset_sps: &SubsetSPS;
                match cur_subset_sps_wrapper {
                    Some(x) => {
                        cur_subset_sps = x;
                    },
                    None => panic!("encode_bitstream - Associated Subset SPS not found for PPS - associated_sps_idx : {}", associated_sps_id),
                }
                let mut vp =
                    VideoParameters::new(&ds.nalu_headers[i], cur_pps, &cur_subset_sps.sps);
                vp.mbaff_frame_flag = ds.slices[slice_idx].sh.mbaff_frame_flag;

                let res = insert_emulation_three_byte(&encode_slice_layer_extension_rbsp(
                    &ds.nalu_headers[i],
                    &ds.slices[slice_idx],
                    cur_subset_sps,
                    cur_pps,
                    &vp,
                    silent_mode,
                ));

                cur_annex_b_nal.extend(res.iter());

                if rtp_out {
                    cur_rtp_nal.extend(res.iter())
                }
                if avcc_out {
                    cur_avcc_nal.extend(res.iter());
                }

                slice_idx += 1;
            }
            21 => {
                if !silent_mode {
                    println!("\t encode_bitstream - NALU {} - Coded slice extension for a depth view component or a 3D-AVC texture view component", i);
                }
                // specified in Annex J
                // slice_layer_extension_rbsp();
                cur_annex_b_nal.extend(insert_emulation_three_byte(
                    &ds.nalu_elements[i].content[1..],
                ));

                if rtp_out {
                    cur_rtp_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));
                }
                if avcc_out {
                    cur_avcc_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));
                }
            }
            22..=23 => {
                if !silent_mode {
                    println!("\t encode_bitstream - NALU {} - RESERVED nal_unit_type of {} - Copying Bytes", i, ds.nalu_headers[i].nal_unit_type);
                }
                cur_annex_b_nal.extend(insert_emulation_three_byte(
                    &ds.nalu_elements[i].content[1..],
                ));

                if rtp_out {
                    cur_rtp_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));
                }
                if avcc_out {
                    cur_avcc_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));
                }
            }
            24..=29 => {
                // The following types are from https://www.ietf.org/rfc/rfc3984.txt and updated in https://datatracker.ietf.org/doc/html/rfc6184
                if !silent_mode {
                    let nalu_type = ds.nalu_headers[i].nal_unit_type;
                    if nalu_type == 24 {
                        // STAP-A    Single-time aggregation packet    5.7.1
                        println!("\t encode_bitstream - NALU {} - {} - RTP STAP-A - Copying Bytes", i, nalu_type);
                    } else if nalu_type == 25 {
                        // STAP-B    Single-time aggregation packet    5.7.1
                        println!("\t encode_bitstream - NALU {} - {} - RTP STAP-B - Copying Bytes", i, nalu_type);
                    } else if nalu_type == 26 {
                        //MTAP16    Multi-time aggregation packet      5.7.2
                        println!("\t encode_bitstream - NALU {} - {} - RTP MTAP16 - Copying Bytes", i, nalu_type);
                    } else if nalu_type == 27 {
                        //MTAP24    Multi-time aggregation packet      5.7.2
                        println!("\t encode_bitstream - NALU {} - {} - RTP MTAP24 - Copying Bytes", i, nalu_type);
                    } else if nalu_type == 28 {
                        //FU-A      Fragmentation unit                 5.8
                        println!("\t encode_bitstream - NALU {} - {} - RTP FU-A - Copying Bytes", i, nalu_type);
                    } else if nalu_type == 29 {
                        //FU-B      Fragmentation unit                 5.8
                        println!("\t encode_bitstream - NALU {} - {} - RTP FU-B - Copying Bytes", i, nalu_type);
                    }
                }
                // Ignore for now
                cur_annex_b_nal.extend(insert_emulation_three_byte(
                    &ds.nalu_elements[i].content[1..],
                ));

                if rtp_out {
                    cur_rtp_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));
                }
                if avcc_out {
                    cur_avcc_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));
                }
            }
            30..=31 => {
                // The following types are from SVC RTP https://datatracker.ietf.org/doc/html/rfc6190
                if !silent_mode {
                    let nalu_type = ds.nalu_headers[i].nal_unit_type;
                    if nalu_type == 30 {
                        // PACSI NAL unit                     4.9
                        println!("\t encode_bitstream - NALU {} - {} - RTP SVC PACSI - Copying Bytes", i, nalu_type);
                    } else if nalu_type == 31 {
                        // This reads a subtype
                        // Type  SubType   NAME
                        // 31     0       reserved                           4.2.1
                        // 31     1       Empty NAL unit                     4.10
                        // 31     2       NI-MTAP                            4.7.1
                        // 31     3-31    reserved                           4.2.1
                        println!("\t encode_bitstream - NALU {} - {} - RTP SVC NALU - Copying Bytes", i, nalu_type);
                    }
                }
                // Ignore for now
                cur_annex_b_nal.extend(insert_emulation_three_byte(
                    &ds.nalu_elements[i].content[1..],
                ));

                if rtp_out {
                    cur_rtp_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));
                }
                if avcc_out {
                    cur_avcc_nal.extend(insert_emulation_three_byte(
                        &ds.nalu_elements[i].content[1..],
                    ));
                }
            }
            _ => panic!(
                "\t encode_bitstream - NALU {} - Unknown nal_unit_type of {}",
                i, ds.nalu_headers[i].nal_unit_type
            ),
        };

        // We skip at the end to properly increment the NALU type idx (e.g., sps_idx, pps_idx, etc.)
        if i as i32 == cut_nalu {
            debug!(target: "encode","");
            debug!(target: "encode","Cutting NALU {}", cut_nalu);
            println!("Cutting above NALU {}", cut_nalu);
            continue;
        } else {
            annex_b_video.extend(cur_annex_b_nal)
        }

        if rtp_out {
            rtp_video.extend(encapsulate_rtp_nalu(cur_rtp_nal, &ds.nalu_headers[i], silent_mode));
        }
        if avcc_out {
            match avcc_mode {
                AvccMode::AvccNalu => avcc_video.nalus.push(cur_avcc_nal),
                AvccMode::AvccPps => avcc_video.pps_list.push(cur_avcc_nal),
                AvccMode::AvccSps => avcc_video.sps_list.push(cur_avcc_nal),
            }
        }
    }

    (annex_b_video, avcc_video, rtp_video)
}
