//! Data structures of decoded syntax elements.

use crate::common::helper::decoder_formatted_print;
use crate::common::helper::encoder_formatted_print;
use crate::common::helper::formatted_print;
use crate::common::helper::inverse_raster_scan;
use log::debug;
use serde::{Deserialize, Serialize};
use std::cmp;

/// The decoded syntax elements from a video
#[derive(Serialize, Deserialize)]
pub struct H264DecodedStream {
    pub nalu_elements: Vec<NALU>,
    pub nalu_headers: Vec<NALUheader>,
    pub spses: Vec<SeqParameterSet>,
    pub subset_spses: Vec<SubsetSPS>,
    pub sps_extensions: Vec<SPSExtension>,
    pub ppses: Vec<PicParameterSet>,
    pub subset_ppses: Vec<PicParameterSet>,
    pub prefix_nalus: Vec<PrefixNALU>,
    pub slices: Vec<Slice>,
    pub seis: Vec<SEINalu>,
    pub auds: Vec<AccessUnitDelim>,
}

impl H264DecodedStream {
    pub fn new() -> H264DecodedStream {
        H264DecodedStream {
            nalu_elements: Vec::new(),
            nalu_headers: Vec::new(),
            spses: Vec::new(),
            subset_spses: Vec::new(),
            sps_extensions: Vec::new(),
            ppses: Vec::new(),
            subset_ppses: Vec::new(),
            prefix_nalus: Vec::new(),
            slices: Vec::new(),
            seis: Vec::new(),
            auds: Vec::new(),
        }
    }

    pub fn clone(&self) -> H264DecodedStream {
        H264DecodedStream {
            nalu_elements: self.nalu_elements.clone(),
            nalu_headers: self.nalu_headers.clone(),
            spses: self.spses.clone(),
            subset_spses: self.subset_spses.clone(),
            sps_extensions: self.sps_extensions.clone(),
            ppses: self.ppses.clone(),
            subset_ppses: self.subset_ppses.clone(),
            prefix_nalus: self.prefix_nalus.clone(),
            slices: self.slices.clone(),
            seis: self.seis.clone(),
            auds: self.auds.clone(),
        }
    }
}

impl Default for H264DecodedStream {
    fn default() -> Self {
        Self::new()
    }
}

/// NALU Header SVC Extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NALUHeaderSVCExtension {
    pub idr_flag: bool,                 // u(1)
    pub priority_id: u8,                // u(6)
    pub no_inter_layer_pred_flag: bool, // u(1)
    pub dependency_id: u8,              // u(3)
    pub quality_id: u8,                 // u(4)
    pub temporal_id: u8,                // u(3)
    pub use_ref_base_pic_flag: bool,    // u(1)
    pub discardable_flag: bool,         // u(1)
    pub output_flag: bool,              // u(1)
    pub reserved_three_2bits: u8,       // u(2)
}

impl NALUHeaderSVCExtension {
    pub fn new() -> NALUHeaderSVCExtension {
        NALUHeaderSVCExtension {
            idr_flag: false,
            priority_id: 0,
            no_inter_layer_pred_flag: false,
            dependency_id: 0,
            quality_id: 0,
            temporal_id: 0,
            use_ref_base_pic_flag: false,
            discardable_flag: false,
            output_flag: false,
            reserved_three_2bits: 0,
        }
    }
}

impl Default for NALUHeaderSVCExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// NALU Header 3D AVC Extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NALUHeader3DAVCExtension {
    pub view_idx: u8,          // u(8)
    pub depth_flag: bool,      // u(1)
    pub non_idr_flag: bool,    // u(1)
    pub temporal_id: u8,       // u(3)
    pub anchor_pic_flag: bool, // u(1)
    pub inter_view_flag: bool, // u(1)
}

impl NALUHeader3DAVCExtension {
    pub fn new() -> NALUHeader3DAVCExtension {
        NALUHeader3DAVCExtension {
            view_idx: 0,
            depth_flag: false,
            non_idr_flag: false,
            temporal_id: 0,
            anchor_pic_flag: false,
            inter_view_flag: false,
        }
    }
}

impl Default for NALUHeader3DAVCExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// NALU Header MVC Extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NALUHeaderMVCExtension {
    pub non_idr_flag: bool,     // u(1)
    pub priority_id: u8,        // u(6)
    pub view_id: u32,           // u(10)
    pub temporal_id: u8,        // u(3)
    pub anchor_pic_flag: bool,  // u(1)
    pub inter_view_flag: bool,  // u(1)
    pub reserved_one_bit: bool, // u(1)
}

impl NALUHeaderMVCExtension {
    pub fn new() -> NALUHeaderMVCExtension {
        NALUHeaderMVCExtension {
            non_idr_flag: false,
            priority_id: 0,
            view_id: 0,
            temporal_id: 0,
            anchor_pic_flag: false,
            inter_view_flag: false,
            reserved_one_bit: false,
        }
    }
}

impl Default for NALUHeaderMVCExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// NALU Header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NALUheader {
    pub forbidden_zero_bit: u8,
    pub nal_ref_idc: u8,
    pub nal_unit_type: u8,
    pub svc_extension_flag: bool,
    pub svc_extension: NALUHeaderSVCExtension,
    pub avc_3d_extension_flag: bool,
    pub avc_3d_extension: NALUHeader3DAVCExtension,
    pub mvc_extension: NALUHeaderMVCExtension,
}

impl NALUheader {
    pub fn new() -> NALUheader {
        NALUheader {
            forbidden_zero_bit: 0,
            nal_ref_idc: 0,
            nal_unit_type: 0,
            svc_extension_flag: false,
            svc_extension: NALUHeaderSVCExtension::new(),
            avc_3d_extension_flag: false,
            avc_3d_extension: NALUHeader3DAVCExtension::new(),
            mvc_extension: NALUHeaderMVCExtension::new(),
        }
    }
}

impl Default for NALUheader {
    fn default() -> Self {
        Self::new()
    }
}

/// Holds the original encoded content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NALU {
    pub longstartcode: bool,
    pub content: Vec<u8>,
}

impl NALU {
    pub fn new() -> NALU {
        NALU {
            longstartcode: true,
            content: Vec::new(),
        }
    }
}

impl Default for NALU {
    fn default() -> Self {
        Self::new()
    }
}

/// NALU Type 14 -- PrefixNALU
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefixNALU {
    pub store_ref_base_pic_flag: bool, // u(1)
    // dec_ref_base_pic_marking() - G.7.3.3.5
    pub adaptive_ref_base_pic_marking_mode_flag: bool, // u(1)
    pub memory_management_base_control_operation: Vec<u32>, // array of ue(v)
    pub difference_of_base_pic_nums_minus1: Vec<u32>,  // array of ue(v)
    pub long_term_base_pic_num: Vec<u32>,              // array of ue(v)
    //
    // below are for future extensions
    pub additional_prefix_nal_unit_extension_flag: bool, // u(1)
    pub additional_prefix_nal_unit_extension_data_flag: Vec<bool>, // u(1)
}

impl PrefixNALU {
    pub fn new() -> PrefixNALU {
        PrefixNALU {
            store_ref_base_pic_flag: false,
            adaptive_ref_base_pic_marking_mode_flag: false,
            memory_management_base_control_operation: Vec::new(),
            difference_of_base_pic_nums_minus1: Vec::new(),
            long_term_base_pic_num: Vec::new(),
            additional_prefix_nal_unit_extension_flag: false,
            additional_prefix_nal_unit_extension_data_flag: Vec::new(),
        }
    }

    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print(
            "Prefix NALU: store_ref_base_pic_flag",
            self.store_ref_base_pic_flag,
            63,
        );
        encoder_formatted_print(
            "Prefix NALU: adaptive_ref_base_pic_marking_mode_flag",
            self.adaptive_ref_base_pic_marking_mode_flag,
            63,
        );
        encoder_formatted_print(
            "Prefix NALU: memory_management_base_control_operation",
            &self.memory_management_base_control_operation,
            63,
        );
        encoder_formatted_print(
            "Prefix NALU: difference_of_base_pic_nums_minus1",
            &self.difference_of_base_pic_nums_minus1,
            63,
        );
        encoder_formatted_print(
            "Prefix NALU: long_term_base_pic_num",
            &self.long_term_base_pic_num,
            63,
        );
        encoder_formatted_print(
            "Prefix NALU: additional_prefix_nal_unit_extension_flag",
            self.additional_prefix_nal_unit_extension_flag,
            63,
        );
        encoder_formatted_print(
            "Prefix NALU: additional_prefix_nal_unit_extension_data_flag",
            &self.additional_prefix_nal_unit_extension_data_flag,
            63,
        );
    }
}

impl Default for PrefixNALU {
    fn default() -> Self {
        Self::new()
    }
}

/// Computed parameters derived from SPS and PPS
///
/// These are used to produce the final image.
///
/// These variables are stylized as camelCase in the spec.
#[derive(Debug, Clone, Copy)]
pub struct VideoParameters {
    // Currently unnecessary parameters are commented out
    pub sub_width_c: u32,   // table 6-1 [0,2,1] where 0 is an undefined value
    pub sub_height_c: u32,  // table 6-1
    pub mb_width_c: u32,    // 6-1
    pub mb_height_c: u32,   // 6-2
    pub idr_pic_flag: bool, // 7-1 - used to determine whether the current slice is of NALU
    // pub depth_flag: bool,      // 7-2 - whether the 3d extension is enabled
    pub chroma_array_type: u8, // defined in separate_colour_plane_flag section on page 74 (pg 96 of PDF)
    pub bit_depth_y: u8,       // 7-3 - range of [8, 14]
    pub qp_bd_offset_y: i32,   // 7-4 - range of [0, 36] multiples of 6
    pub bit_depth_c: u8,       // 7-5
    // pub qp_bd_offset_c: u8,    // 7-6
    // pub raw_mb_bits: u32,      // 7-7
    // //pub flat_4x4_16: [u8; 16], // 7-8
    // //pub flat_8x8_16: [u8; 64], // 7-9
    // //pub default_4x4_intra: [u8; 16], // table 7-3
    // //pub default_8x8_intra: [u8; 64], // table 7-4
    // pub max_frame_num: u32, // 7-10
    // pub max_pic_order_cnt_lsb: u32, // 7-11
    // pub expected_delta_per_pic_order_cnt_cycle: i32, // equation 7-12
    pub pic_width_in_mbs: u32, // 7-13
    // pub pic_width_in_samples_l: u32, // 7-14
    // pub pic_width_in_samples_c: u32, // 7-15
    pub pic_height_in_map_units: u32, // 7-16
    pub pic_size_in_map_units: u32,   // 7-17
    pub frame_height_in_mbs: u32,     // 7-18
    // pub crop_unit_x: u8,       // 7-19/21
    // pub crop_unit_y: u8,       // 7-20/22
    // pub slice_group_change_rate: u32, // 7-23

    // Useful for neighbor decoding
    pub mbaff_frame_flag: bool, // 7-25

    // misc useful values in cabac decoding
    pub nal_unit_type: u8,
    pub pps_constrained_intra_pred_flag: bool,
    pub entropy_coding_mode_flag: bool,
}

impl VideoParameters {
    pub fn new(nh: &NALUheader, p: &PicParameterSet, s: &SeqParameterSet) -> VideoParameters {
        let sub_width_c: u32;
        let sub_height_c: u32;
        let mb_width_c: u32;
        let mb_height_c: u32;
        let mbaff_frame_flag = false;

        // Table 6-1
        match s.chroma_format_idc {
            0 => {
                // Chroma format: monochrome
                if !s.separate_colour_plane_flag {
                    sub_width_c = 0;
                    sub_height_c = 0;
                } else {
                    panic!("update_video_parameters: Unknown combination of chroma_format_idc ({}) and separate_colour_plane_flag ({})", s.chroma_format_idc, s.separate_colour_plane_flag);
                }
            }
            1 => {
                // Chroma format: 4:2:0
                if !s.separate_colour_plane_flag {
                    sub_width_c = 2;
                    sub_height_c = 2;
                } else {
                    panic!("update_vp: Unknown combination of chroma_format_idc ({}) and separate_colour_plane_flag ({})", s.chroma_format_idc, s.separate_colour_plane_flag);
                }
            }
            2 => {
                // Chroma format: 4:2:2
                if !s.separate_colour_plane_flag {
                    sub_width_c = 2;
                    sub_height_c = 1;
                } else {
                    panic!("update_vp: Unknown combination of chroma_format_idc ({}) and separate_colour_plane_flag ({})", s.chroma_format_idc, s.separate_colour_plane_flag);
                }
            }
            3 => {
                // Chroma format: 4:4:4
                if !s.separate_colour_plane_flag {
                    sub_width_c = 1;
                    sub_height_c = 1;
                } else {
                    sub_width_c = 0;
                    sub_height_c = 0;
                }
            }
            _ => {
                // weird values, default to 4:2:0
                // TODO: consider treating this differently
                sub_width_c = 2;
                sub_height_c = 2;
                // panic!("update_vp: unsupported value for chroma_format_idc ({})", s.chroma_format_idc);
            }
        }

        // equation 6-1
        if s.chroma_format_idc == 0 || s.separate_colour_plane_flag {
            // this should only be the case when
            mb_width_c = 0;
            mb_height_c = 0;
        } else {
            mb_width_c = 16 / sub_width_c;
            mb_height_c = 16 / sub_height_c;
        }

        // section 6.4 is used for neighbor calculation

        // equation 7-1
        let idr_pic_flag: bool = nh.nal_unit_type == 5;

        // TODO: equation 7-2 which is the 3d extension
        // depth_flag = match (nh.nal_unit_type != 21) {false => match nh.avc_3d_extension_flag { true => nh.depth-flag, _ => true}, _ => false};

        // page 74/ separate_colour_plane_flag section
        let chroma_array_type: u8 = if !s.separate_colour_plane_flag {
            s.chroma_format_idc
        } else {
            0
        };

        // equation 7-3
        let bit_depth_y: u8 = 8 + s.bit_depth_luma_minus8;

        // equation 7-4a
        let qp_bd_offset_y: i32 = 6 * (s.bit_depth_luma_minus8 as i32);

        // equation 7-5
        let bit_depth_c: u8 = 8 + s.bit_depth_chroma_minus8;

        // equation 7-6
        //qp_bd_offset_c = 6 * s.bit_depth_chroma_minus8;

        // equation 7-7
        //raw_mb_bits = 256u32 * (bit_depth_y as u32)
        //    + 2u32 * (mb_width_c as u32) * (mb_height_c as u32) * (bit_depth_c as u32);

        // only set if s.seq_scaling_matrix_present_flag is present
        // TODO: flat and default values should be set in setup (seq_scaling_list_present_flag[i])

        // equation 7-10
        //let max_frame_num: u32 = 2u32.pow(s.log2_max_frame_num_minus4 as u32 + 4);

        // equation 7-11
        //max_pic_order_cnt_lsb = 2u32.pow(s.log2_max_pic_order_cnt_lsb_minus4 as u32 + 4);

        // equation 7-12
        //if s.pic_order_cnt_type == 1 {
        //    expected_delta_per_pic_order_cnt_cycle = 0;
        //    for i in 0..s.num_ref_frames_in_pic_order_cnt_cycle {
        //        expected_delta_per_pic_order_cnt_cycle += s.offset_for_ref_frame[i as usize];
        //    }
        //}

        // equation 7-13
        let pic_width_in_mbs: u32 = s.pic_width_in_mbs_minus1 + 1;

        // equation 7-14
        //pic_width_in_samples_l = pic_width_in_mbs * 16;

        // equation 7-15
        //pic_width_in_samples_c = pic_width_in_mbs * mb_width_c as u32;

        // equation 7-16
        let pic_height_in_map_units: u32 = s.pic_height_in_map_units_minus1 + 1;

        // equation 7-17
        let pic_size_in_map_units = pic_width_in_mbs * pic_height_in_map_units;

        // equation 7-18
        let frame_height_in_mbs: u32 = (2u32
            - match s.frame_mbs_only_flag {
                true => 1u32,
                _ => 0u32,
            })
            * pic_height_in_map_units;

        // crop values
        //if chroma_array_type == 0 {
        //    // equation 7-19
        //    crop_unit_x = 1;
        //    // equation 7-20
        //    crop_unit_y = 2 - match s.frame_mbs_only_flag {
        //        true => 1,
        //        _ => 0,
        //    };
        //} else {
        //    // equation 7-21
        //    crop_unit_x = sub_width_c;
        //    // equation 7-22
        //    crop_unit_y = sub_height_c
        //        * (2 - match s.frame_mbs_only_flag {
        //            true => 1,
        //            _ => 0,
        //        });
        //}

        // equation 7-23
        //slice_group_change_rate = p.slice_group_change_rate_minus1 + 1;

        // the rest of values are calculated in the slice header

        // misc useful values
        let nal_unit_type: u8 = nh.nal_unit_type;
        let pps_constrained_intra_pred_flag: bool = p.constrained_intra_pred_flag;
        let entropy_coding_mode_flag: bool = p.entropy_coding_mode_flag;

        VideoParameters {
            sub_width_c: sub_width_c,
            sub_height_c: sub_height_c,
            mb_width_c: mb_width_c,
            mb_height_c: mb_height_c,
            idr_pic_flag: idr_pic_flag,
            chroma_array_type: chroma_array_type,
            bit_depth_y: bit_depth_y,
            qp_bd_offset_y: qp_bd_offset_y,
            bit_depth_c: bit_depth_c,
            //max_frame_num: max_frame_num,
            pic_width_in_mbs: pic_width_in_mbs,
            pic_height_in_map_units: pic_height_in_map_units,
            pic_size_in_map_units: pic_size_in_map_units,
            frame_height_in_mbs: frame_height_in_mbs,
            mbaff_frame_flag: mbaff_frame_flag,
            nal_unit_type: nal_unit_type,
            pps_constrained_intra_pred_flag: pps_constrained_intra_pred_flag,
            entropy_coding_mode_flag: entropy_coding_mode_flag,
        }
    }
}

/// Macroblock Types
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum MbType {
    // Added as a starter state
    INONE,
    // Table 7-11
    INxN,
    I16x16_0_0_0,
    I16x16_1_0_0,
    I16x16_2_0_0,
    I16x16_3_0_0,
    I16x16_0_1_0,
    I16x16_1_1_0,
    I16x16_2_1_0,
    I16x16_3_1_0,
    I16x16_0_2_0,
    I16x16_1_2_0,
    I16x16_2_2_0,
    I16x16_3_2_0,
    I16x16_0_0_1,
    I16x16_1_0_1,
    I16x16_2_0_1,
    I16x16_3_0_1,
    I16x16_0_1_1,
    I16x16_1_1_1,
    I16x16_2_1_1,
    I16x16_3_1_1,
    I16x16_0_2_1,
    I16x16_1_2_1,
    I16x16_2_2_1,
    I16x16_3_2_1,
    IPCM,
    // Table 7-12
    SI,
    // Table 7-13
    PL016x16,
    PL0L016x8,
    PL0L08x16,
    P8x8,
    P8x8ref0,
    PSkip,
    // Table 7-14
    BDirect16x16,
    BL016x16,
    BL116x16,
    BBi16x16,
    BL0L016x8,
    BL0L08x16,
    BL1L116x8,
    BL1L18x16,
    BL0L116x8,
    BL0L18x16,
    BL1L016x8,
    BL1L08x16,
    BL0Bi16x8,
    BL0Bi8x16,
    BL1Bi16x8,
    BL1Bi8x16,
    BBiL016x8,
    BBiL08x16,
    BBiL116x8,
    BBiL18x16,
    BBiBi16x8,
    BBiBi8x16,
    B8x8,
    BSkip,
}

/// SubMacroblock Types
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum SubMbType {
    // Added as a starter state
    NA,
    // Table 7-17
    PL08x8,
    PL08x4,
    PL04x8,
    PL04x4,
    // Table 7-18
    BDirect8x8,
    BL08x8,
    BL18x8,
    BBi8x8,
    BL08x4,
    BL04x8,
    BL18x4,
    BL14x8,
    BBi8x4,
    BBi4x8,
    BL04x4,
    BL14x4,
    BBi4x4,
}

/// CAVLC decoded variables
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoeffToken {
    pub total_coeff: usize,
    pub trailing_ones: usize,
    pub n_c: i8,
}

impl CoeffToken {
    pub fn new() -> CoeffToken {
        CoeffToken {
            total_coeff: 0,
            trailing_ones: 0,
            n_c: 0,
        }
    }
}

impl Default for CoeffToken {
    fn default() -> Self {
        Self::new()
    }
}

/// CAVLC residual mode
#[derive(PartialEq)]
pub enum ResidualMode {
    ChromaDCLevel,
    Intra16x16DCLevel,
    Intra16x16ACLevel,
    LumaLevel4x4,
    CbIntra16x16DCLevel,
    CbIntra16x16ACLevel,
    CbLevel4x4,
    CrIntra16x16DCLevel,
    CrIntra16x16ACLevel,
    CrLevel4x4,
    ChromaACLevel,
}

/// Type of macroblock prediction mode
#[derive(Debug, PartialEq, Clone)]
pub enum MbPartPredMode {
    NA,
    Intra4x4,
    Intra8x8,
    Intra16x16,
    PredL0,
    PredL1,
    Direct,
    BiPred,
}

/// Macroblock Residue values
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransformBlock {
    pub available: bool,
    // CABAC decoded values
    pub coded_block_flag: bool,
    pub significant_coeff_flag: Vec<bool>,
    pub last_significant_coeff_flag: Vec<bool>,
    pub coeff_abs_level_minus1: Vec<u32>,
    pub coeff_sign_flag: Vec<bool>,
    // CAVLC decoded values
    pub coeff_token: CoeffToken,
    pub trailing_ones_sign_flag: Vec<bool>,
    pub level_prefix: Vec<u32>,
    pub level_suffix: Vec<u32>,
    pub total_zeros: usize,
    pub run_before: Vec<usize>,
}

