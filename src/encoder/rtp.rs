//! RTP encoding and saving.

use crate::common::data_structures::NALUheader;
use crate::common::data_structures::RTPAggregationState;
use crate::common::data_structures::RTPOptions;
use crate::common::data_structures::StapB;
use crate::common::data_structures::Mtap16;
use crate::common::data_structures::Mtap24;
use crate::encoder::binarization_functions::generate_fixed_length_value;
use crate::encoder::safestart::get_rtp_safe_video;
use std::fs::File;
use std::io::prelude::*;

const FRAGMENT_SIZE: usize = 1400;

fn get_rtp_payload_type_and_timestamp(rtp_nal : &Vec<u8>, timestamp : &mut u32) -> u8 {
    let nal_type = rtp_nal[0] & 0x1f;
    let mut payload_type = 104; // common H.264

    match nal_type {
        1 => {
            // Non-IDR Slice
            payload_type = payload_type + 0x80; // add marker
            *timestamp += 3000;
        }
        5 => {
            // IDR Slice
            payload_type = payload_type + 0x80; // add marker
        }
        24 => {
            // STAP-A
            // From the RFC:
            //  For aggregation packets (STAP and MTAP), the marker bit in the RTP
            //  header MUST be set to the value that the marker bit of the last
            //  NAL unit of the aggregation packet would have been if it were
            //  transported in its own RTP packet.
            // TODO
        }
        28 => {
            // FU-A
            let inner_nal_type = rtp_nal[1] & 0x1f;
            if inner_nal_type == 5 {
                payload_type = payload_type + 0x80; // add marker
            }
            if inner_nal_type == 1 {
                payload_type = payload_type + 0x80; // add marker
                if (rtp_nal[1] & 0x80) != 0 {
                    *timestamp += 3000;
                }
            }
        }
        _ => ()
    }

    payload_type
}

pub fn get_packed_rtp(rtp_nal : &Vec<u8>, seq_num : &mut u16, mut timestamp : &mut u32) -> Vec<u8> {
    let ssrc: u32 = 0x77777777;
    let mut packed = Vec::new();
    let header_byte = 0x80; // version 2, no padding, no extensions, no CSRC, marker = false;

    let payload_type = get_rtp_payload_type_and_timestamp(rtp_nal, &mut timestamp);
    packed.push(header_byte);
    packed.push(payload_type);
    packed.extend(seq_num.to_be_bytes());
    *seq_num += 1;
    packed.extend(timestamp.to_be_bytes());
    packed.extend(ssrc.to_be_bytes());
    packed.extend(rtp_nal.iter());

    packed
}

/// Save encoded stream to RTP dump
pub fn save_rtp_file(rtp_filename: String, rtp_nal: &Vec<Vec<u8>>, enable_safestart: bool) {
    println!("   Writing RTP output: {}", &rtp_filename);

    // Stage 1: NAL to packet (single packet mode for now)

    let mut packets: Vec<Vec<u8>> = Vec::new();
    let mut rtp_nal_mod: Vec<Vec<u8>> = Vec::new();
    let mut seq_num: u16 = 0x1234;
    let mut timestamp: u32 = 0x11223344;
    if enable_safestart {
        rtp_nal_mod.extend(get_rtp_safe_video());
    }

    rtp_nal_mod.extend(rtp_nal.clone());
    for nal in rtp_nal_mod {
        packets.push(get_packed_rtp(&nal, &mut seq_num, &mut timestamp));
    }

    let mut out_bytes: Vec<u8> = Vec::new();
    let s = "#!rtpplay1.0 127.0.0.1/48888\n";
    let header = s.bytes();
    out_bytes.extend(header);

    let start_sec: u32 = 0;
    let start_usec: u32 = 0;
    let source: u32 = 0;
    let port: u16 = 0;
    let padding: u16 = 0;

    out_bytes.extend(start_sec.to_be_bytes());
    out_bytes.extend(start_usec.to_be_bytes());
    out_bytes.extend(source.to_be_bytes());
    out_bytes.extend(port.to_be_bytes());
    out_bytes.extend(padding.to_be_bytes());

    for i in 0..packets.len() {
        let plen: u16 = packets[i].len().try_into().unwrap();
        let blen: u16 = plen + 8;
        let ts: u32 = 0;

        out_bytes.extend(blen.to_be_bytes());
        out_bytes.extend(plen.to_be_bytes());
        out_bytes.extend(ts.to_be_bytes());
        out_bytes.extend(packets[i].iter());
    }

    let mut f = match File::create(&rtp_filename) {
        Err(_) => panic!("couldn't open {}", &rtp_filename),
        Ok(file) => file,
    };

    match f.write_all(out_bytes.as_slice()) {
        Err(_) => panic!("couldn't write to file {}", &rtp_filename),
        Ok(()) => (),
    };
}

pub fn encapsulate_rtp_nalu(nalu : Vec<u8>, nh: &NALUheader, silent_mode : bool, rtp_options: &RTPOptions) -> Vec<Vec<u8>> {
    let mut res = Vec::new();

    if rtp_options.packetization_mode == 0 { // Single NALU mode
        res.push(nalu);
    } else if rtp_options.packetization_mode == 1 { // Non-Interleaved Mode, using NALU, STAP-A, and FU-A
        match rtp_options.aggregation_state {
            RTPAggregationState::None => { // No aggregation NALUs
                // fragment if too large
                if nalu.len() > FRAGMENT_SIZE {
                    if !silent_mode {
                        println!(
                            "Fragmenting {} type {}",
                            nalu.len(),
                            nh.nal_unit_type
                        );
                    }
                    res.extend(encode_fu_a(&nalu, nh));
                } else {
                    res.push(nalu);
                }
            },
            RTPAggregationState::Append => {
                // Append to a STAP-A header
                if !silent_mode {
                    println!(
                        "Appending to STAP-A NALU with {}",
                        nh.nal_unit_type
                    );
                }
                res.push(append_stap_a(&nalu));
            },
            _ => panic!("encapsulate_rtp_nalu - bad RTPAggregationState") // Shouldn't get here
        }
    } else { // packetization_mode == 2, Interleaved Mode, can use STAP-B, MTAP16, MTAP24, FU-A, and FU-B
        res.push(nalu);
    }

    res
}

