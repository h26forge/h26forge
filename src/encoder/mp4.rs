//! MP4 Muxing interface.

use minimp4::Mp4Muxer; // COMMENT OUT IF WANT TO USE WITH AFL++ Binding
use std::fs::File;


/// Save the encoded stream to an MP4 file
pub fn save_mp4_file(
    mp4_filename: String,
    width: i32,
    height: i32,
    is_mp4_fragment: bool,
    is_hevc: bool,
    annex_b_video: &Vec<u8>,
) {
    println!("   Writing MP4 output: {}", mp4_filename);

    // NOTE: AFL++ integration has issues with Mp4Muxer, so this has to be commented out to work with it
    let mut mp4muxer = Mp4Muxer::new(File::create(mp4_filename).unwrap());
    let enable_fragmentation = is_mp4_fragment;
    let is_hevc = is_hevc;
    mp4muxer.init_video(width, height, is_hevc, enable_fragmentation);
    mp4muxer.write_video(annex_b_video);
    mp4muxer.close();
}