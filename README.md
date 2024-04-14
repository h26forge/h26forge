# H26Forge

H26Forge is domain-specific infrastructure for analyzing, generating, and manipulating syntactically correct but semantically spec-non-compliant H.264 video files.

H26Forge has three key features:
1. Given an Annex B H.264 file, randomly mutate the syntax elements.
2. Given an Annex B H.264 file and a Python script, programmatically modify the syntax elements.
3. Generate Annex B H.264 with random syntax elements, which can be written to files or streamed over RTP

This tool and its findings are described in the [H26Forge paper](https://wrv.github.io/h26forge.pdf).

## Motivation

See [MOTIVATION.md](docs/MOTIVATION.md) to see how a H26Forge simplifies a previously manual PoC generation process.

## Release

You can downloaded the latest build on the [releases](https://github.com/h26forge/h26forge/releases) page.

## Building

An up-to-date version of Rust is required to build H26Forge. Be sure to call `rustup update`.

Python 3 is required for editing videos.

**Linux/Mac**
```
./make.sh
```

**Windows**
```
./make.bat
```

These scripts just call `cargo build --release` and copy the binary to the top level directory.

## Quickstart

To generate a video with the default configuration, run:
```
./h26forge generate -o out.264
```

To use a custom randomness range, modify (or copy and create a new file) the syntax element range in [config/default.json](config/default.json) and run:
```
./h26forge generate -o out.264 -c config/default.json
```

A [simple script](scripts/gen_100_videos.sh) is included to generate a batch of 100 videos to a temporary folder. You can run it via:
```
./scripts/gen_100_videos.sh
```
The videos will be output to `tmp/`.


[Another script](scripts/gen_100_chrome_videos.sh) is included to generate 100 videos using [config/chrome.json](config/chrome.json).
```
./scripts/gen_100_chrome_videos.sh
```
You can open the videos in a browser by opening `tmp/<output>/play_videos.html`.


See [GETTINGSTARTED.md](docs/GETTINGSTARTED.md) for more usage details.

See [Example: CVE-2022-22675 PoC](docs/EDITING.md#example-cve-2022-22675-poc) to see the capabilities of editing syntax elements.

## Code

[Contributions](docs/CONTRIBUTING.md) are welcome! Running `cargo doc` will produce the code documentation, available at `target/doc/h26forge/index.html`.

Some unit tests are available. You can run `cargo test` to see their output.

## Spec Coverage

The [H.264 (08/2021) Spec](https://www.itu.int/rec/T-REC-H.264-202108-I/en) is 844 pages long, containing instructions for parsing the bitstream into syntax elements and reproducing video frames.

### Conformance

We can use the ITU test vectors to identify spec conformance for a particular profile. The ITU Test Vectors are available [here](https://www.itu.int/net/ITU-T/sigdb/spevideo/VideoForm-s.aspx?val=102002641).

To run the test vectors, you can run [`./scripts/download_and_run_avcv1_test_vectors.sh`](scripts/download_and_run_avcv1_test_vectors.sh). Alternatively, you can run
```
python scripts/end-to-end-tester.py --testdir <DIRECTORY>
```
where `<DIRECTORY>` points to any folder with `.264` files.

#### Conformance Results

- 99%: Bitstreams for Constrained Baseline, Baseline, Extended and Main profiles. `FM1_BT_B.264` and `FM1_FT_E.264` fail. The two failing videos use FMO, and have issues with CAVLC CoeffToken recovery, seemingly with getting the correct `nC` value based on neighbor values.
- 97%: Bitstreams for Fidelity Range Extensions (High, High 10, and High 4:2:2 profiles). The videos `WB_10bit_QP21_1920x1088` and `WB_10bit_QP21_I-Only_1920x1088` are too big to test.

## How to Cite

> W.R. Vasquez, S. Checkoway, and H. Shacham. [**“The Most Dangerous Codec in the World: Finding and Exploiting Vulnerabilities in H.264 Decoders.”**](https://wrv.github.io/h26forge.pdf) In J. Calandrino and C. Troncoso, eds., *Proceedings of USENIX Security 2023*. USENIX, Aug. 2023.

## Trophies

The following bugs have been found with H26Forge's video generator. If you use H26Forge to find and report an issue, please let us know so we can include it in this list.
- [CVE-2022-48434](https://nvd.nist.gov/vuln/detail/CVE-2022-48434): Use-after-free in FFmpeg as used by VLC due to an SPS change mid-video.
- [CVE-2022-42850](https://support.apple.com/en-us/HT213530): A lack of bounds-checking in SPS StRefPic list parsing leads to an iOS kernel heap overflow.
- CVE-2022-42846 [[1]](https://support.apple.com/en-us/HT213531), [[2]](https://support.apple.com/en-us/HT213530): An IDR Inter predicted first slice leads to an iOS kernel infinite loop during reference picture list modification. 0-clickable.
- CVE-2022-32939 [[1]](https://support.apple.com/en-us/HT213490), [[2]](https://support.apple.com/en-us/HT213489): More than 256 emulation prevention bytes in a correctly encoded H.264 bitstream led to an arbitrary iOS kernel write primitive. 0-clickable.
- [CVE-2022-3266](https://www.mozilla.org/en-US/security/advisories/mfsa2022-40/#CVE-2022-3266): Video width and height was not updated between container and SPS, and also across SPSes in Firefox. This led to a crash of the Firefox GPU process and an information leak.
- [upipe_h264_framer: Fix valgrind warnings on fuzzed files #956](https://github.com/Upipe/upipe/pull/956)
- [CVE-2024-27228](https://bugs.chromium.org/p/project-zero/issues/detail?id=2512): Out-of-bounds quantization parameter leads to an out-of-bounds write in the MFC H.264 hardware video decoder found in the Pixel 7.

## Contributors

- [Natalie Silvanovich](https://github.com/natashenka): Streaming mode and RTP output.