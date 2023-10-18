# Getting Started with H26Forge

## Background
The H.264 codec organizes its instructions for video reconstruction as _syntax elements_. The syntax elements are _entropy_ encoded using algorithms such as binary encodings, exp-Golomb, CABAC, and CAVLC. These entropy encoded syntax elements are bit-oriented, and also dependent on each other. In fact, the "CA" in CABAC and CAVLC stands for context adaptive, meaning some state is used to decode each syntax element.

## H26Forge

H26Forge is a domain-specific infrastructure for working with entropy-encoded H.264 videos. At a high level, H26Forge entropy decodes an input video to produce a *H264DecodedStream* object which contains all the syntax elements of the video. While in this state, syntax elements can be modified to a desired value. H26Forge entropy encodes the modified syntax elements to produce a new encoded video.

When editing videos, it is helpful to a have a copy of the [H.264 spec](https://www.itu.int/rec/T-REC-H.264-202108-I/en) alongside.

### Note on Inputs
Unless specified otherwise, H26Forge expects inputs as **Annex B H.264 bitstreams**.

Starting from an MP4 file, you can recover the bitstream with FFmpeg as so:
```
ffmpeg -i input.mp4 -vcodec copy -an -bsf:v h264_mp4toannexb output.264
```

## Main Modes

Options across modes:
```
  -d, --debug-decode                    Enable decoder syntax element printing
  -e, --debug-encode                    Enable encoder syntax element printing
      --silent                          Reduce the amount of messages going to STDOUT. Warnings and file locations are still output
      --perf                            Output available performance information
      --hevc                            Enable if input is H.265
      --strict-fmo                      If FMO is enabled, use the slice group map. Some malformed videos may not be decodable
      --safestart                       Prepend output video with known good video
      --json                            Generate a JSON of the recovered syntax elements
      --json-no-nalu                    When generating the JSON, do not output the original encoded NALUs
      --avcc                            Output AVCC format video in JavaScript Uint8Array format
      --cut <OUTPUT_CUT>                Cut out a passed in NALU index [default: -1]
      --save-default-config             Save the default configuration used in random video generation
      --mp4                             Output a muxed mp4 file. If `safestart` is enabled, will also output a safe start mp4
      --mp4-frag                        Apply MP4 Fragmentation. Useful for Media Source Extensions
      --mp4-rand-size                   If MP4 output is enabled, will randomize the width/height parameters. Only applied in Randomize or Generate mode
      --mp4-width <OUTPUT_MP4_WIDTH>    Manually set the MP4 width [default: -1]
      --mp4-height <OUTPUT_MP4_HEIGHT>  Manually set the MP4 height [default: -1]
```

### Passthrough Mode

Passthrough mode decodes a video and immediately re-encodes the recovered syntax elements.

Usage: `./h26forge passthrough -i input.264 -o output.264`

### Generating Videos

See [GENERATION.md](GENERATION.md) for details.

### Editing Videos

See [EDITING.md](EDITING.md) for details.

###  Streaming H264 over RTP

See [STREAMING.md](STREAMING.md) for details.

## Other Modes

The following modes may be useful.

### Synthesis

Synthesis mode takes in a JSON file produced by passing the `--json` flag on a decoded video and encodes it. Put in another way, it does a Serde `from_json` for a [`H264DecodedStream`](../src/common/data_structures.rs#L13) object. This is helpful if you want to programmatically modify a video manually or with another tool and produce a valid encoding of the file.

To produce a JSON object from a generated video, run `./h26forge --json generate -o out.264`. This will produce an `out.264.json` file.

To produce a video from a JSON object, run `./h26forge synthesize -i out.264.json -o synth.264`.

### Randomization

Randomization mode will modify the syntax elements in slices, both header and body. It decodes an input video and proceeds to randomize a particular slice, identified by `--slice-idx <index>` or all slices if the `--randomize-all-slices` flag is passed in. You can randomize the slice header by passing in the `--randomize-slice-header` flag.

This mode is helpful for targeting hardware, or for finding stateful bugs. Because slice data is primarily decoded in hardware, randomizing these syntax elements will stress hardware parsing. Also, randomizing later parts of a video may help find bugs once decoders reach a certain state.

Usage: `./h26forge randomize -i vid.264 -o out.264`.

This uses some of the same flags as [generation mode](GENERATION.md#options). Randomization specific flags are:
- `--slice-idx <index>`: Slice index to randomize [default: 0]
- `--randomize-slice-header`: Randomize the slice header along a slice
- `--randomize-all-slices`: Randomize all slices

### Mux

This mode uses minimp4.rs to mux an input H.264 or H.265 video. It does not decode, nor encode, the input video.

Usage: `./h26forge mux -i in.264 -o out.mp4`

The following MP4 options work in this mode:
- `--mp4-frag`: Apply MP4 Fragmentation. Useful for Media Source Extensions
- `--mp4-width <width>`: Manually set the MP4 width [default: -1]
- `--mp4-height <height>`: Manually set the MP4 height [default: -1]

### Experimental

This mode is used to partially decode H.265 videos. This was all written for Section 5.2 of the [H26Forge paper](https://wrv.github.io/h26forge.pdf).

At the moment, this code partially decodes an H.265 SPS and modifies it to create a PoC for CVE-2022-42850.

Usage: `./h26forge --mp4 experimental -i input.265 -o output.265`
