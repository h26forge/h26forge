

/// Decode a Single-Time Aggregation Unit without DON (STAP-A)
pub fn decode_stap_a() {
    // while more_data() {
    //   Decode a NAL size that is 16 bits
    //   The embedded NALU has its own header
    // }
}

/// Decode a Single-Time Aggregation Unit with DON (STAP-B)
pub fn decode_stap_b() {
    // Decode a Decoding Order Number (DON) 16 bits long
    // while more_data() {
    //   Decode a NAL unit size that is 16 bits
    //   Decode a NALU of set size
    // }
}


/// Decode a Multi-Time Aggregation Packet (MTAP) with 16-bit timestamp offset (TS)
pub fn decode_mtap16() {
    // While more_data() {
    //   Decode a NALU Size that is 16 bits
    //   Decode a Decoding Order Number Difference (DOND) that is 8-bits
    //   Decode a 16-bit Timestamp Offset
    //   Decode a NALU of nalu size
    // }
}

/// Decode a Multi-Time Aggregation Packet (MTAP) with 24-bit timestamp offset (TS)
pub fn decode_mtap24() {
    // While more_data() {
    //   Decode a NALU Size that is 16 bits
    //   Decode a Decoding Order Number Difference (DOND) that is 8-bits
    //   Decode a 24-bit Timestamp Offset
    //   Decode a NALU of nalu size
    // }
}

/// Decodes a Fragmentation Unit (FU) without a DON (FU-A)
pub fn decode_fu_a() {
    // Decode FU header
    //  1 bit: Start bit indicating the start of an FU
    //  1 bit: End bit indicating the end of an FU
    //  1 bit: Reserved, please set to 0
    //  5 bit: NALU Payload type
    // The rest is the payload
}

/// Decodes a Fragmentation Unit (FU) with a DON (FU-B)
pub fn decode_fu_b() {
    // Decode FU header
    //  1 bit: Start bit indicating the start of an FU
    //  1 bit: End bit indicating the end of an FU
    //  1 bit: Reserved, please set to 0
    //  5 bit: NALU Payload type
    // Decode 16-bit long DON
    // The rest is the payload
}
