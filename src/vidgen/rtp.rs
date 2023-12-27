use crate::common::data_structures::H264DecodedStream;

use super::film::FilmState;


/// Random a Single-Time Aggregation Unit without DON (STAP-A)
pub fn random_stap_a(stap_a_idx: usize,
    ds: &mut H264DecodedStream,
    _film: &mut FilmState)
    {
    // TODO: incorporate random count

    ds.stap_as[stap_a_idx].count = 1;
}

/// Random a Single-Time Aggregation Unit with DON (STAP-B)
pub fn random_stap_b() {
    // Random a Decoding Order Number (DON) 16 bits long
    // while more_data() {
    //   Random a NAL unit size that is 16 bits
    //   Random a NALU of set size
    // }
}


/// Random a Multi-Time Aggregation Packet (MTAP) with 16-bit timestamp offset (TS)
pub fn random_mtap16() {
    // While more_data() {
    //   Random a NALU Size that is 16 bits
    //   Random a Decoding Order Number Difference (DOND) that is 8-bits
    //   Random a 16-bit Timestamp Offset
    //   Random a NALU of nalu size
    // }
}

/// Random a Multi-Time Aggregation Packet (MTAP) with 24-bit timestamp offset (TS)
pub fn random_mtap24() {
    // While more_data() {
    //   Random a NALU Size that is 16 bits
    //   Random a Decoding Order Number Difference (DOND) that is 8-bits
    //   Random a 24-bit Timestamp Offset
    //   Random a NALU of nalu size
    // }
}

/// Randoms a Fragmentation Unit (FU) without a DON (FU-A)
pub fn random_fu_a() {
    // Random FU header
    //  1 bit: Start bit indicating the start of an FU
    //  1 bit: End bit indicating the end of an FU
    //  1 bit: Reserved, please set to 0
    //  5 bit: NALU Payload type
    // The rest is the payload
}

/// Randoms a Fragmentation Unit (FU) with a DON (FU-B)
pub fn random_fu_b() {
    // Random FU header
    //  1 bit: Start bit indicating the start of an FU
    //  1 bit: End bit indicating the end of an FU
    //  1 bit: Reserved, please set to 0
    //  5 bit: NALU Payload type
    // Random 16-bit long DON
    // The rest is the payload
}
