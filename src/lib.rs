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
/// 
/// * `seed` - Seed to generate video from
/// * `ignore_intra_pred` - If true, does not generate Slices that use Intra prediction
/// * `ignore_edge_intra_pred` - If true, does not generate Slices that use Intra prediction at the edge
/// * `ignore_ipcm` - If true, does not generate PCM macroblocks
/// * `empty_slice_data` - If true, does not Slice data is skipped
/// * `small_video` - If true, will keep output video to 128x128
/// * `silent_mode` - If true, does not output 
/// * `undefined_nalus` - If true, will generate undefined NALU types
pub fn generate_video_from_seed(
    seed: u64,
    ignore_intra_pred : bool,
    ignore_edge_intra_pred : bool,
    ignore_ipcm : bool,
    empty_slice_data : bool,
    small_video : bool,
    silent_mode : bool,
    undefined_nalus : bool) -> Vec<u8> {
    let mut film_state = vidgen::film::FilmState::setup_film_from_seed(seed);

    let rconfig = RandomizeConfig::new();
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

/// Generates a video from a sequence of bytes called a FILM
/// 
/// * `film_contents` - Sequence of bytes to sample random values from
/// * `seed` - Fallback seed for if the bytes run out
/// * `ignore_intra_pred` - If true, does not generate Slices that use Intra prediction
/// * `ignore_edge_intra_pred` - If true, does not generate Slices that use Intra prediction at the edge
/// * `ignore_ipcm` - If true, does not generate PCM macroblocks
/// * `empty_slice_data` - If true, does not Slice data is skipped
/// * `small_video` - If true, will keep output video to 128x128
/// * `silent_mode` - If true, does not output 
/// * `undefined_nalus` - If true, will generate undefined NALU types
pub fn generate_video_from_film_contents(
    film_contents: Vec<u8>,
    seed: u64,
    ignore_intra_pred : bool,
    ignore_edge_intra_pred : bool,
    ignore_ipcm : bool,
    empty_slice_data : bool,
    small_video : bool,
    silent_mode : bool,
    undefined_nalus : bool) -> Vec<u8> {
    let mut film_state = vidgen::film::FilmState::setup_film_from_seed(seed);

    film_state.use_film_file = true;
    film_state.film_file_contents = vidgen::film::FilmStream {
        contents: film_contents,
        read_byte_offset: 0,
        read_bit_offset: 0,
        write_bit_offset: 0,
    };

    let rconfig = RandomizeConfig::new();
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
