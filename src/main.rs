mod common;
mod decoder;
mod encoder;
mod experimental;
mod vidgen;

use clap::{Parser, Subcommand};
use log::{debug, LevelFilter, SetLoggerError};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
};
use std::path::Path;
use std::time::SystemTime;

/// Runtime options for H26Forge
#[derive(Parser)]
#[command(name = "H26Forge")]
#[command(author = "Willy R. Vasquez <wrv@utexas.edu>")]
#[command(version = "1.0")]
#[command(about = "Construct modified or randomized H.264 encoded files")]
struct H26ForgeOptions {
    /// Mode to use H26Forge
    #[command(subcommand)]
    mode: Option<Commands>,

    /// Enable decoder syntax element printing
    #[arg(short = 'd', long)]
    debug_decode: bool,
    /// Enable encoder syntax element printing
    #[arg(short = 'e', long)]
    debug_encode: bool,

    /// Reduce the amount of messages going to STDOUT. Warnings and file locations are still output
    #[arg(long = "silent")]
    print_silent: bool,
    /// Output available performance information
    #[arg(long = "perf")]
    print_perf: bool,

    /// Enable if input is H.265
    #[arg(long = "hevc")]
    input_is_hevc: bool,

    /// If FMO is enabled, use the slice group map. Some malformed videos may not be decodable
    #[arg(long = "strict-fmo")]
    decode_strict_fmo: bool,

    /// Prepend output video with known good video
    #[arg(long = "safestart")]
    include_safestart: bool,

    /// Generate a JSON of the recovered syntax elements
    #[arg(long = "json")]
    output_syntax_json: bool,
    /// When generating the JSON, do not output the original encoded NALUs
    #[arg(long = "json-no-nalu")]
    output_no_nalu_elements: bool,
    /// Output AVCC format video in JavaScript Uint8Array format
    #[arg(long = "avcc")]
    output_avcc: bool,
    /// Cut out a passed in NALU index
    #[arg(long = "cut", default_value = "-1")]
    output_cut: i32,
    /// Save the default configuration used in random video generation
    #[arg(long = "save-default-config")]
    save_default_config: bool,

    /// Output a muxed mp4 file. If `safestart` is enabled, will also output a safe start mp4
    #[arg(long = "mp4")]
    output_mp4: bool,
    /// Apply MP4 Fragmentation. Useful for Media Source Extensions.
    #[arg(long = "mp4-frag")]
    output_mp4_fragment: bool,
    /// If MP4 output is enabled, will randomize the width/height parameters. Only applied in Randomize or Generate mode.
    #[arg(long = "mp4-rand-size")]
    output_mp4_randomsize: bool,
    /// Manually set the MP4 width
    #[arg(long = "mp4-width", default_value = "-1")]
    output_mp4_width: i32,
    /// Manually set the MP4 height
    #[arg(long = "mp4-height", default_value = "-1")]
    output_mp4_height: i32,
    // Output video_replay file for WebRTC, see: https://webrtchacks.com/video_replay/
   #[arg(long = "rtp-replay")]
    output_rtp: bool
}