impl TransformBlock {
    pub fn new() -> TransformBlock {
        TransformBlock {
            available: false,

            coded_block_flag: true, //section 7.4.5.3.3
            significant_coeff_flag: Vec::new(),
            last_significant_coeff_flag: Vec::new(),
            coeff_abs_level_minus1: Vec::new(),
            coeff_sign_flag: Vec::new(),

            coeff_token: CoeffToken::new(),
            trailing_ones_sign_flag: Vec::new(),
            level_prefix: Vec::new(),
            level_suffix: Vec::new(),
            total_zeros: 0,
            run_before: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn decoder_pretty_print(&self) {
        debug!(target: "decode","TransformBlock {{ \n\tavailable: {},\n\tcoded_block_flag: {},\n\tsignificant_coeff_flag: {:?},\n\tlast_significant_coeff_flag: {:?},\n\tcoeff_abs_level_minus1: {:?},\n\tcoeff_sign_flag: {:?},coeff_token : {:?},\n\ttrailing_ones_sign_flag : {:?},\n\tlevel_prefix : {:?},\n\tlevel_suffix : {:?},\n\ttotal_zeros : {:?},\n\trun_before : {:?},\n\t}};",
                self.available,
                self.coded_block_flag,
                self.significant_coeff_flag,
                self.last_significant_coeff_flag,
                self.coeff_abs_level_minus1,
                self.coeff_sign_flag,
                self.coeff_token,
                self.trailing_ones_sign_flag,
                self.level_prefix,
                self.level_suffix,
                self.total_zeros,
                self.run_before,
        );
    }

    #[allow(dead_code)]
    pub fn encoder_pretty_print(&self) {
        debug!(target: "encode","TransformBlock {{ \n\tavailable: {},\n\tcoded_block_flag: {},\n\tsignificant_coeff_flag: {:?},\n\tlast_significant_coeff_flag: {:?},\n\tcoeff_abs_level_minus1: {:?},\n\tcoeff_sign_flag: {:?},coeff_token : {:?},\n\ttrailing_ones_sign_flag : {:?},\n\tlevel_prefix : {:?},\n\tlevel_suffix : {:?},\n\ttotal_zeros : {:?},\n\trun_before : {:?},\n\t}};",
                self.available,
                self.coded_block_flag,
                self.significant_coeff_flag,
                self.last_significant_coeff_flag,
                self.coeff_abs_level_minus1,
                self.coeff_sign_flag,
                self.coeff_token,
                self.trailing_ones_sign_flag,
                self.level_prefix,
                self.level_suffix,
                self.total_zeros,
                self.run_before,
        );
    }
}

impl Default for TransformBlock {
    fn default() -> Self {
        Self::new()
    }
}

/// Macroblock syntax elements
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MacroBlock {
    // implementation specific values
    pub available: bool,
    pub mb_idx: usize, // this is the index in the SliceData structure, which may differ from mb_addr
    pub mb_skip_flag: bool,

    // syntax elements in 7.3.5
    pub mb_addr: usize,
    pub mb_type: MbType,
    pub pcm_sample_luma: Vec<u32>,
    pub pcm_sample_chroma: Vec<u32>,
    pub transform_size_8x8_flag: bool,
    pub coded_block_pattern: u32,
    pub mb_qp_delta: i32,

    // mb_pred
    pub prev_intra4x4_pred_mode_flag: [bool; 16],
    pub rem_intra4x4_pred_mode: [u32; 16],
    pub prev_intra8x8_pred_mode_flag: [bool; 4],
    pub rem_intra8x8_pred_mode: [u32; 4],
    pub intra_chroma_pred_mode: u8,
    pub ref_idx_l0: [u32; 4],
    pub ref_idx_l1: [u32; 4],
    pub mvd_l0: [[[i32; 2]; 4]; 4],
    pub mvd_l1: [[[i32; 2]; 4]; 4],

    // sub_mb_pred
    pub sub_mb_type: [SubMbType; 4],

    // residual_block_cabac or residual_block_cavlc
    pub intra_16x16_dc_level_transform_blocks: TransformBlock,
    pub intra_16x16_ac_level_transform_blocks: Vec<TransformBlock>,
    pub luma_level_4x4_transform_blocks: Vec<TransformBlock>,
    pub luma_level_8x8_transform_blocks: Vec<TransformBlock>,

    pub cb_intra_16x16_dc_level_transform_blocks: TransformBlock,
    pub cb_intra_16x16_ac_level_transform_blocks: Vec<TransformBlock>,
    pub cb_level_4x4_transform_blocks: Vec<TransformBlock>,
    pub cb_level_8x8_transform_blocks: Vec<TransformBlock>,

    pub cr_intra_16x16_dc_level_transform_blocks: TransformBlock,
    pub cr_intra_16x16_ac_level_transform_blocks: Vec<TransformBlock>,
    pub cr_level_4x4_transform_blocks: Vec<TransformBlock>,
    pub cr_level_8x8_transform_blocks: Vec<TransformBlock>,

    pub chroma_dc_level_transform_blocks: Vec<TransformBlock>,
    pub chroma_ac_level_transform_blocks: Vec<Vec<TransformBlock>>,

    // decode variables
    pub no_sub_mb_part_size_less_than_8x8_flag: bool,
    pub coded_block_pattern_luma: u32,
    pub coded_block_pattern_chroma: u32,
    pub qp_y: i32,
    pub qp_y_prime: i32,
    pub transform_bypass_mode_flag: bool,

    // calculated coefficients
    pub intra_16x16_dc_level: Vec<i32>,
    pub intra_16x16_ac_level: Vec<Vec<i32>>,
    pub luma_level_4x4: Vec<Vec<i32>>,
    pub luma_level_8x8: Vec<Vec<i32>>,

    pub cr_intra_16x16_dc_level: Vec<i32>,
    pub cr_intra_16x16_ac_level: Vec<Vec<i32>>,
    pub cr_level_4x4: Vec<Vec<i32>>,
    pub cr_level_8x8: Vec<Vec<i32>>,

    pub cb_intra_16x16_dc_level: Vec<i32>,
    pub cb_intra_16x16_ac_level: Vec<Vec<i32>>,
    pub cb_level_4x4: Vec<Vec<i32>>,
    pub cb_level_8x8: Vec<Vec<i32>>,

    pub num_c8x8: usize,
    pub chroma_dc_level: Vec<Vec<i32>>,
    pub chroma_ac_level: Vec<Vec<Vec<i32>>>,
}

impl MacroBlock {
    pub fn new() -> MacroBlock {
        MacroBlock {
            available: false,
            mb_idx: 0,
            mb_skip_flag: false,

            mb_addr: 0,
            mb_type: MbType::INONE,
            pcm_sample_luma: Vec::new(),
            pcm_sample_chroma: Vec::new(),
            transform_size_8x8_flag: false,
            coded_block_pattern: 0,
            mb_qp_delta: 0,

            prev_intra4x4_pred_mode_flag: [false; 16],
            rem_intra4x4_pred_mode: [0; 16],
            prev_intra8x8_pred_mode_flag: [false; 4],
            rem_intra8x8_pred_mode: [0; 4],
            intra_chroma_pred_mode: 0,
            ref_idx_l0: [0; 4],
            ref_idx_l1: [0; 4],
            mvd_l0: [[[0; 2]; 4]; 4],
            mvd_l1: [[[0; 2]; 4]; 4],

            sub_mb_type: [SubMbType::NA; 4],

            intra_16x16_dc_level_transform_blocks: TransformBlock::new(),
            cb_intra_16x16_dc_level_transform_blocks: TransformBlock::new(),
            cr_intra_16x16_dc_level_transform_blocks: TransformBlock::new(),

            intra_16x16_ac_level_transform_blocks: Vec::new(),
            cb_intra_16x16_ac_level_transform_blocks: Vec::new(),
            cr_intra_16x16_ac_level_transform_blocks: Vec::new(),

            luma_level_4x4_transform_blocks: Vec::new(),
            cb_level_4x4_transform_blocks: Vec::new(),
            cr_level_4x4_transform_blocks: Vec::new(),

            luma_level_8x8_transform_blocks: Vec::new(),
            cb_level_8x8_transform_blocks: Vec::new(),
            cr_level_8x8_transform_blocks: Vec::new(),

            chroma_dc_level_transform_blocks: Vec::new(),
            chroma_ac_level_transform_blocks: Vec::new(),

            no_sub_mb_part_size_less_than_8x8_flag: true,
            coded_block_pattern_luma: 0,
            coded_block_pattern_chroma: 0,
            qp_y: 0,
            qp_y_prime: 0,
            transform_bypass_mode_flag: false,
            intra_16x16_dc_level: Vec::new(),
            cb_intra_16x16_dc_level: Vec::new(),
            cr_intra_16x16_dc_level: Vec::new(),

            intra_16x16_ac_level: Vec::new(),
            cb_intra_16x16_ac_level: Vec::new(),
            cr_intra_16x16_ac_level: Vec::new(),

            luma_level_4x4: Vec::new(),
            cb_level_4x4: Vec::new(),
            cr_level_4x4: Vec::new(),

            luma_level_8x8: Vec::new(),
            cb_level_8x8: Vec::new(),
            cr_level_8x8: Vec::new(),

            num_c8x8: 0,
            chroma_dc_level: Vec::new(),
            chroma_ac_level: Vec::new(),
        }
    }

    /// Returns the partition prediction mode of the macroblock
    pub fn mb_part_pred_mode(&self, mb_part_idx: usize) -> MbPartPredMode {
        if self.mb_type == MbType::P8x8
            || self.mb_type == MbType::P8x8ref0
            || self.mb_type == MbType::B8x8
        {
            return self.sub_mb_part_pred_mode(mb_part_idx);
        }

        if mb_part_idx == 0 {
            // Table 7-11 & 7-12
            if (self.mb_type == MbType::INxN && !self.transform_size_8x8_flag)
                || self.mb_type == MbType::SI
            {
                return MbPartPredMode::Intra4x4; //Intra4x4
            } else if self.mb_type == MbType::INxN {
                return MbPartPredMode::Intra8x8;
            } else if self.mb_type == MbType::IPCM {
                return MbPartPredMode::NA;
            } else if self.mb_type == MbType::I16x16_0_0_0
                || self.mb_type == MbType::I16x16_1_0_0
                || self.mb_type == MbType::I16x16_2_0_0
                || self.mb_type == MbType::I16x16_3_0_0
                || self.mb_type == MbType::I16x16_0_1_0
                || self.mb_type == MbType::I16x16_1_1_0
                || self.mb_type == MbType::I16x16_2_1_0
                || self.mb_type == MbType::I16x16_3_1_0
                || self.mb_type == MbType::I16x16_0_2_0
                || self.mb_type == MbType::I16x16_1_2_0
                || self.mb_type == MbType::I16x16_2_2_0
                || self.mb_type == MbType::I16x16_3_2_0
                || self.mb_type == MbType::I16x16_0_0_1
                || self.mb_type == MbType::I16x16_1_0_1
                || self.mb_type == MbType::I16x16_2_0_1
                || self.mb_type == MbType::I16x16_3_0_1
                || self.mb_type == MbType::I16x16_0_1_1
                || self.mb_type == MbType::I16x16_1_1_1
                || self.mb_type == MbType::I16x16_2_1_1
                || self.mb_type == MbType::I16x16_3_1_1
                || self.mb_type == MbType::I16x16_0_2_1
                || self.mb_type == MbType::I16x16_1_2_1
                || self.mb_type == MbType::I16x16_2_2_1
                || self.mb_type == MbType::I16x16_3_2_1
            {
                return MbPartPredMode::Intra16x16;
            }
            // Table 7-13
            if self.mb_type == MbType::PL016x16
                || self.mb_type == MbType::PL0L016x8
                || self.mb_type == MbType::PL0L08x16
                || self.mb_type == MbType::PSkip
            {
                return MbPartPredMode::PredL0;
            }

            // Table 7-14
            if self.mb_type == MbType::BDirect16x16 || self.mb_type == MbType::BSkip {
                return MbPartPredMode::Direct;
            }
            if self.mb_type == MbType::BL016x16 {
                return MbPartPredMode::PredL0;
            }
            if self.mb_type == MbType::BL116x16 {
                return MbPartPredMode::PredL1;
            }
            if self.mb_type == MbType::BBi16x16 {
                return MbPartPredMode::BiPred;
            }

            if self.mb_type == MbType::BL0L016x8
                || self.mb_type == MbType::BL0L08x16
                || self.mb_type == MbType::BL0L116x8
                || self.mb_type == MbType::BL0L18x16
                || self.mb_type == MbType::BL0Bi16x8
                || self.mb_type == MbType::BL0Bi8x16
            {
                return MbPartPredMode::PredL0;
            }

            if self.mb_type == MbType::BL1L116x8
                || self.mb_type == MbType::BL1L18x16
                || self.mb_type == MbType::BL1L016x8
                || self.mb_type == MbType::BL1L08x16
                || self.mb_type == MbType::BL1Bi16x8
                || self.mb_type == MbType::BL1Bi8x16
            {
                return MbPartPredMode::PredL1;
            }

            if self.mb_type == MbType::BBiL016x8
                || self.mb_type == MbType::BBiL08x16
                || self.mb_type == MbType::BBiL116x8
                || self.mb_type == MbType::BBiL18x16
                || self.mb_type == MbType::BBiBi16x8
                || self.mb_type == MbType::BBiBi8x16
            {
                return MbPartPredMode::BiPred;
            }
        } else if mb_part_idx == 1 {
            // Table 7-13
            if self.mb_type == MbType::PL0L016x8 || self.mb_type == MbType::PL0L08x16 {
                return MbPartPredMode::PredL0;
            }

            // Table 7-14
            if self.mb_type == MbType::BL0L016x8
                || self.mb_type == MbType::BL0L08x16
                || self.mb_type == MbType::BL1L016x8
                || self.mb_type == MbType::BL1L08x16
                || self.mb_type == MbType::BBiL016x8
                || self.mb_type == MbType::BBiL08x16
            {
                return MbPartPredMode::PredL0;
            }

            if self.mb_type == MbType::BBiL116x8
                || self.mb_type == MbType::BBiL18x16
                || self.mb_type == MbType::BL1L116x8
                || self.mb_type == MbType::BL1L18x16
                || self.mb_type == MbType::BL0L116x8
                || self.mb_type == MbType::BL0L18x16
            {
                return MbPartPredMode::PredL1;
            }

            if self.mb_type == MbType::BL0Bi16x8
                || self.mb_type == MbType::BL0Bi8x16
                || self.mb_type == MbType::BL1Bi16x8
                || self.mb_type == MbType::BL1Bi8x16
                || self.mb_type == MbType::BBiBi16x8
                || self.mb_type == MbType::BBiBi8x16
            {
                return MbPartPredMode::BiPred;
            }
        }

        MbPartPredMode::NA
    }

    /// Returns the partition prediction mode of the submacroblock
    pub fn sub_mb_part_pred_mode(&self, mb_part_idx: usize) -> MbPartPredMode {
        match self.sub_mb_type[mb_part_idx] {
            // Table 7-17
            SubMbType::PL08x8 => MbPartPredMode::PredL0,
            SubMbType::PL08x4 => MbPartPredMode::PredL0,
            SubMbType::PL04x8 => MbPartPredMode::PredL0,
            SubMbType::PL04x4 => MbPartPredMode::PredL0,
            // Table 7-18
            SubMbType::BDirect8x8 => MbPartPredMode::Direct,

            SubMbType::BL08x8 => MbPartPredMode::PredL0,
            SubMbType::BL18x8 => MbPartPredMode::PredL1,
            SubMbType::BBi8x8 => MbPartPredMode::BiPred,

            SubMbType::BL08x4 => MbPartPredMode::PredL0,
            SubMbType::BL04x8 => MbPartPredMode::PredL0,

            SubMbType::BL18x4 => MbPartPredMode::PredL1,
            SubMbType::BL14x8 => MbPartPredMode::PredL1,

            SubMbType::BBi8x4 => MbPartPredMode::BiPred,
            SubMbType::BBi4x8 => MbPartPredMode::BiPred,

            SubMbType::BL04x4 => MbPartPredMode::PredL0,
            SubMbType::BL14x4 => MbPartPredMode::PredL1,
            SubMbType::BBi4x4 => MbPartPredMode::BiPred,
            _ => MbPartPredMode::NA,
        }
    }

    /// Returns the number of macroblock partitions used
    pub fn num_mb_part(&self) -> usize {
        match self.mb_type {
            MbType::PL016x16 => 1,
            MbType::PL0L016x8 => 2,
            MbType::PL0L08x16 => 2,
            MbType::P8x8 => 4,
            MbType::P8x8ref0 => 4,
            MbType::PSkip => 1,
            MbType::BDirect16x16 => 0,
            MbType::BL016x16 => 1,
            MbType::BL116x16 => 1,
            MbType::BBi16x16 => 1,
            MbType::BL0L016x8 => 2,
            MbType::BL0L08x16 => 2,
            MbType::BL1L116x8 => 2,
            MbType::BL1L18x16 => 2,
            MbType::BL0L116x8 => 2,
            MbType::BL0L18x16 => 2,
            MbType::BL1L016x8 => 2,
            MbType::BL1L08x16 => 2,
            MbType::BL0Bi16x8 => 2,
            MbType::BL0Bi8x16 => 2,
            MbType::BL1Bi16x8 => 2,
            MbType::BL1Bi8x16 => 2,
            MbType::BBiL016x8 => 2,
            MbType::BBiL08x16 => 2,
            MbType::BBiL116x8 => 2,
            MbType::BBiL18x16 => 2,
            MbType::BBiBi16x8 => 2,
            MbType::BBiBi8x16 => 2,
            MbType::B8x8 => 4,
            MbType::BSkip => 0,
            _ => 0,
        }
    }

    /// Returns the number of sub macroblock partitions used
    pub fn num_sub_mb_part(&self, mb_part_idx: usize) -> usize {
        match self.sub_mb_type[mb_part_idx] {
            // Table 7-17
            SubMbType::PL08x8 => 1,
            SubMbType::PL08x4 => 2,
            SubMbType::PL04x8 => 2,
            SubMbType::PL04x4 => 4,
            // Table 7-18
            SubMbType::BDirect8x8 => 4,

            SubMbType::BL08x8 => 1,
            SubMbType::BL18x8 => 1,
            SubMbType::BBi8x8 => 1,

            SubMbType::BL08x4 => 2,
            SubMbType::BL04x8 => 2,

            SubMbType::BL18x4 => 2,
            SubMbType::BL14x8 => 2,

            SubMbType::BBi8x4 => 2,
            SubMbType::BBi4x8 => 2,

            SubMbType::BL04x4 => 4,
            SubMbType::BL14x4 => 4,
            SubMbType::BBi4x4 => 4,
            _ => 0,
        }
    }

    /// Sets the Coded Block Pattern luma and chroma values based off
    /// Tables 7-11 and 7-12 of the Spec
    pub fn set_cbp_chroma_and_luma(&mut self) {
        // Table 7-11 and 7-12
        if self.mb_type == MbType::I16x16_0_0_0
            || self.mb_type == MbType::I16x16_1_0_0
            || self.mb_type == MbType::I16x16_2_0_0
            || self.mb_type == MbType::I16x16_3_0_0
        {
            self.coded_block_pattern_luma = 0;
            self.coded_block_pattern_chroma = 0;
        }

        if self.mb_type == MbType::I16x16_0_1_0
            || self.mb_type == MbType::I16x16_1_1_0
            || self.mb_type == MbType::I16x16_2_1_0
            || self.mb_type == MbType::I16x16_3_1_0
        {
            self.coded_block_pattern_luma = 0;
            self.coded_block_pattern_chroma = 1;
        }

        if self.mb_type == MbType::I16x16_0_2_0
            || self.mb_type == MbType::I16x16_1_2_0
            || self.mb_type == MbType::I16x16_2_2_0
            || self.mb_type == MbType::I16x16_3_2_0
        {
            self.coded_block_pattern_luma = 0;
            self.coded_block_pattern_chroma = 2;
        }

        if self.mb_type == MbType::I16x16_0_0_1
            || self.mb_type == MbType::I16x16_1_0_1
            || self.mb_type == MbType::I16x16_2_0_1
            || self.mb_type == MbType::I16x16_3_0_1
        {
            self.coded_block_pattern_luma = 15;
            self.coded_block_pattern_chroma = 0;
        }

        if self.mb_type == MbType::I16x16_0_1_1
            || self.mb_type == MbType::I16x16_1_1_1
            || self.mb_type == MbType::I16x16_2_1_1
            || self.mb_type == MbType::I16x16_3_1_1
        {
            self.coded_block_pattern_luma = 15;
            self.coded_block_pattern_chroma = 1;
        }

        if self.mb_type == MbType::I16x16_0_2_1
            || self.mb_type == MbType::I16x16_1_2_1
            || self.mb_type == MbType::I16x16_2_2_1
            || self.mb_type == MbType::I16x16_3_2_1
        {
            self.coded_block_pattern_luma = 15;
            self.coded_block_pattern_chroma = 2;
        }
    }

    /// Implements tables 7-13 and 7-14 --  Returns the (width, height) of an mb_type
    pub fn mb_part_pred_width_and_height(&self) -> (usize, usize) {
        match self.mb_type {
            // Table 7-13
            MbType::PL016x16 => (16, 16),
            MbType::PL0L016x8 => (16, 8),
            MbType::PL0L08x16 => (8, 16),
            MbType::P8x8 => (8, 8),
            MbType::P8x8ref0 => (8, 8),
            MbType::PSkip => (16, 16),
            // Table 7-14
            MbType::BDirect16x16 => (8, 8),
            MbType::BL016x16 => (16, 16),
            MbType::BL116x16 => (16, 16),
            MbType::BBi16x16 => (16, 16),
            MbType::BL0L016x8 => (16, 8),
            MbType::BL0L08x16 => (8, 16),
            MbType::BL1L116x8 => (16, 8),
            MbType::BL1L18x16 => (8, 16),
            MbType::BL0L116x8 => (16, 8),
            MbType::BL0L18x16 => (8, 16),
            MbType::BL1L016x8 => (16, 8),
            MbType::BL1L08x16 => (8, 16),
            MbType::BL0Bi16x8 => (16, 8),
            MbType::BL0Bi8x16 => (8, 16),
            MbType::BL1Bi16x8 => (16, 8),
            MbType::BL1Bi8x16 => (8, 16),
            MbType::BBiL016x8 => (16, 8),
            MbType::BBiL08x16 => (8, 16),
            MbType::BBiL116x8 => (16, 8),
            MbType::BBiL18x16 => (8, 16),
            MbType::BBiBi16x8 => (16, 8),
            MbType::BBiBi8x16 => (8, 16),
            MbType::B8x8 => (8, 8),
            MbType::BSkip => (8, 8),
            _ => (16, 16), // return 16, 16 for all other types for simplicity (PCM + I slices)
        }
    }

    /// Implements tables 7-17 and 7-18
    /// Returns the (width, height) of a sub_mb_type
    pub fn sub_mb_part_pred_width_and_height(&self, mb_part_idx: usize) -> (usize, usize) {
        match self.sub_mb_type[mb_part_idx] {
            // Table 7-17
            SubMbType::PL08x8 => (8, 8),
            SubMbType::PL08x4 => (8, 4),
            SubMbType::PL04x8 => (4, 8),
            SubMbType::PL04x4 => (4, 4),
            // Table 7-18
            SubMbType::BDirect8x8 => (4, 4),

            SubMbType::BL08x8 => (8, 8),
            SubMbType::BL18x8 => (8, 8),
            SubMbType::BBi8x8 => (8, 8),

            SubMbType::BL08x4 => (8, 4),
            SubMbType::BL04x8 => (4, 8),

            SubMbType::BL18x4 => (8, 4),
            SubMbType::BL14x8 => (4, 8),

            SubMbType::BBi8x4 => (8, 4),
            SubMbType::BBi4x8 => (4, 8),

            SubMbType::BL04x4 => (4, 4),
            SubMbType::BL14x4 => (4, 4),
            SubMbType::BBi4x4 => (4, 4),
            _ => (0, 0),
        }
    }

    /// Returns True if the current MacroBlock is inter predicted
    pub fn is_inter(&mut self) -> bool {
        let res = self.mb_part_pred_mode(0);
        res == MbPartPredMode::BiPred
            || res == MbPartPredMode::Direct
            || res == MbPartPredMode::PredL0
            || res == MbPartPredMode::PredL1
    }

    /// Returns True if the current MacroBlock is intra predicted
    pub fn is_intra(&mut self) -> bool {
        let res = self.mb_part_pred_mode(0);
        res == MbPartPredMode::Intra4x4
            || res == MbPartPredMode::Intra8x8
            || res == MbPartPredMode::Intra16x16
    }

    /// A non-mutating version of is_intra()
    pub fn is_intra_non_mut(&self) -> bool {
        let res = self.mb_part_pred_mode(0);
        res == MbPartPredMode::Intra4x4
            || res == MbPartPredMode::Intra8x8
            || res == MbPartPredMode::Intra16x16
    }

    /// Used in CAVLC decoding - returns true if all the AC residue transform coefficient levels of the
    /// neighboring block are equal to 0
    pub fn ac_resid_all_zero(&mut self, mode: u8, blk_idx: usize) -> bool {
        // check the coded_block_pattern_luma or coded_block_pattern_chroma bits

        match mode {
            // i16x16aclevel
            0 => {
                if self.coded_block_pattern_luma & ((1 << blk_idx) >> blk_idx) > 0
                    && self.mb_part_pred_mode(0) == MbPartPredMode::Intra16x16
                {
                    for i in 0..self.intra_16x16_ac_level.len() {
                        for j in 0..self.intra_16x16_ac_level[i].len() {
                            if self.intra_16x16_ac_level[i][j] != 0 {
                                return false;
                            }
                        }
                    }
                } else {
                    return false;
                }
            }
            // cbi16x16aclevel
            1 => {
                if self.coded_block_pattern_chroma & 2 > 0 {
                    for i in 0..self.cb_intra_16x16_ac_level.len() {
                        for j in 0..self.cb_intra_16x16_ac_level[i].len() {
                            if self.cb_intra_16x16_ac_level[i][j] != 0 {
                                return false;
                            }
                        }
                    }
                } else {
                    return false;
                }
            }
            // cri16x16aclevel
            2 => {
                if self.coded_block_pattern_chroma & 2 > 0 {
                    for i in 0..self.cr_intra_16x16_ac_level.len() {
                        for j in 0..self.cr_intra_16x16_ac_level[i].len() {
                            if self.cr_intra_16x16_ac_level[i][j] != 0 {
                                return false;
                            }
                        }
                    }
                } else {
                    return false;
                }
            }
            // chroma AC
            3 => {
                if self.coded_block_pattern_chroma & 2 == 0 {
                    for i in 0..self.chroma_ac_level.len() {
                        for j in 0..self.chroma_ac_level[i].len() {
                            for k in 0..self.chroma_ac_level[i][j].len() {
                                if self.chroma_ac_level[i][j][k] != 0 {
                                    return false;
                                }
                            }
                        }
                    }
                } else {
                    return false;
                }
            }
            _ => (),
        }

        true
    }
}

impl Default for MacroBlock {
    fn default() -> Self {
        Self::new()
    }
}

/// Slice Header syntax elements
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SliceHeader {
    pub first_mb_in_slice: u32,    //ue(v)
    pub slice_type: u8,            //ue(v)
    pub pic_parameter_set_id: u32, //ue(v)
    pub colour_plane_id: u8,       // u(2)
    pub frame_num: u32,
    pub field_pic_flag: bool,
    pub bottom_field_flag: bool,
    pub idr_pic_id: u32, //ue(v)
    pub pic_order_cnt_lsb: u32,
    pub delta_pic_order_cnt_bottom: i32, //se(v)
    pub delta_pic_order_cnt: Vec<i32>,   //se(v) [2]
    pub redundant_pic_cnt: u32,          //ue(v)
    pub direct_spatial_mv_pred_flag: bool,
    pub num_ref_idx_active_override_flag: bool,
    pub num_ref_idx_l0_active_minus1: u32, //ue(v)
    pub num_ref_idx_l1_active_minus1: u32, //ue(v)
    pub ref_pic_list_modification_flag_l0: bool,
    pub ref_pic_list_modification_flag_l1: bool,

    // these variables exist for list 0 and list 1
    pub modification_of_pic_nums_idc_l0: Vec<u32>,
    pub abs_diff_pic_num_minus1_l0: Vec<u32>,
    pub long_term_pic_num_l0: Vec<u32>,
    pub modification_of_pic_nums_idc_l1: Vec<u32>,
    pub abs_diff_pic_num_minus1_l1: Vec<u32>,
    pub long_term_pic_num_l1: Vec<u32>,
    // pred_weight_table
    pub luma_log2_weight_denom: u32,
    pub chroma_log2_weight_denom: u32,
    pub luma_weight_l0_flag: Vec<bool>,
    pub luma_weight_l0: Vec<i32>, //len = num_ref_idx_l1_active_minus1
    pub luma_offset_l0: Vec<i32>, //len = num_ref_idx_l1_active_minus1
    pub chroma_weight_l0_flag: Vec<bool>,
    pub chroma_weight_l0: Vec<Vec<i32>>, // 2 elements each
    pub chroma_offset_l0: Vec<Vec<i32>>,
    pub luma_weight_l1_flag: Vec<bool>,
    pub luma_weight_l1: Vec<i32>, //len = num_ref_idx_l1_active_minus1
    pub luma_offset_l1: Vec<i32>, //len = num_ref_idx_l1_active_minus1
    pub chroma_weight_l1_flag: Vec<bool>,
    pub chroma_weight_l1: Vec<Vec<i32>>, // 2 elements each
    pub chroma_offset_l1: Vec<Vec<i32>>, //
    // dec_ref_pic_marking
    pub no_output_of_prior_pics_flag: bool,
    pub long_term_reference_flag: bool,
    pub adaptive_ref_pic_marking_mode_flag: bool,
    pub memory_management_control_operation: Vec<u32>,
    pub difference_of_pic_nums_minus1: Vec<u32>,
    pub long_term_pic_num: Vec<u32>,
    pub long_term_frame_idx: Vec<u32>,
    pub max_long_term_frame_idx_plus1: Vec<u32>,
    //
    pub cabac_init_idc: u32, //ue(v)
    pub slice_qp_delta: i32, //se(v)
    pub sp_for_switch_flag: bool,
    pub slice_qs_delta: i32,                //se(v)
    pub disable_deblocking_filter_idc: u32, //ue(v)
    pub slice_alpha_c0_offset_div2: i32,    //se(v)
    pub slice_beta_offset_div2: i32,        //se(v)
    pub slice_group_change_cycle: u32,

    //}
    // The following values are calculated from the stream information
    // Spec equation references are included for more information
    pub prev_ref_frame_num: u32,           // page 87
    pub mbaff_frame_flag: bool,            // 7-25
    pub pic_height_in_mbs: u32,            // 7-26
    pub pic_height_in_samples_luma: u32,   // 7-27
    pub pic_height_in_samples_chroma: u32, // 7-28
    pub pic_size_in_mbs: u32,              // 7-29
    pub max_pic_num: u32,                  // end of field_pic_flag area
    pub curr_pic_num: u32,                 // end of field_pic_flag area
    pub slice_qp_y: i32,                   // 7-30
    pub qp_y_prev: i32,
    pub qs_y: u8,             // 7-31
    pub filter_offset_a: i32, // 7-32
    pub filter_offset_b: i32, // 7-33

    // Annex H addendum
    pub abs_diff_view_idx_minus1_l0: Vec<u32>,
    pub abs_diff_view_idx_minus1_l1: Vec<u32>,
}

impl SliceHeader {
    pub fn new() -> SliceHeader {
        SliceHeader {
            first_mb_in_slice: 0,
            slice_type: 0,
            pic_parameter_set_id: 0,
            colour_plane_id: 0,
            frame_num: 0,
            field_pic_flag: false,
            bottom_field_flag: false,
            idr_pic_id: 0,
            pic_order_cnt_lsb: 0,
            delta_pic_order_cnt_bottom: 0,
            delta_pic_order_cnt: Vec::new(),
            redundant_pic_cnt: 0,
            direct_spatial_mv_pred_flag: false,
            num_ref_idx_active_override_flag: false,
            num_ref_idx_l0_active_minus1: 0,
            num_ref_idx_l1_active_minus1: 0,
            ref_pic_list_modification_flag_l0: false,
            ref_pic_list_modification_flag_l1: false,
            modification_of_pic_nums_idc_l0: Vec::new(),
            abs_diff_pic_num_minus1_l0: Vec::new(),
            long_term_pic_num_l0: Vec::new(),
            modification_of_pic_nums_idc_l1: Vec::new(),
            abs_diff_pic_num_minus1_l1: Vec::new(),
            long_term_pic_num_l1: Vec::new(),
            luma_log2_weight_denom: 0,
            chroma_log2_weight_denom: 0,
            luma_weight_l0_flag: Vec::new(),
            luma_weight_l0: Vec::new(),
            luma_offset_l0: Vec::new(),
            chroma_weight_l0_flag: Vec::new(),
            chroma_weight_l0: Vec::new(),
            chroma_offset_l0: Vec::new(),
            luma_weight_l1_flag: Vec::new(),
            luma_weight_l1: Vec::new(),
            luma_offset_l1: Vec::new(),
            chroma_weight_l1_flag: Vec::new(),
            chroma_weight_l1: Vec::new(),
            chroma_offset_l1: Vec::new(),
            no_output_of_prior_pics_flag: false,
            long_term_reference_flag: false,
            adaptive_ref_pic_marking_mode_flag: false,
            memory_management_control_operation: Vec::new(),
            difference_of_pic_nums_minus1: Vec::new(),
            long_term_pic_num: Vec::new(),
            long_term_frame_idx: Vec::new(),
            max_long_term_frame_idx_plus1: Vec::new(),
            cabac_init_idc: 0,
            slice_qp_delta: 0,
            sp_for_switch_flag: false,
            slice_qs_delta: 0,
            disable_deblocking_filter_idc: 0,
            slice_alpha_c0_offset_div2: 0,
            slice_beta_offset_div2: 0,
            slice_group_change_cycle: 0,
            prev_ref_frame_num: 0,
            mbaff_frame_flag: false,
            pic_height_in_mbs: 0,
            pic_height_in_samples_luma: 0,
            pic_height_in_samples_chroma: 0,
            pic_size_in_mbs: 0,
            max_pic_num: 0,
            curr_pic_num: 0,
            slice_qp_y: 0,
            qp_y_prev: 0,
            qs_y: 0,
            filter_offset_a: 0,
            filter_offset_b: 0,
            abs_diff_view_idx_minus1_l0: Vec::new(),
            abs_diff_view_idx_minus1_l1: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn pretty_print(&self) {
        formatted_print("SH: first_mb_in_slice", self.first_mb_in_slice, 63);
        formatted_print("SH: slice_type", self.slice_type, 63);
        formatted_print("SH: pic_parameter_set_id", self.pic_parameter_set_id, 63);
        formatted_print("SH: colour_plane_id", self.colour_plane_id, 63);
        formatted_print("SH: frame_num", self.frame_num, 63);
        formatted_print("SH: field_pic_flag", self.field_pic_flag, 63);
        formatted_print("SH: bottom_field_flag", self.bottom_field_flag, 63);
        formatted_print("SH: idr_pic_id", self.idr_pic_id, 63);
        formatted_print("SH: pic_order_cnt_lsb", self.pic_order_cnt_lsb, 63);
        formatted_print(
            "SH: delta_pic_order_cnt_bottom",
            self.delta_pic_order_cnt_bottom,
            63,
        );
        formatted_print("SH: delta_pic_order_cnt", &self.delta_pic_order_cnt, 63);
        formatted_print("SH: redundant_pic_cnt", self.redundant_pic_cnt, 63);
        formatted_print(
            "SH: direct_spatial_mv_pred_flag",
            self.direct_spatial_mv_pred_flag,
            63,
        );
        formatted_print(
            "SH: num_ref_idx_override_flag",
            self.num_ref_idx_active_override_flag,
            63,
        );
        formatted_print(
            "SH: num_ref_idx_l0_active_minus1",
            self.num_ref_idx_l0_active_minus1,
            63,
        );
        formatted_print(
            "SH: num_ref_idx_l1_active_minus1",
            self.num_ref_idx_l1_active_minus1,
            63,
        );
        formatted_print(
            "SH: ref_pic_list_reordering_flag_l0",
            self.ref_pic_list_modification_flag_l0,
            63,
        );
        formatted_print(
            "SH: modification_of_pic_nums_idc_l0",
            &self.modification_of_pic_nums_idc_l0,
            63,
        );
        formatted_print(
            "SH: abs_diff_pic_num_minus1_l0",
            &self.abs_diff_pic_num_minus1_l0,
            63,
        );
        formatted_print("SH: long_term_pic_num_l0", &self.long_term_pic_num_l0, 63);
        formatted_print(
            "SH: ref_pic_list_reordering_flag_l1",
            self.ref_pic_list_modification_flag_l1,
            63,
        );
        formatted_print(
            "SH: modification_of_pic_nums_idc_l1",
            &self.modification_of_pic_nums_idc_l1,
            63,
        );
        formatted_print(
            "SH: abs_diff_pic_num_minus1_l1",
            &self.abs_diff_pic_num_minus1_l1,
            63,
        );
        formatted_print("SH: long_term_pic_num_l1", &self.long_term_pic_num_l1, 63);
        formatted_print(
            "SH: luma_log2_weight_denom",
            self.luma_log2_weight_denom,
            63,
        );
        formatted_print(
            "SH: chroma_log2_weight_denom",
            self.chroma_log2_weight_denom,
            63,
        );
        formatted_print("SH: luma_weight_flag_l0", &self.luma_weight_l0_flag, 63);
        formatted_print("SH: luma_weight_l0", &self.luma_weight_l0, 63);
        formatted_print("SH: luma_offset_l0", &self.luma_offset_l0, 63);
        formatted_print("SH: chroma_weight_flag_l0", &self.chroma_weight_l0_flag, 63);
        formatted_print("SH: chroma_weight_l0", &self.chroma_weight_l0, 63);
        formatted_print("SH: chroma_offset_l0", &self.chroma_offset_l0, 63);
        formatted_print("SH: luma_weight_l1_flag", &self.luma_weight_l1_flag, 63);
        formatted_print("SH: luma_weight_l1", &self.luma_weight_l1, 63);
        formatted_print("SH: luma_offset_l1", &self.luma_offset_l1, 63);
        formatted_print("SH: chroma_weight_l1_flag", &self.chroma_weight_l1_flag, 63);
        formatted_print("SH: chroma_weight_l1", &self.chroma_weight_l1, 63);
        formatted_print("SH: chroma_offset_l1", &self.chroma_offset_l1, 63);
        formatted_print(
            "SH: no_output_of_prior_pics_flag",
            self.no_output_of_prior_pics_flag,
            63,
        );
        formatted_print(
            "SH: long_term_reference_flag",
            self.long_term_reference_flag,
            63,
        );
        formatted_print(
            "SH: adaptive_ref_pic_buffering_flag",
            self.adaptive_ref_pic_marking_mode_flag,
            63,
        );
        formatted_print(
            "SH: memory_management_control_operation",
            &self.memory_management_control_operation,
            63,
        );
        formatted_print(
            "SH: difference_of_pic_nums_minus1",
            &self.difference_of_pic_nums_minus1,
            63,
        );
        formatted_print("SH: long_term_pic_num", &self.long_term_pic_num, 63);
        formatted_print("SH: long_term_frame_idx", &self.long_term_frame_idx, 63);
        formatted_print(
            "SH: max_long_term_frame_idx_plus1",
            &self.max_long_term_frame_idx_plus1,
            63,
        );
        formatted_print("SH: cabac_init_idc", self.cabac_init_idc, 63);
        formatted_print("SH: slice_qp_delta", self.slice_qp_delta, 63);
        formatted_print("SH: sp_for_switch_flag", self.sp_for_switch_flag, 63);
        formatted_print("SH: slice_qs_delta", self.slice_qs_delta, 63);
        formatted_print(
            "SH: disable_deblocking_filter_idc",
            self.disable_deblocking_filter_idc,
            63,
        );
        formatted_print(
            "SH: slice_alpha_c0_offset_div2",
            self.slice_alpha_c0_offset_div2,
            63,
        );
        formatted_print(
            "SH: slice_beta_offset_div2",
            self.slice_beta_offset_div2,
            63,
        );
        formatted_print(
            "SH: slice_group_change_cycle",
            self.slice_group_change_cycle,
            63,
        );
        formatted_print(
            "SH: abs_diff_view_idx_minus1_l0",
            &self.abs_diff_view_idx_minus1_l0,
            63,
        );
        formatted_print(
            "SH: abs_diff_view_idx_minus1_l1",
            &self.abs_diff_view_idx_minus1_l1,
            63,
        );
    }

    #[allow(dead_code)]
    pub fn clone(&self) -> SliceHeader {
        SliceHeader {
            first_mb_in_slice: self.first_mb_in_slice,
            slice_type: self.slice_type,
            pic_parameter_set_id: self.pic_parameter_set_id,
            colour_plane_id: self.colour_plane_id,
            frame_num: self.frame_num,
            field_pic_flag: self.field_pic_flag,
            bottom_field_flag: self.bottom_field_flag,
            idr_pic_id: self.idr_pic_id,
            pic_order_cnt_lsb: self.pic_order_cnt_lsb,
            delta_pic_order_cnt_bottom: self.delta_pic_order_cnt_bottom,
            delta_pic_order_cnt: self.delta_pic_order_cnt.clone(),
            redundant_pic_cnt: self.redundant_pic_cnt,
            direct_spatial_mv_pred_flag: self.direct_spatial_mv_pred_flag,
            num_ref_idx_active_override_flag: self.num_ref_idx_active_override_flag,
            num_ref_idx_l0_active_minus1: self.num_ref_idx_l0_active_minus1,
            num_ref_idx_l1_active_minus1: self.num_ref_idx_l1_active_minus1,
            ref_pic_list_modification_flag_l0: self.ref_pic_list_modification_flag_l0,
            ref_pic_list_modification_flag_l1: self.ref_pic_list_modification_flag_l1,
            modification_of_pic_nums_idc_l0: self.modification_of_pic_nums_idc_l0.clone(),
            abs_diff_pic_num_minus1_l0: self.abs_diff_pic_num_minus1_l0.clone(),
            long_term_pic_num_l0: self.long_term_pic_num_l0.clone(),
            modification_of_pic_nums_idc_l1: self.modification_of_pic_nums_idc_l1.clone(),
            abs_diff_pic_num_minus1_l1: self.abs_diff_pic_num_minus1_l1.clone(),
            long_term_pic_num_l1: self.long_term_pic_num_l1.clone(),
            luma_log2_weight_denom: self.luma_log2_weight_denom,
            chroma_log2_weight_denom: self.chroma_log2_weight_denom,
            luma_weight_l0_flag: self.luma_weight_l0_flag.clone(),
            luma_weight_l0: self.luma_weight_l0.clone(),
            luma_offset_l0: self.luma_offset_l0.clone(),
            chroma_weight_l0_flag: self.chroma_weight_l0_flag.clone(),
            chroma_weight_l0: self.chroma_weight_l0.clone(),
            chroma_offset_l0: self.chroma_offset_l0.clone(),
            luma_weight_l1_flag: self.luma_weight_l1_flag.clone(),
            luma_weight_l1: self.luma_weight_l1.clone(),
            luma_offset_l1: self.luma_offset_l1.clone(),
            chroma_weight_l1_flag: self.chroma_weight_l1_flag.clone(),
            chroma_weight_l1: self.chroma_weight_l1.clone(),
            chroma_offset_l1: self.chroma_offset_l1.clone(),
            no_output_of_prior_pics_flag: self.no_output_of_prior_pics_flag,
            long_term_reference_flag: self.long_term_reference_flag,
            adaptive_ref_pic_marking_mode_flag: self.adaptive_ref_pic_marking_mode_flag,
            memory_management_control_operation: self.memory_management_control_operation.clone(),
            difference_of_pic_nums_minus1: self.difference_of_pic_nums_minus1.clone(),
            long_term_pic_num: self.long_term_pic_num.clone(),
            long_term_frame_idx: self.long_term_frame_idx.clone(),
            max_long_term_frame_idx_plus1: self.max_long_term_frame_idx_plus1.clone(),
            cabac_init_idc: self.cabac_init_idc,
            slice_qp_delta: self.slice_qp_delta,
            sp_for_switch_flag: self.sp_for_switch_flag,
            slice_qs_delta: self.slice_qs_delta,
            disable_deblocking_filter_idc: self.disable_deblocking_filter_idc,
            slice_alpha_c0_offset_div2: self.slice_alpha_c0_offset_div2,
            slice_beta_offset_div2: self.slice_beta_offset_div2,
            slice_group_change_cycle: self.slice_group_change_cycle,
            prev_ref_frame_num: self.prev_ref_frame_num,
            mbaff_frame_flag: self.mbaff_frame_flag,
            pic_height_in_mbs: self.pic_height_in_mbs,
            pic_height_in_samples_luma: self.pic_height_in_samples_luma,
            pic_height_in_samples_chroma: self.pic_height_in_samples_chroma,
            pic_size_in_mbs: self.pic_size_in_mbs,
            max_pic_num: self.max_pic_num,
            curr_pic_num: self.curr_pic_num,
            slice_qp_y: self.slice_qp_y,
            qp_y_prev: self.qp_y_prev,
            qs_y: self.qs_y,
            filter_offset_a: self.filter_offset_a,
            filter_offset_b: self.filter_offset_b,
            abs_diff_view_idx_minus1_l0: self.abs_diff_view_idx_minus1_l0.clone(),
            abs_diff_view_idx_minus1_l1: self.abs_diff_view_idx_minus1_l1.clone(),
        }
    }

    /// Produce the slice group map that will be used by next_mb_addr to handle FMO.
    /// Follows the process described in section 8.2.2
    pub fn generate_slice_group_map(
        &self,
        s: &SeqParameterSet,
        p: &PicParameterSet,
        vp: &VideoParameters,
    ) -> Vec<u32> {
        let slice_group_change_rate = p.slice_group_change_rate_minus1 + 1;

        // Equation 7-34
        let map_units_in_slice_group0 = cmp::min(
            self.slice_group_change_cycle * slice_group_change_rate,
            vp.pic_size_in_map_units,
        );

        // Start of 8.2.2
        let size_of_upper_left_group = if p.num_slice_groups_minus1 == 1
            && (p.slice_group_map_type == 4 || p.slice_group_map_type == 5)
        {
            if p.slice_group_change_direction_flag {
                vp.pic_size_in_map_units - map_units_in_slice_group0
            } else {
                map_units_in_slice_group0
            }
        } else {
            0
        };

        // construct the mapUnitToSliceGroupMap:
        let map_unit_to_slice_group_map = match p.num_slice_groups_minus1 {
            0 => {
                vec![0; vp.pic_size_in_map_units as usize]
            }
            _ => {
                let mut res = vec![0; vp.pic_size_in_map_units as usize];

                match p.slice_group_map_type {
                    0 => {
                        // Use clause 8.2.2.1
                        let mut i = 0;

                        while i < vp.pic_size_in_map_units {
                            for i_group in 0..=p.num_slice_groups_minus1 {
                                for j in 0..=p.run_length_minus1[i_group as usize] {
                                    if i + j < vp.pic_size_in_map_units {
                                        res[(i + j) as usize] = i_group;
                                    }
                                }
                                i += p.run_length_minus1[i_group as usize] + 1;
                            }
                        }
                        res
                    }
                    1 => {
                        // Use clause 8.2.2.2
                        for i in 0..vp.pic_size_in_map_units {
                            let val = ((i % vp.pic_width_in_mbs)
                                + (((i / vp.pic_width_in_mbs) * (p.num_slice_groups_minus1 + 1))
                                    / 2))
                                % (p.num_slice_groups_minus1 + 1);
                            res[i as usize] = val;
                        }

                        res
                    }
                    2 => {
                        // Use clause 8.2.2.3
                        for i in 0..vp.pic_size_in_map_units {
                            res[i as usize] = p.num_slice_groups_minus1;
                        }

                        for i_group in (0..p.num_slice_groups_minus1).rev() {
                            let y_top_left = p.top_left[i_group as usize] / vp.pic_width_in_mbs;
                            let x_top_left = p.top_left[i_group as usize] % vp.pic_width_in_mbs;
                            let y_bottom_right =
                                p.bottom_right[i_group as usize] / vp.pic_width_in_mbs;
                            let x_bottom_right =
                                p.bottom_right[i_group as usize] % vp.pic_width_in_mbs;
                            for y in y_top_left..=y_bottom_right {
                                for x in x_top_left..=x_bottom_right {
                                    res[(y * vp.pic_width_in_mbs + x) as usize] = i_group;
                                }
                            }
                        }

                        res
                    }
                    3 => {
                        // Use clause 8.2.2.4
                        for i in 0..vp.pic_size_in_map_units {
                            res[i as usize] = 1;
                        }

                        let slice_group_change_direction_int: i32 =
                            match p.slice_group_change_direction_flag {
                                true => 1,
                                _ => 0,
                            };

                        let mut x =
                            (vp.pic_width_in_mbs - slice_group_change_direction_int as u32) / 2;
                        let mut y = (vp.pic_height_in_map_units
                            - slice_group_change_direction_int as u32)
                            / 2;

                        let mut left_bound = x;
                        let mut top_bound = y;
                        let mut right_bound = x;
                        let mut bottom_bound = y;

                        let mut x_dir = slice_group_change_direction_int - 1;
                        let mut y_dir = slice_group_change_direction_int;

                        let mut k = 0;
                        let mut map_unit_vacant: bool;

                        while k < map_units_in_slice_group0 {
                            map_unit_vacant = res[(y * vp.pic_width_in_mbs + x) as usize] == 1;
                            if map_unit_vacant {
                                res[(y * vp.pic_width_in_mbs + x) as usize] = 0;
                            }

                            if x_dir == -1 && x == left_bound {
                                left_bound = cmp::max(left_bound - 1, 0);
                                x = left_bound;

                                x_dir = 0;
                                y_dir = 2 * slice_group_change_direction_int - 1;
                            } else if x_dir == 1 && x == right_bound {
                                right_bound = cmp::min(right_bound + 1, vp.pic_width_in_mbs - 1);
                                x = right_bound;

                                x_dir = 0;
                                y_dir = 1 - 2 * slice_group_change_direction_int;
                            } else if y_dir == -1 && y == top_bound {
                                top_bound = cmp::max(top_bound - 1, 0);
                                y = top_bound;

                                x_dir = 1 - 2 * slice_group_change_direction_int;
                                y_dir = 0;
                            } else if y_dir == 1 && y == bottom_bound {
                                bottom_bound =
                                    cmp::min(bottom_bound + 1, vp.pic_height_in_map_units - 1);
                                y = bottom_bound;

                                x_dir = 2 * slice_group_change_direction_int - 1;
                                y_dir = 0;
                            } else {
                                // x and y are always non-negative
                                x = (x as i32 + x_dir) as u32;
                                y = (y as i32 + y_dir) as u32;
                            }

                            k += match map_unit_vacant {
                                true => 1,
                                _ => 0,
                            };
                        }

                        res
                    }
                    4 => {
                        // Use clause 8.2.2.5
                        for i in 0..vp.pic_size_in_map_units {
                            if i < size_of_upper_left_group {
                                res[i as usize] = match p.slice_group_change_direction_flag {
                                    true => 1,
                                    _ => 0,
                                };
                            } else {
                                res[i as usize] = 1 - match p.slice_group_change_direction_flag {
                                    true => 1,
                                    _ => 0,
                                };
                            }
                        }
                        res
                    }
                    5 => {
                        // Use clause 8.2.2.6
                        let mut k = 0;
                        for j in 0..vp.pic_width_in_mbs {
                            for i in 0..vp.pic_height_in_map_units {
                                if k < size_of_upper_left_group {
                                    res[(i * vp.pic_width_in_mbs + j) as usize] =
                                        match p.slice_group_change_direction_flag {
                                            true => 1,
                                            _ => 0,
                                        };
                                } else {
                                    res[(i * vp.pic_width_in_mbs + j) as usize] =
                                        1 - match p.slice_group_change_direction_flag {
                                            true => 1,
                                            _ => 0,
                                        };
                                }
                                k += 1;
                            }
                        }
                        res
                    }
                    6 => {
                        // Use clause 8.2.2.7
                        res = p.slice_group_id.clone();
                        res
                    }
                    _ => {
                        panic!(
                            "Out-of-bounds num_slice_groups_minus1: {}",
                            p.num_slice_groups_minus1
                        )
                    }
                }
            }
        };

        // Use clause 8.2.2.8
        let sgm = if s.frame_mbs_only_flag || self.field_pic_flag {
            map_unit_to_slice_group_map
        } else if self.mbaff_frame_flag {
            let mut res = Vec::new();
            for i in 0..self.pic_size_in_mbs {
                res.push(map_unit_to_slice_group_map[(i / 2) as usize]);
            }
            res
        } else {
            let mut res = Vec::new();
            for i in 0..self.pic_size_in_mbs {
                res.push(
                    map_unit_to_slice_group_map[((i / (2 * vp.pic_width_in_mbs))
                        * vp.pic_width_in_mbs
                        + (i % vp.pic_width_in_mbs))
                        as usize],
                );
            }
            res
        };

        sgm
    }
}

impl Default for SliceHeader {
    fn default() -> Self {
        Self::new()
    }
}

/// Neighboring macroblock addresses
#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum NeighborMB {
    MbAddrA,
    MbAddrB,
    MbAddrC,
    MbAddrD,
}

/// Slice Data syntax elements
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SliceData {
    pub mb_skip_run: Vec<u32>,
    pub mb_field_decoding_flag: Vec<bool>,
    pub end_of_slice_flag: Vec<bool>,

    pub macroblock_vec: Vec<MacroBlock>,
}

/// Enable to output neighbor debug messages
const NEIGHBOR_DEBUG: bool = false;

impl SliceData {
    pub fn new() -> SliceData {
        SliceData {
            mb_skip_run: Vec::new(),
            mb_field_decoding_flag: Vec::new(),
            end_of_slice_flag: Vec::new(),
            macroblock_vec: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn pretty_print(&self) {
        formatted_print("mb_skip_run", &self.mb_skip_run, 63);
        formatted_print("mb_field_decoding_flag", &self.mb_field_decoding_flag, 63);
        formatted_print("end_of_slice_flag", &self.end_of_slice_flag, 63);
    }

    /// Returns the previously decoded macroblock, not related to neighbor
    pub fn get_previous_macroblock(&self, curr_mb_idx: usize) -> MacroBlock {
        if curr_mb_idx > 0 {
            self.macroblock_vec[curr_mb_idx - 1].clone()
        } else {
            MacroBlock::new()
        }
    }

    /// Returns neighbor macroblocks and availability per Section 6.4.9
    pub fn get_neighbor_macroblock(
        &self,
        curr_mb_idx: usize,
        neighbor_type: NeighborMB,
        vp: &VideoParameters,
    ) -> MacroBlock {
        let result_mb: MacroBlock;

        if NEIGHBOR_DEBUG {
            debug!(target: "decode","get_neighbor_macroblock - curr_mb_idx {} and mb_addr {} and mbtype {:?} and neighbor_type {:?} ", curr_mb_idx,  self.macroblock_vec[curr_mb_idx].mb_addr, self.macroblock_vec[curr_mb_idx].mb_type, neighbor_type);
        }

        // Follow Figure 6-12 for neighboring macroblocks
        match neighbor_type {
            NeighborMB::MbAddrA => {
                if self.macroblock_vec[curr_mb_idx].mb_addr % (vp.pic_width_in_mbs as usize) == 0
                    || curr_mb_idx == 0
                {
                    result_mb = MacroBlock::new();
                } else {
                    // check if mb_a is in this slice or not
                    let mb_a_idx = curr_mb_idx - 1;
                    let mb_a_addr = self.macroblock_vec[curr_mb_idx].mb_addr - 1;
                    if mb_a_addr == self.macroblock_vec[mb_a_idx].mb_addr {
                        result_mb = self.macroblock_vec[mb_a_idx].clone();
                    } else {
                        result_mb = MacroBlock::new();
                    }
                }
            }
            NeighborMB::MbAddrB => {
                if self.macroblock_vec[curr_mb_idx].mb_addr < (vp.pic_width_in_mbs as usize)
                    || curr_mb_idx < (vp.pic_width_in_mbs as usize)
                {
                    result_mb = MacroBlock::new();
                } else {
                    let mb_b_idx = curr_mb_idx - (vp.pic_width_in_mbs as usize);
                    let mb_b_addr =
                        self.macroblock_vec[curr_mb_idx].mb_addr - (vp.pic_width_in_mbs as usize);
                    if mb_b_addr == self.macroblock_vec[mb_b_idx].mb_addr {
                        result_mb = self.macroblock_vec[mb_b_idx].clone();
                    } else {
                        result_mb = MacroBlock::new();
                    }
                }
            }
            NeighborMB::MbAddrC => {
                if self.macroblock_vec[curr_mb_idx].mb_addr < (vp.pic_width_in_mbs as usize)
                    || (self.macroblock_vec[curr_mb_idx].mb_addr + 1)
                        % (vp.pic_width_in_mbs as usize)
                        == 0
                    || curr_mb_idx < (vp.pic_width_in_mbs as usize) - 1
                {
                    result_mb = MacroBlock::new();
                } else {
                    let mb_c_idx = (curr_mb_idx + 1) - (vp.pic_width_in_mbs as usize);
                    let mb_c_addr = self.macroblock_vec[curr_mb_idx].mb_addr
                        - (vp.pic_width_in_mbs as usize)
                        + 1;
                    if mb_c_addr == self.macroblock_vec[mb_c_idx].mb_addr {
                        result_mb = self.macroblock_vec[mb_c_idx].clone();
                    } else {
                        result_mb = MacroBlock::new();
                    }
                }
            }
            NeighborMB::MbAddrD => {
                if self.macroblock_vec[curr_mb_idx].mb_addr % (vp.pic_width_in_mbs as usize) == 0
                    || self.macroblock_vec[curr_mb_idx].mb_addr < (vp.pic_width_in_mbs as usize + 1)
                    || curr_mb_idx < (vp.pic_width_in_mbs as usize) + 1
                {
                    result_mb = MacroBlock::new();
                } else {
                    let mb_d_idx = curr_mb_idx - (vp.pic_width_in_mbs as usize) - 1;
                    let mb_d_addr = self.macroblock_vec[curr_mb_idx].mb_addr
                        - (vp.pic_width_in_mbs as usize)
                        - 1;
                    if mb_d_addr == self.macroblock_vec[mb_d_idx].mb_addr {
                        result_mb = self.macroblock_vec[mb_d_idx].clone();
                    } else {
                        result_mb = MacroBlock::new();
                    }
                }
            }
        }

        result_mb
    }

    /// Returns neighbor MBAFF macroblocks and availability per Section 6.4.10
    pub fn get_neighbor_mbaff_macroblock(
        &self,
        curr_mb_idx: usize,
        neighbor_type: NeighborMB,
        vp: &VideoParameters,
    ) -> MacroBlock {
        let result_mb: MacroBlock;

        if NEIGHBOR_DEBUG {
            debug!(target: "decode","get_neighbor_mbaff_macroblock - curr_mb_idx {} and mb_addr {} and neighbor_type {:?} ", curr_mb_idx,  self.macroblock_vec[curr_mb_idx].mb_addr, neighbor_type);
        }

        // Follow Figure 6-13 for neighboring macroblocks
        match neighbor_type {
            NeighborMB::MbAddrA => {
                if self.macroblock_vec[curr_mb_idx].mb_addr / 2 % (vp.pic_width_in_mbs as usize)
                    == 0
                    || curr_mb_idx < 2
                {
                    // the last check is to make sure mb_a_addr doesn't equal 0
                    result_mb = MacroBlock::new();
                } else {
                    // check if mb_a is in this slice or not
                    let mb_a_idx = 2 * (curr_mb_idx / 2 - 1);
                    let mb_a_addr = 2 * (self.macroblock_vec[curr_mb_idx].mb_addr / 2 - 1);
                    if mb_a_addr == self.macroblock_vec[mb_a_idx].mb_addr {
                        result_mb = self.macroblock_vec[mb_a_idx].clone();
                    } else {
                        result_mb = MacroBlock::new();
                    }
                }
            }
            NeighborMB::MbAddrB => {
                if self.macroblock_vec[curr_mb_idx].mb_addr / 2 < (vp.pic_width_in_mbs as usize)
                    || curr_mb_idx < 2 * (vp.pic_width_in_mbs as usize)
                {
                    result_mb = MacroBlock::new();
                } else {
                    let mb_b_idx = 2 * (curr_mb_idx / 2 - (vp.pic_width_in_mbs as usize));
                    let mb_b_addr = 2
                        * (self.macroblock_vec[curr_mb_idx].mb_addr / 2
                            - (vp.pic_width_in_mbs as usize));
                    if mb_b_addr == self.macroblock_vec[mb_b_idx].mb_addr {
                        result_mb = self.macroblock_vec[mb_b_idx].clone();
                    } else {
                        result_mb = MacroBlock::new();
                    }
                }
            }
            NeighborMB::MbAddrC => {
                // if on the right-most edge, top-most row, or the index is on the top-most row
                // calculate right-most edge to avoid mod 0
                if self.macroblock_vec[curr_mb_idx].mb_addr / 2 % (vp.pic_width_in_mbs as usize)
                    == (vp.pic_width_in_mbs as usize - 1)
                    || self.macroblock_vec[curr_mb_idx].mb_addr / 2 < (vp.pic_width_in_mbs as usize)
                    || curr_mb_idx / 2 < (vp.pic_width_in_mbs as usize) - 1
                {
                    result_mb = MacroBlock::new();
                } else {
                    let mb_c_idx = 2 * (curr_mb_idx / 2 + 1 - (vp.pic_width_in_mbs as usize));
                    let mb_c_addr = 2
                        * (self.macroblock_vec[curr_mb_idx].mb_addr / 2
                            - (vp.pic_width_in_mbs as usize)
                            + 1);
                    if mb_c_addr == self.macroblock_vec[mb_c_idx].mb_addr {
                        result_mb = self.macroblock_vec[mb_c_idx].clone();
                    } else {
                        result_mb = MacroBlock::new();
                    }
                }
            }
            NeighborMB::MbAddrD => {
                // if on the left-most edge, top-most row, or the index is on the top-most row
                if self.macroblock_vec[curr_mb_idx].mb_addr / 2 % (vp.pic_width_in_mbs as usize)
                    == 0
                    || self.macroblock_vec[curr_mb_idx].mb_addr / 2 < (vp.pic_width_in_mbs as usize)
                    || curr_mb_idx / 2 < (vp.pic_width_in_mbs as usize) + 1
                {
                    result_mb = MacroBlock::new();
                } else {
                    let mb_d_idx = 2 * (curr_mb_idx / 2 - (vp.pic_width_in_mbs as usize) - 1);
                    let mb_d_addr = 2
                        * (self.macroblock_vec[curr_mb_idx].mb_addr / 2
                            - (vp.pic_width_in_mbs as usize)
                            - 1);
                    if mb_d_addr == self.macroblock_vec[mb_d_idx].mb_addr {
                        result_mb = self.macroblock_vec[mb_d_idx].clone();
                    } else {
                        result_mb = MacroBlock::new();
                    }
                }
            }
        }

        if NEIGHBOR_DEBUG {
            debug!(target: "decode","get_neighbor_mbaff_macroblock - neighbor_type {:?} is result_mb.available {} result_mb.mb_idx {} and result_mb.mb_addr {} ", neighbor_type, result_mb.available, result_mb.mb_idx, self.macroblock_vec[result_mb.mb_idx].mb_addr);
        }

        result_mb
    }

    /// Returns neighbor location macroblock and x,y coords per section 6.4.12
    fn get_neighbor_locations(
        &self,
        curr_mb_idx: usize,
        x_n: i32,
        y_n: i32,
        luma: bool,
        flip_curr_mb_frame_flag: bool,
        vp: &VideoParameters,
    ) -> (MacroBlock, usize, usize) {
        let result_mb: MacroBlock;
        let x_w: i32;
        let y_w: i32;

        let max_w: i32;
        let max_h: i32;

        if luma {
            max_w = 16;
            max_h = 16;
        } else {
            max_w = vp.mb_width_c as i32;
            max_h = vp.mb_height_c as i32;
        }

        if NEIGHBOR_DEBUG {
            debug!(target: "decode","get_neighbor_locations - mbtype {:?} and x_n {} and y_n {} and luma {} and max_w {} and max_h {}",
                self.macroblock_vec[curr_mb_idx].mb_type,
                x_n,
                y_n,
                luma,
                max_w,
                max_h);
        }

        if !vp.mbaff_frame_flag {
            // 6.4.12.1

            // Table 6-3
            if x_n < 0 {
                if y_n < 0 {
                    // mbAddrD
                    result_mb = self.get_neighbor_macroblock(curr_mb_idx, NeighborMB::MbAddrD, vp);
                } else if 0 <= y_n && y_n < max_h {
                    // mbAddrA
                    result_mb = self.get_neighbor_macroblock(curr_mb_idx, NeighborMB::MbAddrA, vp);
                } else {
                    // not available
                    result_mb = MacroBlock::new();
                }
            } else if 0 <= x_n && x_n < max_w {
                if y_n < 0 {
                    // mbAddrB
                    result_mb = self.get_neighbor_macroblock(curr_mb_idx, NeighborMB::MbAddrB, vp);
                } else if 0 <= y_n && y_n < max_h {
                    // CurrMbAddr
                    result_mb = self.macroblock_vec[curr_mb_idx].clone();
                } else {
                    // not available
                    result_mb = MacroBlock::new();
                }
            } else {
                // x_n >= max_w-
                if y_n < 0 {
                    // mbAddrC
                    result_mb = self.get_neighbor_macroblock(curr_mb_idx, NeighborMB::MbAddrC, vp);
                } else {
                    // not available
                    result_mb = MacroBlock::new();
                }
            }
            x_w = (x_n + max_w) % max_w;
            y_w = (y_n + max_h) % max_h;
        } else {
            // 6.4.12.2

            let mut curr_mb_frame_flag = true;
            if curr_mb_idx < self.mb_field_decoding_flag.len() {
                curr_mb_frame_flag = !self.mb_field_decoding_flag[curr_mb_idx];
            }

            if flip_curr_mb_frame_flag {
                debug!(target: "encode","flipping curr_mb_frame_flag to match predicted");
                curr_mb_frame_flag = !curr_mb_frame_flag;
            }

            let mb_is_top_mb_flag = self.macroblock_vec[curr_mb_idx].mb_addr % 2 == 0;
            let y_m: i32;
            if NEIGHBOR_DEBUG {
                debug!(target: "decode","get_neighbor_locations (mbaff) - mb_addr {} and curr_mb_frame_flag {}", self.macroblock_vec[curr_mb_idx].mb_addr, curr_mb_frame_flag);
            }
            // Table 6-4
            if x_n < 0 {
                if y_n < 0 {
                    if curr_mb_frame_flag {
                        if mb_is_top_mb_flag {
                            let mb_addr_x = self.get_neighbor_mbaff_macroblock(
                                curr_mb_idx,
                                NeighborMB::MbAddrD,
                                vp,
                            );
                            if mb_addr_x.available {
                                result_mb = self.macroblock_vec[mb_addr_x.mb_idx + 1].clone();
                            } else {
                                result_mb = MacroBlock::new();
                            }
                            y_m = y_n;
                        } else {
                            let mb_addr_x = self.get_neighbor_mbaff_macroblock(
                                curr_mb_idx,
                                NeighborMB::MbAddrA,
                                vp,
                            );
                            if mb_addr_x.available {
                                // if mb_addr_x is a frame macroblock
                                let mb_addr_x_frame_flag =
                                    !self.mb_field_decoding_flag[mb_addr_x.mb_idx];
                                if mb_addr_x_frame_flag {
                                    result_mb = mb_addr_x.clone();
                                    y_m = y_n;
                                } else {
                                    result_mb = self.macroblock_vec[mb_addr_x.mb_idx + 1].clone();
                                    y_m = (y_n + max_h) >> 1;
                                }
                            } else {
                                result_mb = MacroBlock::new();
                                y_m = 0;
                            }
                        }
                    } else {
                        let mb_addr_x = self.get_neighbor_mbaff_macroblock(
                            curr_mb_idx,
                            NeighborMB::MbAddrD,
                            vp,
                        );
                        if mb_addr_x.available {
                            if mb_is_top_mb_flag {
                                let mb_addr_x_frame_flag =
                                    !self.mb_field_decoding_flag[mb_addr_x.mb_idx];
                                if mb_addr_x_frame_flag {
                                    result_mb = self.macroblock_vec[mb_addr_x.mb_idx + 1].clone();
                                    y_m = 2 * y_n;
                                } else {
                                    result_mb = mb_addr_x.clone();
                                    y_m = y_n;
                                }
                            } else {
                                result_mb = self.macroblock_vec[mb_addr_x.mb_idx + 1].clone();
                                y_m = y_n;
                            }
                        } else {
                            result_mb = MacroBlock::new();
                            y_m = 0;
                        }
                    }
                } else if 0 <= y_n && y_n < max_h {
                    let mb_addr_x =
                        self.get_neighbor_mbaff_macroblock(curr_mb_idx, NeighborMB::MbAddrA, vp);

                    let mut mb_addr_x_frame_flag = true;
                    if mb_addr_x.mb_idx < self.mb_field_decoding_flag.len() {
                        mb_addr_x_frame_flag = !self.mb_field_decoding_flag[mb_addr_x.mb_idx];
                    }
                    if mb_addr_x.available {
                        if curr_mb_frame_flag {
                            if mb_is_top_mb_flag {
                                if mb_addr_x_frame_flag {
                                    result_mb = mb_addr_x.clone();
                                    y_m = y_n;
                                } else if y_n % 2 == 0 {
                                    result_mb = mb_addr_x.clone();
                                    y_m = y_n >> 1;
                                } else {
                                    result_mb = self.macroblock_vec[mb_addr_x.mb_idx + 1].clone();
                                    y_m = y_n >> 1;
                                }
                            } else if mb_addr_x_frame_flag {
                                result_mb = self.macroblock_vec[mb_addr_x.mb_idx + 1].clone();
                                y_m = y_n;
                            } else if y_n % 2 == 0 {
                                result_mb = mb_addr_x.clone();
                                y_m = (y_n + max_h) >> 1;
                            } else {
                                result_mb = self.macroblock_vec[mb_addr_x.mb_idx + 1].clone();
                                y_m = (y_n + max_h) >> 1;
                            }
                        } else if mb_is_top_mb_flag {
                            if mb_addr_x_frame_flag {
                                if y_n < max_h / 2 {
                                    result_mb = mb_addr_x.clone();
                                    y_m = y_n << 1;
                                } else {
                                    result_mb = self.macroblock_vec[mb_addr_x.mb_idx + 1].clone();
                                    y_m = (y_n << 1) - max_h;
                                }
                            } else {
                                result_mb = mb_addr_x.clone();
                                y_m = y_n;
                            }
                        } else if mb_addr_x_frame_flag {
                            if y_n < max_h / 2 {
                                result_mb = mb_addr_x.clone();
                                y_m = (y_n << 1) + 1;
                            } else {
                                result_mb = self.macroblock_vec[mb_addr_x.mb_idx + 1].clone();
                                y_m = (y_n << 1) + 1 - max_h;
                            }
                        } else {
                            result_mb = self.macroblock_vec[mb_addr_x.mb_idx + 1].clone();
                            y_m = y_n;
                        }
                    } else {
                        result_mb = MacroBlock::new();
                        y_m = 0;
                    }
                } else {
                    result_mb = MacroBlock::new();
                    y_m = 0;
                }
            } else if 0 <= x_n && x_n < max_w {
                if y_n < 0 {
                    if curr_mb_frame_flag {
                        if mb_is_top_mb_flag {
                            // mb_addr_B
                            let mb_addr_x = self.get_neighbor_mbaff_macroblock(
                                curr_mb_idx,
                                NeighborMB::MbAddrB,
                                vp,
                            );
                            if mb_addr_x.available {
                                result_mb = self.macroblock_vec[mb_addr_x.mb_idx + 1].clone();
                                y_m = y_n;
                            } else {
                                result_mb = MacroBlock::new();
                                y_m = 0;
                            }
                        } else {
                            // currMbAddr-1
                            result_mb = self.get_previous_macroblock(curr_mb_idx);
                            y_m = y_n;
                        }
                    } else if mb_is_top_mb_flag {
                        // mb_addr_B
                        let mb_addr_x = self.get_neighbor_mbaff_macroblock(
                            curr_mb_idx,
                            NeighborMB::MbAddrB,
                            vp,
                        );
                        if mb_addr_x.available {
                            let mb_addr_x_frame_flag =
                                !self.mb_field_decoding_flag[mb_addr_x.mb_idx];
                            if mb_addr_x_frame_flag {
                                result_mb = self.macroblock_vec[mb_addr_x.mb_idx + 1].clone();
                                y_m = 2 * y_n;
                            } else {
                                result_mb = mb_addr_x.clone();
                                y_m = y_n;
                            }
                        } else {
                            result_mb = MacroBlock::new();
                            y_m = 0;
                        }
                    } else {
                        // mb_addr_B
                        let mb_addr_x = self.get_neighbor_mbaff_macroblock(
                            curr_mb_idx,
                            NeighborMB::MbAddrB,
                            vp,
                        );
                        if mb_addr_x.available {
                            result_mb = self.macroblock_vec[mb_addr_x.mb_idx + 1].clone();
                            y_m = y_n;
                        } else {
                            result_mb = MacroBlock::new();
                            y_m = 0;
                        }
                    }
                } else if 0 <= y_n && y_n < max_h {
                    // CurrMbAddr
                    result_mb = self.macroblock_vec[curr_mb_idx].clone();
                    y_m = y_n;
                } else {
                    result_mb = MacroBlock::new();
                    y_m = 0;
                }
            } else if y_n < 0 {
                if curr_mb_frame_flag {
                    if mb_is_top_mb_flag {
                        // mb_addr_c
                        let mb_addr_x = self.get_neighbor_mbaff_macroblock(
                            curr_mb_idx,
                            NeighborMB::MbAddrC,
                            vp,
                        );
                        if mb_addr_x.available {
                            result_mb = self.macroblock_vec[mb_addr_x.mb_idx + 1].clone();
                            y_m = y_n;
                        } else {
                            result_mb = MacroBlock::new();
                            y_m = 0;
                        }
                    } else {
                        result_mb = MacroBlock::new();
                        y_m = 0;
                    }
                } else if mb_is_top_mb_flag {
                    // mb_addr_c
                    let mb_addr_x =
                        self.get_neighbor_mbaff_macroblock(curr_mb_idx, NeighborMB::MbAddrC, vp);
                    if mb_addr_x.available {
                        let mb_addr_x_frame_flag = !self.mb_field_decoding_flag[mb_addr_x.mb_idx];
                        if mb_addr_x_frame_flag {
                            result_mb = self.macroblock_vec[mb_addr_x.mb_idx + 1].clone();
                            y_m = 2 * y_n;
                        } else {
                            result_mb = mb_addr_x.clone();
                            y_m = y_n;
                        }
                    } else {
                        result_mb = MacroBlock::new();
                        y_m = 0;
                    }
                } else {
                    // mb_addr_c
                    let mb_addr_x =
                        self.get_neighbor_mbaff_macroblock(curr_mb_idx, NeighborMB::MbAddrC, vp);
                    if mb_addr_x.available {
                        result_mb = self.macroblock_vec[mb_addr_x.mb_idx + 1].clone();
                        y_m = y_n;
                    } else {
                        result_mb = MacroBlock::new();
                        y_m = 0;
                    }
                }
            } else {
                result_mb = MacroBlock::new();
                y_m = 0;
            }
            x_w = (x_n + max_w) % max_w;
            y_w = (y_m + max_h) % max_h;
        }

        (result_mb, x_w as usize, y_w as usize)
    }

    /// Implements section 6.4.2.1
    fn inverse_macroblock_partition_scanning(
        &self,
        mb_part_idx: usize,
        mb_part_width: usize,
        mb_part_height: usize,
    ) -> (usize, usize) {
        // equation 6-11
        let x = inverse_raster_scan(mb_part_idx, mb_part_width, mb_part_height, 16, 0);
        // equation 6-12
        let y = inverse_raster_scan(mb_part_idx, mb_part_width, mb_part_height, 16, 1);

        (x, y)
    }

    /// Implements section 6.4.2.2
    fn inverse_sub_macroblock_partition_scanning(
        &self,
        mb_type: MbType,
        sub_mb_part_idx: usize,
        sub_mb_part_width: usize,
        sub_mb_part_height: usize,
    ) -> (usize, usize) {
        let x: usize;
        let y: usize;

        if mb_type == MbType::P8x8 || mb_type == MbType::P8x8ref0 || mb_type == MbType::B8x8 {
            // equation 6-13
            x = inverse_raster_scan(sub_mb_part_idx, sub_mb_part_width, sub_mb_part_height, 8, 0);
            // equation 6-14
            y = inverse_raster_scan(sub_mb_part_idx, sub_mb_part_width, sub_mb_part_height, 8, 1);
        } else {
            // equation 6-15
            x = inverse_raster_scan(sub_mb_part_idx, 4, 4, 8, 0);
            // equation 6-16
            y = inverse_raster_scan(sub_mb_part_idx, 4, 4, 8, 1);
        }

        (x, y)
    }

    /// Implements section 6.4.3
    fn inverse_4x4_luma_block_scanning(&self, blk_idx: usize) -> (usize, usize) {
        // equation 6-17
        let x = inverse_raster_scan(blk_idx / 4, 8, 8, 16, 0)
            + inverse_raster_scan(blk_idx % 4, 4, 4, 8, 0);
        // equation 6-18
        let y = inverse_raster_scan(blk_idx / 4, 8, 8, 16, 1)
            + inverse_raster_scan(blk_idx % 4, 4, 4, 8, 1);

        (x, y)
    }

    /// Implements section 6.4.7
    fn inverse_4x4_chroma_block_scanning(&self, blk_idx: usize) -> (usize, usize) {
        // equation 6-21
        let x = inverse_raster_scan(blk_idx, 4, 4, 8, 0);
        // equation 6-22
        let y = inverse_raster_scan(blk_idx, 4, 4, 8, 1);

        (x, y)
    }

    /// Implements section 6.4.13.1
    fn get_4x4_luma_idx(&self, x_p: usize, y_p: usize) -> usize {
        // Equation 6-38
        8 * (y_p / 8) + 4 * (x_p / 8) + 2 * ((y_p % 8) / 4) + ((x_p % 8) / 4)
    }

    /// Implements section 6.4.13.2. Only called with ChromaArrayType is equal to 1 or 2
    fn get_4x4_chroma_idx(&self, x_p: usize, y_p: usize) -> usize {
        // Equation 6-39
        2 * (y_p / 4) + (x_p / 4)
    }

    /// Implements section 6.4.13.3
    fn get_8x8_luma_idx(&self, x_p: usize, y_p: usize) -> usize {
        // Equation 6-40
        2 * (y_p / 8) + (x_p / 8)
    }

    /// Implements section 6.4.13.4
    fn get_partition_indices(&self, curr_mb_idx: usize, x_p: usize, y_p: usize) -> (usize, usize) {
        let curr_mb_type = self.macroblock_vec[curr_mb_idx].mb_type;
        let res = self.macroblock_vec[curr_mb_idx].mb_part_pred_width_and_height();
        let curr_mb_part_pred_width = res.0;
        let curr_mb_part_pred_height = res.1;

        if NEIGHBOR_DEBUG {
            debug!(target: "decode","get_partition_indices - mbtype {:?} and x_p {} and y_p {}", self.macroblock_vec[curr_mb_idx].mb_type, x_p, y_p);
            debug!(target: "decode","get_partition_indices - curr_mb_part_pred_width {} and curr_mb_part_pred_height {}", curr_mb_part_pred_width, curr_mb_part_pred_height);
        }

        let mb_part_idx: usize = if self.macroblock_vec[curr_mb_idx].is_intra_non_mut() {
            0
        } else {
            (16 / curr_mb_part_pred_width) * (y_p / curr_mb_part_pred_height)
                + (x_p / curr_mb_part_pred_width)
        };

        let sub_mb_part_idx: usize = if curr_mb_type != MbType::P8x8
            && curr_mb_type != MbType::P8x8ref0
            && curr_mb_type != MbType::B8x8
            && curr_mb_type != MbType::BDirect16x16
        {
            0
        } else if curr_mb_type == MbType::BSkip || curr_mb_type == MbType::BDirect16x16 {
            2 * ((y_p % 8) / 4) + ((x_p % 8) / 4)
        } else {
            let res =
                self.macroblock_vec[curr_mb_idx].sub_mb_part_pred_width_and_height(mb_part_idx);
            let curr_sub_mb_part_pred_width = res.0;
            let curr_sub_mb_part_pred_height = res.1;

            (8 / curr_sub_mb_part_pred_width) * ((y_p % 8) / curr_sub_mb_part_pred_height)
                + ((x_p % 8) / curr_sub_mb_part_pred_width)
        };

        (mb_part_idx, sub_mb_part_idx)
    }

    /// Implements section 6.4.11.1.
    /// NOTE: flip_curr_mb_frame_flag is for when using the predicted mb_field_decoding_flag instead of
    ///       the actually encoded value. This is required when decoding mb_skip_flag in an MBAFF slice.
    pub fn get_neighbor(
        &self,
        curr_mb_idx: usize,
        flip_curr_mb_frame_flag: bool,
        vp: &VideoParameters,
    ) -> (MacroBlock, MacroBlock) {
        // mb_a with Table 6-2 values (-1, 0)
        let res =
            self.get_neighbor_locations(curr_mb_idx, -1, 0, true, flip_curr_mb_frame_flag, vp);
        let mb_a: MacroBlock = res.0;

        // mb_b with Table 6-2 values (0, -1)
        let res =
            self.get_neighbor_locations(curr_mb_idx, 0, -1, true, flip_curr_mb_frame_flag, vp);
        let mb_b: MacroBlock = res.0;

        if NEIGHBOR_DEBUG {
            debug!(target: "decode","get_neighbor - mb_a.mb_addr {} and mb_a.mb_type {:?} and mb_b.mb_addr {} and mb_b.mb_type {:?}", mb_a.mb_addr, mb_a.mb_type, mb_b.mb_addr, mb_b.mb_type);
        }

        (mb_a, mb_b)
    }

    /// Implements section 6.4.11.2
    pub fn get_neighbor_8x8_luma_block(
        &self,
        curr_mb_idx: usize,
        luma: bool,
        luma_8x8_blk_idx: usize,
        vp: &VideoParameters,
    ) -> (MacroBlock, MacroBlock, usize, usize) {
        // mb_a with Table 6-2 values (-1, 0)
        let x_d = -1;
        let y_d = 0;

        let x_n = (luma_8x8_blk_idx as i32 % 2) * 8 + x_d;
        let y_n = (luma_8x8_blk_idx as i32 / 2) * 8 + y_d;

        let res = self.get_neighbor_locations(curr_mb_idx, x_n, y_n, luma, false, vp);
        let mb_a: MacroBlock = res.0;
        let x_w = res.1;
        let y_w = res.2;

        let luma_8x8_blk_idx_a: usize = if !mb_a.available {
            0
        } else {
            self.get_8x8_luma_idx(x_w, y_w)
        };

        // mb_b with Table 6-2 values (0, -1)
        let x_d = 0;
        let y_d = -1;

        let x_n = (luma_8x8_blk_idx as i32 % 2) * 8 + x_d;
        let y_n = (luma_8x8_blk_idx as i32 / 2) * 8 + y_d;

        let res = self.get_neighbor_locations(curr_mb_idx, x_n, y_n, luma, false, vp);
        let mb_b: MacroBlock = res.0;
        let x_w = res.1;
        let y_w = res.2;

        let luma_8x8_blk_idx_b: usize = if !mb_b.available {
            0
        } else {
            self.get_8x8_luma_idx(x_w, y_w)
        };

        (mb_a, mb_b, luma_8x8_blk_idx_a, luma_8x8_blk_idx_b)
    }

    /// Implements section 6.4.11.3
    pub fn get_neighbor_8x8_cr_cb_block(
        &self,
        curr_mb_idx: usize,
        chroma_8x8_blk_idx: usize,
        vp: &VideoParameters,
    ) -> (MacroBlock, MacroBlock, usize, usize) {
        // according to 6.4.11.3 it is the same as above except substituting luma with chroma
        self.get_neighbor_8x8_luma_block(curr_mb_idx, false, chroma_8x8_blk_idx, vp)
    }

    /// Implements section 6.4.11.4
    pub fn get_neighbor_4x4_luma_block(
        &self,
        curr_mb_idx: usize,
        luma: bool,
        luma_4x4_blk_idx: usize,
        vp: &VideoParameters,
    ) -> (MacroBlock, MacroBlock, usize, usize) {
        let res = self.inverse_4x4_luma_block_scanning(luma_4x4_blk_idx);
        let x = res.0 as i32;
        let y = res.1 as i32;

        // mb_a with Table 6-2 values (-1, 0)
        let x_d = -1;
        let y_d = 0;

        let x_n = x + x_d;
        let y_n = y + y_d;

        let res = self.get_neighbor_locations(curr_mb_idx, x_n, y_n, luma, false, vp);
        let mb_a: MacroBlock = res.0;
        let x_w = res.1;
        let y_w = res.2;

        let luma_4x4_blk_idx_a: usize = if !mb_a.available {
            0
        } else {
            self.get_4x4_luma_idx(x_w, y_w)
        };

        // mb_b with Table 6-2 values (0, -1)
        let x_d = 0;
        let y_d = -1;

        let x_n = x + x_d;
        let y_n = y + y_d;

        let res = self.get_neighbor_locations(curr_mb_idx, x_n, y_n, luma, false, vp);
        let mb_b: MacroBlock = res.0;
        let x_w = res.1;
        let y_w = res.2;

        let luma_4x4_blk_idx_b: usize = if !mb_b.available {
            0
        } else {
            self.get_4x4_luma_idx(x_w, y_w)
        };

        (mb_a, mb_b, luma_4x4_blk_idx_a, luma_4x4_blk_idx_b)
    }

    /// Implements section 6.4.11.5. Only called with Chroma Array Type is 1 or 2
    pub fn get_neighbor_4x4_chroma_block(
        &self,
        curr_mb_idx: usize,
        chroma_4x4_blk_idx: usize,
        vp: &VideoParameters,
    ) -> (MacroBlock, MacroBlock, usize, usize) {
        let res = self.inverse_4x4_chroma_block_scanning(chroma_4x4_blk_idx);
        let x = res.0 as i32;
        let y = res.1 as i32;

        // mb_a with Table 6-2 values (-1, 0)
        let x_d = -1;
        let y_d = 0;

        let x_n = x + x_d;
        let y_n = y + y_d;

        // invoke 6.4.12
        let res = self.get_neighbor_locations(curr_mb_idx, x_n, y_n, false, false, vp);
        let mb_a: MacroBlock = res.0;
        let x_w = res.1;
        let y_w = res.2;

        let chroma_4x4_blk_idx_a: usize = if !mb_a.available {
            0
        } else {
            self.get_4x4_chroma_idx(x_w, y_w)
        };

        // mb_b with Table 6-2 values (0, -1)
        let x_d = 0;
        let y_d = -1;

        let x_n = x + x_d;
        let y_n = y + y_d;

        // invoke 6.4.12
        let res = self.get_neighbor_locations(curr_mb_idx, x_n, y_n, false, false, vp);
        let mb_b: MacroBlock = res.0;
        let x_w = res.1;
        let y_w = res.2;

        let chroma_4x4_blk_idx_b: usize = if !mb_b.available {
            0
        } else {
            self.get_4x4_chroma_idx(x_w, y_w)
        };

        (mb_a, mb_b, chroma_4x4_blk_idx_a, chroma_4x4_blk_idx_b)
    }

    /// Implements section 6.4.11.6
    pub fn get_neighbor_4x4_cr_cb_blocks_info(
        &self,
        curr_mb_idx: usize,
        chroma_4x4_blk_idx: usize,
        vp: &VideoParameters,
    ) -> (MacroBlock, MacroBlock, usize, usize) {
        // when Chroma Array Type is 3, it's the same as 6.4.11.4 but for Chroma values
        self.get_neighbor_4x4_luma_block(curr_mb_idx, false, chroma_4x4_blk_idx, vp)
    }

    /// Implements section 6.4.11.7
    pub fn get_neighbor_partitions(
        &self,
        curr_mb_idx: usize,
        mb_part_idx: usize,
        sub_mb_part_idx: usize,
        vp: &VideoParameters,
        _decoder_output: bool,
    ) -> (
        // Macroblocks
        MacroBlock,
        MacroBlock,
        MacroBlock,
        MacroBlock,
        // mb_part_idx_x
        usize,
        usize,
        usize,
        usize,
        // sub_mb_part_idx_x
        usize,
        usize,
        usize,
        usize,
    ) {
        let mb_part_idx_a;
        let mb_part_idx_b;
        let mb_part_idx_c;
        let mb_part_idx_d;

        let sub_mb_part_idx_a;
        let sub_mb_part_idx_b;
        let sub_mb_part_idx_c;
        let sub_mb_part_idx_d;

        let mb_part_pred_width_and_height =
            self.macroblock_vec[curr_mb_idx].mb_part_pred_width_and_height();
        let mb_part_width = mb_part_pred_width_and_height.0;
        let mb_part_height = mb_part_pred_width_and_height.1;

        let sub_mb_part_pred_width_and_height =
            self.macroblock_vec[curr_mb_idx].sub_mb_part_pred_width_and_height(mb_part_idx);
        let sub_mb_part_pred_width = sub_mb_part_pred_width_and_height.0;
        let sub_mb_part_pred_height = sub_mb_part_pred_width_and_height.1;

        let res =
            self.inverse_macroblock_partition_scanning(mb_part_idx, mb_part_width, mb_part_height);
        let x = res.0 as i32;
        let y = res.1 as i32;
        let curr_mb_type = self.macroblock_vec[curr_mb_idx].mb_type;
        let curr_sub_mb_type = self.macroblock_vec[curr_mb_idx].sub_mb_type[sub_mb_part_idx];

        //if decoder_output {
        //    debug!(target: "decode","get_neighbor_partitions - mbtype {:?} and mb_part_idx {} and sub_mb_part_idx {} and x_d {} and y_d {}", self.macroblock_vec[curr_mb_idx].mb_type, mb_part_idx, sub_mb_part_idx, x, y);
        //} else {
        //    debug!(target: "encode","get_neighbor_partitions - mbtype {:?} and mb_part_idx {} and sub_mb_part_idx {} and x_d {} and y_d {}", self.macroblock_vec[curr_mb_idx].mb_type, mb_part_idx, sub_mb_part_idx, x, y);
        //}

        let x_s: i32;
        let y_s: i32;

        if curr_mb_type == MbType::P8x8
            || curr_mb_type == MbType::P8x8ref0
            || curr_mb_type == MbType::B8x8
        {
            // call 6.4.2.2 to get x_s and y_s

            let res = self.inverse_sub_macroblock_partition_scanning(
                curr_mb_type,
                sub_mb_part_idx,
                sub_mb_part_pred_width,
                sub_mb_part_pred_height,
            );
            x_s = res.0 as i32;
            y_s = res.1 as i32;
        } else {
            x_s = 0;
            y_s = 0;
        }

        let pred_part_width: i32;

        if curr_mb_type == MbType::PSkip
            || curr_mb_type == MbType::BSkip
            || curr_mb_type == MbType::BDirect16x16
        {
            pred_part_width = 16
        } else if curr_mb_type == MbType::B8x8 {
            if curr_sub_mb_type == SubMbType::BDirect8x8 {
                pred_part_width = 16;
            } else {
                pred_part_width = sub_mb_part_pred_width as i32;
            }
        } else if curr_mb_type == MbType::P8x8 || curr_mb_type == MbType::P8x8ref0 {
            pred_part_width = sub_mb_part_pred_width as i32;
        } else {
            pred_part_width = mb_part_width as i32;
        }

        // mb_a with Table 6-2 val of (-1, 0)
        let x_d = -1;
        let y_d = 0;

        let x_n = x + x_s + x_d;
        let y_n = y + y_s + y_d;

        // invoke 6.4.12
        let res = self.get_neighbor_locations(curr_mb_idx, x_n, y_n, true, false, vp);
        let mb_a: MacroBlock = res.0;
        let x_w = res.1;
        let y_w = res.2;

        if !mb_a.available {
            mb_part_idx_a = 0;
            sub_mb_part_idx_a = 0;
        } else {
            // follow process 6.4.13.4
            let res = self.get_partition_indices(mb_a.mb_idx, x_w, y_w);
            mb_part_idx_a = res.0;
            sub_mb_part_idx_a = res.1;
        }

        // mb_b with Table 6-2 val of (0, -1)
        let x_d = 0;
        let y_d = -1;

        let x_n = x + x_s + x_d;
        let y_n = y + y_s + y_d;

        // invoke 6.4.12
        let res = self.get_neighbor_locations(curr_mb_idx, x_n, y_n, true, false, vp);
        let mb_b: MacroBlock = res.0;
        let x_w = res.1;
        let y_w = res.2;

        if !mb_b.available {
            mb_part_idx_b = 0;
            sub_mb_part_idx_b = 0;
        } else {
            // follow process 6.4.13.4
            let res = self.get_partition_indices(mb_b.mb_idx, x_w, y_w);
            mb_part_idx_b = res.0;
            sub_mb_part_idx_b = res.1;
        }

        // mb_c with Table 6-2 val of (pred_part_width, -1)
        let x_d = pred_part_width;
        let y_d = -1;

        let x_n = x + x_s + x_d;
        let y_n = y + y_s + y_d;

        // invoke 6.4.12
        let res = self.get_neighbor_locations(curr_mb_idx, x_n, y_n, true, false, vp);
        let mb_c: MacroBlock = res.0;
        let x_w = res.1;
        let y_w = res.2;

        if !mb_c.available {
            mb_part_idx_c = 0;
            sub_mb_part_idx_c = 0;
        } else {
            // follow process 6.4.13.4
            let res = self.get_partition_indices(mb_c.mb_idx, x_w, y_w);
            mb_part_idx_c = res.0;
            sub_mb_part_idx_c = res.1;
        }

        // mb_d with Table 6-2 val of (-1, -1)
        let x_d = -1;
        let y_d = -1;

        let x_n = x + x_s + x_d;
        let y_n = y + y_s + y_d;

        // invoke 6.4.12
        let res = self.get_neighbor_locations(curr_mb_idx, x_n, y_n, true, false, vp);
        let mb_d: MacroBlock = res.0;
        let x_w = res.1;
        let y_w = res.2;

        if !mb_d.available {
            mb_part_idx_d = 0;
            sub_mb_part_idx_d = 0;
        } else {
            // follow process 6.4.13.4
            let res = self.get_partition_indices(mb_d.mb_idx, x_w, y_w);
            mb_part_idx_d = res.0;
            sub_mb_part_idx_d = res.1;
        }

        (
            mb_a,
            mb_b,
            mb_c,
            mb_d,
            mb_part_idx_a,
            mb_part_idx_b,
            mb_part_idx_c,
            mb_part_idx_d,
            sub_mb_part_idx_a,
            sub_mb_part_idx_b,
            sub_mb_part_idx_c,
            sub_mb_part_idx_d,
        )
    }
}

impl Default for SliceData {
    fn default() -> Self {
        Self::new()
    }
}

/// NALU Type 1 and 5 -- Contains slice header and slice data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Slice {
    pub sh: SliceHeader,
    pub sd: SliceData,
}

impl Slice {
    pub fn new() -> Slice {
        Slice {
            sh: SliceHeader::new(),
            sd: SliceData::new(),
        }
    }
}

impl Default for Slice {
    fn default() -> Self {
        Self::new()
    }
}

/// NALU Type 8 -- Picture Parameter Set
#[derive(Serialize, Deserialize, Clone)]
pub struct PicParameterSet {
    pub available: bool, // used to determine if the PicParameterSet has been set or not
    pub pic_parameter_set_id: u32, //ue(v)
    pub seq_parameter_set_id: u32, // ue(v)
    pub entropy_coding_mode_flag: bool,
    pub bottom_field_pic_order_in_frame_present_flag: bool,
    pub num_slice_groups_minus1: u32, //ue(v)
    pub slice_group_map_type: u32,    //ue(v)
    pub run_length_minus1: Vec<u32>,  //ue(v)[num_slice_groups_minus1]
    pub top_left: Vec<u32>,           // ue(v)[num_slice_groups_minus1]
    pub bottom_right: Vec<u32>,       //ue(v)[num_slice_groups_minus1]
    pub slice_group_change_direction_flag: bool,
    pub slice_group_change_rate_minus1: u32, //ue(v)
    // } else if slice_group_map_type == 6 {
    pub pic_size_in_map_units_minus1: u32, // ue(v)
    pub slice_group_id: Vec<u32>,          // u(v)[pic_size_in_map_units_minus1]
    // }
    // }
    pub num_ref_idx_l0_default_active_minus1: u32, // ue(v)
    pub num_ref_idx_l1_default_active_minus1: u32, // ue(v)
    pub weighted_pred_flag: bool,
    pub weighted_bipred_idc: u8,     // u(2)
    pub pic_init_qp_minus26: i32,    // se(v)
    pub pic_init_qs_minus26: i32,    // se(v)
    pub chroma_qp_index_offset: i32, // se(v)
    pub deblocking_filter_control_present_flag: bool,
    pub constrained_intra_pred_flag: bool,
    pub redundant_pic_cnt_present_flag: bool,
    // PPS Extensions
    pub more_data_flag: bool, // not parsed, but exists if more data is present

    pub transform_8x8_mode_flag: bool,

    pub pic_scaling_matrix_present_flag: bool,
    pub pic_scaling_list_present_flag: Vec<bool>,
    pub delta_scale_4x4: Vec<Vec<i32>>,
    pub scaling_list_4x4: Vec<Vec<i32>>, // list (len 6) of list (len 16) of se(v)
    pub delta_scale_8x8: Vec<Vec<i32>>,
    pub scaling_list_8x8: Vec<Vec<i32>>, // list (len 2 or 6) of list (len 64) of se(v)
    pub use_default_scaling_matrix_4x4: Vec<bool>, // list len 6
    pub use_default_scaling_matrix_8x8: Vec<bool>, // list len 2 or 6

    pub second_chroma_qp_index_offset: i32,
}

impl PicParameterSet {
    pub fn new() -> PicParameterSet {
        PicParameterSet {
            available: false,
            pic_parameter_set_id: 0,
            seq_parameter_set_id: 0,
            entropy_coding_mode_flag: false,
            bottom_field_pic_order_in_frame_present_flag: false,
            num_slice_groups_minus1: 0,
            slice_group_map_type: 0,
            run_length_minus1: Vec::new(),
            top_left: Vec::new(),
            bottom_right: Vec::new(),
            slice_group_change_direction_flag: false,
            slice_group_change_rate_minus1: 0,
            pic_size_in_map_units_minus1: 0,
            slice_group_id: Vec::new(),
            num_ref_idx_l0_default_active_minus1: 0,
            num_ref_idx_l1_default_active_minus1: 0,
            weighted_pred_flag: false,
            weighted_bipred_idc: 0,
            pic_init_qp_minus26: 0,
            pic_init_qs_minus26: 0,
            chroma_qp_index_offset: 0,
            deblocking_filter_control_present_flag: false,
            constrained_intra_pred_flag: false,
            redundant_pic_cnt_present_flag: false,

            more_data_flag: false,

            transform_8x8_mode_flag: false,
            pic_scaling_matrix_present_flag: false,
            pic_scaling_list_present_flag: Vec::new(),
            delta_scale_4x4: Vec::new(),
            scaling_list_4x4: Vec::new(),
            delta_scale_8x8: Vec::new(),
            scaling_list_8x8: Vec::new(),
            use_default_scaling_matrix_4x4: Vec::new(),
            use_default_scaling_matrix_8x8: Vec::new(),

            second_chroma_qp_index_offset: 0,
        }
    }

    pub fn clone(&self) -> PicParameterSet {
        PicParameterSet {
            available: self.available,
            pic_parameter_set_id: self.pic_parameter_set_id,
            seq_parameter_set_id: self.seq_parameter_set_id,
            entropy_coding_mode_flag: self.entropy_coding_mode_flag,
            bottom_field_pic_order_in_frame_present_flag: self
                .bottom_field_pic_order_in_frame_present_flag,
            num_slice_groups_minus1: self.num_slice_groups_minus1,
            slice_group_map_type: self.slice_group_map_type,
            run_length_minus1: self.run_length_minus1.clone(),
            top_left: self.top_left.clone(),
            bottom_right: self.bottom_right.clone(),
            slice_group_change_direction_flag: self.slice_group_change_direction_flag,
            slice_group_change_rate_minus1: self.slice_group_change_rate_minus1,
            pic_size_in_map_units_minus1: self.pic_size_in_map_units_minus1,
            slice_group_id: self.slice_group_id.clone(),
            num_ref_idx_l0_default_active_minus1: self.num_ref_idx_l0_default_active_minus1,
            num_ref_idx_l1_default_active_minus1: self.num_ref_idx_l1_default_active_minus1,
            weighted_pred_flag: self.weighted_pred_flag,
            weighted_bipred_idc: self.weighted_bipred_idc,
            pic_init_qp_minus26: self.pic_init_qp_minus26,
            pic_init_qs_minus26: self.pic_init_qs_minus26,
            chroma_qp_index_offset: self.chroma_qp_index_offset,
            deblocking_filter_control_present_flag: self.deblocking_filter_control_present_flag,
            constrained_intra_pred_flag: self.constrained_intra_pred_flag,
            redundant_pic_cnt_present_flag: self.redundant_pic_cnt_present_flag,

            more_data_flag: self.more_data_flag,

            transform_8x8_mode_flag: self.transform_8x8_mode_flag,

            pic_scaling_matrix_present_flag: self.pic_scaling_matrix_present_flag,
            pic_scaling_list_present_flag: self.pic_scaling_list_present_flag.clone(),
            delta_scale_4x4: self.delta_scale_4x4.clone(),
            scaling_list_4x4: self.scaling_list_4x4.clone(),
            delta_scale_8x8: self.delta_scale_8x8.clone(),
            scaling_list_8x8: self.scaling_list_8x8.clone(),
            use_default_scaling_matrix_4x4: self.use_default_scaling_matrix_4x4.clone(),
            use_default_scaling_matrix_8x8: self.use_default_scaling_matrix_8x8.clone(),

            second_chroma_qp_index_offset: self.second_chroma_qp_index_offset,
        }
    }

    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print("PPS: pic_parameter_set_id", self.pic_parameter_set_id, 63);
        encoder_formatted_print("PPS: seq_parameter_set_id", self.seq_parameter_set_id, 63);
        encoder_formatted_print(
            "PPS: entropy_coding_mode_flag",
            self.entropy_coding_mode_flag,
            63,
        );
        encoder_formatted_print(
            "PPS: bottom_field_pic_order_in_frame_present_flag",
            self.bottom_field_pic_order_in_frame_present_flag,
            63,
        );
        encoder_formatted_print(
            "PPS: num_slice_groups_minus1",
            self.num_slice_groups_minus1,
            63,
        );
        if self.num_slice_groups_minus1 > 0 {
            encoder_formatted_print("PPS: slice_group_map_type", self.slice_group_map_type, 63);
            match self.slice_group_map_type {
                0 => encoder_formatted_print(
                    "PPS: run_length_minus1",
                    self.run_length_minus1.clone(),
                    63,
                ),
                2 => {
                    encoder_formatted_print("PPS: top_left", self.top_left.clone(), 63);
                    encoder_formatted_print("PPS: bottom_right", self.bottom_right.clone(), 63);
                }
                3 | 4 | 5 => {
                    encoder_formatted_print(
                        "PPS: slice_group_change_direction_flag",
                        self.slice_group_change_direction_flag,
                        63,
                    );
                    encoder_formatted_print(
                        "PPS: slice_group_change_rate_minus1",
                        self.slice_group_change_rate_minus1,
                        63,
                    );
                }
                6 => {
                    encoder_formatted_print(
                        "PPS: pic_size_in_map_units_minus1",
                        self.pic_size_in_map_units_minus1,
                        63,
                    );
                    encoder_formatted_print("PPS: slice_group_id", self.slice_group_id.clone(), 63);
                }
                _ => (),
            }
        }
        encoder_formatted_print(
            "PPS: num_ref_idx_l0_default_active_minus1",
            self.num_ref_idx_l0_default_active_minus1,
            63,
        );
        encoder_formatted_print(
            "PPS: num_ref_idx_l1_default_active_minus1",
            self.num_ref_idx_l1_default_active_minus1,
            63,
        );
        encoder_formatted_print("PPS: weighted_pred_flag", self.weighted_pred_flag, 63);
        encoder_formatted_print("PPS: weighted_bipred_idc", self.weighted_bipred_idc, 63);
        encoder_formatted_print("PPS: pic_init_qp_minus26", self.pic_init_qp_minus26, 63);
        encoder_formatted_print("PPS: pic_init_qs_minus26", self.pic_init_qs_minus26, 63);
        encoder_formatted_print(
            "PPS: chroma_qp_index_offset",
            self.chroma_qp_index_offset,
            63,
        );
        encoder_formatted_print(
            "PPS: deblocking_filter_control_present_flag",
            self.deblocking_filter_control_present_flag,
            63,
        );
        encoder_formatted_print(
            "PPS: constrained_intra_pred_flag",
            self.constrained_intra_pred_flag,
            63,
        );
        encoder_formatted_print(
            "PPS: redundant_pic_cnt_present_flag",
            self.redundant_pic_cnt_present_flag,
            63,
        );
        if self.more_data_flag {
            encoder_formatted_print(
                "PPS: transform_8x8_mode_flag",
                self.transform_8x8_mode_flag,
                63,
            );
            encoder_formatted_print(
                "PPS: pic_scaling_matrix_present_flag",
                self.pic_scaling_matrix_present_flag,
                63,
            );
            if self.pic_scaling_matrix_present_flag {
                encoder_formatted_print(
                    "PPS: pic_scaling_list_present_flag",
                    self.pic_scaling_list_present_flag.clone(),
                    63,
                );
                encoder_formatted_print("PPS: delta_scale_4x4", self.delta_scale_4x4.clone(), 63);
                encoder_formatted_print("PPS: scaling_list_4x4", self.scaling_list_4x4.clone(), 63);
                encoder_formatted_print("PPS: delta_scale_8x8", self.delta_scale_8x8.clone(), 63);
                encoder_formatted_print("PPS: scaling_list_8x8", self.scaling_list_8x8.clone(), 63);
                encoder_formatted_print(
                    "PPS: use_default_scaling_matrix_4x4",
                    self.use_default_scaling_matrix_4x4.clone(),
                    63,
                );
                encoder_formatted_print(
                    "PPS: use_default_scaling_matrix_8x8",
                    self.use_default_scaling_matrix_8x8.clone(),
                    63,
                );
            }
            encoder_formatted_print(
                "PPS: second_chroma_qp_index_offset",
                self.second_chroma_qp_index_offset,
                63,
            );
        }
    }

    #[allow(dead_code)]
    pub fn decoder_pretty_print(&self) {
        decoder_formatted_print("PPS: pic_parameter_set_id", self.pic_parameter_set_id, 63);
        decoder_formatted_print("PPS: seq_parameter_set_id", self.seq_parameter_set_id, 63);
        decoder_formatted_print(
            "PPS: entropy_coding_mode_flag",
            self.entropy_coding_mode_flag,
            63,
        );
        decoder_formatted_print(
            "PPS: bottom_field_pic_order_in_frame_present_flag",
            self.bottom_field_pic_order_in_frame_present_flag,
            63,
        );
        decoder_formatted_print(
            "PPS: num_slice_groups_minus1",
            self.num_slice_groups_minus1,
            63,
        );
        if self.num_slice_groups_minus1 > 0 {
            decoder_formatted_print("PPS: slice_group_map_type", self.slice_group_map_type, 63);
            match self.slice_group_map_type {
                0 => decoder_formatted_print(
                    "PPS: run_length_minus1",
                    self.run_length_minus1.clone(),
                    63,
                ),
                2 => {
                    decoder_formatted_print("PPS: top_left", self.top_left.clone(), 63);
                    decoder_formatted_print("PPS: bottom_right", self.bottom_right.clone(), 63);
                }
                3 | 4 | 5 => {
                    decoder_formatted_print(
                        "PPS: slice_group_change_direction_flag",
                        self.slice_group_change_direction_flag,
                        63,
                    );
                    decoder_formatted_print(
                        "PPS: slice_group_change_rate_minus1",
                        self.slice_group_change_rate_minus1,
                        63,
                    );
                }
                6 => {
                    decoder_formatted_print(
                        "PPS: pic_size_in_map_units_minus1",
                        self.pic_size_in_map_units_minus1,
                        63,
                    );
                    decoder_formatted_print("PPS: slice_group_id", self.slice_group_id.clone(), 63);
                }
                _ => (),
            }
        }
        decoder_formatted_print(
            "PPS: num_ref_idx_l0_default_active_minus1",
            self.num_ref_idx_l0_default_active_minus1,
            63,
        );
        decoder_formatted_print(
            "PPS: num_ref_idx_l1_default_active_minus1",
            self.num_ref_idx_l1_default_active_minus1,
            63,
        );
        decoder_formatted_print("PPS: weighted_pred_flag", self.weighted_pred_flag, 63);
        decoder_formatted_print("PPS: weighted_bipred_idc", self.weighted_bipred_idc, 63);
        decoder_formatted_print("PPS: pic_init_qp_minus26", self.pic_init_qp_minus26, 63);
        decoder_formatted_print("PPS: pic_init_qs_minus26", self.pic_init_qs_minus26, 63);
        decoder_formatted_print(
            "PPS: chroma_qp_index_offset",
            self.chroma_qp_index_offset,
            63,
        );
        decoder_formatted_print(
            "PPS: deblocking_filter_control_present_flag",
            self.deblocking_filter_control_present_flag,
            63,
        );
        decoder_formatted_print(
            "PPS: constrained_intra_pred_flag",
            self.constrained_intra_pred_flag,
            63,
        );
        decoder_formatted_print(
            "PPS: redundant_pic_cnt_present_flag",
            self.redundant_pic_cnt_present_flag,
            63,
        );
        if self.more_data_flag {
            decoder_formatted_print(
                "PPS: transform_8x8_mode_flag",
                self.transform_8x8_mode_flag,
                63,
            );
            decoder_formatted_print(
                "PPS: pic_scaling_matrix_present_flag",
                self.pic_scaling_matrix_present_flag,
                63,
            );
            if self.pic_scaling_matrix_present_flag {
                decoder_formatted_print(
                    "PPS: pic_scaling_list_present_flag",
                    self.pic_scaling_list_present_flag.clone(),
                    63,
                );
                decoder_formatted_print("PPS: delta_scale_4x4", self.delta_scale_4x4.clone(), 63);
                decoder_formatted_print("PPS: scaling_list_4x4", self.scaling_list_4x4.clone(), 63);
                decoder_formatted_print("PPS: delta_scale_8x8", self.delta_scale_8x8.clone(), 63);
                decoder_formatted_print("PPS: scaling_list_8x8", self.scaling_list_8x8.clone(), 63);
                decoder_formatted_print(
                    "PPS: use_default_scaling_matrix_4x4",
                    self.use_default_scaling_matrix_4x4.clone(),
                    63,
                );
                decoder_formatted_print(
                    "PPS: use_default_scaling_matrix_8x8",
                    self.use_default_scaling_matrix_8x8.clone(),
                    63,
                );
            }
            decoder_formatted_print(
                "PPS: second_chroma_qp_index_offset",
                self.second_chroma_qp_index_offset,
                63,
            );
        }
    }

    #[allow(dead_code)]
    pub fn pretty_print(&self) {
        formatted_print("PPS: pic_parameter_set_id", self.pic_parameter_set_id, 63);
        formatted_print("PPS: seq_parameter_set_id", self.seq_parameter_set_id, 63);
        formatted_print(
            "PPS: entropy_coding_mode_flag",
            self.entropy_coding_mode_flag,
            63,
        );
        formatted_print(
            "PPS: bottom_field_pic_order_in_frame_present_flag",
            self.bottom_field_pic_order_in_frame_present_flag,
            63,
        );
        formatted_print(
            "PPS: num_slice_groups_minus1",
            self.num_slice_groups_minus1,
            63,
        );
        if self.num_slice_groups_minus1 > 0 {
            formatted_print("PPS: slice_group_map_type", self.slice_group_map_type, 63);
            match self.slice_group_map_type {
                0 => formatted_print("PPS: run_length_minus1", self.run_length_minus1.clone(), 63),
                2 => {
                    formatted_print("PPS: top_left", self.top_left.clone(), 63);
                    formatted_print("PPS: bottom_right", self.bottom_right.clone(), 63);
                }
                3 | 4 | 5 => {
                    formatted_print(
                        "PPS: slice_group_change_direction_flag",
                        self.slice_group_change_direction_flag,
                        63,
                    );
                    formatted_print(
                        "PPS: slice_group_change_rate_minus1",
                        self.slice_group_change_rate_minus1,
                        63,
                    );
                }
                6 => {
                    formatted_print(
                        "PPS: pic_size_in_map_units_minus1",
                        self.pic_size_in_map_units_minus1,
                        63,
                    );
                    formatted_print("PPS: slice_group_id", self.slice_group_id.clone(), 63);
                }
                _ => (),
            }
        }
        formatted_print(
            "PPS: num_ref_idx_l0_default_active_minus1",
            self.num_ref_idx_l0_default_active_minus1,
            63,
        );
        formatted_print(
            "PPS: num_ref_idx_l1_default_active_minus1",
            self.num_ref_idx_l1_default_active_minus1,
            63,
        );
        formatted_print("PPS: weighted_pred_flag", self.weighted_pred_flag, 63);
        formatted_print("PPS: weighted_bipred_idc", self.weighted_bipred_idc, 63);
        formatted_print("PPS: pic_init_qp_minus26", self.pic_init_qp_minus26, 63);
        formatted_print("PPS: pic_init_qs_minus26", self.pic_init_qs_minus26, 63);
        formatted_print(
            "PPS: chroma_qp_index_offset",
            self.chroma_qp_index_offset,
            63,
        );
        formatted_print(
            "PPS: deblocking_filter_control_present_flag",
            self.deblocking_filter_control_present_flag,
            63,
        );
        formatted_print(
            "PPS: constrained_intra_pred_flag",
            self.constrained_intra_pred_flag,
            63,
        );
        formatted_print(
            "PPS: redundant_pic_cnt_present_flag",
            self.redundant_pic_cnt_present_flag,
            63,
        );
        if self.more_data_flag {
            formatted_print(
                "PPS: transform_8x8_mode_flag",
                self.transform_8x8_mode_flag,
                63,
            );
            formatted_print(
                "PPS: pic_scaling_matrix_present_flag",
                self.pic_scaling_matrix_present_flag,
                63,
            );
            if self.pic_scaling_matrix_present_flag {
                formatted_print(
                    "PPS: pic_scaling_list_present_flag",
                    self.pic_scaling_list_present_flag.clone(),
                    63,
                );
                formatted_print("PPS: delta_scale_4x4", self.delta_scale_4x4.clone(), 63);
                formatted_print("PPS: scaling_list_4x4", self.scaling_list_4x4.clone(), 63);
                formatted_print("PPS: delta_scale_8x8", self.delta_scale_8x8.clone(), 63);
                formatted_print("PPS: scaling_list_8x8", self.scaling_list_8x8.clone(), 63);
                formatted_print(
                    "PPS: use_default_scaling_matrix_4x4",
                    self.use_default_scaling_matrix_4x4.clone(),
                    63,
                );
                formatted_print(
                    "PPS: use_default_scaling_matrix_8x8",
                    self.use_default_scaling_matrix_8x8.clone(),
                    63,
                );
            }
            formatted_print(
                "PPS: second_chroma_qp_index_offset",
                self.second_chroma_qp_index_offset,
                63,
            );
        }
    }
}

impl Default for PicParameterSet {
    fn default() -> Self {
        Self::new()
    }
}

/// HRD Parameters - part of VUI
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HRDParameters {
    pub cpb_cnt_minus1: u32,                         //ue(v)
    pub bit_rate_scale: u8,                          //u(4)
    pub cpb_size_scale: u8,                          //u(4)
    pub bit_rate_value_minus1: Vec<u32>,             // cpb_cnt_minus1+1 amount of ue(v)
    pub cpb_size_values_minus1: Vec<u32>,            // ^
    pub cbr_flag: Vec<bool>,                         // ^ of u(1)
    pub initial_cpb_removal_delay_length_minus1: u8, //u(5)
    pub cpb_removal_delay_length_minus1: u8,         //u(5)
    pub dpb_output_delay_length_minus1: u8,          //u(5)
    pub time_offset_length: u8,                      //u(5)
}

impl HRDParameters {
    pub fn new() -> HRDParameters {
        HRDParameters {
            cpb_cnt_minus1: 0,
            bit_rate_scale: 0,
            cpb_size_scale: 0,
            bit_rate_value_minus1: Vec::new(),
            cpb_size_values_minus1: Vec::new(),
            cbr_flag: Vec::new(),
            initial_cpb_removal_delay_length_minus1: 0,
            cpb_removal_delay_length_minus1: 23, // Annex E spec default value
            dpb_output_delay_length_minus1: 23,  // Annex E spec default value
            time_offset_length: 24,              // Annex E spec default value
        }
    }

    pub fn clone(&self) -> HRDParameters {
        HRDParameters {
            cpb_cnt_minus1: self.cpb_cnt_minus1,
            bit_rate_scale: self.bit_rate_scale,
            cpb_size_scale: self.cpb_size_scale,
            bit_rate_value_minus1: self.bit_rate_value_minus1.clone(),
            cpb_size_values_minus1: self.cpb_size_values_minus1.clone(),
            cbr_flag: self.cbr_flag.clone(),
            initial_cpb_removal_delay_length_minus1: self.initial_cpb_removal_delay_length_minus1,
            cpb_removal_delay_length_minus1: self.cpb_removal_delay_length_minus1,
            dpb_output_delay_length_minus1: self.dpb_output_delay_length_minus1,
            time_offset_length: self.time_offset_length,
        }
    }

    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print("HRD cpb_cnt_minus1", self.cpb_cnt_minus1, 63);
        encoder_formatted_print("HRD bit_rate_scale", self.bit_rate_scale, 63);
        encoder_formatted_print("HRD cpb_size_scale", self.cpb_size_scale, 63);
        encoder_formatted_print("HRD bit_rate_value_minus1", &self.bit_rate_value_minus1, 63);
        encoder_formatted_print(
            "HRD cpb_size_values_minus1",
            &self.cpb_size_values_minus1,
            63,
        );
        encoder_formatted_print("HRD cbr_flag", &self.cbr_flag, 63);
        encoder_formatted_print(
            "HRD initial_cpb_removal_delay_length_minus1",
            self.initial_cpb_removal_delay_length_minus1,
            63,
        );
        encoder_formatted_print(
            "HRD cpb_removal_delay_length_minus1",
            self.cpb_removal_delay_length_minus1,
            63,
        );
        encoder_formatted_print(
            "HRD dpb_output_delay_length_minus1",
            self.dpb_output_delay_length_minus1,
            63,
        );
        encoder_formatted_print("HRD time_offset_length", self.time_offset_length, 63);
    }
}

impl Default for HRDParameters {
    fn default() -> Self {
        Self::new()
    }
}

/// VUI Parameters - part of SPS
#[derive(Serialize, Deserialize, Clone)]
pub struct VUIParameters {
    pub aspect_ratio_info_present_flag: bool,     // u(1)
    pub aspect_ratio_idc: u8,                     // u(8)
    pub sar_width: u16,                           // u(16)
    pub sar_height: u16,                          // u(16)
    pub overscan_info_present_flag: bool,         // u(1)
    pub overscan_appropriate_flag: bool,          // u(1)
    pub video_signal_type_present_flag: bool,     // u(1)
    pub video_format: u8,                         // u(3)
    pub video_full_range_flag: bool,              //u(1)
    pub colour_description_present_flag: bool,    //u(1)
    pub colour_primaries: u8,                     //u(8)
    pub transfer_characteristics: u8,             //u(8)
    pub matrix_coefficients: u8,                  //u(8)
    pub chroma_loc_info_present_flag: bool,       //u(1)
    pub chroma_sample_loc_type_top_field: u32,    //ue(v)
    pub chroma_sample_loc_type_bottom_field: u32, //ue(v)
    pub timing_info_present_flag: bool,           //u(1)
    pub num_units_in_tick: u32,                   //u(32)
    pub time_scale: u32,                          //u(32)
    pub fixed_frame_rate_flag: bool,              //u(1)
    pub nal_hrd_parameters_present_flag: bool,    //u(1)
    pub nal_hrd_parameters: HRDParameters,
    pub vcl_hrd_parameters_present_flag: bool, //u(1)
    pub vcl_hrd_parameters: HRDParameters,
    pub low_delay_hrd_flag: bool,                      //u(1)
    pub pic_struct_present_flag: bool,                 //u(1)
    pub bitstream_restriction_flag: bool,              //u(1)
    pub motion_vectors_over_pic_boundaries_flag: bool, //u(1)
    pub max_bytes_per_pic_denom: u32,                  //ue(v)
    pub max_bits_per_mb_denom: u32,                    //ue(v)
    pub log2_max_mv_length_horizontal: u32,            //ue(v)
    pub log2_max_mv_length_vertical: u32,              //ue(v)
    pub max_num_reorder_frames: u32,                   //ue(v)
    pub max_dec_frame_buffering: u32,                  //ue(v)
}

impl VUIParameters {
    pub fn new() -> VUIParameters {
        VUIParameters {
            aspect_ratio_info_present_flag: false,
            aspect_ratio_idc: 0,
            sar_width: 0,
            sar_height: 0,
            overscan_info_present_flag: false,
            overscan_appropriate_flag: false,
            video_signal_type_present_flag: false,
            video_format: 0,
            video_full_range_flag: false,
            colour_description_present_flag: false,
            colour_primaries: 0,
            transfer_characteristics: 0,
            matrix_coefficients: 0,
            chroma_loc_info_present_flag: false,
            chroma_sample_loc_type_top_field: 0,
            chroma_sample_loc_type_bottom_field: 0,
            timing_info_present_flag: false,
            num_units_in_tick: 0,
            time_scale: 0,
            fixed_frame_rate_flag: false,
            nal_hrd_parameters_present_flag: false,
            nal_hrd_parameters: HRDParameters::new(),
            vcl_hrd_parameters_present_flag: false,
            vcl_hrd_parameters: HRDParameters::new(),
            low_delay_hrd_flag: false,
            pic_struct_present_flag: false,
            bitstream_restriction_flag: false,
            motion_vectors_over_pic_boundaries_flag: false,
            max_bytes_per_pic_denom: 0,
            max_bits_per_mb_denom: 0,
            log2_max_mv_length_horizontal: 0,
            log2_max_mv_length_vertical: 0,
            max_num_reorder_frames: 0,
            max_dec_frame_buffering: 0,
        }
    }

    pub fn clone(&self) -> VUIParameters {
        VUIParameters {
            aspect_ratio_info_present_flag: self.aspect_ratio_info_present_flag,
            aspect_ratio_idc: self.aspect_ratio_idc,
            sar_width: self.sar_width,
            sar_height: self.sar_height,
            overscan_info_present_flag: self.overscan_info_present_flag,
            overscan_appropriate_flag: self.overscan_appropriate_flag,
            video_signal_type_present_flag: self.video_signal_type_present_flag,
            video_format: self.video_format,
            video_full_range_flag: self.video_full_range_flag,
            colour_description_present_flag: self.colour_description_present_flag,
            colour_primaries: self.colour_primaries,
            transfer_characteristics: self.transfer_characteristics,
            matrix_coefficients: self.matrix_coefficients,
            chroma_loc_info_present_flag: self.chroma_loc_info_present_flag,
            chroma_sample_loc_type_top_field: self.chroma_sample_loc_type_top_field,
            chroma_sample_loc_type_bottom_field: self.chroma_sample_loc_type_bottom_field,
            timing_info_present_flag: self.timing_info_present_flag,
            num_units_in_tick: self.num_units_in_tick,
            time_scale: self.time_scale,
            fixed_frame_rate_flag: self.fixed_frame_rate_flag,
            nal_hrd_parameters_present_flag: self.nal_hrd_parameters_present_flag,
            nal_hrd_parameters: self.nal_hrd_parameters.clone(),
            vcl_hrd_parameters_present_flag: self.vcl_hrd_parameters_present_flag,
            vcl_hrd_parameters: self.vcl_hrd_parameters.clone(),
            low_delay_hrd_flag: self.low_delay_hrd_flag,
            pic_struct_present_flag: self.pic_struct_present_flag,
            bitstream_restriction_flag: self.bitstream_restriction_flag,
            motion_vectors_over_pic_boundaries_flag: self.motion_vectors_over_pic_boundaries_flag,
            max_bytes_per_pic_denom: self.max_bytes_per_pic_denom,
            max_bits_per_mb_denom: self.max_bits_per_mb_denom,
            log2_max_mv_length_horizontal: self.log2_max_mv_length_horizontal,
            log2_max_mv_length_vertical: self.log2_max_mv_length_vertical,
            max_num_reorder_frames: self.max_num_reorder_frames,
            max_dec_frame_buffering: self.max_dec_frame_buffering,
        }
    }

    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print(
            "VUI aspect_ratio_info_present_flag",
            self.aspect_ratio_info_present_flag,
            63,
        );
        encoder_formatted_print("VUI aspect_ratio_idc", self.aspect_ratio_idc, 63);
        encoder_formatted_print("VUI sar_width", self.sar_width, 63);
        encoder_formatted_print("VUI sar_height", self.sar_height, 63);
        encoder_formatted_print(
            "VUI overscan_info_present_flag",
            self.overscan_info_present_flag,
            63,
        );
        encoder_formatted_print(
            "VUI overscan_appropriate_flag",
            self.overscan_appropriate_flag,
            63,
        );
        encoder_formatted_print(
            "VUI video_signal_type_present_flag",
            self.video_signal_type_present_flag,
            63,
        );
        encoder_formatted_print("VUI video_format", self.video_format, 63);
        encoder_formatted_print("VUI video_full_range_flag", self.video_full_range_flag, 63);
        encoder_formatted_print(
            "VUI colour_description_present_flag",
            self.colour_description_present_flag,
            63,
        );
        encoder_formatted_print("VUI colour_primaries", self.colour_primaries, 63);
        encoder_formatted_print(
            "VUI transfer_characteristics",
            self.transfer_characteristics,
            63,
        );
        encoder_formatted_print("VUI matrix_coefficients", self.matrix_coefficients, 63);
        encoder_formatted_print(
            "VUI chroma_loc_info_present_flag",
            self.chroma_loc_info_present_flag,
            63,
        );
        encoder_formatted_print(
            "VUI chroma_sample_loc_type_top_field",
            self.chroma_sample_loc_type_top_field,
            63,
        );
        encoder_formatted_print(
            "VUI chroma_sample_loc_type_bottom_field",
            self.chroma_sample_loc_type_bottom_field,
            63,
        );
        encoder_formatted_print(
            "VUI timing_info_present_flag",
            self.timing_info_present_flag,
            63,
        );
        encoder_formatted_print("VUI num_units_in_tick", self.num_units_in_tick, 63);
        encoder_formatted_print("VUI time_scale", self.time_scale, 63);
        encoder_formatted_print("VUI fixed_frame_rate_flag", self.fixed_frame_rate_flag, 63);
        encoder_formatted_print(
            "VUI nal_hrd_parameters_present_flag",
            self.nal_hrd_parameters_present_flag,
            63,
        );
        if self.nal_hrd_parameters_present_flag {
            self.nal_hrd_parameters.encoder_pretty_print();
        }
        encoder_formatted_print(
            "VUI vcl_hrd_parameters_present_flag",
            self.vcl_hrd_parameters_present_flag,
            63,
        );
        if self.vcl_hrd_parameters_present_flag {
            self.vcl_hrd_parameters.encoder_pretty_print();
        }
        encoder_formatted_print("VUI low_delay_hrd_flag", self.low_delay_hrd_flag, 63);
        encoder_formatted_print(
            "VUI pic_struct_present_flag",
            self.pic_struct_present_flag,
            63,
        );
        encoder_formatted_print(
            "VUI bitstream_restriction_flag",
            self.bitstream_restriction_flag,
            63,
        );
        encoder_formatted_print(
            "VUI motion_vectors_over_pic_boundaries_flag",
            self.motion_vectors_over_pic_boundaries_flag,
            63,
        );
        encoder_formatted_print(
            "VUI max_bytes_per_pic_denom",
            self.max_bytes_per_pic_denom,
            63,
        );
        encoder_formatted_print("VUI max_bits_per_mb_denom", self.max_bits_per_mb_denom, 63);
        encoder_formatted_print(
            "VUI log2_max_mv_length_horizontal",
            self.log2_max_mv_length_horizontal,
            63,
        );
        encoder_formatted_print(
            "VUI log2_max_mv_length_vertical",
            self.log2_max_mv_length_vertical,
            63,
        );
        encoder_formatted_print(
            "VUI max_num_reorder_frames",
            self.max_num_reorder_frames,
            63,
        );
        encoder_formatted_print(
            "VUI max_dec_frame_buffering",
            self.max_dec_frame_buffering,
            63,
        );
    }
}

impl Default for VUIParameters {
    fn default() -> Self {
        Self::new()
    }
}

/// SVC VUI Parameters - part of Subset SPS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SVCVUIParameters {
    pub vui_ext_num_entries_minus1: u32, // ue(v)
    pub vui_ext_dependency_id: Vec<u8>,  // u(3)
    pub vui_ext_quality_id: Vec<u8>,     // u(4)
    pub vui_ext_temporal_id: Vec<u8>,    // u(3)
    pub vui_ext_timing_info_present_flag: Vec<bool>,
    pub vui_ext_num_units_in_tick: Vec<u32>, // u(32)
    pub vui_ext_time_scale: Vec<u32>,        // u(32)
    pub vui_ext_fixed_frame_rate_flag: Vec<bool>,
    pub vui_ext_nal_hrd_parameters_present_flag: Vec<bool>,
    pub vui_ext_nal_hrd_parameters: Vec<HRDParameters>,
    pub vui_ext_vcl_hrd_parameters_present_flag: Vec<bool>,
    pub vui_ext_vcl_hrd_parameters: Vec<HRDParameters>,
    pub vui_ext_low_delay_hrd_flag: Vec<bool>,
    pub vui_ext_pic_struct_present_flag: Vec<bool>,
}

