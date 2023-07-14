mod lib;

use lib::Mp4Muxer;
use std::fs::File;
use std::io::{self, Cursor, Read};
use std::path::Path;

fn read_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut buf = vec![];
    File::open(path)?.read_to_end(&mut buf)?;
    Ok(buf)
}

fn main() {
    let buffer = Cursor::new(vec![]);
    let mut mp4muxer = Mp4Muxer::new(buffer);
    if let Ok(h264) = read_file("1.264") {
        let is_hevc = false;
        let enable_fragmentation = false;
        mp4muxer.init_video(1920, 1200, is_hevc, enable_fragmentation);
        mp4muxer.write_video(&h264);
    }
    if let Ok(h265) = read_file("1.265") {
        let is_hevc = true;
        let enable_fragmentation = false;
        mp4muxer.init_video(1920, 1200, is_hevc, enable_fragmentation);
        mp4muxer.write_video(&h265);
    }
    mp4muxer.write_comment("test comment");
    mp4muxer.close();
}