#[derive(Subcommand)]
enum Commands {
    /// Passthrough files
    Passthrough {
        /// Input H.264 file
        #[arg(short, long, required = true)]
        input: String,
        /// Output H.264 file
        #[arg(short, long, required = true)]
        output: String,
        /// Only decode Parameter Sets and Slice headers
        #[arg(long = "decode-only-headers")]
        decode_only_headers: bool,
    },
    /// Synthesize a JSON file to an encoded video
    Synthesize {
        /// Input JSON file
        #[arg(short, long, required = true)]
        input: String,
        /// Output H.264 file
        #[arg(short, long, required = true)]
        output: String,
    },
    /// Randomize the syntax elements of a particular slice
    Randomize {
        /// Input H.264 file
        #[arg(short, long, required = true)]
        input: String,
        /// Output H.264 file
        #[arg(short, long, required = true)]
        output: String,
        /// Slice index to randomize
        #[arg(short = 'a', long = "slice-idx", default_value = "0")]
        slice_idx: usize,
        /// Randomize the slice header along a slice
        #[arg(long = "randomize-slice-header")]
        random_slice_header: bool,
        /// Randomize all slices
        #[arg(long = "randomize-all-slices")]
        random_all_slices: bool,
        /// Ignores intra prediction in video generation
        #[arg(long = "ignore-intra-pred")]
        ignore_intra_pred: bool,
        /// Ignores intra prediction along the edges of the video
        #[arg(long = "ignore-edge-intra-pred")]
        ignore_edge_intra_pred: bool,
        /// Ignores IPCM Macroblock types
        #[arg(long = "ignore-ipcm")]
        ignore_ipcm: bool,
        /// Seed value for the RNG
        #[arg(short = 's', long)]
        seed: Option<u64>,
        /// Path to configuration file containing the ranges to use in random video generation
        #[arg(short = 'c', long)]
        config: Option<String>,
        /// Produce videos by reading from a film file rather than sampling from a random number generator
        #[arg(long = "film")]
        film_file: Option<String>,
        /// Save the film file that was used to generate a video
        #[arg(long = "output-film")]
        output_film: bool,
    },
    /// Apply a video transform to an input video
    Modify {
        /// Input H.264 file
        #[arg(short, long, required = true)]
        input: String,
        /// Output H.264 file
        #[arg(short, long, required = true)]
        output: String,
        /// Path to a Python video transform
        #[arg(short = 't', long = "transform", required = true)]
        vid_mod_file: String,
        /// Argument for vid_mod_file
        #[arg(short = 'a', long, allow_hyphen_values = true, default_value = "0")]
        arg: i32,
    },
    /// Generate a new random video
    Generate {
        /// Output H.264 file
        #[arg(short, long, required = true)]
        output: String,
        /// Ignores intra prediction in video generation
        #[arg(long = "ignore-intra-pred")]
        ignore_intra_pred: bool,
        /// Ignores intra prediction along the edges of the video
        #[arg(long = "ignore-edge-intra-pred")]
        ignore_edge_intra_pred: bool,
        /// Ignores IPCM Macroblock types
        #[arg(long = "ignore-ipcm")]
        ignore_ipcm: bool,
        /// Limit the produced video to be at most than 128x128 pixels
        #[arg(long = "small")]
        property_small_video: bool,
        /// Produce a video that has empty slice data - i.e. all MBs containing no residue
        #[arg(long = "empty-slice-data")]
        property_empty_slice_data: bool,
        /// Incorporate undefined NALUs (e.g., 17, 18, 22-31) into generated video
        #[arg(long = "include-undefined-nalus")]
        include_undefined_nalus: bool,
        /// Seed value for the RNG
        #[arg(short = 's', long)]
        seed: Option<u64>,
        /// Path to configuration file containing the ranges to use in random video generation
        #[arg(short = 'c', long)]
        config: Option<String>,
        /// Produce videos by reading from a film file rather than sampling from a random number generator
        #[arg(long = "film")]
        film_file: Option<String>,
        /// Save the film file that was used to generate a video
        #[arg(long = "output-film")]
        output_film: bool,
    },
    /// Mux an encoded H.264 video into an MP4
    Mux {
        /// Input H.264 file
        #[arg(short, long, required = true)]
        input: String,
        /// Output MP4 file
        #[arg(short, long, required = true)]
        output: String,
    },
    /// Experimental features
    Experimental {
        /// Input H.265 file
        #[arg(short, long, required = true)]
        input: String,
        /// Output H.265 file
        #[arg(short, long, required = true)]
        output: String,
    },
}

impl H26ForgeOptions {
    fn encoder_debug_print(&self) {
        debug!(target: "encode"," - print_silent: {}", self.print_silent);
        debug!(target: "encode"," - print_perf: {}", self.print_perf);

        debug!(target: "encode"," - input_is_hevc: {}", self.input_is_hevc);
        debug!(target: "encode"," - decode_strict_fmo: {}", self.decode_strict_fmo);
        debug!(target: "encode"," - include_safestart: {}", self.include_safestart);

        debug!(target: "encode"," - output_syntax_json: {}", self.output_syntax_json);
        debug!(target: "encode"," - output_no_nalu_elements: {}", self.output_no_nalu_elements);
        debug!(target: "encode"," - output_avcc: {}", self.output_avcc);

        debug!(target: "encode"," - output_mp4: {}", self.output_mp4);
        debug!(target: "encode"," - output_mp4_fragment: {}", self.output_mp4_fragment);
        debug!(target: "encode"," - output_mp4_randomsize: {}", self.output_mp4_randomsize);
        debug!(target: "encode"," - output_mp4_width: {}", self.output_mp4_width);
        debug!(target: "encode"," - output_mp4_height: {}", self.output_mp4_height);
        debug!(target: "encode"," - output_rtp: {}", self.output_rtp);
    }
}

/// Create debug files for decoder and encoders
fn setup_debug_file(
    decoder_debug: bool,
    encoder_debug: bool,
    input_filename: &str,
    output_filename: &str,
) -> Result<(), SetLoggerError> {
    let cur_time: u64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    // Decoder
    let decoder_level = LevelFilter::Debug;
    let decoder_file_path = format!("{}.h26forge-decoder.{}.log", input_filename, cur_time);

    // Encoder
    let encoder_level = LevelFilter::Debug;
    let encoder_file_path = format!("{}.h26forge-encoder.{}.log", output_filename, cur_time);

    let pattern = "[{l}]\t{m}\n";

    if decoder_debug && !encoder_debug {
        println!("Saving decoder debug log to {}", decoder_file_path);
        let decoder_logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(pattern)))
            .build(decoder_file_path)
            .unwrap();

        let config = Config::builder()
            .appender(Appender::builder().build("decode", Box::new(decoder_logfile)))
            .logger(
                Logger::builder()
                    .appender("decode")
                    .build("decode", decoder_level),
            )
            .build(Root::builder().build(LevelFilter::Off))
            .unwrap();

        let _handle = log4rs::init_config(config)?;
    } else if !decoder_debug && encoder_debug {
        println!("Saving encoder debug log to {}", encoder_file_path);
        let encoder_logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(pattern)))
            .build(encoder_file_path)
            .unwrap();

        let config = Config::builder()
            .appender(Appender::builder().build("encode", Box::new(encoder_logfile)))
            .logger(
                Logger::builder()
                    .appender("encode")
                    .build("encode", encoder_level),
            )
            .build(Root::builder().build(LevelFilter::Off))
            .unwrap();

        let _handle = log4rs::init_config(config)?;
    } else if decoder_debug && encoder_debug {
        println!("Saving decoder debug log to {}", decoder_file_path);
        println!("Saving encoder debug log to {}", encoder_file_path);
        let decoder_logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(pattern)))
            .build(decoder_file_path)
            .unwrap();
        let encoder_logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(pattern)))
            .build(encoder_file_path)
            .unwrap();

        let config = Config::builder()
            .appender(Appender::builder().build("decode", Box::new(decoder_logfile)))
            .appender(Appender::builder().build("encode", Box::new(encoder_logfile)))
            .logger(
                Logger::builder()
                    .appender("decode")
                    .build("decode", decoder_level),
            )
            .logger(
                Logger::builder()
                    .appender("encode")
                    .build("encode", encoder_level),
            )
            .build(Root::builder().build(LevelFilter::Off))
            .unwrap();

        let _handle = log4rs::init_config(config)?;
    }

    Ok(())
}

