# H.264 Types

This document describes some of the different parameters available in H.264 videos.

## NALU Types

Table 7-1 of the spec

| nal_unit_type | Meaning                                                                      |
|---------------| ---------------------                                                        |
|        0      |   Unspecified                                                                |
|        1      |   Coded slice of a non-IDR picture                                           |
|        2      |   Coded slice data partition A                                               |
|        3      |   Coded slice data partition B                                               |
|        4      |   Coded slice data partition C                                               |
|        5      |   Coded slice of an IDR picture                                              |
|        6      |   Supplemental enhancement information (SEI)                                 |
|        7      |   Sequence parameter set                                                     |
|        8      |   Picture parameter set                                                      |
|        9      |   Access unit delimiter                                                      |
|        10     |   End of sequence                                                            |
|        11     |   End of stream                                                              |
|        12     |   Filler data                                                                |
|        13     |   Sequence parameter set extension                                           |
|        14     |   Prefix NAL unit                                                            |
|        15     |   Subset sequence parameter set                                              |
|        16     |   Depth parameter set                                                        |
|      17..18   |   Reserved                                                                   |
|        19     |   Coded slice of an auxiliary coded picture without partitioning             |
|        20     |   Coded slice extension                                                      |
|        21     |   Coded slice extension for a depth view component or a 3D-AVC texture view  |
|      22..23   |   Reserved                                                                   |
|      24..31   |   Unspecified                                                                |


Table 3 from the is from RTP Payload Format Spec: https://datatracker.ietf.org/doc/html/rfc6184

> Table 3. Summary of allowed NAL unit types for each packetization mode (yes = allowed, no = disallowed, ig = ignore)

| Payload Type |  Packet Type   |   Single NAL Unit Mode |   Non-Interleaved Mode  |  Interleaved Mode  |
| -------      | ---------      | -------------          | -------------------     | -----------        |
| 0            | reserved       |      ig                |         ig              |      ig            |
| 1-23         | NAL unit       |     yes                |        yes              |      no            |
| 24           | STAP-A         |      no                |        yes              |      no            |
| 25           | STAP-B         |      no                |         no              |     yes            |
| 26           | MTAP16         |      no                |         no              |     yes            |
| 27           | MTAP24         |      no                |         no              |     yes            |
| 28           | FU-A           |      no                |        yes              |     yes            |
| 29           | FU-B           |      no                |         no              |     yes            |
| 30-31        | reserved       |      ig                |         ig              |      ig            |


## Profile IDCs

This is all the potential `profile_idc` values and their definitions. Much of this information
is found in Annex A of the spec.  A profile dictates what features may have been used to compress the video.

| profile_idc | Description |
| ----------- | ----------- |
|    44       |  CAVLC 4:4:4 Intra profile                                         |
|    66       |  Baseline profile                                                  |
|    77       |  Main profile                                                      |
|    83       |  Scalable Baseline profile (Annex G)                               |
|    86       |  Scalable High profile (Annex G)                                   |
|    88       |  Extended profile                                                  |
|    100      |  High profile                                                      |
|    110      |  High 10 profile                                                   |
|    118      |  Multiview High Profile                                            |
|    119      |  Multiview Field High (Found in JM version 17, not found in spec)  |
|    122      |  High 4:2:2                                                        |
|    128      |  Stereo High Profile (Annex H)                                     |
|    134      |  MFC High Profile (Annex H)                                        |
|    135      |  MFC Depth High Profile (Annex I)                                  |
|    138      |  Multiview Depth High profile (Annex I)                            |
|    139      |  Enhanced Multiview Depth High profile (Annex J)                   |
|    144      |  High 4:4:4 (removed from the spec in 2006)                        |
|    244      |  High Predictive 4:4:4                                             |

## Level IDCs

The `level_idc` specifies the expected playback rate for a decoder. It signals what the
maximum possible frame size will be, how many frames (counted in macroblocks) are
expected to be stored in the decoded picture buffer, as well as other limits on
motion vector lengths.

See Table A-1 in the spec for exact limits.

`level_idc` values: 10, 11, 9 (indicates 1b), 12, 13, 20, 21, 22, 30, 31, 32, 40, 41, 42, 50, 51, 52, 60, 61, 62

Wikipedia also has a [good table](https://en.wikipedia.org/wiki/Advanced_Video_Coding#Levels) summarizing playback information.

## AVCC Extradata Format

All credits go to [szatmary of Stackoverflow](https://stackoverflow.com/a/24890903/8169613) for this
great summary of the AVCC extradata format used in MP4 files and the WebCodecs format.

```
bits
8   version ( always 0x01 )
8   avc profile ( sps[0][1] )
8   avc compatibility ( sps[0][2] )
8   avc level ( sps[0][3] )
6   reserved ( all bits on )
2   NALULengthSizeMinusOne
3   reserved ( all bits on )
5   number of SPS NALUs (usually 1)

repeated once per SPS:
  16         SPS size
  variable   SPS NALU data

8   number of PPS NALUs (usually 1)

repeated once per PPS:
  16       PPS size
  variable PPS NALU data
```

