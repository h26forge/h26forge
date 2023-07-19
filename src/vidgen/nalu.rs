//! NALU header and extensions syntax element decoding

use crate::common::data_structures::H264DecodedStream;
use crate::vidgen::film::FilmState;
use crate::vidgen::generate_configurations::RandomAccessUnitDelim;
use crate::vidgen::generate_configurations::RandomNALUHeader;
use crate::vidgen::generate_configurations::RandomNALUHeader3DAVCExtension;
use crate::vidgen::generate_configurations::RandomNALUHeaderMVCExtension;
use crate::vidgen::generate_configurations::RandomNALUHeaderSVCExtension;
use crate::vidgen::generate_configurations::RandomPrefixNALU;

/// Sample a random NALU type
fn random_nal_unit_type(
    param_sets_exist: bool,
    enable_extensions: bool,
    undefined_nalus: bool,
    rconfig: &RandomNALUHeader,
    film: &mut FilmState,
) -> u8 {
    // if the parameter sets exist then let's bias towards slices
    let mut nalu = if param_sets_exist {
        if rconfig.bias_slice_nalu.sample(film) {
            rconfig.nal_unit_slice_type.sample(film) as u8
        } else {
            if enable_extensions {
                rconfig.nal_unit_extension_type.sample(film) as u8
            } else {
                rconfig.nal_unit_type.sample(film) as u8
            }
        }
    } else {
        if enable_extensions {
            rconfig.nal_unit_extension_type.sample(film) as u8
        } else {
            rconfig.nal_unit_type.sample(film) as u8
        }
    };

    // Only sample for undefined NALUs if the flag is set
    if undefined_nalus {
        if rconfig.bias_undefined_nalu.sample(film) {
            nalu = rconfig.nal_unit_undefined_type.sample(film) as u8;
        }
    }

    nalu
}

/// Generate a random NALU header. This determines the NALU Type
pub fn random_nalu_header(
    nalu_idx: usize,
    param_sets_exist: bool,
    enable_extensions: bool,
    undefined_nalus: bool,
    rconfig: &RandomNALUHeader,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    if nalu_idx >= ds.nalu_headers.len() {
        println!(
            "\t [WARNING] NALU index {} greater than number of nalu headers: {} - Skipping",
            nalu_idx,
            ds.nalu_headers.len()
        );
        return;
    }

    ds.nalu_headers[nalu_idx].forbidden_zero_bit = rconfig.forbidden_zero_bit.sample(film) as u8;
    ds.nalu_headers[nalu_idx].nal_ref_idc = rconfig.nal_ref_idc.sample(film) as u8;
    ds.nalu_headers[nalu_idx].nal_unit_type = random_nal_unit_type(
        param_sets_exist,
        enable_extensions,
        undefined_nalus,
        rconfig,
        film,
    );

    if ds.nalu_headers[nalu_idx].nal_unit_type == 14
        || ds.nalu_headers[nalu_idx].nal_unit_type == 20
        || ds.nalu_headers[nalu_idx].nal_unit_type == 21
    {
        if ds.nalu_headers[nalu_idx].nal_unit_type != 21 {
            ds.nalu_headers[nalu_idx].svc_extension_flag = rconfig.svc_extension_flag.sample(film);
        } else {
            ds.nalu_headers[nalu_idx].avc_3d_extension_flag =
                rconfig.avc_3d_extension_flag.sample(film);
        }

        if ds.nalu_headers[nalu_idx].svc_extension_flag {
            // specified in Annex G
            random_nal_unit_header_svc_extension(nalu_idx, rconfig.random_svc_extension, ds, film);
        } else if ds.nalu_headers[nalu_idx].avc_3d_extension_flag {
            // specified in Annex J
            random_nal_unit_header_3davc_extension(
                nalu_idx,
                rconfig.random_avc_3d_extension,
                ds,
                film,
            );
        } else {
            // specified in Annex H
            random_nal_unit_header_mvc_extension(nalu_idx, rconfig.random_mvc_extension, ds, film);
        }
    }
}

