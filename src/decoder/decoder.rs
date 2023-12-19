//! Decoder entry point.

use crate::common::data_structures::AccessUnitDelim;
use crate::common::data_structures::H264DecodedStream;
use crate::common::data_structures::NALUheader;
use crate::common::data_structures::PicParameterSet;
use crate::common::data_structures::PrefixNALU;
use crate::common::data_structures::SEINalu;
use crate::common::data_structures::SPSExtension;
use crate::common::data_structures::SeqParameterSet;
use crate::common::data_structures::Slice;
use crate::common::data_structures::SubsetSPS;
use crate::common::helper::ByteStream;
use crate::decoder::nalu::decode_access_unit_delimiter;
use crate::decoder::nalu::decode_nalu_header;
use crate::decoder::nalu::decode_prefix_nal_unit_svc;
use crate::decoder::nalu::split_into_nalu;
use crate::decoder::parameter_sets::decode_pic_parameter_set;
use crate::decoder::parameter_sets::decode_seq_parameter_set;
use crate::decoder::parameter_sets::decode_sps_extension;
use crate::decoder::parameter_sets::decode_subset_sps;
use crate::decoder::rtp::decode_fu_a;
use crate::decoder::rtp::decode_fu_b;
use crate::decoder::rtp::decode_stap_a;
use crate::decoder::rtp::decode_stap_b;
use crate::decoder::rtp::decode_mtap16;
use crate::decoder::rtp::decode_mtap24;
use crate::decoder::sei::decode_sei_message;
use crate::decoder::slice::decode_slice_layer_extension_rbsp;
use crate::decoder::slice::decode_slice_layer_without_partitioning_rbsp;
use std::time::SystemTime;

