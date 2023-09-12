


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
pub fn encode_stap_a() {
    // while more_data() {
    //   Encode a NAL size that is 16 bits
    //   The embedded NALU has its own header
    // }
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
pub fn encode_stap_b() {
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
/// 
pub fn encode_mtap16() {
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
pub fn encode_mtap24() {
    // While more_data() {
    //   Encode a NALU Size that is 16 bits
    //   Encode a Decoding Order Number Difference (DOND) that is 8-bits
    //   Encode a 24-bit Timestamp Offset
    //   Encode a NALU of nalu size
    // }
}

/// Encodes a Fragmentation Unit (FU) without a DON (FU-A)
/// 
///   0                   1                   2                   3
///   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   | FU indicator  |   FU header   |                               |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+                               |
///   |                                                               |
///   |                         FU payload                            |
///   |                                                               |
///   |                               +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                               :...OPTIONAL RTP padding        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
pub fn encode_fu_a() {
    // Encode FU header
    //  1 bit: Start bit indicating the start of an FU
    //  1 bit: End bit indicating the end of an FU
    //  1 bit: Reserved, please set to 0
    //  5 bit: NALU Payload type
    // The rest is the payload
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
pub fn encode_fu_b() {
    // Encode FU header
    //  1 bit: Start bit indicating the start of an FU
    //  1 bit: End bit indicating the end of an FU
    //  1 bit: Reserved, please set to 0
    //  5 bit: NALU Payload type
    // Encode 16-bit long DON
    // The rest is the payload
}