/// Given a H264DecodedStream object in json format, output an encoded bitstream
fn mode_synthesize(input_filename: &str, output_filename: &str, options: &H26ForgeOptions) {
    // 1. Use the passed in file to get the decoded_elements
    println!("1. Decoding JSON into H.264 Syntax Elements");
    let mut decoded_elements = vidgen::syntax_to_video::syntax_to_video(input_filename);
    let (width, height) = decoded_elements.spses[0].get_framesize();

    if options.output_syntax_json {
        let filename = format!("{}.new.json", output_filename);
        println!("\t Saving new JSON of generated video to {}", filename);
        vidgen::syntax_to_video::video_to_syntax(
            &decoded_elements,
            options.output_no_nalu_elements,
            filename.as_str(),
        );
    }

    // 2. Encode the file
    println!("2. Writing out Mutated H.264 File");
    let res = encoder::encoder::reencode_syntax_elements(
        &mut decoded_elements,
        options.output_cut,
        options.output_avcc,
        options.print_silent,
        options.output_rtp,
    );
    let encoded_str = res.0;
    let avcc_encoding = res.1;
    let rtp = res.2;
    encoder::encoder::save_encoded_stream(
        encoded_str,
        avcc_encoding,
        output_filename,
        width,
        height,
        options.output_mp4,
        options.output_mp4_fragment,
        false,
        options.output_avcc,
        options.include_safestart,
        rtp,
    );
}