impl SVCVUIParameters {
    pub fn new() -> SVCVUIParameters {
        SVCVUIParameters {
            vui_ext_num_entries_minus1: 0,
            vui_ext_dependency_id: Vec::new(),
            vui_ext_quality_id: Vec::new(),
            vui_ext_temporal_id: Vec::new(),
            vui_ext_timing_info_present_flag: Vec::new(),
            vui_ext_num_units_in_tick: Vec::new(),
            vui_ext_time_scale: Vec::new(),
            vui_ext_fixed_frame_rate_flag: Vec::new(),
            vui_ext_nal_hrd_parameters_present_flag: Vec::new(),
            vui_ext_nal_hrd_parameters: Vec::new(),
            vui_ext_vcl_hrd_parameters_present_flag: Vec::new(),
            vui_ext_vcl_hrd_parameters: Vec::new(),
            vui_ext_low_delay_hrd_flag: Vec::new(),
            vui_ext_pic_struct_present_flag: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print(
            "SVC VUI: vui_ext_num_entries_minus1",
            &self.vui_ext_num_entries_minus1,
            63,
        );
        encoder_formatted_print(
            "SVC VUI: vui_ext_dependency_id",
            &self.vui_ext_dependency_id,
            63,
        );
        encoder_formatted_print("SVC VUI: vui_ext_quality_id", &self.vui_ext_quality_id, 63);
        encoder_formatted_print(
            "SVC VUI: vui_ext_temporal_id",
            &self.vui_ext_temporal_id,
            63,
        );
        encoder_formatted_print(
            "SVC VUI: vui_ext_timing_info_present_flag",
            &self.vui_ext_timing_info_present_flag,
            63,
        );
        encoder_formatted_print(
            "SVC VUI: vui_ext_num_units_in_tick",
            &self.vui_ext_num_units_in_tick,
            63,
        );
        encoder_formatted_print("SVC VUI: vui_ext_time_scale", &self.vui_ext_time_scale, 63);
        encoder_formatted_print(
            "SVC VUI: vui_ext_fixed_frame_rate_flag",
            &self.vui_ext_fixed_frame_rate_flag,
            63,
        );
        encoder_formatted_print(
            "SVC VUI: vui_ext_nal_hrd_parameters_present_flag",
            &self.vui_ext_nal_hrd_parameters_present_flag,
            63,
        );
        encoder_formatted_print(
            "SVC VUI: vui_ext_nal_hrd_parameters",
            &self.vui_ext_nal_hrd_parameters,
            63,
        );
        encoder_formatted_print(
            "SVC VUI: vui_ext_vcl_hrd_parameters_present_flag",
            &self.vui_ext_vcl_hrd_parameters_present_flag,
            63,
        );
        encoder_formatted_print(
            "SVC VUI: vui_ext_vcl_hrd_parameters",
            &self.vui_ext_vcl_hrd_parameters,
            63,
        );
        encoder_formatted_print(
            "SVC VUI: vui_ext_low_delay_hrd_flag",
            &self.vui_ext_low_delay_hrd_flag,
            63,
        );
        encoder_formatted_print(
            "SVC VUI: vui_ext_pic_struct_present_flag",
            &self.vui_ext_pic_struct_present_flag,
            63,
        );
    }
}

impl Default for SVCVUIParameters {
    fn default() -> Self {
        Self::new()
    }
}

/// MVC VUI Parameters -- part of Subset SPS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MVCVUIParameters {
    pub vui_mvc_num_ops_minus1: u32,                      // ue(v)
    pub vui_mvc_temporal_id: Vec<u8>,                     // u(3)
    pub vui_mvc_num_target_output_views_minus1: Vec<u32>, // ue(v)
    pub vui_mvc_view_id: Vec<Vec<u32>>,                   // ue(v)
    pub vui_mvc_timing_info_present_flag: Vec<bool>,
    pub vui_mvc_num_units_in_tick: Vec<u32>, // u(32)
    pub vui_mvc_time_scale: Vec<u32>,        // u(32)
    pub vui_mvc_fixed_frame_rate_flag: Vec<bool>,
    pub vui_mvc_nal_hrd_parameters_present_flag: Vec<bool>,
    pub vui_mvc_nal_hrd_parameters: Vec<HRDParameters>,
    pub vui_mvc_vcl_hrd_parameters_present_flag: Vec<bool>,
    pub vui_mvc_vcl_hrd_parameters: Vec<HRDParameters>,
    pub vui_mvc_low_delay_hrd_flag: Vec<bool>,
    pub vui_mvc_pic_struct_present_flag: Vec<bool>,
}

