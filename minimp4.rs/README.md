# minimp4.rs

A [minimp4](https://github.com/lieff/minimp4) Rust binding.

# Features

- H264 stream mux
- H265 stream mux
- set track title
- set comment

# Usage

``` rust
    let mut mp4muxer = Mp4Muxer::new(File::create("1.mp4").unwrap());
    let mut buf = Vec::new();
    File::open("1.264").unwrap().read_to_end(&mut buf).unwrap();
    mp4muxer.init_video(316, 342, false, "title");
    mp4muxer.write_video(&buf);
    mp4muxer.close();
```

# TODO

- [x] Support hevc mux
- [x] Support multiple track
- [ ] Support audio track
- [x] Support set track title
- [ ] Support set metadata
- [ ] Better error handling

# Contributing

Pull request :)

# WRV: Changes made to MiniMP4

The original MiniMP4 is located at https://github.com/darkskygit/minimp4.rs.

This version contains the following changes:
- Skip parameter set and slice header rewriting to incremental values. Instead, keep the original SPS_ID and PPS_IDs. This impacts playback on some players that expect incremental IDs, per the avcC standard.
- For H.265 muxing, only the first SPS is added to the hvcC atom -- subsequent ones are part of the mdat atom.
- Do not skip AUD types.
- The `first_mb_in_slice` syntax element is ignored.
- Skip an IDR slice requirement.
- Removed some audio processing code. 