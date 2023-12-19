use crate::common::data_structures::AVCCFormat;
use std::fs::File;
use std::io::prelude::*;
pub enum AvccMode {
    AvccNalu,
    AvccSps,
    AvccPps,
}


/// Creates the AVCC header which contains version bits, and all SPSes & PPSes
/// Reference: https://stackoverflow.com/a/24890903/8169613
fn encode_avcc_extradata(avcc_encoding: &AVCCFormat) -> Vec<u8> {
    /*
        bits
        8   version ( always 0x01 )
        8   avc profile ( sps[0][1] )
        8   avc compatibility ( sps[0][2] )
        8   avc level ( sps[0][3] )
        6   reserved ( all bits on )
        2   NALULengthSizeMinusOne (this is how many bytes to use for the length element of a NALU)
        3   reserved ( all bits on )
        5   number of SPS NALUs (usually 1)

        repeated once per SPS:
        16         SPS size
        variable   SPS NALU data

        8   number of PPS NALUs (usually 1)

        repeated once per PPS:
        16       PPS size
        variable PPS NALU data
    */

    let mut avcc_extradata: Vec<u8> = vec![1, avcc_encoding.initial_sps.profile_idc]; // version and avc profile

    // collect the constraint_set flags
    let compatibility = ((match avcc_encoding.initial_sps.constraint_set0_flag {
        true => 1,
        false => 0,
    }) << 7)
        | ((match avcc_encoding.initial_sps.constraint_set1_flag {
            true => 1,
            false => 0,
        }) << 6)
        | ((match avcc_encoding.initial_sps.constraint_set2_flag {
            true => 1,
            false => 0,
        }) << 5)
        | ((match avcc_encoding.initial_sps.constraint_set3_flag {
            true => 1,
            false => 0,
        }) << 4)
        | ((match avcc_encoding.initial_sps.constraint_set4_flag {
            true => 1,
            false => 0,
        }) << 3)
        | ((match avcc_encoding.initial_sps.constraint_set5_flag {
            true => 1,
            false => 0,
        }) << 2);
    avcc_extradata.push(compatibility); // compatibility
    avcc_extradata.push(avcc_encoding.initial_sps.level_idc); // avc level

    avcc_extradata.push(0xff); // 6 bits are on; the last two bits are how many length bytes there are before each NALU (in our case, we'll use 4)

    // number of SPSes
    if avcc_encoding.sps_list.len() > 31 {
        println!(
            "[WARNING] AVCC can only support at most 5 bits worth of SPSes, there may be an error"
        );
    }
    avcc_extradata.push(0xe0 | (0x1F & (avcc_encoding.sps_list.len() as u8))); // the top 3 reserved bits + 5 bits of length

    // for each SPS, 16 bits of SPS size and the amount of bytes
    for s in &avcc_encoding.sps_list {
        if s.len() > 0xffff {
            println!("[WARNING] AVCC can only support 65,536 bytes of SPS; There will be an issue parsing")
        }

        let first_byte = (s.len() & 0xff00) >> 8;
        let second_byte = s.len() & 0xff;

        avcc_extradata.push(first_byte as u8);
        avcc_extradata.push(second_byte as u8);

        avcc_extradata.extend(s);
    }

    // number of PPSes
    if avcc_encoding.pps_list.len() > 255 {
        println!(
            "[WARNING] AVCC can only support at most 8 bits worth of PPSes, there may be an error"
        );
    }
    avcc_extradata.push(avcc_encoding.pps_list.len() as u8);

    // for each PPS, 16 bits of PPS size and then the bytes
    for p in &avcc_encoding.pps_list {
        if p.len() > 0xffff {
            println!("[WARNING] AVCC can only support 65,536 bytes of PPS; There will be an issue parsing")
        }

        let first_byte = (p.len() & 0xff00) >> 8;
        let second_byte = p.len() & 0xff;

        avcc_extradata.push(first_byte as u8);
        avcc_extradata.push(second_byte as u8);

        avcc_extradata.extend(p);
    }

    avcc_extradata
}

/// Encoded Slice data into AVCC format, which is prepended by its length, rather than a start code
fn encode_avcc_data(slice_list: Vec<Vec<u8>>) -> Vec<u8> {
    let mut encoded: Vec<u8> = Vec::new();

    for sl in slice_list {
        // we use the default of 4 bytes to indicate NALU length
        let first_byte = (sl.len() & 0xff000000) >> 24;
        let second_byte = (sl.len() & 0x00ff0000) >> 16;
        let third_byte = (sl.len() & 0x0000ff00) >> 8;
        let fourth_byte = sl.len() & 0x000000ff;

        encoded.push(first_byte as u8);
        encoded.push(second_byte as u8);
        encoded.push(third_byte as u8);
        encoded.push(fourth_byte as u8);

        encoded.extend(sl);
    }

    encoded
}

/// Save an encoded video in AVCC format that is playable by WebCodecs
pub fn save_avcc_file(avcc_encoding: AVCCFormat, filename: &str) {
    let avcc_extradata = encode_avcc_extradata(&avcc_encoding);
    let avcc_data = encode_avcc_data(avcc_encoding.nalus);

    let mut avcc_extradata_filename: String = filename.to_owned();
    avcc_extradata_filename.push_str(".avcc.js");

    let mut avcc_data_filename: String = filename.to_owned();
    avcc_data_filename.push_str(".avcc.264");

    println!(
        "   Writing AVCC file output: \n\t Extradata: {}\n\t AVCC Data: {}",
        avcc_extradata_filename, avcc_data_filename
    );
    println!("   NOTE: AVCC data may not be playable (e.g. SPS or PPS IDs non-incrementing)");

    let hex_avcc_extradata =
        avcc_extradata
            .iter()
            .enumerate()
            .fold(String::new(), |mut acc, (i, _)| {
                if i < (avcc_extradata.len() - 1) {
                    acc.push_str(&format!("0x{}, ", hex::encode(vec![avcc_extradata[i]])))
                } else {
                    acc.push_str(&format!("0x{}", hex::encode(vec![avcc_extradata[i]])))
                }

                if i % 16 == 0 && i != 0 {
                    acc.push_str("\n\t\t")
                }
                acc
            });

    // var allows us to overwrite if dynamically loading in the browser
    let complete_string = format!(
        "var avcC = new Uint8Array(\n\t[\n\t\t{}\n]);",
        hex_avcc_extradata
    );

    let mut avcc_f_ed = match File::create(&avcc_extradata_filename) {
        Err(_) => panic!("couldn't open {}", avcc_extradata_filename),
        Ok(file) => file,
    };

    match avcc_f_ed.write_all(complete_string.as_bytes()) {
        Err(_) => panic!("couldn't write to file {}", avcc_extradata_filename),
        Ok(()) => (),
    };

    let mut avcc_f_d = match File::create(&avcc_data_filename) {
        Err(_) => panic!("couldn't open {}", avcc_data_filename),
        Ok(file) => file,
    };

    match avcc_f_d.write_all(avcc_data.as_slice()) {
        Err(_) => panic!("couldn't write to file {}", avcc_data_filename),
        Ok(()) => (),
    };
}