/// Generate a random NALU SVC Extension
fn random_nal_unit_header_svc_extension(
    nalu_idx: usize,
    rconfig: RandomNALUHeaderSVCExtension,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    ds.nalu_headers[nalu_idx].svc_extension.idr_flag = rconfig.idr_flag.sample(film);
    ds.nalu_headers[nalu_idx].svc_extension.priority_id = rconfig.priority_id.sample(film) as u8;
    ds.nalu_headers[nalu_idx]
        .svc_extension
        .no_inter_layer_pred_flag = rconfig.no_inter_layer_pred_flag.sample(film);
    ds.nalu_headers[nalu_idx].svc_extension.dependency_id =
        rconfig.dependency_id.sample(film) as u8;
    ds.nalu_headers[nalu_idx].svc_extension.quality_id = rconfig.quality_id.sample(film) as u8;
    ds.nalu_headers[nalu_idx].svc_extension.temporal_id = rconfig.temporal_id.sample(film) as u8;
    ds.nalu_headers[nalu_idx]
        .svc_extension
        .use_ref_base_pic_flag = rconfig.use_ref_base_pic_flag.sample(film);
    ds.nalu_headers[nalu_idx].svc_extension.discardable_flag =
        rconfig.discardable_flag.sample(film);
    ds.nalu_headers[nalu_idx].svc_extension.output_flag = rconfig.output_flag.sample(film);
    ds.nalu_headers[nalu_idx].svc_extension.reserved_three_2bits =
        rconfig.reserved_three_2bits.sample(film) as u8;
}

/// Generate a random NALU 3DAVC Extension
fn random_nal_unit_header_3davc_extension(
    nalu_idx: usize,
    rconfig: RandomNALUHeader3DAVCExtension,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    ds.nalu_headers[nalu_idx].avc_3d_extension.view_idx = rconfig.view_idx.sample(film) as u8;
    ds.nalu_headers[nalu_idx].avc_3d_extension.depth_flag = rconfig.depth_flag.sample(film);
    ds.nalu_headers[nalu_idx].avc_3d_extension.non_idr_flag = rconfig.non_idr_flag.sample(film);
    ds.nalu_headers[nalu_idx].avc_3d_extension.temporal_id = rconfig.temporal_id.sample(film) as u8;
    ds.nalu_headers[nalu_idx].avc_3d_extension.anchor_pic_flag =
        rconfig.anchor_pic_flag.sample(film);
    ds.nalu_headers[nalu_idx].avc_3d_extension.inter_view_flag =
        rconfig.inter_view_flag.sample(film);
}

/// Generate a random NALU MVC Extension
///
/// Described in H.7.3.1.1 NAL unit header MVC extension syntax
fn random_nal_unit_header_mvc_extension(
    nalu_idx: usize,
    rconfig: RandomNALUHeaderMVCExtension,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    ds.nalu_headers[nalu_idx].mvc_extension.non_idr_flag = rconfig.non_idr_flag.sample(film);
    ds.nalu_headers[nalu_idx].mvc_extension.priority_id = rconfig.priority_id.sample(film) as u8;
    ds.nalu_headers[nalu_idx].mvc_extension.view_id = rconfig.view_id.sample(film);
    ds.nalu_headers[nalu_idx].mvc_extension.temporal_id = rconfig.temporal_id.sample(film) as u8;
    ds.nalu_headers[nalu_idx].mvc_extension.anchor_pic_flag = rconfig.anchor_pic_flag.sample(film);
    ds.nalu_headers[nalu_idx].mvc_extension.inter_view_flag = rconfig.inter_view_flag.sample(film);
    ds.nalu_headers[nalu_idx].mvc_extension.reserved_one_bit =
        rconfig.reserved_one_bit.sample(film);
}

