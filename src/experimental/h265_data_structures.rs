//! H.265 Data structures.

use crate::common::data_structures::NALU;
use crate::common::helper::{decoder_formatted_print, encoder_formatted_print};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum NalUnitType {
    NalUnitCodedSliceTrailN, // 0
    NalUnitCodedSliceTrailR, // 1
    NalUnitCodedSliceTsaN,   // 2
    NalUnitCodedSliceTsaR,   // 3
    NalUnitCodedSliceStsaN,  // 4
    NalUnitCodedSliceStsaR,  // 5
    NalUnitCodedSliceRadlN,  // 6
    NalUnitCodedSliceRadlR,  // 7
    NalUnitCodedSliceRaslN,  // 8
    NalUnitCodedSliceRaslR,  // 9
    NalUnitReservedVclN10,
    NalUnitReservedVclR11,
    NalUnitReservedVclN12,
    NalUnitReservedVclR13,
    NalUnitReservedVclN14,
    NalUnitCodedSliceBlaWLp,
    NalUnitReservedVclR15,     // 16
    NalUnitCodedSliceBlaWRadl, // 17
    NalUnitCodedSliceBlaNLp,   // 18
    NalUnitCodedSliceIdrWRadl, // 19
    NalUnitCodedSliceIdrNLp,   // 20
    NalUnitCodedSliceCra,      // 21
    NalUnitReservedIrapVcl22,
    NalUnitReservedIrapVcl23,
    NalUnitReservedVcl24,
    NalUnitReservedVcl25,
    NalUnitReservedVcl26,
    NalUnitReservedVcl27,
    NalUnitReservedVcl28,
    NalUnitReservedVcl29,
    NalUnitReservedVcl30,
    NalUnitReservedVcl31,
    NalUnitVps,                 // 32
    NalUnitSps,                 // 33
    NalUnitPps,                 // 34
    NalUnitAccessUnitDelimiter, // 35
    NalUnitEos,                 // 36
    NalUnitEob,                 // 37
    NalUnitFillerData,          // 38
    NalUnitPrefixSei,           // 39
    NalUnitSuffixSei,           // 40
    NalUnitReservedNvcl41,
    NalUnitReservedNvcl42,
    NalUnitReservedNvcl43,
    NalUnitReservedNvcl44,
    NalUnitReservedNvcl45,
    NalUnitReservedNvcl46,
    NalUnitReservedNvcl47,
    NalUnitUnspecified48,
    NalUnitUnspecified49,
    NalUnitUnspecified50,
    NalUnitUnspecified51,
    NalUnitUnspecified52,
    NalUnitUnspecified53,
    NalUnitUnspecified54,
    NalUnitUnspecified55,
    NalUnitUnspecified56,
    NalUnitUnspecified57,
    NalUnitUnspecified58,
    NalUnitUnspecified59,
    NalUnitUnspecified60,
    NalUnitUnspecified61,
    NalUnitUnspecified62,
    NalUnitUnspecified63,
    NalUnitInvalid,
}