/// Given an input seed, randomize elements inside the video and output an encoded bitstream
fn mode_randomize(
    input_filename: &str,
    output_filename: &str,
    slice_idx: usize,
    use_seed: bool,
    use_film_file: bool,
    film_file: &str,
    manual_seed: u64,
    rconfig: vidgen::generate_configurations::RandomizeConfig,
    random_all_slices: bool,
    ignore_intra_pred: bool,
    ignore_edge_intra_pred: bool,
    ignore_ipcm: bool,
    randomize_header: bool,
    output_film: bool,
    options: &H26ForgeOptions,
) {
    // 1. Decode the bitstream to get the Syntax Elements
    println!("1. Decoding bitstream into H.264 Syntax Elements");
    let mut decoded_elements = decoder::decoder::decode_bitstream(
        input_filename,
        false,
        options.print_perf,
        options.decode_strict_fmo,
    );

    println!("2. Randomly mutating H.264 Syntax Elements");

    let mut film_state: vidgen::film::FilmState;

    if use_seed && use_film_file {
        film_state = vidgen::film::FilmState::setup_film_from_file_and_seed(film_file, manual_seed);
    } else if use_seed && !use_film_file {
        film_state = vidgen::film::FilmState::setup_film_from_seed(manual_seed);
    } else if !use_seed && use_film_file {
        film_state = vidgen::film::FilmState::setup_film_from_file(film_file);
    } else {
        film_state = vidgen::film::FilmState::setup_film();
    }

    println!("\t seed value: {}", film_state.seed);
    debug!(target: "encode","Randomly generated video seed value: {}", film_state.seed);
    debug!(target: "encode","Flags:");
    options.encoder_debug_print();
    debug!(target: "encode"," - random_all_slices : {}", random_all_slices);
    debug!(target: "encode"," - ignore_intra_pred : {}", ignore_intra_pred);
    debug!(target: "encode"," - ignore_edge_intra_pred : {}", ignore_edge_intra_pred);
    debug!(target: "encode"," - ignore_ipcm : {}", ignore_ipcm);
    debug!(target: "encode"," - randomize_header : {}", randomize_header);
    debug!(target: "encode"," - output_film : {}", output_film);

    if random_all_slices {
        println!("2.1 Mutating all slices");

        let mut slice_count = 0;
        for nalu_idx in 0..decoded_elements.nalu_headers.len() {
            if decoded_elements.nalu_headers[nalu_idx].nal_unit_type == 1
                || decoded_elements.nalu_headers[nalu_idx].nal_unit_type == 5
            {
                // choose pps and sps to use
                let mut pps_idx = 0;
                let mut sps_idx = 0;
                while pps_idx < decoded_elements.ppses.len() {
                    if decoded_elements.ppses[pps_idx].pic_parameter_set_id
                        == decoded_elements.slices[slice_idx].sh.pic_parameter_set_id
                    {
                        break;
                    }
                    pps_idx += 1;
                }
                if pps_idx == decoded_elements.ppses.len() {
                    panic!(
                        "mode_randomize - PPS with id {} not found",
                        decoded_elements.slices[slice_idx].sh.pic_parameter_set_id
                    );
                }

                while sps_idx < decoded_elements.spses.len() {
                    if decoded_elements.spses[sps_idx].seq_parameter_set_id
                        == decoded_elements.ppses[pps_idx].seq_parameter_set_id
                    {
                        break;
                    }
                    sps_idx += 1;
                }
                if sps_idx == decoded_elements.spses.len() {
                    panic!(
                        "mode_randomize - SPS with id {} not found",
                        decoded_elements.ppses[pps_idx].seq_parameter_set_id
                    );
                }
                let cur_sps = &decoded_elements.spses[sps_idx].clone();
                let cur_pps = &decoded_elements.ppses[pps_idx].clone();
                let empty_slice_data = false;
                vidgen::slice::random_slice(
                    nalu_idx,
                    slice_count,
                    cur_pps,
                    cur_sps,
                    ignore_intra_pred,
                    ignore_edge_intra_pred,
                    ignore_ipcm,
                    empty_slice_data,
                    randomize_header,
                    options.print_silent,
                    &rconfig,
                    &mut decoded_elements,
                    &mut film_state,
                );
                slice_count += 1;
            }
        }
    } else {
        let mut slice_count = 0;
        for nalu_idx in 0..decoded_elements.nalu_headers.len() {
            if decoded_elements.nalu_headers[nalu_idx].nal_unit_type == 1
                || decoded_elements.nalu_headers[nalu_idx].nal_unit_type == 5
            {
                if slice_count == slice_idx {
                    // choose pps and sps to use
                    let mut pps_idx = 0;
                    let mut sps_idx = 0;
                    while pps_idx < decoded_elements.ppses.len() {
                        if decoded_elements.ppses[pps_idx].pic_parameter_set_id
                            == decoded_elements.slices[slice_idx].sh.pic_parameter_set_id
                        {
                            break;
                        }
                        pps_idx += 1;
                    }
                    if pps_idx == decoded_elements.ppses.len() {
                        panic!(
                            "mode_randomize - PPS with id {} not found",
                            decoded_elements.slices[slice_idx].sh.pic_parameter_set_id
                        );
                    }

                    while sps_idx < decoded_elements.spses.len() {
                        if decoded_elements.spses[sps_idx].seq_parameter_set_id
                            == decoded_elements.ppses[pps_idx].seq_parameter_set_id
                        {
                            break;
                        }
                        sps_idx += 1;
                    }
                    if sps_idx == decoded_elements.spses.len() {
                        panic!(
                            "mode_randomize - SPS with id {} not found",
                            decoded_elements.ppses[pps_idx].seq_parameter_set_id
                        );
                    }
                    let cur_sps = &decoded_elements.spses[sps_idx].clone();
                    let cur_pps = &decoded_elements.ppses[pps_idx].clone();
                    let empty_slice_data = false;

                    // Randomize the header first
                    let param_sets_exist = true;
                    let enable_extensions = true;
                    let undefined_nalus = false;

                    vidgen::nalu::random_nalu_header(
                        nalu_idx,
                        param_sets_exist,
                        enable_extensions,
                        undefined_nalus,
                        &rconfig.random_nalu_range,
                        &mut decoded_elements,
                        &mut film_state,
                    );

                    // set the header to a slice type if not already
                    if decoded_elements.nalu_headers[nalu_idx].nal_unit_type != 1
                        && decoded_elements.nalu_headers[nalu_idx].nal_unit_type != 5
                    {
                        // randomly assign to 1 or 5
                        if rconfig
                            .random_nalu_range
                            .bias_idr_nalu
                            .sample(&mut film_state)
                        {
                            decoded_elements.nalu_headers[nalu_idx].nal_unit_type = 5;
                        } else {
                            decoded_elements.nalu_headers[nalu_idx].nal_unit_type = 1;
                        }
                    }

                    // Randomize the slice
                    vidgen::slice::random_slice(
                        nalu_idx,
                        slice_idx,
                        cur_pps,
                        cur_sps,
                        ignore_intra_pred,
                        ignore_edge_intra_pred,
                        ignore_ipcm,
                        empty_slice_data,
                        randomize_header,
                        options.print_silent,
                        &rconfig,
                        &mut decoded_elements,
                        &mut film_state,
                    );
                    break;
                }
                slice_count += 1;
            }
        }
    }

    if options.output_syntax_json {
        let filename = format!("{}.json", output_filename);
        println!("\t Saving JSON of randomly generated video to {}", filename);
        vidgen::syntax_to_video::video_to_syntax(
            &decoded_elements,
            options.output_no_nalu_elements,
            filename.as_str(),
        );
    }

    let (mut width, mut height) = match options.output_mp4_randomsize && options.output_mp4 {
        true => {
            let width = rconfig
                .random_video_config
                .mp4_width
                .sample(&mut film_state);
            let height = rconfig
                .random_video_config
                .mp4_height
                .sample(&mut film_state);
            if !options.print_silent {
                println!(
                    "\t Setting random MP4 Width x Height: {} x {}",
                    width, height
                );
            }
            (width as i32, height as i32)
        } // max usual support is 8k video
        false => decoded_elements.spses[0].get_framesize(),
    };

    if options.output_mp4_width > -1 {
        if !options.print_silent {
            println!(
                "\t Overwriting the MP4 width with passed in value: {}",
                options.output_mp4_width
            );
        }
        width = options.output_mp4_width;
    }

    if options.output_mp4_height > -1 {
        if !options.print_silent {
            println!(
                "\t Overwriting the MP4 height with passed in value: {}",
                options.output_mp4_height
            );
        }
        height = options.output_mp4_height;
    }

    if output_film {
        if !options.print_silent {
            println!("\t Saving film file!");
        }
        film_state.save_film(output_filename);
    }

    // 3. Re-encode the file
    println!("3. Writing out Mutated H.264 File");
    let res = encoder::encoder::reencode_syntax_elements(
        &mut decoded_elements,
        options.output_cut,
        options.output_avcc,
        options.print_silent,
        options.output_rtp,
    );
    let encoded_str = res.0;
    let avcc_encoding = res.1;
    let rtp = res.2;
    encoder::encoder::save_encoded_stream(
        encoded_str,
        avcc_encoding,
        output_filename,
        width,
        height,
        options.output_mp4,
        options.output_mp4_fragment,
        false,
        options.output_avcc,
        options.include_safestart,
        rtp,
    );
}