impl MVCVUIParameters {
    pub fn new() -> MVCVUIParameters {
        MVCVUIParameters {
            vui_mvc_num_ops_minus1: 0,
            vui_mvc_temporal_id: Vec::new(),
            vui_mvc_num_target_output_views_minus1: Vec::new(),
            vui_mvc_view_id: Vec::new(),
            vui_mvc_timing_info_present_flag: Vec::new(),
            vui_mvc_num_units_in_tick: Vec::new(),
            vui_mvc_time_scale: Vec::new(),
            vui_mvc_fixed_frame_rate_flag: Vec::new(),
            vui_mvc_nal_hrd_parameters_present_flag: Vec::new(),
            vui_mvc_nal_hrd_parameters: Vec::new(),
            vui_mvc_vcl_hrd_parameters_present_flag: Vec::new(),
            vui_mvc_vcl_hrd_parameters: Vec::new(),
            vui_mvc_low_delay_hrd_flag: Vec::new(),
            vui_mvc_pic_struct_present_flag: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print(
            "MVC VUI: vui_mvc_num_ops_minus1",
            &self.vui_mvc_num_ops_minus1,
            63,
        );
        encoder_formatted_print(
            "MVC VUI: vui_mvc_temporal_id",
            &self.vui_mvc_temporal_id,
            63,
        );
        encoder_formatted_print(
            "MVC VUI: vui_mvc_num_target_output_views_minus1",
            &self.vui_mvc_num_target_output_views_minus1,
            63,
        );
        encoder_formatted_print("MVC VUI: vui_mvc_view_id", &self.vui_mvc_view_id, 63);
        encoder_formatted_print(
            "MVC VUI: vui_mvc_timing_info_present_flag",
            &self.vui_mvc_timing_info_present_flag,
            63,
        );
        encoder_formatted_print(
            "MVC VUI: vui_mvc_num_units_in_tick",
            &self.vui_mvc_num_units_in_tick,
            63,
        );
        encoder_formatted_print("MVC VUI: vui_mvc_time_scale", &self.vui_mvc_time_scale, 63);
        encoder_formatted_print(
            "MVC VUI: vui_mvc_fixed_frame_rate_flag",
            &self.vui_mvc_fixed_frame_rate_flag,
            63,
        );
        encoder_formatted_print(
            "MVC VUI: vui_mvc_nal_hrd_parameters_present_flag",
            &self.vui_mvc_nal_hrd_parameters_present_flag,
            63,
        );
        encoder_formatted_print(
            "MVC VUI: vui_mvc_nal_hrd_parameters",
            &self.vui_mvc_nal_hrd_parameters,
            63,
        );
        encoder_formatted_print(
            "MVC VUI: vui_mvc_vcl_hrd_parameters_present_flag",
            &self.vui_mvc_vcl_hrd_parameters_present_flag,
            63,
        );
        encoder_formatted_print(
            "MVC VUI: vui_mvc_vcl_hrd_parameters",
            &self.vui_mvc_vcl_hrd_parameters,
            63,
        );
        encoder_formatted_print(
            "MVC VUI: vui_mvc_low_delay_hrd_flag",
            &self.vui_mvc_low_delay_hrd_flag,
            63,
        );
        encoder_formatted_print(
            "MVC VUI: vui_mvc_pic_struct_present_flag",
            &self.vui_mvc_pic_struct_present_flag,
            63,
        );
    }
}

impl Default for MVCVUIParameters {
    fn default() -> Self {
        Self::new()
    }
}

/// MVCD VUI Parameters -- part of Subset SPS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MVCDVUIParameters {
    pub vui_mvcd_num_ops_minus1: u32,                      // ue(v)
    pub vui_mvcd_temporal_id: Vec<u8>,                     // u(3)
    pub vui_mvcd_num_target_output_views_minus1: Vec<u32>, // ue(v)
    pub vui_mvcd_view_id: Vec<Vec<u32>>,                   // ue(v)
    pub vui_mvcd_depth_flag: Vec<Vec<bool>>,
    pub vui_mvcd_texture_flag: Vec<Vec<bool>>,
    pub vui_mvcd_timing_info_present_flag: Vec<bool>,
    pub vui_mvcd_num_units_in_tick: Vec<u32>, // u(32)
    pub vui_mvcd_time_scale: Vec<u32>,        // u(32)
    pub vui_mvcd_fixed_frame_rate_flag: Vec<bool>,
    pub vui_mvcd_nal_hrd_parameters_present_flag: Vec<bool>,
    pub vui_mvcd_nal_hrd_parameters: Vec<HRDParameters>,
    pub vui_mvcd_vcl_hrd_parameters_present_flag: Vec<bool>,
    pub vui_mvcd_vcl_hrd_parameters: Vec<HRDParameters>,
    pub vui_mvcd_low_delay_hrd_flag: Vec<bool>,
    pub vui_mvcd_pic_struct_present_flag: Vec<bool>,
}

