//! H.264 video entropy de/encoding and randomization.
//!
//! Domain-specific infrastructure for analyzing, generating,
//! and manipulating syntactically correct but semantically
//! spec-non-compliant video files.

pub mod common;
pub mod decoder;
pub mod encoder;
pub mod vidgen;

use common::data_structures::H264DecodedStream;
use common::data_structures::NALUheader;
use common::data_structures::PicParameterSet;
use common::data_structures::SeqParameterSet;
use common::data_structures::Slice;
use common::data_structures::VideoParameters;
use encoder::encoder::insert_emulation_three_byte;
use encoder::encoder::reencode_syntax_elements;
use encoder::nalu::encode_nalu_header;
use encoder::parameter_sets::encode_pps;
use encoder::parameter_sets::encode_sps;
use encoder::slice::encode_slice;
use vidgen::film::FilmState;
use vidgen::generate_configurations::RandomizeConfig;
use vidgen::nalu::random_nalu_header;
use vidgen::parameter_sets::random_pps;
use vidgen::parameter_sets::random_sps;
use vidgen::slice::random_slice_header;
use vidgen::vidgen::random_video;


/// Generates a single SPS without initializing the random state
fn generate_single_sps(
    enable_extensions: bool,
    small_video: bool,
    silent_mode: bool,
    rconfig : &RandomizeConfig,
    mut film : &mut FilmState,
) -> (SeqParameterSet, Vec<u8>) {
    let mut sps = SeqParameterSet::new();
    let mut encoded_sps = vec![0, 0, 0, 1];

    // generate the NALU header for the SPS

    let mut nalu_header = 7u8;
    // leave the top most bits, set the type to SPS (7)
    nalu_header |= film.read_film_bytes(1)[0] & 0xe0;
    encoded_sps.push(nalu_header);

    // Generate a random Sequence Parameter Set
    random_sps(&mut sps, enable_extensions, &rconfig.random_sps_range, small_video, silent_mode, &mut film);
    encoded_sps.extend_from_slice(&insert_emulation_three_byte(&encode_sps(&sps, false)));

    (sps, encoded_sps)
}

/// Generates a single PPS without initializing the random state
fn generate_single_pps(
    sps : &SeqParameterSet,
    rconfig : &RandomizeConfig,
    mut film : &mut FilmState,
) -> (PicParameterSet, Vec<u8>) {
    let mut encoded_pps = vec![0, 0, 0, 1];

    // NALU type of 8
    let mut nalu_header = 8u8;
    // Randomize top most bits
    nalu_header |= film.read_film_bytes(1)[0] & 0xe0;
    encoded_pps.push(nalu_header);

    let mut ds = H264DecodedStream::new();
    ds.ppses.push(PicParameterSet::new());

    // Generate a random Picture Parameter Set
    random_pps(
        0,
        sps,
        rconfig.random_pps_range,
        &mut ds,
        &mut film);

    encoded_pps.extend_from_slice(&insert_emulation_three_byte(&encode_pps(&ds.ppses[0], sps)));

    (ds.ppses[0].clone(), encoded_pps)
}


fn generate_single_slice_header(
    sps : &SeqParameterSet,
    pps : &PicParameterSet,
    silent_mode: bool,
    rconfig : &RandomizeConfig,
    film : &mut FilmState,
) -> Vec<u8> {
    let mut encoded_slice_header = vec![0, 0, 0, 1];


    let mut ds = H264DecodedStream::new();
    ds.nalu_headers.push(NALUheader::new());
    ds.slices.push(Slice::new());

    random_nalu_header(
        0,
        true,
        false,
        false,
        &rconfig.random_nalu_range,
        &mut ds,
        film);

    // NALU Type of 1 or 5
    ds.nalu_headers[0].nal_unit_type = 1;
    let is_idr = film.read_film_bool(0, 1, 1);
    if is_idr {
        ds.nalu_headers[0].nal_unit_type = 5;
    }

    let vp = VideoParameters::new(&ds.nalu_headers[0], pps, sps);

    random_slice_header(
        0,
        0,
        pps,
        sps,
        &vp,
        silent_mode,
        &rconfig.random_slice_header_range,
        &mut ds,
        film,
    );

    // Random slice header generation will extend the macroblock vec
    // for fields, so this just zeroes out that extension to skip trying
    // to encode slice data.
    ds.slices[0].sd.macroblock_vec = Vec::new();

    encoded_slice_header.extend_from_slice(&encode_nalu_header(&ds.nalu_headers[0]));
    encoded_slice_header.extend_from_slice(&insert_emulation_three_byte(&encode_slice(
        &ds.nalu_headers[0],
        &ds.slices[0],
        &sps,
        &pps,
        &vp,
        silent_mode,
        )));


    encoded_slice_header
}


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
    ignore_intra_pred: bool,
    ignore_edge_intra_pred: bool,
    ignore_ipcm: bool,
    empty_slice_data: bool,
    small_video: bool,
    silent_mode: bool,
    undefined_nalus: bool,
) -> Vec<u8> {
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
    let encoded_vid = reencode_syntax_elements(&mut vid, cut_nalu, avcc_out, silent_mode, false);

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
    ignore_intra_pred: bool,
    ignore_edge_intra_pred: bool,
    ignore_ipcm: bool,
    empty_slice_data: bool,
    small_video: bool,
    silent_mode: bool,
    undefined_nalus: bool,
) -> Vec<u8> {
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
    let encoded_vid = reencode_syntax_elements(&mut vid, cut_nalu, avcc_out, silent_mode, false);

    encoded_vid.0
}

