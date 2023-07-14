//! Converts JSON to H264DecodedStream and vice-versa.

use crate::common::data_structures::H264DecodedStream;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

/// Takes in a json encoding of the H264 Decoded Stream and
/// produces the H264DecodedStream to encode out
pub fn syntax_to_video(input_file: &str) -> H264DecodedStream {
    // recover the JSON and fill it into an H264DecodedStream object
    let json_file = match File::open(input_file) {
        Err(_) => panic!("couldn't open {}", input_file),
        Ok(file) => file,
    };

    let reader = BufReader::new(json_file);

    let res: H264DecodedStream = match serde_json::from_reader(reader) {
        Ok(x) => x, // copy over the new result
        Err(y) => panic!("Error reading modified H264DecodedStream: {:?}", y),
    };

    res
}

/// Takes in a H264DecodedStream and a file_name and saves
/// the json to that file
pub fn video_to_syntax(ds: &H264DecodedStream, no_nalu_elements: bool, filename: &str) {
    // json_file will store our H264DecodedStream elements
    let mut json_file = match File::create(filename) {
        Err(_) => panic!("couldn't create {}", filename),
        Ok(file) => file,
    };

    let serialized: String;

    if no_nalu_elements {
        // create a clone with no nalu_elements
        // TODO: verify if this impacts syntax_to_video re-encoding for non-supported NALUs
        let mut new_ds = H264DecodedStream::new();

        new_ds.nalu_headers = ds.nalu_headers.clone();
        new_ds.spses = ds.spses.clone();
        new_ds.subset_spses = ds.subset_spses.clone();
        new_ds.sps_extensions = ds.sps_extensions.clone();
        new_ds.ppses = ds.ppses.clone();
        new_ds.subset_ppses = ds.subset_ppses.clone();
        new_ds.prefix_nalus = ds.prefix_nalus.clone();
        new_ds.slices = ds.slices.clone();

        serialized = serde_json::to_string(&new_ds).unwrap();
    } else {
        serialized = serde_json::to_string(ds).unwrap();
    }

    match json_file.write_all(serialized.as_bytes()) {
        Err(_) => panic!("couldn't write to file {}", filename),
        Ok(()) => (),
    };
}
