//! Random video generation entry point.

use crate::common::data_structures::AccessUnitDelim;
use crate::common::data_structures::H264DecodedStream;
use crate::common::data_structures::MacroBlock;
use crate::common::data_structures::NALUheader;
use crate::common::data_structures::PicParameterSet;
use crate::common::data_structures::PrefixNALU;
use crate::common::data_structures::SEINalu;
use crate::common::data_structures::SeqParameterSet;
use crate::common::data_structures::Slice;
use crate::common::data_structures::SubsetSPS;
use crate::common::data_structures::NALU;
use crate::vidgen::film::FilmState;
use crate::vidgen::generate_configurations::RandomizeConfig;
use crate::vidgen::nalu::random_access_unit_delimiter;
use crate::vidgen::nalu::random_nalu_header;
use crate::vidgen::nalu::random_prefix_nalu;
use crate::vidgen::parameter_sets::random_pps;
use crate::vidgen::parameter_sets::random_sps;
use crate::vidgen::parameter_sets::random_subset_sps;
use crate::vidgen::sei::random_sei;
use crate::vidgen::slice::random_slice;
use crate::vidgen::slice::random_slice_layer_extension;

/// Generate a random video
pub fn random_video(
    ignore_intra_pred: bool,
    ignore_edge_intra_pred: bool,
    ignore_ipcm: bool,
    empty_slice_data: bool,
    small_video: bool,
    silent_mode: bool,
    undefined_nalus: bool,
    rconfig: &RandomizeConfig,
    film: &mut FilmState,
) -> H264DecodedStream {
    let number_nalus = rconfig.random_video_config.num_nalus.sample(film) as usize;
    let enable_extensions = rconfig.random_video_config.enable_extensions.sample(film);

    let mut ds = H264DecodedStream::new();

    let mut prefix_nalu_idx = 0;
    let mut sps_idx = 0;
    let mut subset_sps_idx = 0;
    let mut pps_idx = 0;
    let mut sei_idx = 0;
    let mut slice_idx = 0;
    let mut aud_idx = 0;

    let mut generated_nalu_type_str = String::new();

    for nalu_idx in 0..number_nalus {
        ds.nalu_elements.push(NALU::new());
        ds.nalu_headers.push(NALUheader::new());
        let param_sets_exist = pps_idx > 0 && sps_idx > 0;
        random_nalu_header(
            nalu_idx,
            param_sets_exist,
            enable_extensions,
            undefined_nalus,
            &rconfig.random_nalu_range,
            &mut ds,
            film,
        );

        if nalu_idx == 0 {
            // The first SPS is forced to be an SPS
            ds.nalu_headers[nalu_idx].nal_unit_type = 7;
        } else if ds.nalu_headers[nalu_idx - 1].nal_unit_type == 7
            || ds.nalu_headers[nalu_idx - 1].nal_unit_type == 15
        {
            // all SPS or subsetSPS should be followed by a PPS
            ds.nalu_headers[nalu_idx].nal_unit_type = 8;
        }

        // set the first frame to an IDR one for now
        if ds.nalu_headers[nalu_idx].nal_unit_type == 1 && slice_idx == 0 {
            ds.nalu_headers[nalu_idx].nal_unit_type = 5;
        }

        // if we have a coded slice extension without a subsetSPS then we'll create a new subsetSPS
        // TODO: create an unreferenced coded slice extension
        if ds.nalu_headers[nalu_idx].nal_unit_type == 20 && subset_sps_idx == 0 {
            ds.nalu_headers[nalu_idx].nal_unit_type = 15;
        }

        match ds.nalu_headers[nalu_idx].nal_unit_type {
            1 | 5 => {
                // slices
                if !silent_mode {
                    println!(
                        "\t random_video - NALU {} - Generating Coded slice of {}",
                        nalu_idx,
                        match ds.nalu_headers[nalu_idx].nal_unit_type {
                            1 => "a non-IDR picture",
                            _ => "an IDR picture",
                        }
                    );
                }
                ds.slices.push(Slice::new());
                // for amount of macroblocks, assume frames only
                let macroblock_amount = ((ds.spses[sps_idx - 1].pic_width_in_mbs_minus1 + 1)
                    * (ds.spses[sps_idx - 1].pic_height_in_map_units_minus1 + 1))
                    as usize;

                ds.slices[slice_idx].sd.macroblock_vec = vec![MacroBlock::new(); macroblock_amount];
                // use the most recent PPS and SPS

                let cur_pps = &ds.ppses[pps_idx - 1].clone();
                let cur_sps: SeqParameterSet;
                if cur_pps.is_subset_pps {
                    cur_sps = ds.subset_spses[subset_sps_idx - 1].sps.clone();
                } else {
                    cur_sps = ds.spses[sps_idx - 1].clone();
                }

                let randomize_header = true;
                random_slice(
                    nalu_idx,
                    slice_idx,
                    cur_pps,
                    &cur_sps,
                    ignore_intra_pred,
                    ignore_edge_intra_pred,
                    ignore_ipcm,
                    empty_slice_data,
                    randomize_header,
                    silent_mode,
                    rconfig,
                    &mut ds,
                    film,
                );
                if ds.nalu_headers[nalu_idx].nal_unit_type == 1 {
                    generated_nalu_type_str += "Non-IDR Slice(1);";
                } else {
                    generated_nalu_type_str += "IDR Slice(5);";
                }
                slice_idx += 1;
            }
            6 => {
                if !silent_mode {
                    println!(
                        "\t random_video - NALU {} - Generating Supplemental Enhancement Info",
                        nalu_idx
                    );
                }
                ds.seis.push(SEINalu::new());
                random_sei(sei_idx, &rconfig.random_sei_range, &mut ds, film);
                generated_nalu_type_str += "SEI(6);";
                sei_idx += 1;
            }
            7 => {
                // SPS
                if !silent_mode {
                    println!(
                        "\t random_video - NALU {} - Generating Sequence Parameter Set",
                        nalu_idx
                    );
                }
                ds.spses.push(SeqParameterSet::new());
                random_sps(
                    &mut ds.spses[sps_idx],
                    enable_extensions,
                    &rconfig.random_sps_range,
                    small_video,
                    silent_mode,
                    film,
                );
                generated_nalu_type_str += "SPS(7);";
                sps_idx += 1;
            }
            8 => {
                // PPS
                if !silent_mode {
                    println!(
                        "\t random_video - NALU {} - Generating Picture Parameter Set",
                        nalu_idx
                    );
                }
                // use the most recent SPS
                // choose either the most recent SPS or subsetSPS
                let cur_sps: SeqParameterSet;
                ds.ppses.push(PicParameterSet::new());

                if ds.nalu_headers[nalu_idx - 1].nal_unit_type == 15 {
                    cur_sps = ds.subset_spses[subset_sps_idx - 1].sps.clone();
                    ds.ppses[pps_idx].is_subset_pps = true;
                } else {
                    cur_sps = ds.spses[sps_idx - 1].clone();
                }

                random_pps(pps_idx, &cur_sps, rconfig.random_pps_range, &mut ds, film);
                generated_nalu_type_str += "PPS(8);";
                pps_idx += 1;
            }
            9 => {
                // Access Unit Delimiter
                if !silent_mode {
                    println!(
                        "\t random_video - NALU {} - Access Unit Delimiter",
                        nalu_idx
                    );
                }
                ds.auds.push(AccessUnitDelim::new());
                random_access_unit_delimiter(
                    aud_idx,
                    rconfig.random_access_unit_delim_range,
                    &mut ds,
                    film,
                );
                aud_idx += 1;
                generated_nalu_type_str += "AUD(9);";
            }
            10 => {
                // End of sequence - signals the next NALU should be an IDR
                if !silent_mode {
                    println!("\t random_video - NALU {} - End of Sequence", nalu_idx);
                }
                generated_nalu_type_str += "EndOfSequence(10);";
                // TODO: nothing to do here, but could throw junk in the future
            }
            11 => {
                // End of Stream - signals
                if !silent_mode {
                    println!("\t random_video - NALU {} - End of Stream", nalu_idx);
                }
                generated_nalu_type_str += "EndOfStream(11);";
            }
            12 => {
                // Filler data RBSP - should be all 0xff

                let filler_data_length = rconfig
                    .random_nalu_range
                    .filler_data_nalu_length
                    .sample(film);
                if !silent_mode {
                    println!(
                        "\t random_video - NALU {} - Filler Data of length {}",
                        nalu_idx, filler_data_length
                    );
                }
                ds.nalu_elements[nalu_idx].content.push(0); // the first byte is the header, which is ignored
                ds.nalu_elements[nalu_idx]
                    .content
                    .extend(film.read_film_bytes(filler_data_length)); // the rest is random bytes of random length

                generated_nalu_type_str += "FillerData(12);";
            }
            14 => {
                // Prefix NALU
                if !silent_mode {
                    println!(
                        "\t random_video - NALU {} - Generating Prefix NALU",
                        nalu_idx
                    );
                }
                ds.prefix_nalus.push(PrefixNALU::new());
                random_prefix_nalu(
                    nalu_idx,
                    prefix_nalu_idx,
                    rconfig.random_prefix_nalu_range,
                    &mut ds,
                    film,
                );
                generated_nalu_type_str += "PrefixNALU(14);";
                prefix_nalu_idx += 1;
            }
            15 => {
                // Subset sps
                if !silent_mode {
                    println!(
                        "\t random_video - NALU {} - Generating Subset SPS",
                        nalu_idx
                    );
                }
                ds.subset_spses.push(SubsetSPS::new());
                random_subset_sps(
                    subset_sps_idx,
                    enable_extensions,
                    &rconfig.random_subset_sps_range,
                    small_video,
                    silent_mode,
                    &mut ds,
                    film,
                );
                generated_nalu_type_str += "SubsetSPS(15);";
                subset_sps_idx += 1;
            }
            20 => {
                // Coded slice extension
                if !silent_mode {
                    println!(
                        "\t random_video - NALU {} - Generating Coded Slice Extension",
                        nalu_idx
                    );
                }
                ds.slices.push(Slice::new());

                // Get the most recent PPS associated with a subset SPS, which
                // should also be the most recent subset SPS
                let mut subset_pps_idx: usize = ds.ppses.len() - 1;
                for i in (0..ds.ppses.len()).rev() {
                    if ds.ppses[i].is_subset_pps {
                        subset_pps_idx = i;
                        break;
                    }
                }

                // for amount of macroblocks, assume frames only
                let macroblock_amount = ((ds.subset_spses[subset_sps_idx - 1]
                    .sps
                    .pic_width_in_mbs_minus1
                    + 1)
                    * (ds.subset_spses[subset_sps_idx - 1]
                        .sps
                        .pic_height_in_map_units_minus1
                        + 1)) as usize;
                ds.slices[slice_idx].sd.macroblock_vec = vec![MacroBlock::new(); macroblock_amount];
                // use the most recent PPS and SPS
                let randomize_header = true;
                random_slice_layer_extension(
                    nalu_idx,
                    slice_idx,
                    subset_pps_idx,
                    subset_sps_idx - 1,
                    ignore_intra_pred,
                    ignore_edge_intra_pred,
                    ignore_ipcm,
                    empty_slice_data,
                    randomize_header,
                    silent_mode,
                    &rconfig,
                    &mut ds,
                    film,
                );
                generated_nalu_type_str += "CodedSliceExt(20);";
                slice_idx += 1;
            }
            0 | 17 | 18 | 22..=31 => {
                let random_byte_length =
                    rconfig.random_nalu_range.undefined_nalu_length.sample(film);
                if !silent_mode {
                    println!("\t random_video - NALU {} - Generating Undefined NALU type {} of length {}", nalu_idx, ds.nalu_headers[nalu_idx].nal_unit_type, random_byte_length);
                }
                ds.nalu_elements[nalu_idx].content.push(0); // the first byte is the header, which is ignored
                ds.nalu_elements[nalu_idx]
                    .content
                    .extend(film.read_film_bytes(random_byte_length));
                generated_nalu_type_str +=
                    &format!("Undefined({});", ds.nalu_headers[nalu_idx].nal_unit_type)[..];
            }
            _ => println!(
                "Not currently supported nal_unit_type {}",
                ds.nalu_headers[nalu_idx].nal_unit_type
            ),
        }
    }
    if !silent_mode {
        println!("\t Generated Sequence: {}", generated_nalu_type_str);
    }

    ds
}