impl MVCDVUIParameters {
    pub fn new() -> MVCDVUIParameters {
        MVCDVUIParameters {
            vui_mvcd_num_ops_minus1: 0,
            vui_mvcd_temporal_id: Vec::new(),
            vui_mvcd_num_target_output_views_minus1: Vec::new(),
            vui_mvcd_view_id: Vec::new(),
            vui_mvcd_depth_flag: Vec::new(),
            vui_mvcd_texture_flag: Vec::new(),
            vui_mvcd_timing_info_present_flag: Vec::new(),
            vui_mvcd_num_units_in_tick: Vec::new(),
            vui_mvcd_time_scale: Vec::new(),
            vui_mvcd_fixed_frame_rate_flag: Vec::new(),
            vui_mvcd_nal_hrd_parameters_present_flag: Vec::new(),
            vui_mvcd_nal_hrd_parameters: Vec::new(),
            vui_mvcd_vcl_hrd_parameters_present_flag: Vec::new(),
            vui_mvcd_vcl_hrd_parameters: Vec::new(),
            vui_mvcd_low_delay_hrd_flag: Vec::new(),
            vui_mvcd_pic_struct_present_flag: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print(
            "MVCD VUI: vui_mvcd_num_ops_minus1",
            &self.vui_mvcd_num_ops_minus1,
            63,
        );
        encoder_formatted_print(
            "MVCD VUI: vui_mvcd_temporal_id",
            &self.vui_mvcd_temporal_id,
            63,
        );
        encoder_formatted_print(
            "MVCD VUI: vui_mvcd_num_target_output_views_minus1",
            &self.vui_mvcd_num_target_output_views_minus1,
            63,
        );
        encoder_formatted_print("MVCD VUI: vui_mvcd_view_id", &self.vui_mvcd_view_id, 63);
        encoder_formatted_print(
            "MVCD VUI: vui_mvcd_depth_flag",
            &self.vui_mvcd_depth_flag,
            63,
        );
        encoder_formatted_print(
            "MVCD VUI: vui_mvcd_texture_flag",
            &self.vui_mvcd_texture_flag,
            63,
        );
        encoder_formatted_print(
            "MVCD VUI: vui_mvcd_timing_info_present_flag",
            &self.vui_mvcd_timing_info_present_flag,
            63,
        );
        encoder_formatted_print(
            "MVCD VUI: vui_mvcd_num_units_in_tick",
            &self.vui_mvcd_num_units_in_tick,
            63,
        );
        encoder_formatted_print(
            "MVCD VUI: vui_mvcd_time_scale",
            &self.vui_mvcd_time_scale,
            63,
        );
        encoder_formatted_print(
            "MVCD VUI: vui_mvcd_fixed_frame_rate_flag",
            &self.vui_mvcd_fixed_frame_rate_flag,
            63,
        );
        encoder_formatted_print(
            "MVCD VUI: vui_mvcd_nal_hrd_parameters_present_flag",
            &self.vui_mvcd_nal_hrd_parameters_present_flag,
            63,
        );
        encoder_formatted_print(
            "MVCD VUI: vui_mvcd_nal_hrd_parameters",
            &self.vui_mvcd_nal_hrd_parameters,
            63,
        );
        encoder_formatted_print(
            "MVCD VUI: vui_mvcd_vcl_hrd_parameters_present_flag",
            &self.vui_mvcd_vcl_hrd_parameters_present_flag,
            63,
        );
        encoder_formatted_print(
            "MVCD VUI: vui_mvcd_vcl_hrd_parameters",
            &self.vui_mvcd_vcl_hrd_parameters,
            63,
        );
        encoder_formatted_print(
            "MVCD VUI: vui_mvcd_low_delay_hrd_flag",
            &self.vui_mvcd_low_delay_hrd_flag,
            63,
        );
        encoder_formatted_print(
            "MVCD VUI: vui_mvcd_pic_struct_present_flag",
            &self.vui_mvcd_pic_struct_present_flag,
            63,
        );
    }
}

impl Default for MVCDVUIParameters {
    fn default() -> Self {
        Self::new()
    }
}

/// SVC SPS Parameters -- part of Subset SPS
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SVCSPSExtension {
    pub inter_layer_deblocking_filter_control_present_flag: bool,
    pub extended_spatial_scalability_idc: u8,
    pub chroma_phase_x_plus1_flag: bool,
    pub chroma_phase_y_plus1: u8,
    pub seq_ref_layer_chroma_phase_x_plus1_flag: bool,
    pub seq_ref_layer_chroma_phase_y_plus1: u8,
    pub seq_scaled_ref_layer_left_offset: i32,
    pub seq_scaled_ref_layer_top_offset: i32,
    pub seq_scaled_ref_layer_right_offset: i32,
    pub seq_scaled_ref_layer_bottom_offset: i32,
    pub seq_tcoeff_level_prediction_flag: bool,
    pub adaptive_tcoeff_level_prediction_flag: bool,
    pub slice_header_restriction_flag: bool,
}

impl SVCSPSExtension {
    pub fn new() -> SVCSPSExtension {
        SVCSPSExtension {
            inter_layer_deblocking_filter_control_present_flag: false,
            extended_spatial_scalability_idc: 0,
            chroma_phase_x_plus1_flag: false,
            chroma_phase_y_plus1: 0,
            seq_ref_layer_chroma_phase_x_plus1_flag: false,
            seq_ref_layer_chroma_phase_y_plus1: 0,
            seq_scaled_ref_layer_left_offset: 0,
            seq_scaled_ref_layer_top_offset: 0,
            seq_scaled_ref_layer_right_offset: 0,
            seq_scaled_ref_layer_bottom_offset: 0,
            seq_tcoeff_level_prediction_flag: false,
            adaptive_tcoeff_level_prediction_flag: false,
            slice_header_restriction_flag: false,
        }
    }