impl NalUnitType {
    pub fn from(val : u32) -> NalUnitType {
        match val {
            0 => return NalUnitType::NalUnitCodedSliceTrailN, // 0
            1 => return NalUnitType::NalUnitCodedSliceTrailR, // 1
            2 => return NalUnitType::NalUnitCodedSliceTsaN,   // 2
            3 => return NalUnitType::NalUnitCodedSliceTsaR,   // 3
            4 => return NalUnitType::NalUnitCodedSliceStsaN,  // 4
            5 => return NalUnitType::NalUnitCodedSliceStsaR,  // 5
            6 => return NalUnitType::NalUnitCodedSliceRadlN,  // 6
            7 => return NalUnitType::NalUnitCodedSliceRadlR,  // 7
            8 => return NalUnitType::NalUnitCodedSliceRaslN,  // 8
            9 => return NalUnitType::NalUnitCodedSliceRaslR,  // 9
           10 => return NalUnitType::NalUnitReservedVclN10,
           11 => return NalUnitType::NalUnitReservedVclR11,
           12 => return NalUnitType::NalUnitReservedVclN12,
           13 => return NalUnitType::NalUnitReservedVclR13,
           14 => return NalUnitType::NalUnitReservedVclN14,
           15 => return NalUnitType::NalUnitCodedSliceBlaWLp,
           16 => return NalUnitType::NalUnitReservedVclR15,     // 16
           17 => return NalUnitType::NalUnitCodedSliceBlaWRadl, // 17
           18 => return NalUnitType::NalUnitCodedSliceBlaNLp,   // 18
           19 => return NalUnitType::NalUnitCodedSliceIdrWRadl, // 19
           20 => return NalUnitType::NalUnitCodedSliceIdrNLp,   // 20
           21 => return NalUnitType::NalUnitCodedSliceCra,      // 21
           22 => return NalUnitType::NalUnitReservedIrapVcl22,
           23 => return NalUnitType::NalUnitReservedIrapVcl23,
           24 => return NalUnitType::NalUnitReservedVcl24,
           25 => return NalUnitType::NalUnitReservedVcl25,
           26 => return NalUnitType::NalUnitReservedVcl26,
           27 => return NalUnitType::NalUnitReservedVcl27,
           28 => return NalUnitType::NalUnitReservedVcl28,
           29 => return NalUnitType::NalUnitReservedVcl29,
           30 => return NalUnitType::NalUnitReservedVcl30,
           31 => return NalUnitType::NalUnitReservedVcl31,
           32 => return NalUnitType::NalUnitVps,                 // 32
           33 => return NalUnitType::NalUnitSps,                 // 33
           34 => return NalUnitType::NalUnitPps,                 // 34
           35 => return NalUnitType::NalUnitAccessUnitDelimiter, // 35
           36 => return NalUnitType::NalUnitEos,                 // 36
           37 => return NalUnitType::NalUnitEob,                 // 37
           38 => return NalUnitType::NalUnitFillerData,          // 38
           39 => return NalUnitType::NalUnitPrefixSei,           // 39
           40 => return NalUnitType::NalUnitSuffixSei,           // 40
           41 => return NalUnitType::NalUnitReservedNvcl41,
           42 => return NalUnitType::NalUnitReservedNvcl42,
           43 => return NalUnitType::NalUnitReservedNvcl43,
           44 => return NalUnitType::NalUnitReservedNvcl44,
           45 => return NalUnitType::NalUnitReservedNvcl45,
           46 => return NalUnitType::NalUnitReservedNvcl46,
           47 => return NalUnitType::NalUnitReservedNvcl47,
           48 => return NalUnitType::NalUnitUnspecified48,
           49 => return NalUnitType::NalUnitUnspecified49,
           50 => return NalUnitType::NalUnitUnspecified50,
           51 => return NalUnitType::NalUnitUnspecified51,
           52 => return NalUnitType::NalUnitUnspecified52,
           53 => return NalUnitType::NalUnitUnspecified53,
           54 => return NalUnitType::NalUnitUnspecified54,
           55 => return NalUnitType::NalUnitUnspecified55,
           56 => return NalUnitType::NalUnitUnspecified56,
           57 => return NalUnitType::NalUnitUnspecified57,
           58 => return NalUnitType::NalUnitUnspecified58,
           59 => return NalUnitType::NalUnitUnspecified59,
           60 => return NalUnitType::NalUnitUnspecified60,
           61 => return NalUnitType::NalUnitUnspecified61,
           62 => return NalUnitType::NalUnitUnspecified62,
           63 => return NalUnitType::NalUnitUnspecified63,
            _ => return NalUnitType::NalUnitInvalid,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H265NALUHeader {
    pub forbidden_zero_bit: u8,
    pub nal_unit_type: NalUnitType,
    pub nuh_layer_id: u8,
    pub nuh_temporal_id_plus1: u8,
}

impl H265NALUHeader {
    pub fn new() -> H265NALUHeader {
        H265NALUHeader {
            forbidden_zero_bit: 0,
            nal_unit_type: NalUnitType::NalUnitInvalid,
            nuh_layer_id: 0,
            nuh_temporal_id_plus1: 0,
        }
    }
}

impl Default for H265NALUHeader {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize)]
pub struct H265DecodedStream {
    pub nalu_elements: Vec<NALU>,
    pub nalu_headers: Vec<H265NALUHeader>,
    pub vpses: Vec<H265VideoParameterSet>,
    pub spses: Vec<H265SeqParameterSet>,
    pub ppses: Vec<H265PicParameterSet>,
}

impl H265DecodedStream {
    pub fn new() -> H265DecodedStream {
        H265DecodedStream {
            nalu_elements: Vec::new(),
            nalu_headers: Vec::new(),
            vpses: Vec::new(),
            spses: Vec::new(),
            ppses: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn clone(&self) -> H265DecodedStream {
        H265DecodedStream {
            nalu_elements: self.nalu_elements.clone(),
            nalu_headers: self.nalu_headers.clone(),
            vpses: self.vpses.clone(),
            spses: self.spses.clone(),
            ppses: self.ppses.clone(),
        }
    }
}

impl Default for H265DecodedStream {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct H265SeqParameterSet {
    pub sps_video_parameter_set_id: u8, // u(4)
    pub sps_max_sub_layers_minus1: u8,  // u(3)
    pub sps_temporal_id_nesting_flag: bool,
    pub profile_tier_level: ProfileTierLevel,
    pub sps_seq_parameter_set_id: u32,
    pub chroma_format_idc: u32,
    pub separate_colour_plane_flag: bool,
    pub pic_width_in_luma_samples: u32,
    pub pic_height_in_luma_samples: u32,
    pub conformance_window_flag: bool,
    pub conf_win_left_offset: u32,
    pub conf_win_right_offset: u32,
    pub conf_win_top_offset: u32,
    pub conf_win_bottom_offset: u32,
    pub bit_depth_luma_minus8: u32,
    pub bit_depth_chroma_minus8: u32,
    pub log2_max_pic_order_cnt_lsb_minus4: u32,
    pub sps_sub_layer_ordering_info_present_flag: bool,
    pub sps_max_dec_pic_buffering_minus1: Vec<u32>,
    pub sps_max_num_reorder_pics: Vec<u32>,
    pub sps_max_latency_increase_plus1: Vec<u32>,
    pub log2_min_luma_coding_block_size_minus3: u32,
    pub log2_diff_max_min_luma_coding_block_size: u32,
    pub log2_min_luma_transform_block_size_minus2: u32,
    pub log2_diff_max_min_luma_transform_block_size: u32,
    pub max_transform_hierarchy_depth_inter: u32,
    pub max_transform_hierarchy_depth_intra: u32,
    pub scaling_list_enabled_flag: bool,
    pub sps_scaling_list_data_present_flag: bool,
    pub scaling_list_data: ScalingListData,
    pub amp_enabled_flag: bool,
    pub sample_adaptive_offset_enabled_flag: bool,
    pub pcm_enabled_flag: bool,
    pub pcm_sample_bit_depth_luma_minus1: u8,   //u(4)
    pub pcm_sample_bit_depth_chroma_minus1: u8, //u(4)
    pub log2_min_pcm_luma_coding_block_size_minus3: u32,
    pub log2_diff_max_min_pcm_luma_coding_block_size: u32,
    pub pcm_loop_filter_disabled_flag: bool,
    pub num_short_term_ref_pic_sets: u32,
    pub st_ref_pic_set: Vec<ShortTermRefPic>,
    pub long_term_ref_pics_present_flag: bool,
    pub num_long_term_ref_pics_sps: u32,
    pub lt_ref_pic_poc_lsb_sps: Vec<u32>, // u(v)
    pub used_by_curr_pic_lt_sps_flag: Vec<bool>,
    pub sps_temporal_mvp_enabled_flag: bool,
    pub strong_intra_smoothing_enabled_flag: bool,
    pub vui_parameters_present_flag: bool,
    pub vui_parameters: H265VuiParameters,
    pub sps_extension_present_flag: bool,
    pub sps_range_extension_flag: bool,
    pub sps_multilayer_extension_flag: bool,
    pub sps_3d_extension_flag: bool,
    pub sps_scc_extension_flag: bool,
    pub sps_extension_4bits: u8, //u(4)
    pub sps_range_extension: H265SPSRangeExtension,
    pub sps_multilayer_extension: H265SPSMultilayerExtension,
    pub sps_3d_extension: H265SPS3DExtension,
    pub sps_scc_extension: H265SPSSCCExtension,
    pub sps_extension_data_flag: bool,
}

impl H265SeqParameterSet {
    pub fn new() -> H265SeqParameterSet {
        H265SeqParameterSet {
            sps_video_parameter_set_id: 0,
            sps_max_sub_layers_minus1: 0,
            sps_temporal_id_nesting_flag: false,
            profile_tier_level: ProfileTierLevel::new(),
            sps_seq_parameter_set_id: 0,
            chroma_format_idc: 0,
            separate_colour_plane_flag: false,
            pic_width_in_luma_samples: 0,
            pic_height_in_luma_samples: 0,
            conformance_window_flag: false,
            conf_win_left_offset: 0,
            conf_win_right_offset: 0,
            conf_win_top_offset: 0,
            conf_win_bottom_offset: 0,
            bit_depth_luma_minus8: 0,
            bit_depth_chroma_minus8: 0,
            log2_max_pic_order_cnt_lsb_minus4: 0,
            sps_sub_layer_ordering_info_present_flag: false,
            sps_max_dec_pic_buffering_minus1: Vec::new(),
            sps_max_num_reorder_pics: Vec::new(),
            sps_max_latency_increase_plus1: Vec::new(),
            log2_min_luma_coding_block_size_minus3: 0,
            log2_diff_max_min_luma_coding_block_size: 0,
            log2_min_luma_transform_block_size_minus2: 0,
            log2_diff_max_min_luma_transform_block_size: 0,
            max_transform_hierarchy_depth_inter: 0,
            max_transform_hierarchy_depth_intra: 0,
            scaling_list_enabled_flag: false,
            sps_scaling_list_data_present_flag: false,
            scaling_list_data: ScalingListData::new(),
            amp_enabled_flag: false,
            sample_adaptive_offset_enabled_flag: false,
            pcm_enabled_flag: false,
            pcm_sample_bit_depth_luma_minus1: 0,
            pcm_sample_bit_depth_chroma_minus1: 0,
            log2_min_pcm_luma_coding_block_size_minus3: 0,
            log2_diff_max_min_pcm_luma_coding_block_size: 0,
            pcm_loop_filter_disabled_flag: false,
            num_short_term_ref_pic_sets: 0,
            st_ref_pic_set: Vec::new(),
            long_term_ref_pics_present_flag: false,
            num_long_term_ref_pics_sps: 0,
            lt_ref_pic_poc_lsb_sps: Vec::new(),
            used_by_curr_pic_lt_sps_flag: Vec::new(),
            sps_temporal_mvp_enabled_flag: false,
            strong_intra_smoothing_enabled_flag: false,
            vui_parameters_present_flag: false,
            vui_parameters: H265VuiParameters::new(),
            sps_extension_present_flag: false,
            sps_range_extension_flag: false,
            sps_multilayer_extension_flag: false,
            sps_3d_extension_flag: false,
            sps_scc_extension_flag: false,
            sps_extension_4bits: 0,
            sps_range_extension: H265SPSRangeExtension::new(),
            sps_multilayer_extension: H265SPSMultilayerExtension::new(),
            sps_3d_extension: H265SPS3DExtension::new(),
            sps_scc_extension: H265SPSSCCExtension::new(),
            sps_extension_data_flag: false,
        }
    }

    #[allow(dead_code)]
    pub fn clone(&self) -> H265SeqParameterSet {
        H265SeqParameterSet {
            sps_video_parameter_set_id: self.sps_video_parameter_set_id,
            sps_max_sub_layers_minus1: self.sps_max_sub_layers_minus1,
            sps_temporal_id_nesting_flag: self.sps_temporal_id_nesting_flag,
            profile_tier_level: self.profile_tier_level.clone(),
            sps_seq_parameter_set_id: self.sps_seq_parameter_set_id,
            chroma_format_idc: self.chroma_format_idc,
            separate_colour_plane_flag: self.separate_colour_plane_flag,
            pic_width_in_luma_samples: self.pic_width_in_luma_samples,
            pic_height_in_luma_samples: self.pic_height_in_luma_samples,
            conformance_window_flag: self.conformance_window_flag,
            conf_win_left_offset: self.conf_win_left_offset,
            conf_win_right_offset: self.conf_win_right_offset,
            conf_win_top_offset: self.conf_win_top_offset,
            conf_win_bottom_offset: self.conf_win_bottom_offset,
            bit_depth_luma_minus8: self.bit_depth_luma_minus8,
            bit_depth_chroma_minus8: self.bit_depth_chroma_minus8,
            log2_max_pic_order_cnt_lsb_minus4: self.log2_max_pic_order_cnt_lsb_minus4,
            sps_sub_layer_ordering_info_present_flag: self.sps_sub_layer_ordering_info_present_flag,
            sps_max_dec_pic_buffering_minus1: self.sps_max_dec_pic_buffering_minus1.clone(),
            sps_max_num_reorder_pics: self.sps_max_num_reorder_pics.clone(),
            sps_max_latency_increase_plus1: self.sps_max_latency_increase_plus1.clone(),
            log2_min_luma_coding_block_size_minus3: self.log2_min_luma_coding_block_size_minus3,
            log2_diff_max_min_luma_coding_block_size: self.log2_diff_max_min_luma_coding_block_size,
            log2_min_luma_transform_block_size_minus2: self
                .log2_min_luma_transform_block_size_minus2,
            log2_diff_max_min_luma_transform_block_size: self
                .log2_diff_max_min_luma_transform_block_size,
            max_transform_hierarchy_depth_inter: self.max_transform_hierarchy_depth_inter,
            max_transform_hierarchy_depth_intra: self.max_transform_hierarchy_depth_intra,
            scaling_list_enabled_flag: self.scaling_list_enabled_flag,
            sps_scaling_list_data_present_flag: self.sps_scaling_list_data_present_flag,
            scaling_list_data: self.scaling_list_data.clone(),
            amp_enabled_flag: self.amp_enabled_flag,
            sample_adaptive_offset_enabled_flag: self.sample_adaptive_offset_enabled_flag,
            pcm_enabled_flag: self.pcm_enabled_flag,
            pcm_sample_bit_depth_luma_minus1: self.pcm_sample_bit_depth_luma_minus1,
            pcm_sample_bit_depth_chroma_minus1: self.pcm_sample_bit_depth_chroma_minus1,
            log2_min_pcm_luma_coding_block_size_minus3: self
                .log2_min_pcm_luma_coding_block_size_minus3,
            log2_diff_max_min_pcm_luma_coding_block_size: self
                .log2_diff_max_min_pcm_luma_coding_block_size,
            pcm_loop_filter_disabled_flag: self.pcm_loop_filter_disabled_flag,
            num_short_term_ref_pic_sets: self.num_short_term_ref_pic_sets,
            st_ref_pic_set: self.st_ref_pic_set.clone(),
            long_term_ref_pics_present_flag: self.long_term_ref_pics_present_flag,
            num_long_term_ref_pics_sps: self.num_long_term_ref_pics_sps,
            lt_ref_pic_poc_lsb_sps: self.lt_ref_pic_poc_lsb_sps.clone(),
            used_by_curr_pic_lt_sps_flag: self.used_by_curr_pic_lt_sps_flag.clone(),
            sps_temporal_mvp_enabled_flag: self.sps_temporal_mvp_enabled_flag,
            strong_intra_smoothing_enabled_flag: self.strong_intra_smoothing_enabled_flag,
            vui_parameters_present_flag: self.vui_parameters_present_flag,
            vui_parameters: self.vui_parameters.clone(),
            sps_extension_present_flag: self.sps_extension_present_flag,
            sps_range_extension_flag: self.sps_range_extension_flag,
            sps_multilayer_extension_flag: self.sps_multilayer_extension_flag,
            sps_3d_extension_flag: self.sps_3d_extension_flag,
            sps_scc_extension_flag: self.sps_scc_extension_flag,
            sps_extension_4bits: self.sps_extension_4bits,
            sps_range_extension: self.sps_range_extension.clone(),
            sps_multilayer_extension: self.sps_multilayer_extension.clone(),
            sps_3d_extension: self.sps_3d_extension.clone(),
            sps_scc_extension: self.sps_scc_extension.clone(),
            sps_extension_data_flag: self.sps_extension_data_flag,
        }
    }

    pub fn debug_print(&self) {
        decoder_formatted_print(
            "SPS: sps_video_parameter_set_id",
            &self.sps_video_parameter_set_id,
            63,
        );
        decoder_formatted_print(
            "SPS: sps_max_sub_layers_minus1",
            &self.sps_max_sub_layers_minus1,
            63,
        );
        decoder_formatted_print(
            "SPS: sps_temporal_id_nesting_flag",
            &self.sps_temporal_id_nesting_flag,
            63,
        );
        self.profile_tier_level.debug_print();
        decoder_formatted_print(
            "SPS: sps_seq_parameter_set_id",
            &self.sps_seq_parameter_set_id,
            63,
        );
        decoder_formatted_print("SPS: chroma_format_idc", &self.chroma_format_idc, 63);
        if self.chroma_format_idc == 3 {
            decoder_formatted_print(
                "SPS: separate_colour_plane_flag",
                &self.separate_colour_plane_flag,
                63,
            );
        }

        decoder_formatted_print(
            "SPS: pic_width_in_luma_samples",
            &self.pic_width_in_luma_samples,
            63,
        );
        decoder_formatted_print(
            "SPS: pic_height_in_luma_samples",
            &self.pic_height_in_luma_samples,
            63,
        );
        decoder_formatted_print(
            "SPS: conformance_window_flag",
            &self.conformance_window_flag,
            63,
        );
        if self.conformance_window_flag {
            decoder_formatted_print("SPS: conf_win_left_offset", &self.conf_win_left_offset, 63);
            decoder_formatted_print(
                "SPS: conf_win_right_offset",
                &self.conf_win_right_offset,
                63,
            );
            decoder_formatted_print("SPS: conf_win_top_offset", &self.conf_win_top_offset, 63);
            decoder_formatted_print(
                "SPS: conf_win_bottom_offset",
                &self.conf_win_bottom_offset,
                63,
            );
        }

        decoder_formatted_print(
            "SPS: bit_depth_luma_minus8",
            &self.bit_depth_luma_minus8,
            63,
        );
        decoder_formatted_print(
            "SPS: bit_depth_chroma_minus8",
            &self.bit_depth_chroma_minus8,
            63,
        );
        decoder_formatted_print(
            "SPS: log2_max_pic_order_cnt_lsb_minus4",
            &self.log2_max_pic_order_cnt_lsb_minus4,
            63,
        );
        decoder_formatted_print(
            "SPS: sps_sub_layer_ordering_info_present_flag",
            &self.sps_sub_layer_ordering_info_present_flag,
            63,
        );

        decoder_formatted_print(
            "SPS: sps_max_dec_pic_buffering_minus1",
            &self.sps_max_dec_pic_buffering_minus1,
            63,
        );
        decoder_formatted_print(
            "SPS: sps_max_num_reorder_pics",
            &self.sps_max_num_reorder_pics,
            63,
        );
        decoder_formatted_print(
            "SPS: sps_max_latency_increase_plus1",
            &self.sps_max_latency_increase_plus1,
            63,
        );

        decoder_formatted_print(
            "SPS: log2_min_luma_coding_block_size_minus3",
            &self.log2_min_luma_coding_block_size_minus3,
            63,
        );
        decoder_formatted_print(
            "SPS: log2_diff_max_min_luma_coding_block_size",
            &self.log2_diff_max_min_luma_coding_block_size,
            63,
        );
        decoder_formatted_print(
            "SPS: log2_min_luma_transform_block_size_minus2",
            &self.log2_min_luma_transform_block_size_minus2,
            63,
        );
        decoder_formatted_print(
            "SPS: log2_diff_max_min_luma_transform_block_size",
            &self.log2_diff_max_min_luma_transform_block_size,
            63,
        );
        decoder_formatted_print(
            "SPS: max_transform_hierarchy_depth_inter",
            &self.max_transform_hierarchy_depth_inter,
            63,
        );
        decoder_formatted_print(
            "SPS: max_transform_hierarchy_depth_intra",
            &self.max_transform_hierarchy_depth_intra,
            63,
        );
        decoder_formatted_print(
            "SPS: scaling_list_enabled_flag",
            &self.scaling_list_enabled_flag,
            63,
        );
        if self.scaling_list_enabled_flag {
            decoder_formatted_print(
                "SPS: sps_scaling_list_data_present_flag",
                &self.sps_scaling_list_data_present_flag,
                63,
            );
            if self.sps_scaling_list_data_present_flag {
                self.scaling_list_data.debug_print();
            }
        }

        decoder_formatted_print("SPS: amp_enabled_flag", &self.amp_enabled_flag, 63);
        decoder_formatted_print(
            "SPS: sample_adaptive_offset_enabled_flag",
            &self.sample_adaptive_offset_enabled_flag,
            63,
        );
        decoder_formatted_print("SPS: pcm_enabled_flag", &self.pcm_enabled_flag, 63);
        if self.pcm_enabled_flag {
            decoder_formatted_print(
                "SPS: pcm_sample_bit_depth_luma_minus1",
                &self.pcm_sample_bit_depth_luma_minus1,
                63,
            );
            decoder_formatted_print(
                "SPS: pcm_sample_bit_depth_chroma_minus1",
                &self.pcm_sample_bit_depth_chroma_minus1,
                63,
            );
            decoder_formatted_print(
                "SPS: log2_min_pcm_luma_coding_block_size_minus3",
                &self.log2_min_pcm_luma_coding_block_size_minus3,
                63,
            );
            decoder_formatted_print(
                "SPS: log2_diff_max_min_pcm_luma_coding_block_size",
                &self.log2_diff_max_min_pcm_luma_coding_block_size,
                63,
            );
            decoder_formatted_print(
                "SPS: pcm_loop_filter_disabled_flag",
                &self.pcm_loop_filter_disabled_flag,
                63,
            );
        }
        decoder_formatted_print(
            "SPS: num_short_term_ref_pic_sets",
            &self.num_short_term_ref_pic_sets,
            63,
        );
        for pic in &self.st_ref_pic_set {
            pic.debug_print();
        }
        decoder_formatted_print(
            "SPS: long_term_ref_pics_present_flag",
            &self.long_term_ref_pics_present_flag,
            63,
        );
        if self.long_term_ref_pics_present_flag {
            decoder_formatted_print(
                "SPS: num_long_term_ref_pics_sps",
                &self.num_long_term_ref_pics_sps,
                63,
            );
            decoder_formatted_print(
                "SPS: lt_ref_pic_poc_lsb_sps",
                &self.lt_ref_pic_poc_lsb_sps,
                63,
            );
            decoder_formatted_print(
                "SPS: used_by_curr_pic_lt_sps_flag",
                &self.used_by_curr_pic_lt_sps_flag,
                63,
            );
        }
        decoder_formatted_print(
            "SPS: sps_temporal_mvp_enabled_flag",
            &self.sps_temporal_mvp_enabled_flag,
            63,
        );
        decoder_formatted_print(
            "SPS: strong_intra_smoothing_enabled_flag",
            &self.strong_intra_smoothing_enabled_flag,
            63,
        );
        decoder_formatted_print(
            "SPS: vui_parameters_present_flag",
            &self.vui_parameters_present_flag,
            63,
        );
        if self.vui_parameters_present_flag {
            self.vui_parameters.debug_print();
        }
        decoder_formatted_print(
            "SPS: sps_extension_present_flag",
            &self.sps_extension_present_flag,
            63,
        );
        if self.sps_extension_present_flag {
            decoder_formatted_print(
                "SPS: sps_range_extension_flag",
                &self.sps_range_extension_flag,
                63,
            );
            decoder_formatted_print(
                "SPS: sps_multilayer_extension_flag",
                &self.sps_multilayer_extension_flag,
                63,
            );
            decoder_formatted_print(
                "SPS: sps_3d_extension_flag",
                &self.sps_3d_extension_flag,
                63,
            );
            decoder_formatted_print(
                "SPS: sps_scc_extension_flag",
                &self.sps_scc_extension_flag,
                63,
            );
            decoder_formatted_print("SPS: sps_extension_4bits", &self.sps_extension_4bits, 63);
        }

        if self.sps_range_extension_flag {
            self.sps_range_extension.debug_print();
        }
        if self.sps_multilayer_extension_flag {
            self.sps_multilayer_extension.debug_print();
        }
        if self.sps_3d_extension_flag {
            self.sps_3d_extension.debug_print();
        }
        if self.sps_scc_extension_flag {
            self.sps_scc_extension.debug_print();
        }

        decoder_formatted_print(
            "SPS: sps_extension_data_flag",
            &self.sps_extension_data_flag,
            63,
        );
    }

    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print(
            "SPS: sps_video_parameter_set_id",
            &self.sps_video_parameter_set_id,
            63,
        );
        encoder_formatted_print(
            "SPS: sps_max_sub_layers_minus1",
            &self.sps_max_sub_layers_minus1,
            63,
        );
        encoder_formatted_print(
            "SPS: sps_temporal_id_nesting_flag",
            &self.sps_temporal_id_nesting_flag,
            63,
        );
        self.profile_tier_level.encoder_pretty_print();
        encoder_formatted_print(
            "SPS: sps_seq_parameter_set_id",
            &self.sps_seq_parameter_set_id,
            63,
        );
        encoder_formatted_print("SPS: chroma_format_idc", &self.chroma_format_idc, 63);
        if self.chroma_format_idc == 3 {
            encoder_formatted_print(
                "SPS: separate_colour_plane_flag",
                &self.separate_colour_plane_flag,
                63,
            );
        }

        encoder_formatted_print(
            "SPS: pic_width_in_luma_samples",
            &self.pic_width_in_luma_samples,
            63,
        );
        encoder_formatted_print(
            "SPS: pic_height_in_luma_samples",
            &self.pic_height_in_luma_samples,
            63,
        );
        encoder_formatted_print(
            "SPS: conformance_window_flag",
            &self.conformance_window_flag,
            63,
        );
        if self.conformance_window_flag {
            encoder_formatted_print("SPS: conf_win_left_offset", &self.conf_win_left_offset, 63);
            encoder_formatted_print(
                "SPS: conf_win_right_offset",
                &self.conf_win_right_offset,
                63,
            );
            encoder_formatted_print("SPS: conf_win_top_offset", &self.conf_win_top_offset, 63);
            encoder_formatted_print(
                "SPS: conf_win_bottom_offset",
                &self.conf_win_bottom_offset,
                63,
            );
        }

        encoder_formatted_print(
            "SPS: bit_depth_luma_minus8",
            &self.bit_depth_luma_minus8,
            63,
        );
        encoder_formatted_print(
            "SPS: bit_depth_chroma_minus8",
            &self.bit_depth_chroma_minus8,
            63,
        );
        encoder_formatted_print(
            "SPS: log2_max_pic_order_cnt_lsb_minus4",
            &self.log2_max_pic_order_cnt_lsb_minus4,
            63,
        );
        encoder_formatted_print(
            "SPS: sps_sub_layer_ordering_info_present_flag",
            &self.sps_sub_layer_ordering_info_present_flag,
            63,
        );

        encoder_formatted_print(
            "SPS: sps_max_dec_pic_buffering_minus1",
            &self.sps_max_dec_pic_buffering_minus1,
            63,
        );
        encoder_formatted_print(
            "SPS: sps_max_num_reorder_pics",
            &self.sps_max_num_reorder_pics,
            63,
        );
        encoder_formatted_print(
            "SPS: sps_max_latency_increase_plus1",
            &self.sps_max_latency_increase_plus1,
            63,
        );

        encoder_formatted_print(
            "SPS: log2_min_luma_coding_block_size_minus3",
            &self.log2_min_luma_coding_block_size_minus3,
            63,
        );
        encoder_formatted_print(
            "SPS: log2_diff_max_min_luma_coding_block_size",
            &self.log2_diff_max_min_luma_coding_block_size,
            63,
        );
        encoder_formatted_print(
            "SPS: log2_min_luma_transform_block_size_minus2",
            &self.log2_min_luma_transform_block_size_minus2,
            63,
        );
        encoder_formatted_print(
            "SPS: log2_diff_max_min_luma_transform_block_size",
            &self.log2_diff_max_min_luma_transform_block_size,
            63,
        );
        encoder_formatted_print(
            "SPS: max_transform_hierarchy_depth_inter",
            &self.max_transform_hierarchy_depth_inter,
            63,
        );
        encoder_formatted_print(
            "SPS: max_transform_hierarchy_depth_intra",
            &self.max_transform_hierarchy_depth_intra,
            63,
        );
        encoder_formatted_print(
            "SPS: scaling_list_enabled_flag",
            &self.scaling_list_enabled_flag,
            63,
        );
        if self.scaling_list_enabled_flag {
            encoder_formatted_print(
                "SPS: sps_scaling_list_data_present_flag",
                &self.sps_scaling_list_data_present_flag,
                63,
            );
            if self.sps_scaling_list_data_present_flag {
                self.scaling_list_data.encoder_pretty_print();
            }
        }

        encoder_formatted_print("SPS: amp_enabled_flag", &self.amp_enabled_flag, 63);
        encoder_formatted_print(
            "SPS: sample_adaptive_offset_enabled_flag",
            &self.sample_adaptive_offset_enabled_flag,
            63,
        );
        encoder_formatted_print("SPS: pcm_enabled_flag", &self.pcm_enabled_flag, 63);
        if self.pcm_enabled_flag {
            encoder_formatted_print(
                "SPS: pcm_sample_bit_depth_luma_minus1",
                &self.pcm_sample_bit_depth_luma_minus1,
                63,
            );
            encoder_formatted_print(
                "SPS: pcm_sample_bit_depth_chroma_minus1",
                &self.pcm_sample_bit_depth_chroma_minus1,
                63,
            );
            encoder_formatted_print(
                "SPS: log2_min_pcm_luma_coding_block_size_minus3",
                &self.log2_min_pcm_luma_coding_block_size_minus3,
                63,
            );
            encoder_formatted_print(
                "SPS: log2_diff_max_min_pcm_luma_coding_block_size",
                &self.log2_diff_max_min_pcm_luma_coding_block_size,
                63,
            );
            encoder_formatted_print(
                "SPS: pcm_loop_filter_disabled_flag",
                &self.pcm_loop_filter_disabled_flag,
                63,
            );
        }
        encoder_formatted_print(
            "SPS: num_short_term_ref_pic_sets",
            &self.num_short_term_ref_pic_sets,
            63,
        );
        for pic in &self.st_ref_pic_set {
            pic.encoder_pretty_print();
        }
        encoder_formatted_print(
            "SPS: long_term_ref_pics_present_flag",
            &self.long_term_ref_pics_present_flag,
            63,
        );
        if self.long_term_ref_pics_present_flag {
            encoder_formatted_print(
                "SPS: num_long_term_ref_pics_sps",
                &self.num_long_term_ref_pics_sps,
                63,
            );
            encoder_formatted_print(
                "SPS: lt_ref_pic_poc_lsb_sps",
                &self.lt_ref_pic_poc_lsb_sps,
                63,
            );
            encoder_formatted_print(
                "SPS: used_by_curr_pic_lt_sps_flag",
                &self.used_by_curr_pic_lt_sps_flag,
                63,
            );
        }
        encoder_formatted_print(
            "SPS: sps_temporal_mvp_enabled_flag",
            &self.sps_temporal_mvp_enabled_flag,
            63,
        );
        encoder_formatted_print(
            "SPS: strong_intra_smoothing_enabled_flag",
            &self.strong_intra_smoothing_enabled_flag,
            63,
        );
        encoder_formatted_print(
            "SPS: vui_parameters_present_flag",
            &self.vui_parameters_present_flag,
            63,
        );
        if self.vui_parameters_present_flag {
            self.vui_parameters.encoder_pretty_print();
        }
        encoder_formatted_print(
            "SPS: sps_extension_present_flag",
            &self.sps_extension_present_flag,
            63,
        );
        if self.sps_extension_present_flag {
            encoder_formatted_print(
                "SPS: sps_range_extension_flag",
                &self.sps_range_extension_flag,
                63,
            );
            encoder_formatted_print(
                "SPS: sps_multilayer_extension_flag",
                &self.sps_multilayer_extension_flag,
                63,
            );
            encoder_formatted_print(
                "SPS: sps_3d_extension_flag",
                &self.sps_3d_extension_flag,
                63,
            );
            encoder_formatted_print(
                "SPS: sps_scc_extension_flag",
                &self.sps_scc_extension_flag,
                63,
            );
            encoder_formatted_print("SPS: sps_extension_4bits", &self.sps_extension_4bits, 63);
        }

        if self.sps_range_extension_flag {
            self.sps_range_extension.encoder_pretty_print();
        }
        if self.sps_multilayer_extension_flag {
            self.sps_multilayer_extension.encoder_pretty_print();
        }
        if self.sps_3d_extension_flag {
            self.sps_3d_extension.encoder_pretty_print();
        }
        if self.sps_scc_extension_flag {
            self.sps_scc_extension.encoder_pretty_print();
        }

        encoder_formatted_print(
            "SPS: sps_extension_data_flag",
            &self.sps_extension_data_flag,
            63,
        );
    }
}

impl Default for H265SeqParameterSet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProfileTierLevel {
    pub general_profile_space: u8,                            // u(2)
    pub general_tier_flag: bool,                              // u(1)
    pub general_profile_idc: u8,                              // u(5)
    pub general_profile_compatibility_flag: [bool; 32],       // len(32) u(1)
    pub general_progressive_source_flag: bool,                // u(1)
    pub general_interlaced_source_flag: bool,                 // u(1)
    pub general_non_packed_constraint_flag: bool,             // u(1)
    pub general_frame_only_constraint_flag: bool,             // u(1)
    pub general_max_12bit_constraint_flag: bool,              // u(1)
    pub general_max_10bit_constraint_flag: bool,              // u(1)
    pub general_max_8bit_constraint_flag: bool,               // u(1)
    pub general_max_422chroma_constraint_flag: bool,          // u(1)
    pub general_max_420chroma_constraint_flag: bool,          // u(1)
    pub general_max_monochrome_constraint_flag: bool,         // u(1)
    pub general_intra_constraint_flag: bool,                  // u(1)
    pub general_one_picture_only_constraint_flag: bool,       // u(1)
    pub general_lower_bit_rate_constraint_flag: bool,         // u(1)
    pub general_max_14bit_constraint_flag: bool,              // u(1)
    pub general_reserved_zero_33bits: u64,                    // u(33)
    pub general_reserved_zero_34bits: u64,                    // u(34)
    pub general_reserved_zero_7bits: u8,                      // u(7)
    pub general_reserved_zero_35bits: u64,                    // u(35)
    pub general_reserved_zero_43bits: u64,                    // u(43)
    pub general_inbld_flag: bool,                             // u(1)
    pub general_reserved_zero_bit: u8,                        // u(1)
    pub general_level_idc: u8,                                // u(8)
    pub sub_layer_profile_present_flag: Vec<bool>,            //
    pub sub_layer_level_present_flag: Vec<bool>,              //
    pub reserved_zero_2bits: Vec<u8>,                         // u(2)
    pub sub_layer_profile_space: Vec<u8>,                     // u(2)
    pub sub_layer_tier_flag: Vec<bool>,                       // u(1)
    pub sub_layer_profile_idc: Vec<u8>,                       // u(5)
    pub sub_layer_profile_compatibility_flag: Vec<Vec<bool>>, // u(1)
    pub sub_layer_progressive_source_flag: Vec<bool>,         // u(1)
    pub sub_layer_interlaced_source_flag: Vec<bool>,          // u(1)
    pub sub_layer_non_packed_constraint_flag: Vec<bool>,      // u(1)
    pub sub_layer_frame_only_constraint_flag: Vec<bool>,      // u(1)
    pub sub_layer_max_12bit_constraint_flag: Vec<bool>,
    pub sub_layer_max_10bit_constraint_flag: Vec<bool>,
    pub sub_layer_max_8bit_constraint_flag: Vec<bool>,
    pub sub_layer_max_422chroma_constraint_flag: Vec<bool>,
    pub sub_layer_max_420chroma_constraint_flag: Vec<bool>,
    pub sub_layer_max_monochrome_constraint_flag: Vec<bool>,
    pub sub_layer_intra_constraint_flag: Vec<bool>,
    pub sub_layer_one_picture_only_constraint_flag: Vec<bool>,
    pub sub_layer_lower_bit_rate_constraint_flag: Vec<bool>,
    pub sub_layer_max_14bit_constraint_flag: Vec<bool>,
    pub sub_layer_reserved_zero_33bits: Vec<u64>,
    pub sub_layer_reserved_zero_34bits: Vec<u64>,
    pub sub_layer_reserved_zero_7bits: Vec<u8>,   // u(7)
    pub sub_layer_reserved_zero_35bits: Vec<u64>, // u(35)
    pub sub_layer_reserved_zero_43bits: Vec<u64>, // u(43)
    pub sub_layer_inbld_flag: Vec<bool>,
    pub sub_layer_reserved_zero_bit: Vec<u8>, // u(1)
    pub sub_layer_level_idc: Vec<u8>,         // u(8)
}

impl ProfileTierLevel {
    pub fn new() -> ProfileTierLevel {
        ProfileTierLevel {
            general_profile_space: 0,
            general_tier_flag: false,
            general_profile_idc: 0,
            general_profile_compatibility_flag: [false; 32],
            general_progressive_source_flag: false,
            general_interlaced_source_flag: false,
            general_non_packed_constraint_flag: false,
            general_frame_only_constraint_flag: false,
            general_max_12bit_constraint_flag: false,
            general_max_10bit_constraint_flag: false,
            general_max_8bit_constraint_flag: false,
            general_max_422chroma_constraint_flag: false,
            general_max_420chroma_constraint_flag: false,
            general_max_monochrome_constraint_flag: false,
            general_intra_constraint_flag: false,
            general_one_picture_only_constraint_flag: false,
            general_lower_bit_rate_constraint_flag: false,
            general_max_14bit_constraint_flag: false,
            general_reserved_zero_33bits: 0,
            general_reserved_zero_34bits: 0,
            general_reserved_zero_7bits: 0,
            general_reserved_zero_35bits: 0,
            general_reserved_zero_43bits: 0,
            general_inbld_flag: false,
            general_reserved_zero_bit: 0,
            general_level_idc: 0,
            sub_layer_profile_present_flag: Vec::new(),
            sub_layer_level_present_flag: Vec::new(),
            reserved_zero_2bits: Vec::new(),
            sub_layer_profile_space: Vec::new(),
            sub_layer_tier_flag: Vec::new(),
            sub_layer_profile_idc: Vec::new(),
            sub_layer_profile_compatibility_flag: Vec::new(),
            sub_layer_progressive_source_flag: Vec::new(),
            sub_layer_interlaced_source_flag: Vec::new(),
            sub_layer_non_packed_constraint_flag: Vec::new(),
            sub_layer_frame_only_constraint_flag: Vec::new(),
            sub_layer_max_12bit_constraint_flag: Vec::new(),
            sub_layer_max_10bit_constraint_flag: Vec::new(),
            sub_layer_max_8bit_constraint_flag: Vec::new(),
            sub_layer_max_422chroma_constraint_flag: Vec::new(),
            sub_layer_max_420chroma_constraint_flag: Vec::new(),
            sub_layer_max_monochrome_constraint_flag: Vec::new(),
            sub_layer_intra_constraint_flag: Vec::new(),
            sub_layer_one_picture_only_constraint_flag: Vec::new(),
            sub_layer_lower_bit_rate_constraint_flag: Vec::new(),
            sub_layer_max_14bit_constraint_flag: Vec::new(),
            sub_layer_reserved_zero_33bits: Vec::new(),
            sub_layer_reserved_zero_34bits: Vec::new(),
            sub_layer_reserved_zero_7bits: Vec::new(),
            sub_layer_reserved_zero_35bits: Vec::new(),
            sub_layer_reserved_zero_43bits: Vec::new(),
            sub_layer_inbld_flag: Vec::new(),
            sub_layer_reserved_zero_bit: Vec::new(),
            sub_layer_level_idc: Vec::new(),
        }
    }

    fn debug_print(&self) {
        // TODO: proper ProfileTierLevel formatting
        decoder_formatted_print(
            "ProfileTierLevel: general_profile_space",
            &self.general_profile_space,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_tier_flag",
            &self.general_tier_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_profile_idc",
            &self.general_profile_idc,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_profile_compatibility_flag",
            &self.general_profile_compatibility_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_progressive_source_flag",
            &self.general_progressive_source_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_interlaced_source_flag",
            &self.general_interlaced_source_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_non_packed_constraint_flag",
            &self.general_non_packed_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_frame_only_constraint_flag",
            &self.general_frame_only_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_max_12bit_constraint_flag",
            &self.general_max_12bit_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_max_10bit_constraint_flag",
            &self.general_max_10bit_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_max_8bit_constraint_flag",
            &self.general_max_8bit_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_max_422chroma_constraint_flag",
            &self.general_max_422chroma_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_max_420chroma_constraint_flag",
            &self.general_max_420chroma_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_max_monochrome_constraint_flag",
            &self.general_max_monochrome_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_intra_constraint_flag",
            &self.general_intra_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_one_picture_only_constraint_flag",
            &self.general_one_picture_only_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_lower_bit_rate_constraint_flag",
            &self.general_lower_bit_rate_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_max_14bit_constraint_flag",
            &self.general_max_14bit_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_reserved_zero_33bits",
            &self.general_reserved_zero_33bits,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_reserved_zero_34bits",
            &self.general_reserved_zero_34bits,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_reserved_zero_7bits",
            &self.general_reserved_zero_7bits,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_reserved_zero_35bits",
            &self.general_reserved_zero_35bits,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_reserved_zero_43bits",
            &self.general_reserved_zero_43bits,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_inbld_flag",
            &self.general_inbld_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_reserved_zero_bit",
            &self.general_reserved_zero_bit,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: general_level_idc",
            &self.general_level_idc,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_profile_present_flag",
            &self.sub_layer_profile_present_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_level_present_flag",
            &self.sub_layer_level_present_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: reserved_zero_2bits",
            &self.reserved_zero_2bits,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_profile_space",
            &self.sub_layer_profile_space,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_tier_flag",
            &self.sub_layer_tier_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_profile_idc",
            &self.sub_layer_profile_idc,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_profile_compatibility_flag",
            &self.sub_layer_profile_compatibility_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_progressive_source_flag",
            &self.sub_layer_progressive_source_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_interlaced_source_flag",
            &self.sub_layer_interlaced_source_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_non_packed_constraint_flag",
            &self.sub_layer_non_packed_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_frame_only_constraint_flag",
            &self.sub_layer_frame_only_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_max_12bit_constraint_flag",
            &self.sub_layer_max_12bit_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_max_10bit_constraint_flag",
            &self.sub_layer_max_10bit_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_max_8bit_constraint_flag",
            &self.sub_layer_max_8bit_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_max_422chroma_constraint_flag",
            &self.sub_layer_max_422chroma_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_max_420chroma_constraint_flag",
            &self.sub_layer_max_420chroma_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_max_monochrome_constraint_flag",
            &self.sub_layer_max_monochrome_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_intra_constraint_flag",
            &self.sub_layer_intra_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_one_picture_only_constraint_flag",
            &self.sub_layer_one_picture_only_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_lower_bit_rate_constraint_flag",
            &self.sub_layer_lower_bit_rate_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_max_14bit_constraint_flag",
            &self.sub_layer_max_14bit_constraint_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_reserved_zero_33bits",
            &self.sub_layer_reserved_zero_33bits,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_reserved_zero_34bits",
            &self.sub_layer_reserved_zero_34bits,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_reserved_zero_7bits",
            &self.sub_layer_reserved_zero_7bits,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_reserved_zero_35bits",
            &self.sub_layer_reserved_zero_35bits,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_reserved_zero_43bits",
            &self.sub_layer_reserved_zero_43bits,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_inbld_flag",
            &self.sub_layer_inbld_flag,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_reserved_zero_bit",
            &self.sub_layer_reserved_zero_bit,
            63,
        );
        decoder_formatted_print(
            "ProfileTierLevel: sub_layer_level_idc",
            &self.sub_layer_level_idc,
            63,
        );
    }

    fn encoder_pretty_print(&self) {
        // TODO: proper ProfileTierLevel formatting
        encoder_formatted_print(
            "ProfileTierLevel: general_profile_space",
            &self.general_profile_space,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_tier_flag",
            &self.general_tier_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_profile_idc",
            &self.general_profile_idc,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_profile_compatibility_flag",
            &self.general_profile_compatibility_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_progressive_source_flag",
            &self.general_progressive_source_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_interlaced_source_flag",
            &self.general_interlaced_source_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_non_packed_constraint_flag",
            &self.general_non_packed_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_frame_only_constraint_flag",
            &self.general_frame_only_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_max_12bit_constraint_flag",
            &self.general_max_12bit_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_max_10bit_constraint_flag",
            &self.general_max_10bit_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_max_8bit_constraint_flag",
            &self.general_max_8bit_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_max_422chroma_constraint_flag",
            &self.general_max_422chroma_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_max_420chroma_constraint_flag",
            &self.general_max_420chroma_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_max_monochrome_constraint_flag",
            &self.general_max_monochrome_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_intra_constraint_flag",
            &self.general_intra_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_one_picture_only_constraint_flag",
            &self.general_one_picture_only_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_lower_bit_rate_constraint_flag",
            &self.general_lower_bit_rate_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_max_14bit_constraint_flag",
            &self.general_max_14bit_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_reserved_zero_33bits",
            &self.general_reserved_zero_33bits,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_reserved_zero_34bits",
            &self.general_reserved_zero_34bits,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_reserved_zero_7bits",
            &self.general_reserved_zero_7bits,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_reserved_zero_35bits",
            &self.general_reserved_zero_35bits,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_reserved_zero_43bits",
            &self.general_reserved_zero_43bits,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_inbld_flag",
            &self.general_inbld_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_reserved_zero_bit",
            &self.general_reserved_zero_bit,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: general_level_idc",
            &self.general_level_idc,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_profile_present_flag",
            &self.sub_layer_profile_present_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_level_present_flag",
            &self.sub_layer_level_present_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: reserved_zero_2bits",
            &self.reserved_zero_2bits,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_profile_space",
            &self.sub_layer_profile_space,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_tier_flag",
            &self.sub_layer_tier_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_profile_idc",
            &self.sub_layer_profile_idc,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_profile_compatibility_flag",
            &self.sub_layer_profile_compatibility_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_progressive_source_flag",
            &self.sub_layer_progressive_source_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_interlaced_source_flag",
            &self.sub_layer_interlaced_source_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_non_packed_constraint_flag",
            &self.sub_layer_non_packed_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_frame_only_constraint_flag",
            &self.sub_layer_frame_only_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_max_12bit_constraint_flag",
            &self.sub_layer_max_12bit_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_max_10bit_constraint_flag",
            &self.sub_layer_max_10bit_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_max_8bit_constraint_flag",
            &self.sub_layer_max_8bit_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_max_422chroma_constraint_flag",
            &self.sub_layer_max_422chroma_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_max_420chroma_constraint_flag",
            &self.sub_layer_max_420chroma_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_max_monochrome_constraint_flag",
            &self.sub_layer_max_monochrome_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_intra_constraint_flag",
            &self.sub_layer_intra_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_one_picture_only_constraint_flag",
            &self.sub_layer_one_picture_only_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_lower_bit_rate_constraint_flag",
            &self.sub_layer_lower_bit_rate_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_max_14bit_constraint_flag",
            &self.sub_layer_max_14bit_constraint_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_reserved_zero_33bits",
            &self.sub_layer_reserved_zero_33bits,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_reserved_zero_34bits",
            &self.sub_layer_reserved_zero_34bits,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_reserved_zero_7bits",
            &self.sub_layer_reserved_zero_7bits,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_reserved_zero_35bits",
            &self.sub_layer_reserved_zero_35bits,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_reserved_zero_43bits",
            &self.sub_layer_reserved_zero_43bits,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_inbld_flag",
            &self.sub_layer_inbld_flag,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_reserved_zero_bit",
            &self.sub_layer_reserved_zero_bit,
            63,
        );
        encoder_formatted_print(
            "ProfileTierLevel: sub_layer_level_idc",
            &self.sub_layer_level_idc,
            63,
        );
    }
}

impl Default for ProfileTierLevel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ScalingListData {
    pub scaling_list_pred_mode_flag: Vec<Vec<bool>>, // sizeId (4) * matrixId (6)
    pub scaling_list_pred_matrix_id_delta: Vec<Vec<u32>>, // sizeId (4) * matrixId (6)
    pub scaling_list_dc_coef_minus8: Vec<Vec<i32>>,  // sizeId (4) * matrixId (6)
    pub scaling_list_delta_coef: Vec<Vec<Vec<i32>>>, // // sizeId (4) * matrixId (6) * coefNum
    // derived values
    pub scaling_list: Vec<Vec<Vec<u32>>>,
}

impl ScalingListData {
    pub fn new() -> ScalingListData {
        ScalingListData {
            scaling_list_pred_mode_flag: Vec::new(),
            scaling_list_pred_matrix_id_delta: Vec::new(),
            scaling_list_dc_coef_minus8: Vec::new(),
            scaling_list_delta_coef: Vec::new(),
            scaling_list: Vec::new(),
        }
    }

    pub fn debug_print(&self) {
        decoder_formatted_print(
            "Scaling List: scaling_list_pred_mode_flag",
            &self.scaling_list_pred_mode_flag,
            63,
        );
        decoder_formatted_print(
            "Scaling List: scaling_list_pred_matrix_id_delta",
            &self.scaling_list_pred_matrix_id_delta,
            63,
        );
        decoder_formatted_print(
            "Scaling List: scaling_list_dc_coef_minus8",
            &self.scaling_list_dc_coef_minus8,
            63,
        );
        decoder_formatted_print(
            "Scaling List: scaling_list_delta_coef",
            &self.scaling_list_delta_coef,
            63,
        );
        decoder_formatted_print("Scaling List: scaling_list", &self.scaling_list, 63);
    }

    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print(
            "Scaling List: scaling_list_pred_mode_flag",
            &self.scaling_list_pred_mode_flag,
            63,
        );
        encoder_formatted_print(
            "Scaling List: scaling_list_pred_matrix_id_delta",
            &self.scaling_list_pred_matrix_id_delta,
            63,
        );
        encoder_formatted_print(
            "Scaling List: scaling_list_dc_coef_minus8",
            &self.scaling_list_dc_coef_minus8,
            63,
        );
        encoder_formatted_print(
            "Scaling List: scaling_list_delta_coef",
            &self.scaling_list_delta_coef,
            63,
        );
        encoder_formatted_print("Scaling List: scaling_list", &self.scaling_list, 63);
    }
}

impl Default for ScalingListData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct H265VuiParameters {}

impl H265VuiParameters {
    pub fn new() -> H265VuiParameters {
        H265VuiParameters {}
    }

    pub fn debug_print(&self) {}

    pub fn encoder_pretty_print(&self) {}
}

impl Default for H265VuiParameters {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct H265SPSRangeExtension {}

impl H265SPSRangeExtension {
    pub fn new() -> H265SPSRangeExtension {
        H265SPSRangeExtension {}
    }

    pub fn debug_print(&self) {}

    pub fn encoder_pretty_print(&self) {}
}

impl Default for H265SPSRangeExtension {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct H265SPSMultilayerExtension {}

impl H265SPSMultilayerExtension {
    pub fn new() -> H265SPSMultilayerExtension {
        H265SPSMultilayerExtension {}
    }

    pub fn debug_print(&self) {}

    pub fn encoder_pretty_print(&self) {}
}

impl Default for H265SPSMultilayerExtension {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct H265SPS3DExtension {}

impl H265SPS3DExtension {
    pub fn new() -> H265SPS3DExtension {
        H265SPS3DExtension {}
    }

    pub fn debug_print(&self) {}

    pub fn encoder_pretty_print(&self) {}
}

impl Default for H265SPS3DExtension {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]

pub struct H265SPSSCCExtension {}

impl H265SPSSCCExtension {
    pub fn new() -> H265SPSSCCExtension {
        H265SPSSCCExtension {}
    }

    pub fn debug_print(&self) {}

    pub fn encoder_pretty_print(&self) {}
}

impl Default for H265SPSSCCExtension {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ShortTermRefPic {
    pub inter_ref_pic_set_prediction_flag: bool, //u(1)
    pub delta_idx_minus1: u32,                   //ue(v)
    pub delta_rps_sign: bool,                    //u(1)
    pub abs_delta_rps_minus1: u32,               //ue(v)
    pub used_by_curr_pic_flag: Vec<bool>,
    pub use_delta_flag: Vec<bool>,
    pub num_negative_pics: u32, //ue(v)
    pub num_positive_pics: u32, //ue(v)
    pub delta_poc_s0_minus1: Vec<u32>,
    pub used_by_curr_pic_s0_flag: Vec<bool>,
    pub delta_poc_s1_minus1: Vec<u32>,
    pub used_by_curr_pic_s1_flag: Vec<bool>,

    // Derived from bitstream
    pub num_delta_pics: u32, // num_negative_pics + num_positive_pics
}

impl ShortTermRefPic {
    pub fn new() -> ShortTermRefPic {
        ShortTermRefPic {
            inter_ref_pic_set_prediction_flag: false,
            delta_idx_minus1: 0,
            delta_rps_sign: false,
            abs_delta_rps_minus1: 0,
            used_by_curr_pic_flag: Vec::new(),
            use_delta_flag: Vec::new(),
            num_negative_pics: 0,
            num_positive_pics: 0,
            delta_poc_s0_minus1: Vec::new(),
            used_by_curr_pic_s0_flag: Vec::new(),
            delta_poc_s1_minus1: Vec::new(),
            used_by_curr_pic_s1_flag: Vec::new(),
            num_delta_pics: 0,
        }
    }

    pub fn debug_print(&self) {
        decoder_formatted_print(
            "StRefPic: inter_ref_pic_set_prediction_flag",
            &self.inter_ref_pic_set_prediction_flag,
            63,
        );
        decoder_formatted_print("StRefPic: delta_idx_minus1", &self.delta_idx_minus1, 63);
        decoder_formatted_print("StRefPic: delta_rps_sign", &self.delta_rps_sign, 63);
        decoder_formatted_print(
            "StRefPic: abs_delta_rps_minus1",
            &self.abs_delta_rps_minus1,
            63,
        );
        decoder_formatted_print(
            "StRefPic: used_by_curr_pic_flag",
            &self.used_by_curr_pic_flag,
            63,
        );
        decoder_formatted_print("StRefPic: use_delta_flag", &self.use_delta_flag, 63);
        decoder_formatted_print("StRefPic: num_negative_pics", &self.num_negative_pics, 63);
        decoder_formatted_print("StRefPic: num_positive_pics", &self.num_positive_pics, 63);
        decoder_formatted_print(
            "StRefPic: delta_poc_s0_minus1",
            &self.delta_poc_s0_minus1,
            63,
        );
        decoder_formatted_print(
            "StRefPic: used_by_curr_pic_s0_flag",
            &self.used_by_curr_pic_s0_flag,
            63,
        );
        decoder_formatted_print(
            "StRefPic: delta_poc_s1_minus1",
            &self.delta_poc_s1_minus1,
            63,
        );
        decoder_formatted_print(
            "StRefPic: used_by_curr_pic_s1_flag",
            &self.used_by_curr_pic_s1_flag,
            63,
        );
        decoder_formatted_print("StRefPic: num_delta_pics", &self.num_delta_pics, 63);
    }

    pub fn encoder_pretty_print(&self) {
        encoder_formatted_print(
            "StRefPic: inter_ref_pic_set_prediction_flag",
            &self.inter_ref_pic_set_prediction_flag,
            63,
        );
        encoder_formatted_print("StRefPic: delta_idx_minus1", &self.delta_idx_minus1, 63);
        encoder_formatted_print("StRefPic: delta_rps_sign", &self.delta_rps_sign, 63);
        encoder_formatted_print(
            "StRefPic: abs_delta_rps_minus1",
            &self.abs_delta_rps_minus1,
            63,
        );
        encoder_formatted_print(
            "StRefPic: used_by_curr_pic_flag",
            &self.used_by_curr_pic_flag,
            63,
        );
        encoder_formatted_print("StRefPic: use_delta_flag", &self.use_delta_flag, 63);
        encoder_formatted_print("StRefPic: num_negative_pics", &self.num_negative_pics, 63);
        encoder_formatted_print("StRefPic: num_positive_pics", &self.num_positive_pics, 63);
        encoder_formatted_print(
            "StRefPic: delta_poc_s0_minus1",
            &self.delta_poc_s0_minus1,
            63,
        );
        encoder_formatted_print(
            "StRefPic: used_by_curr_pic_s0_flag",
            &self.used_by_curr_pic_s0_flag,
            63,
        );
        encoder_formatted_print(
            "StRefPic: delta_poc_s1_minus1",
            &self.delta_poc_s1_minus1,
            63,
        );
        encoder_formatted_print(
            "StRefPic: used_by_curr_pic_s1_flag",
            &self.used_by_curr_pic_s1_flag,
            63,
        );
        encoder_formatted_print("StRefPic: num_delta_pics", &self.num_delta_pics, 63);
    }
}

impl Default for ShortTermRefPic {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct H265HRDParameters {
    pub nal_hrd_parameters_present_flag : bool, // u(1)
    pub vcl_hrd_parameters_present_flag : bool, // u(1)
    pub sub_pic_hrd_params_present_flag : bool, // u(1)
    pub tick_divisor_minus2 : u8, // u(8)
    pub du_cpb_removal_delay_increment_length_minus1 : u8, // u(5)
    pub sub_pic_cpb_params_in_pic_timing_sei_flag : bool, // u(1)
    pub dpb_output_delay_du_length_minus1 : u8, // u(5)
    pub bit_rate_scale : u8, // u(4)
    pub cpb_size_scale : u8, // u(4)
    pub cpb_size_du_scale : u8, // u(4)
    pub initial_cpb_removal_delay_length_minus1 : u8, // u(5)
    pub au_cpb_removal_delay_length_minus1 : u8, // u(5)
    pub dpb_output_delay_length_minus1 : u8, // u(5)
    pub fixed_pic_rate_general_flag : Vec<bool>, // u(1)
    pub fixed_pic_rate_within_cvs_flag : Vec<bool>, // u(1)
    pub elemental_duration_in_tc_minus1 : Vec<u32>, // ue(v)
    pub low_delay_hrd_flag : Vec<bool>, // u(1)
    pub cpb_cnt_minus1 : Vec<u32>, // ue(v)
    pub nal_sub_layer_hrd_parameters : Vec<H265SubLayerHRDParameters>,
    pub vcl_sub_layer_hrd_parameters : Vec<H265SubLayerHRDParameters>,
}

impl H265HRDParameters {
    pub fn new() -> H265HRDParameters {
        H265HRDParameters {
            nal_hrd_parameters_present_flag : false,
            vcl_hrd_parameters_present_flag : false,
            sub_pic_hrd_params_present_flag : false,
            tick_divisor_minus2 : 0,
            du_cpb_removal_delay_increment_length_minus1 : 0,
            sub_pic_cpb_params_in_pic_timing_sei_flag : false,
            dpb_output_delay_du_length_minus1 : 0,
            bit_rate_scale : 0,
            cpb_size_scale : 0,
            cpb_size_du_scale : 0,
            initial_cpb_removal_delay_length_minus1 : 0,
            au_cpb_removal_delay_length_minus1 : 0,
            dpb_output_delay_length_minus1 : 0,
            fixed_pic_rate_general_flag : Vec::new(),
            fixed_pic_rate_within_cvs_flag : Vec::new(),
            elemental_duration_in_tc_minus1 : Vec::new(),
            low_delay_hrd_flag : Vec::new(),
            cpb_cnt_minus1 : Vec::new(),
            nal_sub_layer_hrd_parameters : Vec::new(),
            vcl_sub_layer_hrd_parameters : Vec::new(),
        }
    }

    #[allow(unused)]
    pub fn debug_print(&self) {}

    #[allow(unused)]
    pub fn encoder_pretty_print(&self) {}
}

impl Default for H265HRDParameters {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct H265SubLayerHRDParameters {
    pub bit_rate_value_minus1 : Vec<u32>, // ue(v)
    pub cpb_size_value_minus1 : Vec<u32>, // ue(v)
    pub cpb_size_du_value_minus1 : Vec<u32>, // ue(v)
    pub bit_rate_du_value_minus1 : Vec<u32>, // ue(v)
    pub cbr_flag : Vec<bool>, // 
}

impl H265SubLayerHRDParameters {
    pub fn new() -> H265SubLayerHRDParameters {
        H265SubLayerHRDParameters {
            bit_rate_value_minus1 : Vec::new(),
            cpb_size_value_minus1 : Vec::new(),
            cpb_size_du_value_minus1 : Vec::new(),
            bit_rate_du_value_minus1 : Vec::new(),
            cbr_flag : Vec::new(),
        }
    }

    #[allow(unused)]
    pub fn debug_print(&self) {}

    #[allow(unused)]
    pub fn encoder_pretty_print(&self) {}
}

impl Default for H265SubLayerHRDParameters {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct H265VideoParameterSet {
    pub vps_video_parameter_set_id : u8,      // u(4)
    pub vps_base_layer_internal_flag : bool,  // u(1)
    pub vps_base_layer_available_flag : bool, // u(1)
    pub vps_max_layers_minus1 : u8,           // u(6)
    pub vps_max_sub_layers_minus1 : u8,       // u(3)
    pub vps_temporal_id_nesting_flag : bool,  // u(1)
    pub vps_reserved_0xffff_16bits : u32,     // u(16)
    pub ptl : ProfileTierLevel, //
    pub vps_sub_layer_ordering_info_present_flag : bool, // u(1)
    pub vps_max_dec_pic_buffering_minus1 : Vec<u32>, // ue(v)
    pub vps_max_num_reorder_pics : Vec<u32>,         // ue(v)
    pub vps_max_latency_increase_plus1 : Vec<u32>,   // ue(v)
    pub vps_max_layer_id : u8, // u(6)
    pub vps_num_layer_sets_minus1 : u32, // ue(v)
    pub layer_id_included_flag: Vec<Vec<bool>>, //  u(1)
    pub vps_timing_info_present_flag : bool, // u(1)
    pub vps_num_units_in_tick : u32, // u(32)
    pub vps_time_scale : u32, // u(32)
    pub vps_poc_proportional_to_timing_flag : bool, // u(1)
    pub vps_num_ticks_poc_diff_one_minus1 : u32, // ue(v)
    pub vps_num_hrd_parameters : u32, // ue(v)
    pub hrd_layer_set_idx : Vec<u32>, //  ue(v)
    pub cprms_present_flag : Vec<bool>, // u(1)
    pub hrd_parameters : Vec<H265HRDParameters>,
    pub vps_extension_flag : bool, // u(1)
    pub vps_extension_data_flag : bool, // u(1)
}

impl H265VideoParameterSet {
    pub fn new() -> H265VideoParameterSet {
        H265VideoParameterSet {
            vps_video_parameter_set_id : 0,
            vps_base_layer_internal_flag : false,
            vps_base_layer_available_flag : false,
            vps_max_layers_minus1 : 0,
            vps_max_sub_layers_minus1 : 0,
            vps_temporal_id_nesting_flag : false,
            vps_reserved_0xffff_16bits : 0,
            ptl : ProfileTierLevel::new(),
            vps_sub_layer_ordering_info_present_flag : false,
            vps_max_dec_pic_buffering_minus1 : Vec::new(),
            vps_max_num_reorder_pics : Vec::new(),
            vps_max_latency_increase_plus1 : Vec::new(),
            vps_max_layer_id : 0,
            vps_num_layer_sets_minus1 : 0,
            layer_id_included_flag : Vec::new(),
            vps_timing_info_present_flag : false,
            vps_num_units_in_tick : 0,
            vps_time_scale : 0,
            vps_poc_proportional_to_timing_flag : false,
            vps_num_ticks_poc_diff_one_minus1 : 0,
            vps_num_hrd_parameters : 0,
            hrd_layer_set_idx : Vec::new(),
            cprms_present_flag : Vec::new(),
            hrd_parameters : Vec::new(),
            vps_extension_flag : false,
            vps_extension_data_flag : false,
        }
    }

    #[allow(unused)]
    pub fn debug_print(&self) {}

    #[allow(unused)]
    pub fn encoder_pretty_print(&self) {}
}

impl Default for H265VideoParameterSet {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct H265PicParameterSet {
    pub pps_pic_parameter_set_id : u32,                 // ue(v)
    pub pps_seq_parameter_set_id : u32,                 // ue(v)
    pub dependent_slice_segments_enabled_flag : bool,   // u(1)
    pub output_flag_present_flag : bool,                // u(1)
    pub num_extra_slice_header_bits : u8,               // u(3)
    pub sign_data_hiding_enabled_flag : bool,           // u(1)
    pub cabac_init_present_flag : bool,                 // u(1)
    pub num_ref_idx_l0_default_active_minus1 : u32,     // ue(v)
    pub num_ref_idx_l1_default_active_minus1 : u32,     // ue(v)
    pub init_qp_minus26 : i32,                          // se(v)
    pub constrained_intra_pred_flag : bool,             // u(1)
    pub transform_skip_enabled_flag : bool,             // u(1)
    pub cu_qp_delta_enabled_flag : bool,                // u(1)
    pub diff_cu_qp_delta_depth : u32,                   // ue(v)
    pub pps_cb_qp_offset : i32,                         // se(v)
    pub pps_cr_qp_offset : i32,                         // se(v)
    pub pps_slice_chroma_qp_offsets_present_flag : bool, // u(1)
    pub weighted_pred_flag : bool,                      // u(1)
    pub weighted_bipred_flag : bool,                    // u(1)
    pub transquant_bypass_enabled_flag : bool,          // u(1)
    pub tiles_enabled_flag : bool,                      // u(1)
    pub entropy_coding_sync_enabled_flag : bool,        // u(1)
    pub num_tile_columns_minus1 : u32,                  // ue(v)
    pub num_tile_rows_minus1 : u32,                     // ue(v)
    pub uniform_spacing_flag : bool,                    // u(1)
    pub column_width_minus1 : Vec<u32>,                 //[ i ] ue(v)
    pub row_height_minus1 : Vec<u32>,                   //[ i ] ue(v)
    pub loop_filter_across_tiles_enabled_flag : bool,   // u(1)
    pub pps_loop_filter_across_slices_enabled_flag : bool, // u(1)
    pub deblocking_filter_control_present_flag : bool,  // u(1)
    pub deblocking_filter_override_enabled_flag : bool, // u(1)
    pub pps_deblocking_filter_disabled_flag : bool,     // u(1)
    pub pps_beta_offset_div2 : i32,                     // se(v)
    pub pps_tc_offset_div2 : i32,                       // se(v)
    pub pps_scaling_list_data_present_flag : bool,      // u(1)
    pub scaling_list_data : ScalingListData,
    pub lists_modification_present_flag : bool,         // u(1)
    pub log2_parallel_merge_level_minus2 : u32,         // ue(v)
    pub slice_segment_header_extension_present_flag : bool, // u(1)
    pub pps_extension_present_flag : bool,              // u(1)
    pub pps_range_extension_flag : bool,                // u(1)
    pub pps_multilayer_extension_flag : bool,           // u(1)
    pub pps_3d_extension_flag : bool,                   // u(1)
    pub pps_scc_extension_flag : bool,                  // u(1)
    pub pps_extension_4bits : u8,                       // u(4)

}

impl H265PicParameterSet {
    pub fn new() -> H265PicParameterSet {
        H265PicParameterSet {
            pps_pic_parameter_set_id : 0,
            pps_seq_parameter_set_id : 0,
            dependent_slice_segments_enabled_flag : false,
            output_flag_present_flag : false,
            num_extra_slice_header_bits : 0,
            sign_data_hiding_enabled_flag : false,
            cabac_init_present_flag : false,
            num_ref_idx_l0_default_active_minus1 : 0,
            num_ref_idx_l1_default_active_minus1 : 0,
            init_qp_minus26 : 0,
            constrained_intra_pred_flag : false,
            transform_skip_enabled_flag : false,
            cu_qp_delta_enabled_flag : false,
            diff_cu_qp_delta_depth : 0,
            pps_cb_qp_offset : 0,
            pps_cr_qp_offset : 0,
            pps_slice_chroma_qp_offsets_present_flag : false,
            weighted_pred_flag : false,
            weighted_bipred_flag : false,
            transquant_bypass_enabled_flag : false,
            tiles_enabled_flag : false,
            entropy_coding_sync_enabled_flag : false,
            num_tile_columns_minus1 : 0,
            num_tile_rows_minus1 : 0,
            uniform_spacing_flag : false,
            column_width_minus1 : Vec::new(),
            row_height_minus1 : Vec::new(),
            loop_filter_across_tiles_enabled_flag : false,
            pps_loop_filter_across_slices_enabled_flag : false,
            deblocking_filter_control_present_flag : false,
            deblocking_filter_override_enabled_flag : false,
            pps_deblocking_filter_disabled_flag : false,
            pps_beta_offset_div2 : 0,
            pps_tc_offset_div2 : 0,
            pps_scaling_list_data_present_flag : false,
            scaling_list_data : ScalingListData::new(),
            lists_modification_present_flag : false,
            log2_parallel_merge_level_minus2 : 0,
            slice_segment_header_extension_present_flag : false,
            pps_extension_present_flag : false,
            pps_range_extension_flag : false,
            pps_multilayer_extension_flag : false,
            pps_3d_extension_flag : false,
            pps_scc_extension_flag : false,
            pps_extension_4bits : 0,
        }
    }
}

impl Default for H265PicParameterSet {
    fn default() -> Self {
        Self::new()
    }
}