// Packetization Mode = 1 (Non-Interleaved Mode)

/// Encode a Single-Time Aggregation Unit (STAP-A)
pub fn append_stap_a(nalu : &Vec<u8>) -> Vec<u8> {
    let mut stap_a_bytes: Vec<u8> = Vec::new();
    let nal_size = (nalu.len() as u16).to_be_bytes();
    stap_a_bytes.extend(nal_size.iter());
    stap_a_bytes.extend(nalu.iter());

    stap_a_bytes
}

/// Encode a Fragmentation Unit (FU) without a DON (FU-A)
fn encode_fu_a(nal : &Vec<u8>, nh : &NALUheader) -> Vec<Vec<u8>> {
    let mut res = Vec::new();

    // 28 is FU-A
    let fu_indicator: u8 = 28 | nh.forbidden_zero_bit << 7 | (nh.nal_ref_idc << 5);
    let fua_chunks = nal.chunks(FRAGMENT_SIZE);
    let last_idx = fua_chunks.len() - 1;
    for (i, chunk) in fua_chunks.enumerate() {
        let mut fua_bytes: Vec<u8> = Vec::new();
        fua_bytes.push(fu_indicator);

        // Encode FU header
        // +---------------+
        // |0|1|2|3|4|5|6|7|
        // +-+-+-+-+-+-+-+-+
        // |S|E|R|  Type   |
        // +---------------+
        //  S: Start bit indicating the start of an FU
        //  E: End bit indicating the end of an FU
        //  R: Reserved, please set to 0
        //  Type: NALU Payload type
        let mut fu_header = nh.nal_unit_type;
        if i == 0 {
            fu_header |= 0x80; // S = 1
        }
        if i == last_idx {
            fu_header |= 0x40; // E = 1
        }
        fua_bytes.push(fu_header);
        fua_bytes.extend(chunk);
        res.push(fua_bytes);
    }
    res
}

// Packetization Mode 2 (Interleaved Mode)

/// Encode a Single-Time Aggregation Unit with DON (STAP-B)
#[allow(dead_code)]
pub fn encode_stap_b(_p : StapB) {
    // Encode a Decoding Order Number (DON) 16 bits long
    // while more_data() {
    //   Encode a NAL unit size that is 16 bits
    //   Encode a NALU of set size
    // }
}


/// Encode a Multi-Time Aggregation Packet (MTAP) with 16-bit timestamp offset (TS)
#[allow(dead_code)]
pub fn encode_mtap16(_p : Mtap16) {
    // While more_data() {
    //   Encode a NALU Size that is 16 bits
    //   Encode a Decoding Order Number Difference (DOND) that is 8-bits
    //   Encode a 16-bit Timestamp Offset
    //   Encode a NALU of nalu size
    // }
}

/// Encode a Multi-Time Aggregation Packet (MTAP) with 24-bit timestamp offset (TS)
#[allow(dead_code)]
pub fn encode_mtap24(_p : Mtap24, nh: & NALUheader) -> Vec<u8> {
    let mut res = Vec::new();
    // 27 is MTAP24
    let mtap24_hdr: u8 = 27 | nh.forbidden_zero_bit << 7 | (nh.nal_ref_idc << 5);

    res.push(mtap24_hdr);

    // While more_data() {
    //   Encode a NALU Size that is 16 bits
    //   Encode a Decoding Order Number Difference (DOND) that is 8-bits
    //   Encode a 24-bit Timestamp Offset
    //   Encode a NALU of nalu size
    // }

    res
}

/// Encode a Fragmentation Unit (FU) with a DON (FU-B)
///
/// NOTE: uses the same DON for each FU atm
#[allow(dead_code)]
fn encode_fu_b(nal : &Vec<u8>, nh : &NALUheader, don : u16) -> Vec<Vec<u8>> {
    let mut res = Vec::new();

    // 29 is FU-B
    let fu_indicator: u8 = 29 | nh.forbidden_zero_bit << 7 | (nh.nal_ref_idc << 5);
    let encoded_don = generate_fixed_length_value(don as u32, 16);
    let fub_chunks = nal.chunks(FRAGMENT_SIZE);
    let last_idx = fub_chunks.len() - 1;
    for (i, chunk) in fub_chunks.enumerate() {
        let mut fub_bytes: Vec<u8> = Vec::new();
        fub_bytes.push(fu_indicator);

        // Encode FU header
        // +---------------+
        // |0|1|2|3|4|5|6|7|
        // +-+-+-+-+-+-+-+-+
        // |S|E|R|  Type   |
        // +---------------+
        //  S: Start bit indicating the start of an FU
        //  E: End bit indicating the end of an FU
        //  R: Reserved, please set to 0
        //  Type: NALU Payload type
        let mut fu_header = nh.nal_unit_type;
        if i == 0 {
            fu_header |= 0x80; // S = 1
        }
        if i == last_idx {
            fu_header |= 0x40; // E = 1
        }
        fub_bytes.push(fu_header);
        // Encode DON
        fub_bytes.extend(encoded_don.iter());
        // Add the rest of the payload
        fub_bytes.extend(chunk);
        res.push(fub_bytes);
    }

    // TODO: allow empty FUs

    return res;
}