    #[allow(dead_code)]
    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print(
            "SVC SPS: inter_layer_deblocking_filter_control_present_flag",
            self.inter_layer_deblocking_filter_control_present_flag,
            63,
        );
        encoder_formatted_print(
            "SVC SPS: extended_spatial_scalability_idc",
            self.extended_spatial_scalability_idc,
            63,
        );
        encoder_formatted_print(
            "SVC SPS: chroma_phase_x_plus1_flag",
            self.chroma_phase_x_plus1_flag,
            63,
        );
        encoder_formatted_print(
            "SVC SPS: chroma_phase_y_plus1",
            self.chroma_phase_y_plus1,
            63,
        );
        encoder_formatted_print(
            "SVC SPS: seq_ref_layer_chroma_phase_x_plus1_flag",
            self.seq_ref_layer_chroma_phase_x_plus1_flag,
            63,
        );
        encoder_formatted_print(
            "SVC SPS: seq_ref_layer_chroma_phase_y_plus1",
            self.seq_ref_layer_chroma_phase_y_plus1,
            63,
        );
        encoder_formatted_print(
            "SVC SPS: seq_scaled_ref_layer_left_offset",
            self.seq_scaled_ref_layer_left_offset,
            63,
        );
        encoder_formatted_print(
            "SVC SPS: seq_scaled_ref_layer_top_offset",
            self.seq_scaled_ref_layer_top_offset,
            63,
        );
        encoder_formatted_print(
            "SVC SPS: seq_scaled_ref_layer_right_offset",
            self.seq_scaled_ref_layer_right_offset,
            63,
        );
        encoder_formatted_print(
            "SVC SPS: seq_scaled_ref_layer_bottom_offset",
            self.seq_scaled_ref_layer_bottom_offset,
            63,
        );
        encoder_formatted_print(
            "SVC SPS: seq_tcoeff_level_prediction_flag",
            self.seq_tcoeff_level_prediction_flag,
            63,
        );
        encoder_formatted_print(
            "SVC SPS: adaptive_tcoeff_level_prediction_flag",
            self.adaptive_tcoeff_level_prediction_flag,
            63,
        );
        encoder_formatted_print(
            "SVC SPS: slice_header_restriction_flag",
            self.slice_header_restriction_flag,
            63,
        );
    }
}