/// Given an input video and modification file, output an encoded bitstream of the modified video
fn mode_modify(
    input_filename: &str,
    output_filename: &str,
    mod_file: &str,
    args: i32,
    options: &H26ForgeOptions,
) {
    // 1. Decode the bitstream to get the Syntax Elements
    println!("1. Decoding bitstream into H.264 Syntax Elements");
    let mut decoded_elements = decoder::decoder::decode_bitstream(
        input_filename,
        false,
        options.print_perf,
        options.decode_strict_fmo,
    );

    println!("2. Mutating H.264 Syntax Elements with provided code");

    let mut success = true;

    if !mod_file.eq_ignore_ascii_case("") {
        success =
            vidgen::modify_video::perform_video_modification(mod_file, args, &mut decoded_elements);
    } else {
        println!("\t [WARNING] Video modification file required for this option - Skipping");
    }

    if success {
        let (width, height) = decoded_elements.spses[0].get_framesize();
        if options.output_syntax_json {
            let filename = format!("{}.json", output_filename);
            println!("\t Saving JSON of modified video to {}", filename);
            vidgen::syntax_to_video::video_to_syntax(
                &decoded_elements,
                options.output_no_nalu_elements,
                filename.as_str(),
            );
        }

        // 3. Re-encode the file
        println!("3. Writing out Mutated H.264 File");
        let res = encoder::encoder::reencode_syntax_elements(
            &mut decoded_elements,
            options.output_cut,
            options.output_avcc,
            options.print_silent,
            options.output_rtp,
        );
        let encoded_str = res.0;
        let avcc_encoding = res.1;
        let rtp = res.2;
        encoder::encoder::save_encoded_stream(
            encoded_str,
            avcc_encoding,
            output_filename,
            width,
            height,
            options.output_mp4,
            options.output_mp4_fragment,
            false,
            options.output_avcc,
            options.include_safestart,
            rtp,
        );
    } else {
        println!("Skipping writing out new file");
    }
}