/// Generates an SPS from a sequence of bytes called a FILM
///
/// * `film_contents` - Sequence of bytes to sample random values from
/// * `seed` - Fallback seed for if the bytes run out
/// * `enable_extensions` - If true, will choose extension Profiles
/// * `small_video` - If true, will keep output video to 128x128
/// * `silent_mode` - If true, does not output
pub fn generate_sps_from_film_contents(
    film_contents: Vec<u8>,
    seed: u64,
    enable_extensions: bool,
    small_video: bool,
    silent_mode: bool,
) -> Vec<u8> {
    let mut film = vidgen::film::FilmState::setup_film_from_seed(seed);

    film.use_film_file = true;
    film.film_file_contents = vidgen::film::FilmStream {
        contents: film_contents,
        read_byte_offset: 0,
        read_bit_offset: 0,
        write_bit_offset: 0,
    };

    let rconfig = RandomizeConfig::new();

    generate_single_sps(enable_extensions, small_video, silent_mode, &rconfig, &mut film).1
}


/// Generates a collection of Parameter Sets from a sequence of bytes called a FILM
///
/// * `film_contents` - Sequence of bytes to sample random values from
/// * `seed` - Fallback seed for if the bytes run out
/// * `enable_extensions` - If true, will choose extension Profiles
/// * `small_video` - If true, will keep output video to 128x128
/// * `silent_mode` - If true, does not output
pub fn generate_parameter_sets_from_film_contents(
    film_contents: Vec<u8>,
    seed: u64,
    count : u8,
    enable_extensions: bool,
    small_video: bool,
    silent_mode: bool,
) -> Vec<u8> {

    let mut film = vidgen::film::FilmState::setup_film_from_seed(seed);

    film.use_film_file = true;
    film.film_file_contents = vidgen::film::FilmStream {
        contents: film_contents,
        read_byte_offset: 0,
        read_bit_offset: 0,
        write_bit_offset: 0,
    };

    let rconfig = RandomizeConfig::new();

    let mut encoded_stream : Vec<u8> = Vec::new();

    let mut sps_array : Vec<SeqParameterSet> = Vec::new();

    // Generate an initial SPS
    let init_sps = generate_single_sps(enable_extensions, small_video, silent_mode, &rconfig, &mut film);

    sps_array.push(init_sps.0);
    encoded_stream.extend(init_sps.1.iter());


    for _ in 0..count {
        let gen_pps = film.read_film_bool(0, 1, 1);

        if gen_pps {
            // use any previously generated SPS as a basis for the PPS
            let sps_idx = film.read_film_u32(0, sps_array.len() as u32 -1);
            encoded_stream.extend(generate_single_pps(&sps_array[sps_idx as usize], &rconfig, &mut film).1.iter());
        } else {
            let new_sps = generate_single_sps(enable_extensions, small_video, silent_mode, &rconfig, &mut film);
            sps_array.push(new_sps.0);
            encoded_stream.extend(new_sps.1.iter());
        }
    }

    encoded_stream
}

/// Generates a collection of Parameter Sets from a sequence of bytes called a FILM
///
/// * `film_contents` - Sequence of bytes to sample random values from
/// * `seed` - Fallback seed for if the bytes run out
/// * `enable_extensions` - If true, will choose extension Profiles
/// * `small_video` - If true, will keep output video to 128x128
/// * `silent_mode` - If true, does not output
pub fn generate_parameters_and_slices_from_film_contents(
    film_contents: Vec<u8>,
    seed: u64,
    count : u8,
    enable_extensions: bool,
    small_video: bool,
    silent_mode: bool,
) -> Vec<u8> {

    let mut film = vidgen::film::FilmState::setup_film_from_seed(seed);

    film.use_film_file = true;
    film.film_file_contents = vidgen::film::FilmStream {
        contents: film_contents,
        read_byte_offset: 0,
        read_bit_offset: 0,
        write_bit_offset: 0,
    };

    let rconfig = RandomizeConfig::new();

    let mut encoded_stream : Vec<u8> = Vec::new();

    let mut sps_array : Vec<SeqParameterSet> = Vec::new();
    let mut pps_array : Vec<(usize, PicParameterSet)> = Vec::new();

    // Generate an initial SPS
    let init_sps = generate_single_sps(enable_extensions, small_video, silent_mode, &rconfig, &mut film);
    sps_array.push(init_sps.0);
    encoded_stream.extend(init_sps.1.iter());

    // Generate an initial PPS
    let sps_idx = film.read_film_u32(0, sps_array.len() as u32 -1) as usize;
    let init_pps = generate_single_pps(&sps_array[sps_idx], &rconfig, &mut film);
    pps_array.push((sps_idx, init_pps.0));
    encoded_stream.extend(init_pps.1.iter());


    for _ in 0..count {
        let gen_type = film.read_film_u32(0, 2);

        match gen_type {
            0 => {
                // new SPS
                let new_sps = generate_single_sps(enable_extensions, small_video, silent_mode, &rconfig, &mut film);
                sps_array.push(new_sps.0);
                encoded_stream.extend(new_sps.1.iter());
            },
            1 => {
                // use any previously generated SPS as a basis for the PPS
                let sps_idx = film.read_film_u32(0, sps_array.len() as u32 -1) as usize;
                let new_pps = generate_single_pps(&sps_array[sps_idx], &rconfig, &mut film);
                pps_array.push((sps_idx, new_pps.0));
                encoded_stream.extend(new_pps.1.iter());
            },
            _ => {

                let pps_idx = film.read_film_u32(0, pps_array.len() as u32 -1) as usize;
                let sps = &sps_array[pps_array[pps_idx].0];
                let pps = &pps_array[pps_idx].1;

                generate_single_slice_header(sps, pps, silent_mode, &rconfig, &mut film);
            }
        };
    }

    encoded_stream
}