impl Default for SVCSPSExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// MVC SPS Parameters -- part of Subset SPS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MVCSPSExtension {
    pub num_views_minus1: usize,                              // ue(v)
    pub view_id: Vec<u32>,                                    // num_views_minus1+1 number of ue(v)
    pub num_anchor_refs_l0: Vec<u32>,                         // num_views_minus1+1 number of ue(v)
    pub anchor_refs_l0: Vec<Vec<u32>>, // (num_views_minus1+1) * num_anchor_refs_l0[i] number of ue(v)
    pub num_anchor_refs_l1: Vec<u32>,  // num_views_minus1+1 number of ue(v)
    pub anchor_refs_l1: Vec<Vec<u32>>, // (num_views_minus1+1) * num_anchor_refs_l1[i] number of ue(v)
    pub num_non_anchor_refs_l0: Vec<u32>, // num_views_minus1+1 number of ue(v)
    pub non_anchor_refs_l0: Vec<Vec<u32>>, // (num_views_minus1+1) * num_non_anchor_refs_l0[i] number of ue(v)
    pub num_non_anchor_refs_l1: Vec<u32>,  // num_views_minus1+1 number of ue(v)
    pub non_anchor_refs_l1: Vec<Vec<u32>>, // (num_views_minus1+1) * num_non_anchor_refs_l1[i] number of ue(v)
    pub num_level_values_signalled_minus1: usize, // ue(v)
    pub level_idc: Vec<u8>,                // num_level_values_signalled_minus1+1 of u(8)
    pub num_applicable_ops_minus1: Vec<usize>, // num_level_values_signalled_minus1+1 of ue(v)
    pub applicable_op_temporal_id: Vec<Vec<u8>>, // (num_level_values_signalled_minus1+1) * (num_applicable_ops_minus1+1) of u(3)
    pub applicable_op_num_target_views_minus1: Vec<Vec<u32>>, // (num_level_values_signalled_minus1+1) * (num_applicable_ops_minus1+1) of ue(v)
    pub applicable_op_target_view_id: Vec<Vec<Vec<u32>>>, // (num_level_values_signalled_minus1+1) * (num_applicable_ops_minus1+1) * (applicable_op_num_target_views_minus1+1) of ue(v)
    pub applicable_op_num_views_minus1: Vec<Vec<u32>>, // (num_level_values_signalled_minus1+1) * (num_applicable_ops_minus1+1) of ue(v)
    pub mfc_format_idc: u8,                            // u(6)
    pub default_grid_position_flag: bool,              // u(1)
    pub view0_grid_position_x: u8,                     // u(4)
    pub view0_grid_position_y: u8,                     // u(4)
    pub view1_grid_position_x: u8,                     // u(4)
    pub view1_grid_position_y: u8,                     // u(4)
    pub rpu_filter_enabled_flag: bool,                 // u(1)
    pub rpu_field_processing_flag: bool,               // u(1)
}

impl MVCSPSExtension {
    pub fn new() -> MVCSPSExtension {
        MVCSPSExtension {
            num_views_minus1: 0,
            view_id: Vec::new(),
            num_anchor_refs_l0: Vec::new(),
            anchor_refs_l0: Vec::new(),
            num_anchor_refs_l1: Vec::new(),
            anchor_refs_l1: Vec::new(),
            num_non_anchor_refs_l0: Vec::new(),
            non_anchor_refs_l0: Vec::new(),
            num_non_anchor_refs_l1: Vec::new(),
            non_anchor_refs_l1: Vec::new(),
            num_level_values_signalled_minus1: 0,
            level_idc: Vec::new(),
            num_applicable_ops_minus1: Vec::new(),
            applicable_op_temporal_id: Vec::new(),
            applicable_op_num_target_views_minus1: Vec::new(),
            applicable_op_target_view_id: Vec::new(),
            applicable_op_num_views_minus1: Vec::new(),
            mfc_format_idc: 0,
            default_grid_position_flag: false,
            view0_grid_position_x: 0,
            view0_grid_position_y: 0,
            view1_grid_position_x: 0,
            view1_grid_position_y: 0,
            rpu_filter_enabled_flag: false,
            rpu_field_processing_flag: false,
        }
    }

    #[allow(dead_code)]
    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print("MVC SPS: num_views_minus1", self.num_views_minus1, 63);
        encoder_formatted_print("MVC SPS: view_id", self.view_id.clone(), 63);
        encoder_formatted_print(
            "MVC SPS: num_anchor_refs_l0",
            self.num_anchor_refs_l0.clone(),
            63,
        );
        encoder_formatted_print("MVC SPS: anchor_refs_l0", self.anchor_refs_l0.clone(), 63);
        encoder_formatted_print(
            "MVC SPS: num_anchor_refs_l1",
            self.num_anchor_refs_l1.clone(),
            63,
        );
        encoder_formatted_print("MVC SPS: anchor_refs_l1", self.anchor_refs_l1.clone(), 63);
        encoder_formatted_print(
            "MVC SPS: num_non_anchor_refs_l0",
            self.num_non_anchor_refs_l0.clone(),
            63,
        );
        encoder_formatted_print(
            "MVC SPS: non_anchor_refs_l0",
            self.non_anchor_refs_l0.clone(),
            63,
        );
        encoder_formatted_print(
            "MVC SPS: num_non_anchor_refs_l1",
            self.num_non_anchor_refs_l1.clone(),
            63,
        );
        encoder_formatted_print(
            "MVC SPS: non_anchor_refs_l1",
            self.non_anchor_refs_l1.clone(),
            63,
        );
        encoder_formatted_print(
            "MVC SPS: num_level_values_signalled_minus1",
            self.num_level_values_signalled_minus1,
            63,
        );
        encoder_formatted_print("MVC SPS: level_idc", self.level_idc.clone(), 63);
        encoder_formatted_print(
            "MVC SPS: num_applicable_ops_minus1",
            self.num_applicable_ops_minus1.clone(),
            63,
        );
        encoder_formatted_print(
            "MVC SPS: applicable_op_temporal_id",
            self.applicable_op_temporal_id.clone(),
            63,
        );
        encoder_formatted_print(
            "MVC SPS: applicable_op_num_target_views_minus1",
            self.applicable_op_num_target_views_minus1.clone(),
            63,
        );
        encoder_formatted_print(
            "MVC SPS: applicable_op_target_view_id",
            self.applicable_op_target_view_id.clone(),
            63,
        );
        encoder_formatted_print(
            "MVC SPS: applicable_op_num_views_minus1",
            self.applicable_op_num_views_minus1.clone(),
            63,
        );
        encoder_formatted_print("MVC SPS: mfc_format_idc", self.mfc_format_idc, 63);
        encoder_formatted_print(
            "MVC SPS: default_grid_position_flag",
            self.default_grid_position_flag,
            63,
        );
        encoder_formatted_print(
            "MVC SPS: view0_grid_position_x",
            self.view0_grid_position_x,
            63,
        );
        encoder_formatted_print(
            "MVC SPS: view0_grid_position_y",
            self.view0_grid_position_y,
            63,
        );
        encoder_formatted_print(
            "MVC SPS: view1_grid_position_x",
            self.view1_grid_position_x,
            63,
        );
        encoder_formatted_print(
            "MVC SPS: view1_grid_position_y",
            self.view1_grid_position_y,
            63,
        );
        encoder_formatted_print(
            "MVC SPS: rpu_filter_enabled_flag",
            self.rpu_filter_enabled_flag,
            63,
        );
        encoder_formatted_print(
            "MVC SPS: rpu_field_processing_flag",
            self.rpu_field_processing_flag,
            63,
        );
    }
}

impl Default for MVCSPSExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// MVCD SPS Parameters -- part of Subset SPS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MVCDSPSExtension {
    pub num_views_minus1: u32, // ue(v)
    pub view_id: Vec<u32>,     // ue(v)
    pub depth_view_present_flag: Vec<bool>,
    pub texture_view_present_flag: Vec<bool>,
    pub num_anchor_refs_l0: Vec<u32>,            // ue(v)
    pub anchor_ref_l0: Vec<Vec<u32>>,            // ue(v)
    pub num_anchor_refs_l1: Vec<u32>,            // ue(v)
    pub anchor_ref_l1: Vec<Vec<u32>>,            // ue(v)
    pub num_non_anchor_refs_l0: Vec<u32>,        // ue(v)
    pub non_anchor_ref_l0: Vec<Vec<u32>>,        // ue(v)
    pub num_non_anchor_refs_l1: Vec<u32>,        // ue(v)
    pub non_anchor_ref_l1: Vec<Vec<u32>>,        // ue(v)
    pub num_level_values_signalled_minus1: u32,  //ue(v)
    pub level_idc: Vec<u8>,                      // u(8)
    pub num_applicable_ops_minus1: Vec<u32>,     // ue(v)
    pub applicable_op_temporal_id: Vec<Vec<u8>>, // u(3)
    pub applicable_op_num_target_views_minus1: Vec<Vec<u32>>, // ue(v)
    pub applicable_op_target_view_id: Vec<Vec<Vec<u32>>>, // ue(v)
    pub applicable_op_depth_flag: Vec<Vec<Vec<bool>>>,
    pub applicable_op_texture_flag: Vec<Vec<Vec<u32>>>,
    pub applicable_op_num_texture_views_minus1: Vec<Vec<u32>>, // ue(v)
    pub applicable_op_num_depth_views: Vec<Vec<u32>>,          // ue(v)
    pub mvcd_vui_parameters_present_flag: bool,
    pub mvcd_vui_parameters: MVCDVUIParameters,
    pub texture_vui_parameters_present_flag: bool,
    pub mvc_vui_parameters_extension: MVCVUIParameters,
}

impl MVCDSPSExtension {
    pub fn new() -> MVCDSPSExtension {
        MVCDSPSExtension {
            num_views_minus1: 0,
            view_id: Vec::new(),
            depth_view_present_flag: Vec::new(),
            texture_view_present_flag: Vec::new(),
            num_anchor_refs_l0: Vec::new(),
            anchor_ref_l0: Vec::new(),
            num_anchor_refs_l1: Vec::new(),
            anchor_ref_l1: Vec::new(),
            num_non_anchor_refs_l0: Vec::new(),
            non_anchor_ref_l0: Vec::new(),
            num_non_anchor_refs_l1: Vec::new(),
            non_anchor_ref_l1: Vec::new(),
            num_level_values_signalled_minus1: 0,
            level_idc: Vec::new(),
            num_applicable_ops_minus1: Vec::new(),
            applicable_op_temporal_id: Vec::new(),
            applicable_op_num_target_views_minus1: Vec::new(),
            applicable_op_target_view_id: Vec::new(),
            applicable_op_depth_flag: Vec::new(),
            applicable_op_texture_flag: Vec::new(),
            applicable_op_num_texture_views_minus1: Vec::new(),
            applicable_op_num_depth_views: Vec::new(),
            mvcd_vui_parameters_present_flag: false,
            mvcd_vui_parameters: MVCDVUIParameters::new(),
            texture_vui_parameters_present_flag: false,
            mvc_vui_parameters_extension: MVCVUIParameters::new(),
        }
    }

    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print("MVCD SPS: num_views_minus1", &self.num_views_minus1, 63);
        encoder_formatted_print("MVCD SPS: view_id", &self.view_id, 63);
        encoder_formatted_print(
            "MVCD SPS: depth_view_present_flag",
            &self.depth_view_present_flag,
            63,
        );
        encoder_formatted_print(
            "MVCD SPS: texture_view_present_flag",
            &self.texture_view_present_flag,
            63,
        );
        encoder_formatted_print("MVCD SPS: num_anchor_refs_l0", &self.num_anchor_refs_l0, 63);
        encoder_formatted_print("MVCD SPS: anchor_ref_l0", &self.anchor_ref_l0, 63);
        encoder_formatted_print("MVCD SPS: num_anchor_refs_l1", &self.num_anchor_refs_l1, 63);
        encoder_formatted_print("MVCD SPS: anchor_ref_l1", &self.anchor_ref_l1, 63);
        encoder_formatted_print(
            "MVCD SPS: num_non_anchor_refs_l0",
            &self.num_non_anchor_refs_l0,
            63,
        );
        encoder_formatted_print("MVCD SPS: non_anchor_ref_l0", &self.non_anchor_ref_l0, 63);
        encoder_formatted_print(
            "MVCD SPS: num_non_anchor_refs_l1",
            &self.num_non_anchor_refs_l1,
            63,
        );
        encoder_formatted_print("MVCD SPS: non_anchor_ref_l1", &self.non_anchor_ref_l1, 63);
        encoder_formatted_print(
            "MVCD SPS: num_level_values_signalled_minus1",
            &self.num_level_values_signalled_minus1,
            63,
        );
        encoder_formatted_print("MVCD SPS: level_idc", &self.level_idc, 63);
        encoder_formatted_print(
            "MVCD SPS: num_applicable_ops_minus1",
            &self.num_applicable_ops_minus1,
            63,
        );
        encoder_formatted_print(
            "MVCD SPS: applicable_op_temporal_id",
            &self.applicable_op_temporal_id,
            63,
        );
        encoder_formatted_print(
            "MVCD SPS: applicable_op_num_target_views_minus1",
            &self.applicable_op_num_target_views_minus1,
            63,
        );
        encoder_formatted_print(
            "MVCD SPS: applicable_op_target_view_id",
            &self.applicable_op_target_view_id,
            63,
        );
        encoder_formatted_print(
            "MVCD SPS: applicable_op_depth_flag",
            &self.applicable_op_depth_flag,
            63,
        );
        encoder_formatted_print(
            "MVCD SPS: applicable_op_texture_flag",
            &self.applicable_op_texture_flag,
            63,
        );
        encoder_formatted_print(
            "MVCD SPS: applicable_op_num_texture_views_minus1",
            &self.applicable_op_num_texture_views_minus1,
            63,
        );
        encoder_formatted_print(
            "MVCD SPS: applicable_op_num_depth_views",
            &self.applicable_op_num_depth_views,
            63,
        );
        encoder_formatted_print(
            "MVCD SPS: mvcd_vui_parameters_present_flag",
            &self.mvcd_vui_parameters_present_flag,
            63,
        );
        self.mvcd_vui_parameters.encoder_pretty_print();
        encoder_formatted_print(
            "MVCD SPS: texture_vui_parameters_present_flag",
            &self.texture_vui_parameters_present_flag,
            63,
        );
        self.mvc_vui_parameters_extension.encoder_pretty_print();
    }
}

impl Default for MVCDSPSExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// AVC-3D SPS Parameters -- part of Subset SPS
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AVC3DSPSExtension {
    // TODO: AVC 3D SPS Extension
}

impl AVC3DSPSExtension {
    pub fn new() -> AVC3DSPSExtension {
        AVC3DSPSExtension {}
    }

    pub fn encoder_pretty_print(&self) {}
}

impl Default for AVC3DSPSExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// NALU Type 7 -- Sequence Parameter Set
#[derive(Serialize, Deserialize, Clone)]
pub struct SeqParameterSet {
    pub available: bool, // used to determine if the SPS has been set or not

    pub profile_idc: u8,
    pub constraint_set0_flag: bool,
    pub constraint_set1_flag: bool,
    pub constraint_set2_flag: bool,
    pub constraint_set3_flag: bool,
    pub constraint_set4_flag: bool,
    pub constraint_set5_flag: bool,
    pub reserved_zero_2bits: u8, //u(2)
    pub level_idc: u8,
    pub seq_parameter_set_id: u32, //ue(v)

    pub chroma_format_idc: u8, //ue(v)
    pub separate_colour_plane_flag: bool,
    pub bit_depth_luma_minus8: u8,   //ue(v)
    pub bit_depth_chroma_minus8: u8, //ue(v)
    pub qpprime_y_zero_transform_bypass_flag: bool,

    // seq_scaling
    pub seq_scaling_matrix_present_flag: bool,
    pub seq_scaling_list_present_flag: Vec<bool>, //u(1)[8 or 12]
    pub delta_scale_4x4: Vec<Vec<i32>>, // used to calculate scaling list, but store for ease in encoding
    pub scaling_list_4x4: Vec<Vec<i32>>, // list (len 6) of list (len 16) of se(v)
    pub delta_scale_8x8: Vec<Vec<i32>>, // used to calculate scaling list, but store for ease in encoding
    pub scaling_list_8x8: Vec<Vec<i32>>, // list (len 2 or 6) of list (len 64) of se(v)
    pub use_default_scaling_matrix_4x4: Vec<bool>, // list len 6
    pub use_default_scaling_matrix_8x8: Vec<bool>, // list len 2 or 6

    pub log2_max_frame_num_minus4: u32, //ue(v)

    pub pic_order_cnt_type: u32,               //ue(v)
    pub log2_max_pic_order_cnt_lsb_minus4: u8, //ue(v) - the range of this must be between 0 to 12 inclusive

    pub delta_pic_order_always_zero_flag: bool,
    pub offset_for_non_ref_pic: i32,                //se(v)
    pub offset_for_top_to_bottom_field: i32,        //se(v)
    pub num_ref_frames_in_pic_order_cnt_cycle: u32, //ue(v)
    pub offset_for_ref_frame: Vec<i32>,             //se(v)[num_ref_frames_in_pic_order_cnt_cycle]

    pub max_num_ref_frames: u32, //ue(v)
    pub gaps_in_frame_num_value_allowed_flag: bool,
    pub pic_width_in_mbs_minus1: u32,        //ue(v)
    pub pic_height_in_map_units_minus1: u32, //ue(v)

    pub frame_mbs_only_flag: bool,
    pub mb_adaptive_frame_field_flag: bool,

    pub direct_8x8_inference_flag: bool,

    pub frame_cropping_flag: bool,
    pub frame_crop_left_offset: u32,   //ue(v)
    pub frame_crop_right_offset: u32,  //ue(v)
    pub frame_crop_top_offset: u32,    //ue(v)
    pub frame_crop_bottom_offset: u32, //ue(v)

    pub vui_parameters_present_flag: bool,
    pub vui_parameters: VUIParameters,
}

impl SeqParameterSet {
    pub fn new() -> SeqParameterSet {
        SeqParameterSet {
            available: false,
            profile_idc: 0,
            constraint_set0_flag: false,
            constraint_set1_flag: false,
            constraint_set2_flag: false,
            constraint_set3_flag: false,
            constraint_set4_flag: false,
            constraint_set5_flag: false,
            reserved_zero_2bits: 0,
            level_idc: 0,
            seq_parameter_set_id: 0,

            chroma_format_idc: 1, // section 7.4.2.1.1: when not present, assume to be 1 (4:2:0 chroma format)
            separate_colour_plane_flag: false,
            bit_depth_luma_minus8: 0,
            bit_depth_chroma_minus8: 0,
            qpprime_y_zero_transform_bypass_flag: false,

            seq_scaling_matrix_present_flag: false,
            seq_scaling_list_present_flag: Vec::new(),
            delta_scale_4x4: Vec::new(),
            scaling_list_4x4: Vec::new(),
            delta_scale_8x8: Vec::new(),
            scaling_list_8x8: Vec::new(),
            use_default_scaling_matrix_4x4: Vec::new(),
            use_default_scaling_matrix_8x8: Vec::new(),

            log2_max_frame_num_minus4: 0,

            pic_order_cnt_type: 0,
            log2_max_pic_order_cnt_lsb_minus4: 0,

            delta_pic_order_always_zero_flag: false,
            offset_for_non_ref_pic: 0,
            offset_for_top_to_bottom_field: 0,
            num_ref_frames_in_pic_order_cnt_cycle: 0,
            offset_for_ref_frame: Vec::new(),

            max_num_ref_frames: 0,
            gaps_in_frame_num_value_allowed_flag: false,
            pic_width_in_mbs_minus1: 0,
            pic_height_in_map_units_minus1: 0,

            frame_mbs_only_flag: false,
            mb_adaptive_frame_field_flag: false,

            direct_8x8_inference_flag: false,

            frame_cropping_flag: false,
            frame_crop_left_offset: 0,
            frame_crop_right_offset: 0,
            frame_crop_top_offset: 0,
            frame_crop_bottom_offset: 0,

            vui_parameters_present_flag: false,
            vui_parameters: VUIParameters::new(),
        }
    }

    pub fn clone(&self) -> SeqParameterSet {
        SeqParameterSet {
            available: self.available,
            profile_idc: self.profile_idc,
            constraint_set0_flag: self.constraint_set0_flag,
            constraint_set1_flag: self.constraint_set1_flag,
            constraint_set2_flag: self.constraint_set2_flag,
            constraint_set3_flag: self.constraint_set3_flag,
            constraint_set4_flag: self.constraint_set4_flag,
            constraint_set5_flag: self.constraint_set5_flag,
            reserved_zero_2bits: self.reserved_zero_2bits,
            level_idc: self.level_idc,
            seq_parameter_set_id: self.seq_parameter_set_id,

            chroma_format_idc: self.chroma_format_idc,
            separate_colour_plane_flag: self.separate_colour_plane_flag,
            bit_depth_luma_minus8: self.bit_depth_luma_minus8,
            bit_depth_chroma_minus8: self.bit_depth_chroma_minus8,
            qpprime_y_zero_transform_bypass_flag: self.qpprime_y_zero_transform_bypass_flag,

            seq_scaling_matrix_present_flag: self.seq_scaling_matrix_present_flag,
            seq_scaling_list_present_flag: self.seq_scaling_list_present_flag.clone(),
            delta_scale_4x4: self.delta_scale_4x4.clone(),
            scaling_list_4x4: self.scaling_list_4x4.clone(),
            delta_scale_8x8: self.delta_scale_8x8.clone(),
            scaling_list_8x8: self.scaling_list_8x8.clone(),
            use_default_scaling_matrix_4x4: self.use_default_scaling_matrix_4x4.clone(),
            use_default_scaling_matrix_8x8: self.use_default_scaling_matrix_8x8.clone(),

            log2_max_frame_num_minus4: self.log2_max_frame_num_minus4,

            pic_order_cnt_type: self.pic_order_cnt_type,
            log2_max_pic_order_cnt_lsb_minus4: self.log2_max_pic_order_cnt_lsb_minus4,

            delta_pic_order_always_zero_flag: self.delta_pic_order_always_zero_flag,
            offset_for_non_ref_pic: self.offset_for_non_ref_pic,
            offset_for_top_to_bottom_field: self.offset_for_top_to_bottom_field,
            num_ref_frames_in_pic_order_cnt_cycle: self.num_ref_frames_in_pic_order_cnt_cycle,
            offset_for_ref_frame: self.offset_for_ref_frame.clone(),

            max_num_ref_frames: self.max_num_ref_frames,
            gaps_in_frame_num_value_allowed_flag: self.gaps_in_frame_num_value_allowed_flag,
            pic_width_in_mbs_minus1: self.pic_width_in_mbs_minus1,
            pic_height_in_map_units_minus1: self.pic_height_in_map_units_minus1,

            frame_mbs_only_flag: self.frame_mbs_only_flag,
            mb_adaptive_frame_field_flag: self.mb_adaptive_frame_field_flag,

            direct_8x8_inference_flag: self.direct_8x8_inference_flag,

            frame_cropping_flag: self.frame_cropping_flag,
            frame_crop_left_offset: self.frame_crop_left_offset,
            frame_crop_right_offset: self.frame_crop_right_offset,
            frame_crop_top_offset: self.frame_crop_top_offset,
            frame_crop_bottom_offset: self.frame_crop_bottom_offset,

            vui_parameters_present_flag: self.vui_parameters_present_flag,
            vui_parameters: self.vui_parameters.clone(),
        }
    }

    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print("SPS: profile_idc", self.profile_idc, 63);
        encoder_formatted_print("SPS: constraint_set0_flag", self.constraint_set0_flag, 63);
        encoder_formatted_print("SPS: constraint_set1_flag", self.constraint_set1_flag, 63);
        encoder_formatted_print("SPS: constraint_set2_flag", self.constraint_set2_flag, 63);
        encoder_formatted_print("SPS: constraint_set3_flag", self.constraint_set3_flag, 63);
        encoder_formatted_print("SPS: constraint_set4_flag", self.constraint_set4_flag, 63);
        encoder_formatted_print("SPS: constraint_set5_flag", self.constraint_set5_flag, 63);
        encoder_formatted_print("SPS: reserved_zero_2bits", self.reserved_zero_2bits, 63);
        encoder_formatted_print("SPS: level_idc", self.level_idc, 63);
        encoder_formatted_print("SPS: seq_parameter_set_id", self.seq_parameter_set_id, 63);
        encoder_formatted_print("SPS: chroma_format_idc", self.chroma_format_idc, 63);
        if self.chroma_format_idc == 3 {
            encoder_formatted_print(
                "SPS: separate_colour_plane_flag",
                self.separate_colour_plane_flag,
                63,
            );
        }
        encoder_formatted_print("SPS: bit_depth_luma_minus8", self.bit_depth_luma_minus8, 63);
        encoder_formatted_print(
            "SPS: bit_depth_chroma_minus8",
            self.bit_depth_chroma_minus8,
            63,
        );
        encoder_formatted_print(
            "SPS: qpprime_y_zero_transform_bypass_flag",
            self.qpprime_y_zero_transform_bypass_flag,
            63,
        );
        encoder_formatted_print(
            "SPS: seq_scaling_matrix_present_flag",
            self.seq_scaling_matrix_present_flag,
            63,
        );
        if self.seq_scaling_matrix_present_flag {
            encoder_formatted_print(
                "SPS: seq_scaling_list_present_flag",
                self.seq_scaling_list_present_flag.clone(),
                63,
            );
            encoder_formatted_print("SPS: delta_scale_4x4", self.delta_scale_4x4.clone(), 63);
            encoder_formatted_print("SPS: scaling_list_4x4", self.scaling_list_4x4.clone(), 63);
            encoder_formatted_print("SPS: delta_scale_8x8", self.delta_scale_8x8.clone(), 63);
            encoder_formatted_print("SPS: scaling_list_8x8", self.scaling_list_8x8.clone(), 63);
            encoder_formatted_print(
                "SPS: use_default_scaling_matrix_4x4",
                self.use_default_scaling_matrix_4x4.clone(),
                63,
            );
            encoder_formatted_print(
                "SPS: use_default_scaling_matrix_8x8",
                self.use_default_scaling_matrix_8x8.clone(),
                63,
            );
        }
        encoder_formatted_print(
            "SPS: log2_max_frame_num_minus4",
            self.log2_max_frame_num_minus4,
            63,
        );
        encoder_formatted_print("SPS: pic_order_cnt_type", self.pic_order_cnt_type, 63);
        if self.pic_order_cnt_type == 0 {
            encoder_formatted_print(
                "SPS: log2_max_pic_order_cnt_lsb_minus4",
                self.log2_max_pic_order_cnt_lsb_minus4,
                63,
            );
        } else if self.pic_order_cnt_type == 1 {
            encoder_formatted_print(
                "SPS: delta_pic_order_always_zero_flag",
                self.delta_pic_order_always_zero_flag,
                63,
            );
            encoder_formatted_print(
                "SPS: offset_for_non_ref_pic",
                self.offset_for_non_ref_pic,
                63,
            );
            encoder_formatted_print(
                "SPS: offset_for_top_to_bottom_field",
                self.offset_for_top_to_bottom_field,
                63,
            );
            encoder_formatted_print(
                "SPS: num_ref_frames_in_pic_order_cnt_cycle",
                self.num_ref_frames_in_pic_order_cnt_cycle,
                63,
            );
            for i in 0..self.num_ref_frames_in_pic_order_cnt_cycle {
                encoder_formatted_print(
                    "SPS: offset_for_ref_frame,",
                    self.offset_for_ref_frame[i as usize],
                    63,
                );
            }
        }
        encoder_formatted_print("SPS: max_num_ref_frames", self.max_num_ref_frames, 63);
        encoder_formatted_print(
            "SPS: gaps_in_frame_num_value_allowed_flag",
            self.gaps_in_frame_num_value_allowed_flag,
            63,
        );
        encoder_formatted_print(
            "SPS: pic_width_in_mbs_minus1",
            self.pic_width_in_mbs_minus1,
            63,
        );
        encoder_formatted_print(
            "SPS: pic_height_in_map_units_minus1",
            self.pic_height_in_map_units_minus1,
            63,
        );
        encoder_formatted_print("SPS: frame_mbs_only_flag", self.frame_mbs_only_flag, 63);
        if !self.frame_mbs_only_flag {
            encoder_formatted_print(
                "SPS: mb_adaptive_frame_field_flag",
                self.mb_adaptive_frame_field_flag,
                63,
            );
        }