/// Generate a random Prefix NALU (NALU Type 14)
pub fn random_prefix_nalu(
    nalu_idx: usize,
    prefix_nalu_idx: usize,
    rconfig: RandomPrefixNALU,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    if ds.nalu_headers[nalu_idx].nal_ref_idc != 0 {
        ds.prefix_nalus[prefix_nalu_idx].store_ref_base_pic_flag =
            rconfig.store_ref_base_pic_flag.sample(film);
        if (ds.prefix_nalus[prefix_nalu_idx].store_ref_base_pic_flag
            || ds.nalu_headers[nalu_idx]
                .svc_extension
                .use_ref_base_pic_flag)
            && !ds.nalu_headers[nalu_idx].svc_extension.idr_flag
        {
            random_ref_base_pic_marking(prefix_nalu_idx, rconfig, ds, film);
        }
        ds.prefix_nalus[prefix_nalu_idx].additional_prefix_nal_unit_extension_flag = rconfig
            .additional_prefix_nal_unit_extension_flag
            .sample(film);
        if ds.prefix_nalus[prefix_nalu_idx].additional_prefix_nal_unit_extension_flag {
            let num_data_extensions = rconfig.num_data_extensions.sample(film);
            for _ in 0..num_data_extensions {
                ds.prefix_nalus[prefix_nalu_idx]
                    .additional_prefix_nal_unit_extension_data_flag
                    .push(
                        rconfig
                            .additional_prefix_nal_unit_extension_data_flag
                            .sample(film),
                    );
            }
        }
    }

    let num_data_extensions = rconfig.num_data_extensions.sample(film);
    for _ in 0..num_data_extensions {
        ds.prefix_nalus[prefix_nalu_idx]
            .additional_prefix_nal_unit_extension_data_flag
            .push(
                rconfig
                    .additional_prefix_nal_unit_extension_data_flag
                    .sample(film),
            );
    }
}

/// Generate a random ref_base_pic_marking as a part of a Prefix NALU
fn random_ref_base_pic_marking(
    prefix_nalu_idx: usize,
    rconfig: RandomPrefixNALU,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    ds.prefix_nalus[prefix_nalu_idx].adaptive_ref_base_pic_marking_mode_flag =
        rconfig.adaptive_ref_base_pic_marking_mode_flag.sample(film);
    if ds.prefix_nalus[prefix_nalu_idx].adaptive_ref_base_pic_marking_mode_flag {
        let num_modifications = rconfig.num_modifications.sample(film) as usize;
        if num_modifications > 0 {
            for i in 0..num_modifications - 1 {
                ds.prefix_nalus[prefix_nalu_idx]
                    .memory_management_base_control_operation
                    .push(
                        rconfig
                            .memory_management_base_control_operation
                            .sample(film),
                    );

                if ds.prefix_nalus[prefix_nalu_idx].memory_management_base_control_operation[i] == 1 {
                    ds.prefix_nalus[prefix_nalu_idx]
                        .difference_of_base_pic_nums_minus1
                        .push(rconfig.difference_of_base_pic_nums_minus1.sample(film));
                } else {
                    ds.prefix_nalus[prefix_nalu_idx]
                        .difference_of_base_pic_nums_minus1
                        .push(0);
                }

                if ds.prefix_nalus[prefix_nalu_idx].memory_management_base_control_operation[i] == 2 {
                    ds.prefix_nalus[prefix_nalu_idx]
                        .long_term_base_pic_num
                        .push(rconfig.long_term_base_pic_num.sample(film));
                } else {
                    ds.prefix_nalus[prefix_nalu_idx]
                        .long_term_base_pic_num
                        .push(0);
                }
            }
        }
        // stop condition
        ds.prefix_nalus[prefix_nalu_idx]
            .memory_management_base_control_operation
            .push(0);
    }
}

/// Generate a random Access Unit Delimiter (NALU Type 9)
pub fn random_access_unit_delimiter(
    aud_idx: usize,
    rconfig: RandomAccessUnitDelim,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    ds.auds[aud_idx].primary_pic_type = rconfig.primary_pic_type.sample(film) as u8;
}
