use crate::common::data_structures::NALUheader;
use crate::common::data_structures::StapA;
use crate::common::data_structures::StapB;
use crate::common::data_structures::Mtap16;
use crate::common::data_structures::Mtap24;
use crate::encoder::binarization_functions::generate_fixed_length_value;
use crate::encoder::safestart::get_rtp_safe_video;
use std::fs::File;
use std::io::prelude::*;


pub const FRAGMENT_SIZE: usize = 1400;

/// Save encoded stream to RTP dump
pub fn save_rtp_file(rtp_filename: String, rtp_nal: &Vec<Vec<u8>>, enable_safestart: bool) {
    println!("   Writing RTP output: {}", &rtp_filename);

    // Stage 1: NAL to packet (single packet mode for now)

    let mut packets: Vec<Vec<u8>> = Vec::new();
    let mut rtp_nal_mod: Vec<Vec<u8>> = Vec::new();
    let mut seq_num: u16 = 0x1234;
    let mut timestamp: u32 = 0x11223344;
    let ssrc: u32 = 0x77777777;
    if enable_safestart {
        rtp_nal_mod.extend(get_rtp_safe_video());
    }

    rtp_nal_mod.extend(rtp_nal.clone());
    for i in 0..rtp_nal_mod.len() {
        let header_byte = 0x80; // version 2, no padding, no extensions, no CSRC, marker = false;
        let nal_type = rtp_nal_mod[i][0] & 0x1f;

        packets.push(Vec::new());
        packets[i].push(header_byte);
        let mut payload_type = 104; // common H.264
        if nal_type == 5 {
            payload_type = payload_type + 0x80; // add marker
        }
        if nal_type == 1 {
            payload_type = payload_type + 0x80; // add marker
            timestamp += 3000;
        }
        if nal_type == 28 {
            let inner_nal_type = rtp_nal_mod[i][1] & 0x1f;
            if inner_nal_type == 5 {
                payload_type = payload_type + 0x80; // add marker
            }
            if inner_nal_type == 1 {
                payload_type = payload_type + 0x80; // add marker
                if (rtp_nal_mod[i][1] & 0x80) != 0 {
                    timestamp += 3000;
                }
            }
        }
        packets[i].push(payload_type);
        packets[i].extend(seq_num.to_be_bytes());
        seq_num += 1;
        packets[i].extend(timestamp.to_be_bytes());
        packets[i].extend(ssrc.to_be_bytes());
        packets[i].extend(rtp_nal_mod[i].iter());
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


/// Encode a Single-Time Aggregation Unit without DON (STAP-A)
///
///   0                   1                   2                   3
///   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                          RTP Header                           |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |STAP-A NAL HDR |         NALU 1 Size           | NALU 1 HDR    |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                         NALU 1 Data                           |
///   :                                                               :
///   +               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |               | NALU 2 Size                   | NALU 2 HDR    |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                         NALU 2 Data                           |
///   :                                                               :
///   |                               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                               :...OPTIONAL RTP padding        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///
///   Figure 7.  An example of an RTP packet including an STAP-A
///              containing two single-time aggregation units
#[allow(dead_code)]
pub fn encode_stap_a(p : StapA) -> Vec<u8> {
    let mut res = Vec::new();
    for i in 0..p.nalus.len() {
        res.extend(generate_fixed_length_value(p.nalu_sizes[i] as u32, 16));
        res.extend(p.nalus[i].content.iter());
    }

    return res;
}

/// Encode a Single-Time Aggregation Unit with DON (STAP-B)
///
///   0                   1                   2                   3
///   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                          RTP Header                           |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |STAP-B NAL HDR | DON                           | NALU 1 Size   |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   | NALU 1 Size   | NALU 1 HDR    | NALU 1 Data                   |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+                               +
///   :                                                               :
///   +               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |               | NALU 2 Size                   | NALU 2 HDR    |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                       NALU 2 Data                             |
///   :                                                               :
///   |                               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                               :...OPTIONAL RTP padding        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#[allow(dead_code)]
pub fn encode_stap_b(_p : StapB) {
    // Encode a Decoding Order Number (DON) 16 bits long
    // while more_data() {
    //   Encode a NAL unit size that is 16 bits
    //   Encode a NALU of set size
    // }
}


/// Encode a Multi-Time Aggregation Packet (MTAP) with 16-bit timestamp offset (TS)
///
///   0                   1                   2                   3
///   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                          RTP Header                           |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |MTAP16 NAL HDR |  decoding order number base   | NALU 1 Size   |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |  NALU 1 Size  |  NALU 1 DOND  |       NALU 1 TS offset        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |  NALU 1 HDR   |  NALU 1 DATA                                  |
///   +-+-+-+-+-+-+-+-+                                               +
///   :                                                               :
///   +               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |               | NALU 2 SIZE                   |  NALU 2 DOND  |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |       NALU 2 TS offset        |  NALU 2 HDR   |  NALU 2 DATA  |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+               |
///   :                                                               :
///   |                               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                               :...OPTIONAL RTP padding        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
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
///
///   0                   1                   2                   3
///   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                          RTP Header                           |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |MTAP24 NAL HDR |  decoding order number base   | NALU 1 Size   |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |  NALU 1 Size  |  NALU 1 DOND  |       NALU 1 TS offs          |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |NALU 1 TS offs |  NALU 1 HDR   |  NALU 1 DATA                  |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+                               +
///   :                                                               :
///   +               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |               | NALU 2 SIZE                   |  NALU 2 DOND  |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |       NALU 2 TS offset                        |  NALU 2 HDR   |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |  NALU 2 DATA                                                  |
///   :                                                               :
///   |                               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                               :...OPTIONAL RTP padding        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
#[allow(dead_code)]
pub fn encode_mtap24(_p : Mtap24) {
    // While more_data() {
    //   Encode a NALU Size that is 16 bits
    //   Encode a Decoding Order Number Difference (DOND) that is 8-bits
    //   Encode a 24-bit Timestamp Offset
    //   Encode a NALU of nalu size
    // }
}

/// Encapsulate a Fragmentation Unit (FU) without a DON (FU-A)
///   0                   1
///   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   | FU indicator  |   FU header   |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
pub fn encapsulate_fu_a(nal : &Vec<u8>, nh : &NALUheader) -> Vec<Vec<u8>> {
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

    // TODO: allow empty FUs

    return res;
}

/// Encodes a Fragmentation Unit (FU) with a DON (FU-B)
///
///   0                   1                   2                   3
///   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   | FU indicator  |   FU header   |               DON             |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-|
///   |                                                               |
///   |                         FU payload                            |
///   |                                                               |
///   |                               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                               :...OPTIONAL RTP padding        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///
/// NOTE: uses the same DON for each FU atm
#[allow(dead_code)]
pub fn encapsulate_fu_b(nal : &Vec<u8>, nh : &NALUheader, don : u16) -> Vec<Vec<u8>> {
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