        encoder_formatted_print(
            "SPS: direct_8x8_inference_flag",
            self.direct_8x8_inference_flag,
            63,
        );
        encoder_formatted_print("SPS: frame_cropping_flag", self.frame_cropping_flag, 63);
        if self.frame_cropping_flag {
            encoder_formatted_print(
                "SPS: frame_crop_left_offset",
                self.frame_crop_left_offset,
                63,
            );
            encoder_formatted_print(
                "SPS: frame_crop_right_offset",
                self.frame_crop_right_offset,
                63,
            );
            encoder_formatted_print("SPS: frame_crop_top_offset", self.frame_crop_top_offset, 63);
            encoder_formatted_print(
                "SPS: frame_crop_bottom_offset",
                self.frame_crop_bottom_offset,
                63,
            );
        }

        encoder_formatted_print(
            "SPS: vui_parameters_present_flag",
            self.vui_parameters_present_flag,
            63,
        );
        if self.vui_parameters_present_flag {
            self.vui_parameters.encoder_pretty_print();
        }
    }

    pub fn get_framesize(&self) -> (i32, i32) {
        /*
        From the spec 7.4.2.1.1:
            The variables CropUnitX and CropUnitY are derived as follows:
            – If ChromaArrayType is equal to 0, CropUnitX and CropUnitY are derived as:
                    CropUnitX = 1 (7-19)
                    CropUnitY = 2 − frame_mbs_only_flag (7-20)
            – Otherwise (ChromaArrayType is equal to 1, 2, or 3), CropUnitX and CropUnitY are derived as:
                    CropUnitX = SubWidthC (7-21)
                    CropUnitY = SubHeightC * ( 2 − frame_mbs_only_flag ) (7-22)

            The frame cropping rectangle contains luma samples with horizontal frame coordinates from
            CropUnitX * frame_crop_left_offset to PicWidthInSamplesL − ( CropUnitX * frame_crop_right_offset + 1 ) and vertical
            frame coordinates from CropUnitY * frame_crop_top_offset to ( 16 * FrameHeightInMbs ) −
            ( CropUnitY * frame_crop_bottom_offset + 1 ), inclusive.

            The value of frame_crop_left_offset shall be in the range of 0 to ( PicWidthInSamplesL / CropUnitX ) − ( frame_crop_right_offset + 1 ), inclusive;
            and the value of frame_crop_top_offset shall be in the range of 0 to ( 16 * FrameHeightInMbs / CropUnitY ) − ( frame_crop_bottom_offset + 1 ), inclusive.

            When frame_cropping_flag is equal to 0, the values of frame_crop_left_offset, frame_crop_right_offset,
            frame_crop_top_offset, and frame_crop_bottom_offset shall be inferred to be equal to 0.

            When ChromaArrayType is not equal to 0, the corresponding specified samples of the two chroma arrays are the samples
            having frame coordinates ( x / SubWidthC, y / SubHeightC ), where ( x, y ) are the frame coordinates of the specified luma
            samples.

            For decoded fields, the specified samples of the decoded field are the samples that fall within the rectangle specified in
            frame coordinates.
        */

        let width;
        let height;
        // check for underflow
        if ((self.pic_width_in_mbs_minus1 as i64 + 1) * 16)
            - 2 * (self.frame_crop_left_offset as i64 + self.frame_crop_right_offset as i64)
            < 0
        {
            println!("\t\t [WARNING] Underflowing horizontal frame cropping - ignoring cropping in picture size calculation");
            width = (self.pic_width_in_mbs_minus1 + 1) * 16;
        } else {
            // check for overflow
            if 2 * (self.frame_crop_left_offset as i64 + self.frame_crop_right_offset as i64)
                > (std::i32::MAX as i64)
            {
                println!("\t\t [WARNING] Overflowing horizontal frame cropping - ignoring cropping in picture size calculation");
                width = (self.pic_width_in_mbs_minus1 + 1) * 16;
            } else {
                width = ((self.pic_width_in_mbs_minus1 + 1) * 16)
                    - 2 * (self.frame_crop_left_offset + self.frame_crop_right_offset);
            }
        }

        if ((self.pic_height_in_map_units_minus1 as i64 + 1) * 16)
            - 2 * (self.frame_crop_top_offset as i64 + self.frame_crop_bottom_offset as i64)
            < 0
        {
            println!("\t\t [WARNING] Underflowing vertical frame cropping - ignoring cropping in picture size calculation");
            height = (self.pic_height_in_map_units_minus1 + 1) * 16;
        } else {
            if 2 * (self.frame_crop_top_offset as i64 + self.frame_crop_bottom_offset as i64)
                > (std::i32::MAX as i64)
            {
                println!("\t\t [WARNING] Overflowing vertical frame cropping - ignoring cropping in picture size calculation");
                height = (self.pic_height_in_map_units_minus1 + 1) * 16;
            } else {
                height = ((self.pic_height_in_map_units_minus1 + 1) * 16)
                    - 2 * (self.frame_crop_top_offset + self.frame_crop_bottom_offset);
            }
        }

        (width as i32, height as i32)
    }
}

impl Default for SeqParameterSet {
    fn default() -> Self {
        Self::new()
    }
}

/// NALU Type 13 -- Sequence Parameter Set Extension
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SPSExtension {
    pub seq_parameter_set_id: u32,       // ue(v)
    pub aux_format_idc: u32,             // ue(v)
    pub bit_depth_aux_minus8: u32,       // ue(v)
    pub alpha_incr_flag: bool,           // u(1)
    pub alpha_opaque_value: u32,         // u(v)
    pub alpha_transparent_value: u32,    // u(v)
    pub additional_extension_flag: bool, // u(1)
}

impl SPSExtension {
    pub fn new() -> SPSExtension {
        SPSExtension {
            seq_parameter_set_id: 0,
            aux_format_idc: 0,
            bit_depth_aux_minus8: 0,
            alpha_incr_flag: false,
            alpha_opaque_value: 0,
            alpha_transparent_value: 0,
            additional_extension_flag: false,
        }
    }
}

impl Default for SPSExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// NALU Type 15 -- Subset Sequence Parameter Set
#[derive(Serialize, Deserialize, Clone)]
pub struct SubsetSPS {
    pub sps: SeqParameterSet,
    // SubsetSPS components
    pub sps_svc: SVCSPSExtension,
    pub svc_vui_parameters_present_flag: bool, // u(1)
    pub svc_vui: SVCVUIParameters,
    pub bit_equal_to_one: u8, // u(1)
    pub sps_mvc: MVCSPSExtension,
    pub mvc_vui_parameters_present_flag: bool, // u(1)
    pub mvc_vui: MVCVUIParameters,
    pub sps_mvcd: MVCDSPSExtension,
    pub sps_3davc: AVC3DSPSExtension,
    pub additional_extension2_flag: Vec<bool>,
}

impl SubsetSPS {
    pub fn new() -> SubsetSPS {
        SubsetSPS {
            sps: SeqParameterSet::new(),
            sps_svc: SVCSPSExtension::new(),
            svc_vui_parameters_present_flag: false,
            svc_vui: SVCVUIParameters::new(),
            bit_equal_to_one: 0,
            sps_mvc: MVCSPSExtension::new(),
            mvc_vui_parameters_present_flag: false,
            mvc_vui: MVCVUIParameters::new(),
            sps_mvcd: MVCDSPSExtension::new(),
            sps_3davc: AVC3DSPSExtension::new(),
            additional_extension2_flag: Vec::new(),
        }
    }

    pub fn encoder_pretty_print(&self) {
        self.sps.encoder_pretty_print();

        self.sps_svc.encoder_pretty_print();
        encoder_formatted_print(
            "Subset SPS: svc_vui_parameters_present_flag",
            self.svc_vui_parameters_present_flag,
            63,
        );
        if self.svc_vui_parameters_present_flag {
            self.svc_vui.encoder_pretty_print();
        }
        encoder_formatted_print("Subset SPS: bit_equal_to_one", self.bit_equal_to_one, 63);
        self.sps_mvc.encoder_pretty_print();
        encoder_formatted_print(
            "Subset SPS: mvc_vui_parameters_present_flag",
            self.mvc_vui_parameters_present_flag,
            63,
        );
        if self.mvc_vui_parameters_present_flag {
            self.mvc_vui.encoder_pretty_print();
        }
        self.sps_mvcd.encoder_pretty_print();
        self.sps_3davc.encoder_pretty_print();

        encoder_formatted_print(
            "Subset SPS: additional_extension2_flag",
            self.additional_extension2_flag.clone(),
            63,
        );
    }
}

impl Default for SubsetSPS {
    fn default() -> Self {
        Self::new()
    }
}

/// AVCC Output Format
#[derive(Serialize, Deserialize)]
pub struct AVCCFormat {
    pub initial_sps: SeqParameterSet,
    pub sps_list: Vec<Vec<u8>>,
    pub pps_list: Vec<Vec<u8>>,
    pub nalus: Vec<Vec<u8>>,
}

impl AVCCFormat {
    pub fn new() -> AVCCFormat {
        AVCCFormat {
            initial_sps: SeqParameterSet::new(),
            sps_list: Vec::new(),
            pps_list: Vec::new(),
            nalus: Vec::new(),
        }
    }
}

impl Default for AVCCFormat {
    fn default() -> Self {
        Self::new()
    }
}

/// SEI Type 0; Described in Annex D.2.2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SEIBufferingPeriod {
    pub seq_parameter_set_id: u32,                      //  ue(v)
    pub nal_initial_cpb_removal_delay: Vec<u32>, //  vector of length cpb_cnt_minus1 from HRD parameters; bit length is initial_cpb_removal_delay_length_minus1 + 1
    pub nal_initial_cpb_removal_delay_offset: Vec<u32>, //  vector of length cpb_cnt_minus1 from HRD parameters; bit length is initial_cpb_removal_delay_length_minus1 + 1
    pub vcl_initial_cpb_removal_delay: Vec<u32>, //  vector of length cpb_cnt_minus1 from HRD parameters; bit length is initial_cpb_removal_delay_length_minus1 + 1
    pub vcl_initial_cpb_removal_delay_offset: Vec<u32>, //  vector of length cpb_cnt_minus1 from HRD parameters; bit length is initial_cpb_removal_delay_length_minus1 + 1
}

impl SEIBufferingPeriod {
    pub fn new() -> SEIBufferingPeriod {
        SEIBufferingPeriod {
            seq_parameter_set_id: 0,
            nal_initial_cpb_removal_delay: Vec::new(),
            nal_initial_cpb_removal_delay_offset: Vec::new(),
            vcl_initial_cpb_removal_delay: Vec::new(),
            vcl_initial_cpb_removal_delay_offset: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print(
            "Buffering Period SEI: seq_parameter_set_id",
            self.seq_parameter_set_id,
            63,
        );
        encoder_formatted_print(
            "Buffering Period SEI: NAL initial_cpb_removal_delay",
            self.nal_initial_cpb_removal_delay.clone(),
            63,
        );
        encoder_formatted_print(
            "Buffering Period SEI: NAL initial_cpb_removal_delay_offset",
            self.nal_initial_cpb_removal_delay_offset.clone(),
            63,
        );
        encoder_formatted_print(
            "Buffering Period SEI: VCL initial_cpb_removal_delay",
            self.vcl_initial_cpb_removal_delay.clone(),
            63,
        );
        encoder_formatted_print(
            "Buffering Period SEI: VCL initial_cpb_removal_delay_offset",
            self.vcl_initial_cpb_removal_delay_offset.clone(),
            63,
        );
    }
}

impl Default for SEIBufferingPeriod {
    fn default() -> Self {
        Self::new()
    }
}

/// SEI Type 1; Described in Annex D.2.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SEIPicTiming {
    pub cpb_removal_delay: u32,
    pub dpb_output_delay: u32,
    pub pic_struct: u32, // u(4)
    // vec length is NumClockTS
    pub clock_timestamp_flag: Vec<bool>,  // u(1)
    pub ct_type: Vec<u32>,                // u(2)
    pub nuit_field_based_flag: Vec<bool>, // u(1)
    pub counting_type: Vec<u32>,          // u(5)
    pub full_timestamp_flag: Vec<bool>,   // u(1)
    pub discontinuity_flag: Vec<bool>,    // u(1)
    pub cnt_dropped_flag: Vec<bool>,      // u(1)
    pub n_frames: Vec<u32>,               // u(8)
    pub seconds_value: Vec<u32>,          // u(6), range: 0..59
    pub minutes_value: Vec<u32>,          // u(6), range: 0..59
    pub hours_value: Vec<u32>,            // u(5), range: 0..23
    pub seconds_flag: Vec<bool>,          // u(1)
    pub minutes_flag: Vec<bool>,          // u(1)
    pub hours_flag: Vec<bool>,            // u(1)
    pub time_offset: Vec<u32>,            // i(v)
}

impl SEIPicTiming {
    pub fn new() -> SEIPicTiming {
        SEIPicTiming {
            cpb_removal_delay: 0,
            dpb_output_delay: 0,
            pic_struct: 0,
            clock_timestamp_flag: Vec::new(),
            ct_type: Vec::new(),
            nuit_field_based_flag: Vec::new(),
            counting_type: Vec::new(),
            full_timestamp_flag: Vec::new(),
            discontinuity_flag: Vec::new(),
            cnt_dropped_flag: Vec::new(),
            n_frames: Vec::new(),
            seconds_value: Vec::new(),
            minutes_value: Vec::new(),
            hours_value: Vec::new(),
            seconds_flag: Vec::new(),
            minutes_flag: Vec::new(),
            hours_flag: Vec::new(),
            time_offset: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn encoder_pretty_print(&self) {}
}

impl Default for SEIPicTiming {
    fn default() -> Self {
        Self::new()
    }
}

/// Number of known UUIDs
pub const KNOWN_UUIDS: u32 = 3;
/// Unknown Apple 1
pub const UUID_APPLE1: [u8; 16] = [
    0x03, 0x87, 0xF4, 0x4E, 0xCD, 0x0A, 0x4B, 0xDC, 0xA1, 0x94, 0x3A, 0xC3, 0xD4, 0x9B, 0x17, 0x1F,
];
/// Unknown Apple 2
pub const UUID_APPLE2: [u8; 16] = [
    0x47, 0x56, 0x4A, 0xDC, 0x5C, 0x4C, 0x43, 0x3F, 0x94, 0xEF, 0xC5, 0x11, 0x3C, 0xD1, 0x43, 0xA8,
];
/// Found in H264H8.videodecoder
pub const UUID_APPLE3: [u8; 16] = [
    0x23, 0xF2, 0x8D, 0xDC, 0xE2, 0xC3, 0x46, 0x56, 0xBC, 0x51, 0x57, 0xA5, 0x1C, 0xDE, 0x4F, 0xDE,
];
// maybe I have the endian-ness wrong and it's 5646c3e2dc8df223 and de4fde1ca55751bc: 5646c3e2dc8df223de4fde1ca55751bc

/// UUID: 0x0387F44ECD0A4BDCA1943AC3D49B171F (recovered from AppleD5500.kext)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SEIUnregisteredDataApple1 {
    pub mystery_param1: u32, // u(8)
                             // if mystery_param1 is less than 4 then it derives another parameter
}

impl SEIUnregisteredDataApple1 {
    pub fn new() -> SEIUnregisteredDataApple1 {
        SEIUnregisteredDataApple1 { mystery_param1: 0 }
    }

    #[allow(dead_code)]
    pub fn encoder_pretty_print(&self) {}
}

impl Default for SEIUnregisteredDataApple1 {
    fn default() -> Self {
        Self::new()
    }
}

/// UUID: 0x47564ADC5C4C433F94EFC5113CD143A8 (recovered from AppleD5500.kext)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SEIUnregisteredDataApple2 {
    pub mystery_param1: u32, // u(8)
    pub mystery_param2: u32, // u(8)
    pub mystery_param3: u32, // u(8)
    pub mystery_param4: u32, // u(8)
    pub mystery_param5: u32, // u(8)
    pub mystery_param6: u32, // u(8)
    pub mystery_param7: u32, // u(8)
    pub mystery_param8: u32, // u(8)
}

impl SEIUnregisteredDataApple2 {
    pub fn new() -> SEIUnregisteredDataApple2 {
        SEIUnregisteredDataApple2 {
            mystery_param1: 0,
            mystery_param2: 0,
            mystery_param3: 0,
            mystery_param4: 0,
            mystery_param5: 0,
            mystery_param6: 0,
            mystery_param7: 0,
            mystery_param8: 0,
        }
    }

    #[allow(dead_code)]
    pub fn encoder_pretty_print(&self) {}
}

impl Default for SEIUnregisteredDataApple2 {
    fn default() -> Self {
        Self::new()
    }
}

/// SEI Type 5; Described in Annex D.2.7
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SEIUserDataUnregistered {
    pub uuid_iso_iec_11578: [u8; 16], // u(128)
    pub user_data_apple1: SEIUnregisteredDataApple1,
    pub user_data_apple2: SEIUnregisteredDataApple2,
    pub user_data_payload_byte: Vec<u8>, // default for unknown UUIDs
}

impl SEIUserDataUnregistered {
    pub fn new() -> SEIUserDataUnregistered {
        SEIUserDataUnregistered {
            uuid_iso_iec_11578: [0; 16],
            user_data_apple1: SEIUnregisteredDataApple1::new(),
            user_data_apple2: SEIUnregisteredDataApple2::new(),
            user_data_payload_byte: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn encoder_pretty_print(&self) {}
}

impl Default for SEIUserDataUnregistered {
    fn default() -> Self {
        Self::new()
    }
}

/// SEI Type 6; Described in Annex D.2.8
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SEIRecoveryPoint {
    pub recovery_frame_cnt: u32, // ue(v)
    pub exact_match_flag: bool,
    pub broken_link_flag: bool,
    pub changing_slice_group_idc: u8, // u(2)
}

impl SEIRecoveryPoint {
    pub fn new() -> SEIRecoveryPoint {
        SEIRecoveryPoint {
            recovery_frame_cnt: 0,
            exact_match_flag: false,
            broken_link_flag: false,
            changing_slice_group_idc: 0,
        }
    }

    #[allow(dead_code)]
    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print(
            "SEI (Recovery Point): recovery_frame_cnt",
            self.recovery_frame_cnt,
            63,
        );
        encoder_formatted_print(
            "SEI (Recovery Point): exact_match_flag",
            self.exact_match_flag,
            63,
        );
        encoder_formatted_print(
            "SEI (Recovery Point): broken_link_flag",
            self.broken_link_flag,
            63,
        );
        encoder_formatted_print(
            "SEI (Recovery Point): changing_slice_group_idc",
            self.changing_slice_group_idc,
            63,
        );
    }
}

impl Default for SEIRecoveryPoint {
    fn default() -> Self {
        Self::new()
    }
}

/// SEI Type 19; Described in Annex D.2.21
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SEIFilmGrainCharacteristics {
    pub film_grain_characteristics_cancel_flag: bool,
    pub film_grain_model_id: u8, // u(2)
    pub separate_colour_description_present_flag: bool,
    pub film_grain_bit_depth_luma_minus8: u8,   // u(3)
    pub film_grain_bit_depth_chroma_minus8: u8, // u(3)
    pub film_grain_full_range_flag: bool,
    pub film_grain_colour_primaries: u8,         // u(8)
    pub film_grain_transfer_characteristics: u8, // u(8)
    pub film_grain_matrix_coefficients: u8,      // u(8)
    pub blending_mode_id: u8,                    // u(2)
    pub log2_scale_factor: u8,                   // u(4)
    pub comp_model_present_flag: Vec<bool>,
    pub num_intensity_intervals_minus1: Vec<u8>, // u(8)
    pub num_model_values_minus1: Vec<u8>,        // u(3)
    pub intensity_interval_lower_bound: Vec<Vec<u8>>, // u(8)
    pub intensity_interval_upper_bound: Vec<Vec<u8>>, // u(8)
    pub comp_model_value: Vec<Vec<Vec<i32>>>,    // se(v)
    pub film_grain_characteristics_repetition_period: u32, // ue(v)
}

impl SEIFilmGrainCharacteristics {
    pub fn new() -> SEIFilmGrainCharacteristics {
        SEIFilmGrainCharacteristics {
            film_grain_characteristics_cancel_flag: false,
            film_grain_model_id: 0,
            separate_colour_description_present_flag: false,
            film_grain_bit_depth_luma_minus8: 0,
            film_grain_bit_depth_chroma_minus8: 0,
            film_grain_full_range_flag: false,
            film_grain_colour_primaries: 0,
            film_grain_transfer_characteristics: 0,
            film_grain_matrix_coefficients: 0,
            blending_mode_id: 0,
            log2_scale_factor: 0,
            comp_model_present_flag: Vec::new(),
            num_intensity_intervals_minus1: Vec::new(),
            num_model_values_minus1: Vec::new(),
            intensity_interval_lower_bound: Vec::new(),
            intensity_interval_upper_bound: Vec::new(),
            comp_model_value: Vec::new(),
            film_grain_characteristics_repetition_period: 0,
        }
    }

    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print(
            "SEI (Film Grain Char) film_grain_characteristics_cancel_flag",
            self.film_grain_characteristics_cancel_flag,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) film_grain_model_id",
            self.film_grain_model_id,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) separate_colour_description_present_flag",
            self.separate_colour_description_present_flag,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) film_grain_bit_depth_luma_minus8",
            self.film_grain_bit_depth_luma_minus8,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) film_grain_bit_depth_chroma_minus8",
            self.film_grain_bit_depth_chroma_minus8,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) film_grain_full_range_flag",
            self.film_grain_full_range_flag,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) film_grain_colour_primaries",
            self.film_grain_colour_primaries,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) film_grain_transfer_characteristics",
            self.film_grain_transfer_characteristics,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) film_grain_matrix_coefficients",
            self.film_grain_matrix_coefficients,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) blending_mode_id",
            self.blending_mode_id,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) log2_scale_factor",
            self.log2_scale_factor,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) comp_model_present_flag",
            &self.comp_model_present_flag,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) num_intensity_intervals_minus1",
            &self.num_intensity_intervals_minus1,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) num_model_values_minus1",
            &self.num_model_values_minus1,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) intensity_interval_lower_bound",
            &self.intensity_interval_lower_bound,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) intensity_interval_upper_bound",
            &self.intensity_interval_upper_bound,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) comp_model_value",
            &self.comp_model_value,
            63,
        );
        encoder_formatted_print(
            "SEI (Film Grain Char) film_grain_characteristics_repetition_period",
            self.film_grain_characteristics_repetition_period,
            63,
        );
    }
}

impl Default for SEIFilmGrainCharacteristics {
    fn default() -> Self {
        Self::new()
    }
}

/// SEI Type 45; Described in Annex D.2.26
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SEIFramePacking {}

impl SEIFramePacking {
    pub fn new() -> SEIFramePacking {
        SEIFramePacking {}
    }

    #[allow(dead_code)]
    pub fn encoder_pretty_print(&self) {}
}

impl Default for SEIFramePacking {
    fn default() -> Self {
        Self::new()
    }
}

/// SEI Payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SEIPayload {
    pub available: bool,                      // Enabled if the SEI has been parsed
    pub buffering_period: SEIBufferingPeriod, // SEI type 0
    pub pic_timing: SEIPicTiming,             // SEI type 1
    pub unregistered_user_data: SEIUserDataUnregistered, // SEI type 5
    pub recovery_point: SEIRecoveryPoint,     // SEI type 6
    pub film_grain_characteristics: SEIFilmGrainCharacteristics, // SEI Type 19
    pub frame_packing: SEIFramePacking,       // SEI type 45
}

impl SEIPayload {
    pub fn new() -> SEIPayload {
        SEIPayload {
            available: false,
            buffering_period: SEIBufferingPeriod::new(),
            pic_timing: SEIPicTiming::new(),
            unregistered_user_data: SEIUserDataUnregistered::new(),
            recovery_point: SEIRecoveryPoint::new(),
            film_grain_characteristics: SEIFilmGrainCharacteristics::new(),
            frame_packing: SEIFramePacking::new(),
        }
    }
}

/// NALU Type 6 -- SEI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SEINalu {
    // Each SEI can have multiple types so keep one data structure for each NALU SEI
    pub payload_type: Vec<u32>,
    pub payload_size: Vec<u32>,
    // embed specific SEI type into here
    pub payload: Vec<SEIPayload>,
}

impl SEINalu {
    pub fn new() -> SEINalu {
        SEINalu {
            payload_type: Vec::new(),
            payload_size: Vec::new(),
            payload: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn encoder_pretty_print(&self) {}
}

impl Default for SEINalu {
    fn default() -> Self {
        Self::new()
    }
}

/// NALU Type 9 -- AUD
///
/// NALU Type 9 used to indicate the type of slices present in a primary
/// coded picture and used by decoder to simplify the detection of the
/// boundary between access units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessUnitDelim {
    // Table 7-5 – Meaning of primary_pic_type
    // primary_pic_type    | slice_type values that may be present in the primary coded picture
    // ------------------------------------------------------------------------------------------
    //       0             | 2, 7
    //       1             | 0, 2, 5, 7
    //       2             | 0, 1, 2, 5, 6, 7
    //       3             | 4, 9
    //       4             | 3, 4, 8, 9
    //       5             | 2, 4, 7, 9
    //       6             | 0, 2, 3, 4, 5, 7, 8, 9
    //       7             | 0, 1, 2, 3, 4, 5, 6, 7, 8, 9
    pub primary_pic_type: u8, // u(3)
}

impl AccessUnitDelim {
    pub fn new() -> AccessUnitDelim {
        AccessUnitDelim {
            primary_pic_type: 0,
        }
    }

    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print("AUD: primary_pic_type", self.primary_pic_type, 63);
    }
}

impl Default for AccessUnitDelim {
    fn default() -> Self {
        Self::new()
    }
}