/// Generate a completely random video without a seed
fn mode_generate(
    output_filename: &str,
    use_seed: bool,
    manual_seed: u64,
    rconfig: vidgen::generate_configurations::RandomizeConfig,
    use_film_file: bool,
    film_file: &str,
    ignore_intra_pred: bool,
    ignore_edge_intra_pred: bool,
    ignore_ipcm: bool,
    property_empty_slice_data: bool,
    property_small_video: bool,
    include_undefined_nalus: bool,
    output_film: bool,
    options: &H26ForgeOptions,
) {
    // 1. Generate video
    if !options.print_silent {
        println!("1. Generating random video");
    }
    let mut film_state: vidgen::film::FilmState;
    let start_time = SystemTime::now();
    if use_seed && use_film_file {
        film_state = vidgen::film::FilmState::setup_film_from_file_and_seed(film_file, manual_seed);
    } else if use_seed && !use_film_file {
        film_state = vidgen::film::FilmState::setup_film_from_seed(manual_seed);
    } else if !use_seed && use_film_file {
        film_state = vidgen::film::FilmState::setup_film_from_file(film_file);
    } else {
        film_state = vidgen::film::FilmState::setup_film();
    }
    if options.print_perf {
        let duration = start_time.elapsed();
        match duration {
            Ok(elapsed) => {
                println!(
                    "[PERF] mode_generate;setup FilmState;{} ns",
                    elapsed.as_nanos()
                );
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
    if !options.print_silent {
        println!("\t seed value: {}", film_state.seed);
    }
    debug!(target: "encode","Randomly generated video seed value: {}", film_state.seed);
    debug!(target: "encode","Flags:");
    options.encoder_debug_print();
    debug!(target: "encode"," - ignore_intra_pred : {}", ignore_intra_pred);
    debug!(target: "encode"," - ignore_edge_intra_pred : {}", ignore_edge_intra_pred);
    debug!(target: "encode"," - ignore_ipcm : {}", ignore_ipcm);
    debug!(target: "encode"," - property_empty_slice_data : {}", property_empty_slice_data);
    debug!(target: "encode"," - property_small_video : {}", property_small_video);
    debug!(target: "encode"," - include_undefined_nalus : {}", include_undefined_nalus);
    debug!(target: "encode"," - output_film : {}", output_film);

    let start_time = SystemTime::now();
    let mut decoded_elements = vidgen::vidgen::random_video(
        ignore_intra_pred,
        ignore_edge_intra_pred,
        ignore_ipcm,
        property_empty_slice_data,
        property_small_video,
        options.print_silent,
        include_undefined_nalus,
        &rconfig,
        &mut film_state,
    );

    if options.print_perf {
        let duration = start_time.elapsed();
        match duration {
            Ok(elapsed) => {
                println!(
                    "[PERF] mode_generate;random_video;{} ns",
                    elapsed.as_nanos()
                );
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }

    if options.output_syntax_json {
        let filename = format!("{}.json", output_filename);
        if !options.print_silent {
            println!("\t Saving JSON of randomly generated video to {}", filename);
        }
        vidgen::syntax_to_video::video_to_syntax(
            &decoded_elements,
            options.output_no_nalu_elements,
            filename.as_str(),
        );
    }

    // call this before saving the film
    let (mut width, mut height) = match options.output_mp4_randomsize && options.output_mp4 {
        true => {
            let width = rconfig
                .random_video_config
                .mp4_width
                .sample(&mut film_state);
            let height = rconfig
                .random_video_config
                .mp4_height
                .sample(&mut film_state);
            if !options.print_silent {
                println!(
                    "\t Setting random MP4 Width x Height: {} x {}",
                    width, height
                );
            }
            (width as i32, height as i32)
        } // max usual support is 8k video
        false => decoded_elements.spses[0].get_framesize(),
    };

    if options.output_mp4_width > -1 {
        if !options.print_silent {
            println!(
                "\t Overwriting the MP4 width with passed in value: {}",
                options.output_mp4_width
            );
        }
        width = options.output_mp4_width;
    }

    if options.output_mp4_height > -1 {
        if !options.print_silent {
            println!(
                "\t Overwriting the MP4 height with passed in value: {}",
                options.output_mp4_height
            );
        }
        height = options.output_mp4_height;
    }

    if output_film {
        if !options.print_silent {
            println!("\t Saving film file!");
        }
        film_state.save_film(output_filename);
    }

    // 2. Re-encode the file
    if !options.print_silent {
        println!("2. Writing out Mutated H.264 File");
    }
    let start_time = SystemTime::now();
    let res = encoder::encoder::reencode_syntax_elements(
        &mut decoded_elements,
        options.output_cut,
        options.output_avcc,
        options.print_silent,
        options.output_rtp,
    );
    if options.print_perf {
        let duration = start_time.elapsed();
        match duration {
            Ok(elapsed) => {
                println!(
                    "[PERF] mode_generate;reencode_syntax_elements;{} ns",
                    elapsed.as_nanos()
                );
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
    let encoded_str = res.0;
    let avcc_encoding = res.1;
    let rtp = res.2;
    let start_time = SystemTime::now();
    encoder::encoder::save_encoded_stream(
        encoded_str,
        avcc_encoding,
        output_filename,
        width,
        height,
        options.output_mp4,
        options.output_mp4_fragment,
        false,
        options.output_avcc,
        options.include_safestart,
        rtp,
    );
    if options.print_perf {
        let duration = start_time.elapsed();
        match duration {
            Ok(elapsed) => {
                println!(
                    "[PERF] mode_generate;save_encoded_stream;{} ns",
                    elapsed.as_nanos()
                );
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}

fn main() {
    let options = H26ForgeOptions::parse();

    if options.save_default_config {
        println!("[Default Config] Outputting Video Generation default configuration");
        vidgen::generate_configurations::save_config();
        println!("[Default Config] Done: available at ./default.json");
    }

    match &options.mode {
        Some(Commands::Passthrough {
            input,
            output,
            decode_only_headers,
        }) => {
            if options.debug_decode || options.debug_encode {
                let res =
                    setup_debug_file(options.debug_decode, options.debug_encode, input, output);
                match res {
                    Ok(_) => println!("Set up debug logs"),
                    _ => println!("Issue setting up debug logs"),
                }
            }

            if *decode_only_headers && !options.print_silent {
                println!("\tOnly running headers through passthrough");
            }

            if !Path::new(input).exists() {
                println!("ERROR - unable to find {}", input);
                std::process::exit(1);
            }

            if !options.print_silent {
                println!("Using input file: {}", input);

                println!("Running in passthrough mode");
                // 1. Decode the bitstream to get the Syntax Elements
                println!("1. Decoding bitstream into H.264 Syntax Elements");
            }

            let start_time = SystemTime::now();
            let mut decoded_elements = decoder::decoder::decode_bitstream(
                input,
                *decode_only_headers,
                options.print_perf,
                options.decode_strict_fmo,
            );
            let (width, height) = decoded_elements.spses[0].get_framesize();
            if options.print_perf {
                let duration = start_time.elapsed();
                match duration {
                    Ok(elapsed) => {
                        println!(
                            "[PERF] main_passthrough;decode_bitstream;{} ns",
                            elapsed.as_nanos()
                        );
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }

            // 2. Re-encode the file
            if !options.print_silent {
                println!("2. Writing out unmodified H.264 File: {}", output);
            }

            if options.output_syntax_json {
                let filename = format!("{}.json", output);
                println!("\t Saving JSON of modified video to {}", filename);
                vidgen::syntax_to_video::video_to_syntax(
                    &decoded_elements,
                    options.output_no_nalu_elements,
                    filename.as_str(),
                );
            }

            let start_time = SystemTime::now();
            let res = encoder::encoder::reencode_syntax_elements(
                &mut decoded_elements,
                options.output_cut,
                options.output_avcc,
                options.print_silent,
                options.output_rtp,
            );

            if options.print_perf {
                let duration = start_time.elapsed();
                match duration {
                    Ok(elapsed) => {
                        println!(
                            "[PERF] main_passthrough;reencode_syntax_elements;{} ns",
                            elapsed.as_nanos()
                        );
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }
            let encoded_str = res.0;
            let avcc_encoding = res.1;
            let rtp = res.2;
            let start_time = SystemTime::now();
            encoder::encoder::save_encoded_stream(
                encoded_str,
                avcc_encoding,
                output,
                width,
                height,
                options.output_mp4,
                options.output_mp4_fragment,
                false,
                options.output_avcc,
                options.include_safestart,
                rtp,
            );
            if options.print_perf {
                let duration = start_time.elapsed();
                match duration {
                    Ok(elapsed) => {
                        println!(
                            "[PERF] main_passthrough;save_encoded_stream;{} ns",
                            elapsed.as_nanos()
                        );
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }
        }
        Some(Commands::Modify {
            input,
            output,
            vid_mod_file,
            arg,
        }) => {
            if options.debug_decode || options.debug_encode {
                let res =
                    setup_debug_file(options.debug_decode, options.debug_encode, input, output);
                match res {
                    Ok(_) => println!("Set up debug logs"),
                    _ => println!("Issue setting up debug logs"),
                }
            }

            if !options.print_silent {
                println!("Using input file: {}", input);

                println!("Running in modify mode");
            }

            mode_modify(input, &output, &vid_mod_file, *arg, &options);
        }
        Some(Commands::Generate {
            output,
            ignore_intra_pred,
            ignore_edge_intra_pred,
            ignore_ipcm,
            property_small_video,
            property_empty_slice_data,
            include_undefined_nalus,
            seed,
            config,
            film_file,
            output_film,
        }) => {
            if options.debug_encode {
                let res = setup_debug_file(false, options.debug_encode, "", output);
                match res {
                    Ok(_) => println!("Set up debug logs"),
                    _ => println!("Issue setting up debug logs"),
                }
            }

            if !options.print_silent {
                println!("Generating a new video");
            }

            let use_seed;

            let manual_seed = match seed {
                Some(x) => {
                    use_seed = true;
                    *x
                }
                None => {
                    use_seed = false;
                    0
                }
            };

            let rconfig = match config {
                Some(x) => {
                    if !options.print_silent {
                        println!("\t loading config file {}", x);
                    }
                    debug!(target: "encode","\t loading config file {}", x);
                    vidgen::generate_configurations::load_config(x)
                }
                _ => {
                    if !options.print_silent {
                        println!("\t using default random value ranges");
                    }
                    debug!(target: "encode","\t using default random value ranges");
                    vidgen::generate_configurations::RandomizeConfig::new()
                }
            };

            let use_film_file;
            let film_file = match film_file {
                Some(x) => {
                    use_film_file = true;
                    if !options.print_silent {
                        println!("Using film file {}", x);
                    }
                    x
                }
                _ => {
                    use_film_file = false;
                    ""
                }
            };

            if use_film_file && use_seed {
                println!("[WARNING] Passed both a film file and a random seed --- defaulting to file; seed will be used if file terminates early");
            }

            let start_time = SystemTime::now();
            mode_generate(
                output,
                use_seed,
                manual_seed,
                rconfig,
                use_film_file,
                film_file,
                *ignore_intra_pred,
                *ignore_edge_intra_pred,
                *ignore_ipcm,
                *property_empty_slice_data,
                *property_small_video,
                *include_undefined_nalus,
                *output_film,
                &options,
            );
            if options.print_perf {
                let duration = start_time.elapsed();
                match duration {
                    Ok(elapsed) => {
                        println!(
                            "[PERF] main_generate;mode_generate;{} ns",
                            elapsed.as_nanos()
                        );
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }
        }
        Some(Commands::Randomize {
            input,
            output,
            slice_idx,
            random_slice_header,
            random_all_slices,
            ignore_intra_pred,
            ignore_edge_intra_pred,
            ignore_ipcm,
            seed,
            film_file,
            output_film,
            config,
        }) => {
            if options.debug_decode || options.debug_encode {
                let res =
                    setup_debug_file(options.debug_decode, options.debug_encode, input, output);
                match res {
                    Ok(_) => println!("Set up debug logs"),
                    _ => println!("Issue setting up debug logs"),
                }
            }

            if !options.print_silent {
                println!("Using input file: {}", input);

                println!("Running in randomize mode");
            }

            let use_seed;

            let seed = match seed {
                Some(x) => {
                    use_seed = true;
                    *x
                }
                _ => {
                    use_seed = false;
                    0
                }
            };

            let use_film_file;
            let film_file = match film_file {
                Some(x) => {
                    use_film_file = true;
                    if !options.print_silent {
                        println!("Using film file {}", x);
                    }
                    x
                }
                _ => {
                    use_film_file = false;
                    ""
                }
            };

            if use_film_file && use_seed {
                println!("[WARNING] Passed both a film file and a random seed --- defaulting to file; seed will be used if file terminates early");
            }

            let rconfig = match config {
                Some(x) => {
                    if !options.print_silent {
                        println!("\t loading config file {}", x);
                    }
                    debug!(target: "encode","\t loading config file {}", x);
                    vidgen::generate_configurations::load_config(x)
                }
                _ => {
                    if !options.print_silent {
                        println!("\t using default random value ranges");
                    }
                    debug!(target: "encode","\t using default random value ranges");
                    vidgen::generate_configurations::RandomizeConfig::new()
                }
            };

            let start_time = SystemTime::now();
            mode_randomize(
                input,
                output,
                *slice_idx,
                use_seed,
                use_film_file,
                film_file,
                seed,
                rconfig,
                *random_all_slices,
                *ignore_intra_pred,
                *ignore_edge_intra_pred,
                *ignore_ipcm,
                *random_slice_header,
                *output_film,
                &options,
            );
            if options.print_perf {
                let duration = start_time.elapsed();
                match duration {
                    Ok(elapsed) => {
                        println!(
                            "[PERF] main_randomize;mode_randomize;{} ns",
                            elapsed.as_nanos()
                        );
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }
        }
        Some(Commands::Synthesize { input, output }) => {
            if options.debug_decode || options.debug_encode {
                let res =
                    setup_debug_file(options.debug_decode, options.debug_encode, input, output);
                match res {
                    Ok(_) => println!("Set up debug logs"),
                    _ => println!("Issue setting up debug logs"),
                }
            }

            if !options.print_silent {
                println!("Using input file: {}", *input);
                println!("Running in synth mode");
            }

            mode_synthesize(input, output, &options);
        }
        Some(Commands::Mux { input, output }) => {
            if !options.print_silent {
                println!("Using input file: {}", input);
                println!("Running in Wrap mode");
            }

            let encoded_str = std::fs::read(input).unwrap();

            let mut width = 720;
            let mut height = 480;

            if options.output_mp4_width > -1 {
                if !options.print_silent {
                    println!(
                        "\t Overwriting the MP4 width with passed in value: {}",
                        options.output_mp4_width
                    );
                }
                width = options.output_mp4_width;
            }

            if options.output_mp4_height > -1 {
                if !options.print_silent {
                    println!(
                        "\t Overwriting the MP4 height with passed in value: {}",
                        options.output_mp4_height
                    );
                }
                height = options.output_mp4_height;
            }

            encoder::encoder::save_mp4_file(
                output.to_string(),
                width,
                height,
                options.output_mp4_fragment,
                options.input_is_hevc,
                &encoded_str,
            )
        }
        Some(Commands::Experimental { input, output }) => {
            if !options.print_silent {
                println!("Using input file: {}", input);
                println!("Running in Wrap mode");
            }

            println!("1. Decoding H.265 Stream");
            let mut ds = experimental::h265_decoder::decode_bitstream(input, false);

            println!("2. Modifying H.265 Stream");

            //let pc_value = match matches.value_of("ARG") {
            //    Some(y) => {
            //        match y.parse::<u64>() {
            //            Ok(n) => n,
            //            _ => {println!("\t Argument must be a positive integer - defaulting to 0");
            //                0
            //            },
            //        }
            //    },
            //    _ => {
            //        println!("\t No pc_value provided, default to 0x4141414141414142");
            //        0x4141414141414142
            //    },
            //};
            //experimental::h265_modify::cve_2022_42850_exploit(pc_value, &mut ds);

            experimental::h265_modify::cve_2022_42850_poc(&mut ds);

            println!("2. Saving modified H.265 Stream");
            let start_time = SystemTime::now();
            let encoded_str = experimental::h265_encoder::reencode_syntax_elements(&mut ds);
            if options.print_perf {
                let duration = start_time.elapsed();
                match duration {
                    Ok(elapsed) => {
                        println!(
                            "[PERF] main_passthrough;reencode_syntax_elements;{} ns",
                            elapsed.as_nanos()
                        );
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }

            let width = 1920;
            let height = 1080;
            let start_time = SystemTime::now();
            let rtp:Vec<Vec<u8>> = Vec::new();
            encoder::encoder::save_encoded_stream(
                encoded_str,
                common::data_structures::AVCCFormat::new(),
                output,
                width,
                height,
                options.output_mp4,
                options.output_mp4_fragment,
                true,
                false,
                false,
                rtp,
            );

            if options.print_perf {
                let duration = start_time.elapsed();
                match duration {
                    Ok(elapsed) => {
                        println!(
                            "[PERF] main_passthrough;save_encoded_stream;{} ns",
                            elapsed.as_nanos()
                        );
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
            }
        }
        None => {
            println!("\tNo mode provided");
        }
    };
}