/// Given the bytestream, it returns the decoded syntax elements
pub fn decode_bitstream(
    filename: &str,
    only_headers: bool,
    perf_output: bool,
    decode_strict_fmo: bool,
) -> H264DecodedStream {
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

    let mut nalu_headers: Vec<NALUheader> = Vec::new();
    let mut spses: Vec<SeqParameterSet> = Vec::new();
    let mut subset_spses: Vec<SubsetSPS> = Vec::new();
    let mut sps_extensions: Vec<SPSExtension> = Vec::new();
    let mut ppses: Vec<PicParameterSet> = Vec::new();
    let mut prefix_nalus: Vec<PrefixNALU> = Vec::new();
    let mut seis: Vec<SEINalu> = Vec::new();
    let mut slices: Vec<Slice> = Vec::new();
    let mut auds: Vec<AccessUnitDelim> = Vec::new();

    println!("\tFound {:?} NALUs", nalu_elements.len());

    for (i, n) in nalu_elements.iter().enumerate() {
        let mut nalu_data = ByteStream::new(n.content.clone());

        let header = decode_nalu_header(n.longstartcode, &mut nalu_data);
        nalu_headers.push(header.clone());

        // Check RTP NALU types which may contain aggregation or fragmentation types

        match header.nal_unit_type {
            0 => {
                println!("\t decode_bitstream - NALU {} - {} - Unknown nal_unit_type of 0 - not affecting decoding process", i,  header.nal_unit_type)
            }
            1 => {
                println!(
                    "\t decode_bitstream - NALU {} - {} - Coded slice of a non-IDR picture",
                    i, header.nal_unit_type
                );
                let start_time = SystemTime::now();
                let slice = decode_slice_layer_without_partitioning_rbsp(
                    &mut nalu_data,
                    &header,
                    &spses,
                    &ppses,
                    only_headers,
                    decode_strict_fmo,
                );
                if perf_output {
                    let duration = start_time.elapsed();
                    match duration {
                        Ok(elapsed) => {
                            println!("[PERF] decode_bitstream - decode_slice_layer_without_partitioning_rbsp - duration: {} ns", elapsed.as_nanos());
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                }
                slices.push(slice);
            }
            2 => {
                println!(
                    "\t decode_bitstream - NALU {} - {} - Coded slice data partition A",
                    i, header.nal_unit_type
                );
            }
            3 => {
                println!(
                    "\t decode_bitstream - NALU {} - {} - Coded slice data partition B",
                    i, header.nal_unit_type
                );
            }
            4 => {
                println!(
                    "\t decode_bitstream - NALU {} - {} - Coded slice data partition C",
                    i, header.nal_unit_type
                );
            }
            5 => {
                println!(
                    "\t decode_bitstream - NALU {} - {} - Coded slice of an IDR picture",
                    i, header.nal_unit_type
                );
                let start_time = SystemTime::now();
                let slice = decode_slice_layer_without_partitioning_rbsp(
                    &mut nalu_data,
                    &header,
                    &spses,
                    &ppses,
                    only_headers,
                    decode_strict_fmo,
                );
                if perf_output {
                    let duration = start_time.elapsed();
                    match duration {
                        Ok(elapsed) => {
                            println!("[PERF] decode_bitstream - decode_slice_layer_without_partitioning_rbsp - duration: {} ns", elapsed.as_nanos());
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                }
                slices.push(slice);
            }
            6 => {
                println!("\t decode_bitstream - NALU {} - {} - Supplemental enhancement information (SEI)", i,  header.nal_unit_type);
                // not all SEI units need SPSes
                seis.push(decode_sei_message(&spses, &mut nalu_data));
            }
            7 => {
                println!(
                    "\t decode_bitstream - NALU {} - {} - Decoding Sequence Parameter Set (SPS)",
                    i, header.nal_unit_type
                );
                let start_time = SystemTime::now();
                let sps = decode_seq_parameter_set(&mut nalu_data);
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
            8 => {
                println!(
                    "\t decode_bitstream - NALU {} - {} - Decoding Picture Parameter Set (PPS)",
                    i, header.nal_unit_type
                );
                let start_time = SystemTime::now();
                let r = decode_pic_parameter_set(&mut nalu_data, &spses, &subset_spses);
                if perf_output {
                    let duration = start_time.elapsed();
                    match duration {
                        Ok(elapsed) => {
                            println!("[PERF] decode_bitstream - decode_pic_parameter_set - duration: {} ns", elapsed.as_nanos());
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                }
                ppses.push(r);
            }
            9 => {
                println!(
                    "\t decode_bitstream - NALU {} - {} - Access unit delimiter (AUD)",
                    i, header.nal_unit_type
                );
                let start_time = SystemTime::now();
                let aud = decode_access_unit_delimiter(&mut nalu_data);
                auds.push(aud);

                if perf_output {
                    let duration = start_time.elapsed();
                    match duration {
                        Ok(elapsed) => {
                            println!("[PERF] decode_bitstream - decode_pic_parameter_set - duration: {} ns", elapsed.as_nanos());
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                }
            }
            10 => {
                println!(
                    "\t decode_bitstream - NALU {} - {} - End of Sequence",
                    i, header.nal_unit_type
                );
                // According to 7.3.2.5 there is nothing to parse
                // According to 7.4.2.5 this signals that the next NALU shall be an IDR
            }
            11 => {
                println!(
                    "\t decode_bitstream - NALU {} - {} - End of Stream",
                    i, header.nal_unit_type
                );
                // According to 7.3.2.6 there is nothing to parse
                // According to 7.4.2.6 this signals that there is nothing else to decode, so technically the decoder could `break;`
            }
            12 => {
                println!(
                    "\t decode_bitstream - NALU {} - {} - Filler Data",
                    i, header.nal_unit_type
                );
                // According to 7.3.2.7 and 7.4.2.7 this is, as the name describes, filler data
                // that should be all 0xff bytes
                // TODO: implement 7.3.2.7
                //filler_data_rbsp();
            }
            13 => {
                println!(
                    "\t decode_bitstream - NALU {} - {} - Sequence parameter set extension",
                    i, header.nal_unit_type
                );
                let start_time = SystemTime::now();
                let sps_ext = decode_sps_extension(&mut nalu_data);
                if perf_output {
                    let duration = start_time.elapsed();
                    match duration {
                        Ok(elapsed) => {
                            println!(
                                "[PERF] decode_bitstream - decode_sps_extension - duration: {} ns",
                                elapsed.as_nanos()
                            );
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                }
                sps_extensions.push(sps_ext);
            }
            14 => {
                println!(
                    "\t decode_bitstream - NALU {} - {} - Prefix NAL unit",
                    i, header.nal_unit_type
                );
                // described in 7.3.2.12
                if header.svc_extension_flag {
                    // described in G.7.3.2.12.1
                    let start_time = SystemTime::now();
                    let prefix_nal = decode_prefix_nal_unit_svc(header, &mut nalu_data);
                    if perf_output {
                        let duration = start_time.elapsed();
                        match duration {
                            Ok(elapsed) => {
                                println!("[PERF] decode_bitstream - decode_prefix_nal_unit_svc - duration: {} ns", elapsed.as_nanos());
                            }
                            Err(e) => {
                                println!("Error: {:?}", e);
                            }
                        }
                    }
                    prefix_nalus.push(prefix_nal);
                }
            }
            15 => {
                println!(
                    "\t decode_bitstream - NALU {} - {} - Subset sequence parameter set",
                    i, header.nal_unit_type
                );
                let start_time = SystemTime::now();
                let sub_sps = decode_subset_sps(&mut nalu_data);
                if perf_output {
                    let duration = start_time.elapsed();
                    match duration {
                        Ok(elapsed) => {
                            println!(
                                "[PERF] decode_bitstream - decode_subset_sps - duration: {} ns",
                                elapsed.as_nanos()
                            );
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                }
                subset_spses.push(sub_sps);
            }
            16 => {
                println!(
                    "\t decode_bitstream - NALU {} - {} -  Depth parameter set",
                    i, header.nal_unit_type
                );
                // TODO: Annex J
                //depth_parameter_set_rbsp();
            }
            17..=18 => println!(
                "\t decode_bitstream - NALU {} - {} - RESERVED nal_unit_type ignoring",
                i, header.nal_unit_type
            ),
            19 => {
                println!("\t decode_bitstream - NALU {} - {} - Coded slice of an auxiliary coded picture without partitioning", i,  header.nal_unit_type);
                // slice_layer_without_partitioning_rbsp(); // but non-VCL
            }
            20 => {
                // Multiview Coding is specified in Annex H, Scalable Video Coding in Annex G, 3D AVC in Annex J
                println!(
                    "\t decode_bitstream - NALU {} - {} - Coded slice extension",
                    i, header.nal_unit_type
                );
                let start_time = SystemTime::now();
                let slice = decode_slice_layer_extension_rbsp(
                    &mut nalu_data,
                    &header,
                    &subset_spses,
                    &ppses,
                    only_headers,
                    decode_strict_fmo,
                );
                if perf_output {
                    let duration = start_time.elapsed();
                    match duration {
                        Ok(elapsed) => {
                            println!("[PERF] decode_bitstream - decode_slice_layer_extension_rbsp - duration: {} ns", elapsed.as_nanos());
                        }
                        Err(e) => {
                            println!("Error: {:?}", e);
                        }
                    }
                }
                slices.push(slice);
            }
            21 => {
                // specified in Annex J
                println!("\t decode_bitstream - NALU {} - {} - Coded slice extension for a depth view component or a 3D-AVC texture view component", i,  header.nal_unit_type);
                // slice_layer_extension_rbsp();
            }
            22..=23 => println!(
                "\t decode_bitstream - NALU {} - {} - RESERVED nal_unit_type ignoring",
                i, header.nal_unit_type
            ),
            // The following types are from https://www.ietf.org/rfc/rfc3984.txt and updated in https://datatracker.ietf.org/doc/html/rfc6184
            24 => {
                // STAP-A    Single-time aggregation packet     5.7.1
                println!(
                    "\t decode_bitstream - NALU {} - {} - RTP STAP-A",
                    i, header.nal_unit_type
                );
                decode_stap_a();
            }
            25 => {
                // STAP-B    Single-time aggregation packet     5.7.1
                println!(
                    "\t decode_bitstream - NALU {} - {} - RTP STAP-B",
                    i, header.nal_unit_type
                );
                decode_stap_b();
            }
            26 => {
                //MTAP16    Multi-time aggregation packet      5.7.2
                println!(
                    "\t decode_bitstream - NALU {} - {} - RTP MTAP16",
                    i, header.nal_unit_type
                );
                decode_mtap16();
            }
            27 => {
                //MTAP24    Multi-time aggregation packet      5.7.2
                println!(
                    "\t decode_bitstream - NALU {} - {} - RTP MTAP24",
                    i, header.nal_unit_type
                );
                decode_mtap24();
            }
            28 => {
                //FU-A      Fragmentation unit                 5.8
                println!(
                    "\t decode_bitstream - NALU {} - {} - RTP FU-A",
                    i, header.nal_unit_type
                );
                decode_fu_a();
            }
            29 => {
                //FU-B      Fragmentation unit                 5.8
                println!(
                    "\t decode_bitstream - NALU {} - {} - RTP FU-B",
                    i, header.nal_unit_type
                );
                decode_fu_b();
            }
            // The following types are from SVC RTP https://datatracker.ietf.org/doc/html/rfc6190
            30 => {
                // PACSI NAL unit                     4.9
                println!(
                    "\t decode_bitstream - NALU {} - {} - RTP SVC PACSI",
                    i, header.nal_unit_type
                );
            }
            31 => {
                // This reads a subtype
                // Type  SubType   NAME
                // 31     0       reserved                           4.2.1
                // 31     1       Empty NAL unit                     4.10
                // 31     2       NI-MTAP                            4.7.1
                // 31     3-31    reserved                           4.2.1
                println!(
                    "\t decode_bitstream - NALU {} - {} - RTP SVC NALU",
                    i, header.nal_unit_type
                );
            }
            _ => println!(
                "\t decode_bitstream - NALU {} - {} - Unknown nal_unit_type ",
                i, header.nal_unit_type
            ),
        };
    }

    println!(
        "\t decode_bitstream - Decoded a total of {} slices",
        slices.len()
    );

    H264DecodedStream {
        nalu_elements,
        nalu_headers,
        spses,
        subset_spses,
        sps_extensions,
        ppses,
        prefix_nalus,
        slices,
        seis,
        auds,
    }
}
