//! H.264 video entropy de/encoding and randomization.
//!
//! Domain-specific infrastructure for analyzing, generating,
//! and manipulating syntactically correct but semantically
//! spec-non-compliant video files.

pub mod common;
pub mod decoder;
pub mod encoder;
pub mod vidgen;

use crate::encoder::encoder::reencode_syntax_elements;
use crate::vidgen::generate_configurations::RandomizeConfig;
use crate::vidgen::vidgen::random_video;

/// Generates a video from a seed value
pub fn generate_video_from_seed(seed: u64) -> Vec<u8> {
    let mut film_state = vidgen::film::FilmState::setup_film_from_seed(seed);

    let rconfig = RandomizeConfig::new();

    let ignore_intra_pred = false;
    let ignore_edge_intra_pred = true;
    let ignore_ipcm = true;
    let empty_slice_data = false;
    let small_video = true;
    let silent_mode = true;
    let undefined_nalus = true;
    let cut_nalu = -1;

    let mut vid = random_video(
        ignore_intra_pred,
        ignore_edge_intra_pred,
        ignore_ipcm,
        empty_slice_data,
        small_video,
        silent_mode,
        undefined_nalus,
        &rconfig,
        &mut film_state,
    );

    let avcc_out = false;
    let encoded_vid = reencode_syntax_elements(&mut vid, cut_nalu, avcc_out, silent_mode);

    encoded_vid.0
}

/// Generates a video from a film contents file
pub fn generate_video_from_film_contents(film_file_contents: Vec<u8>) -> Vec<u8> {
    let mut film_state = vidgen::film::FilmState::setup_film();

    film_state.use_film_file = true;
    film_state.film_file_contents = vidgen::film::FilmStream {
        contents: film_file_contents,
        read_byte_offset: 0,
        read_bit_offset: 0,
        write_bit_offset: 0,
    };

    let rconfig = RandomizeConfig::new();

    let ignore_intra_pred = false;
    let ignore_edge_intra_pred = true;
    let ignore_ipcm = true;
    let empty_slice_data = false;
    let small_video = true;
    let silent_mode = true;
    let undefined_nalus = true;
    let cut_nalu = -1;

    let mut vid = random_video(
        ignore_intra_pred,
        ignore_edge_intra_pred,
        ignore_ipcm,
        empty_slice_data,
        small_video,
        silent_mode,
        undefined_nalus,
        &rconfig,
        &mut film_state,
    );

    let avcc_out = false;
    let encoded_vid = reencode_syntax_elements(&mut vid, cut_nalu, avcc_out, silent_mode);

    encoded_vid.0
}
