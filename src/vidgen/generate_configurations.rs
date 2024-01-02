//! Data Structures containing ranges for random value generation.

use crate::common::data_structures::KNOWN_UUIDS;
use crate::vidgen::film::FilmState;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

/// Maintains ranges for bool types
///
/// Generate a number between [min, max].
/// If the number is greater than or equal to
/// the threshold then return true
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomBoolRange {
    pub min: u32,
    pub max: u32,
    pub threshold: u32,
}

impl RandomBoolRange {
    // If the generated number is greater than or equal to the threshold
    // then return true, else return false;
    pub fn sample(&self, film: &mut FilmState) -> bool {
        film.read_film_bool(self.min, self.max, self.threshold)
    }
    pub fn new(min: u32, max: u32, threshold: u32) -> RandomBoolRange {
        RandomBoolRange {
            min: min,
            max: max,
            threshold: threshold,
        }
    }
}

/// Maintains ranges for i32 type
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomI32Range {
    pub min: i32,
    pub max: i32,
}

impl RandomI32Range {
    pub fn sample(&self, film: &mut FilmState) -> i32 {
        film.read_film_i32(self.min, self.max)
    }
    pub fn new(min: i32, max: i32) -> RandomI32Range {
        RandomI32Range { min: min, max: max }
    }
}

/// Maintains ranges for i32 type  with dependent range sampling
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomDependentI32Range {
    pub min: i32,
    pub max: i32,
    pub use_dependency: bool,
}

impl RandomDependentI32Range {
    pub fn sample(&self, dependent_min: i32, dependent_max: i32, film: &mut FilmState) -> i32 {
        if self.use_dependency {
            film.read_film_i32(dependent_min, dependent_max)
        } else {
            film.read_film_i32(self.min, self.max)
        }
    }
    pub fn non_dependent_sample(&self, film: &mut FilmState) -> i32 {
        film.read_film_i32(self.min, self.max)
    }
    pub fn new(min: i32, max: i32, use_dependency: bool) -> RandomDependentI32Range {
        RandomDependentI32Range {
            min: min,
            max: max,
            use_dependency: use_dependency,
        }
    }
}

/// Maintains ranges for u32 type
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomU32Range {
    pub min: u32,
    pub max: u32,
}

impl RandomU32Range {
    pub fn sample(&self, film: &mut FilmState) -> u32 {
        film.read_film_u32(self.min, self.max)
    }

    pub fn sample_custom_max(&self, custom_max: u32, film: &mut FilmState) -> u32 {
        if custom_max < self.max as u32 {
            film.read_film_u32(0, custom_max)
        } else {
            self.sample(film)
        }
    }

    pub fn new(min: u32, max: u32) -> RandomU32Range {
        RandomU32Range { min: min, max: max }
    }
}

/// Maintains ranges for u32 type with dependent range sampling
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomDependentU32Range {
    pub min: u32,
    pub max: u32,
    pub use_dependency: bool,
}

impl RandomDependentU32Range {
    pub fn sample(&self, dependent_min: u32, dependent_max: u32, film: &mut FilmState) -> u32 {
        if self.use_dependency {
            film.read_film_u32(dependent_min, dependent_max)
        } else {
            film.read_film_u32(self.min, self.max)
        }
    }

    pub fn new(min: u32, max: u32, use_dependency: bool) -> RandomDependentU32Range {
        RandomDependentU32Range {
            min: min,
            max: max,
            use_dependency: use_dependency,
        }
    }
}

/// Samples from an enum
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RandomU32Enum {
    pub values: Vec<u32>,
}

impl RandomU32Enum {
    pub fn sample(&self, film: &mut FilmState) -> u32 {
        if self.values.len() > 0 {
            let idx = film.read_film_u32(0, self.values.len() as u32 - 1);
            self.values[idx as usize]
        } else {
            0
        }
    }

    pub fn new(values: Vec<u32>) -> RandomU32Enum {
        RandomU32Enum {
            values: values.clone(),
        }
    }
}

/// Macroblock syntax elements
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RandomMBRange {
    // CABAC residuals
    // coded_block_flag is always set to true
    pub significant_coeff_flag: RandomBoolRange, // bool
    pub last_significant_coeff_flag: RandomBoolRange, // bool
    pub coeff_abs_level_minus1: RandomU32Range,  // u32
    pub coeff_sign_flag: RandomBoolRange,        // bool
    // CAVLC residuals
    // coeff_token is made up of total_coeff, trailing_ones, n_c
    // n_c is context adaptive, so we just randomize total_coeff and trailing_ones
    pub total_coeff: RandomU32Range,   // usize - can be u32
    pub trailing_ones: RandomU32Range, // usize - can be u32
    pub trailing_ones_sign_flag: RandomBoolRange, // bool
    pub level_prefix: RandomU32Range,  // u32
    pub level_suffix: RandomU32Range,  // u32
    pub total_zeros: RandomU32Range,   // usize - can be u32
    pub run_before: RandomU32Range,    // usize - can be u32
    // mb_pred
    pub prev_intra4x4_pred_mode_flag: RandomBoolRange, // bool
    pub rem_intra4x4_pred_mode: RandomU32Range,        // u32; value is truncated to 3 bits
    pub prev_intra8x8_pred_mode_flag: RandomBoolRange, // bool
    pub rem_intra8x8_pred_mode: RandomU32Range,        // u32; value is truncated to 3 bits
    pub intra_chroma_pred_mode: RandomU32Range,        // u8; value is truncated to 2 bits
    // mb_pred and sub_mb_pred
    pub ref_idx_l0: RandomU32Range, // u32; this is an index into a list 0 reference buffer; large values can lead to crashes
    pub ref_idx_l1: RandomU32Range, // u32; same as above for list 1
    pub mvd_l0: RandomI32Range,     // i32
    pub mvd_l1: RandomI32Range,     // i32
    // slice_data
    pub mb_skip_flag: RandomBoolRange,            // bool
    pub mb_skip_run: RandomU32Range,              // u32
    pub mb_field_decoding_flag: RandomBoolRange,  // bool
    pub mb_i_type: RandomU32Enum,                 // follows Table 7-11
    pub mb_si_type: RandomU32Enum,                // follows Table 7-12
    pub mb_p_type: RandomU32Enum,                 // follows Table 7-13
    pub mb_b_type: RandomU32Enum,                 // follows Table 7-14
    pub sub_mb_type_p: RandomU32Enum,             // follows Table 7-17
    pub sub_mb_type_b: RandomU32Enum,             // follows Table 7-18
    pub pcm_sample_luma: RandomU32Range, // u32; TODO: change to dependent on bit_depth_luma
    pub pcm_sample_chroma: RandomU32Range, // u32; TODO: change to dependent. Values are clipped to the range [0, 2^bit_depth_chroma) (bit_depth is usually 8)
    pub transform_size_8x8_flag: RandomBoolRange, // bool
    pub coded_block_pattern: RandomU32Range, // u32; value is truncated to 6 bits and max read value is 47 (top two bits only go to 2)
    pub mb_qp_delta: RandomI32Range, // i32; too big a range was killing our decoding too quickly; range: −(26+QpBdOffsetY/2) to+(25+QpBdOffsetY/2)
    // Biases
    pub bias_b_p_no_residue: RandomBoolRange, // if true, B and P slices have no residue
    pub bias_zero_mb_ref_idx_l0: RandomBoolRange, // if true, ref_idx_l0 is set to 0
    pub bias_zero_mb_ref_idx_l1: RandomBoolRange, // if true, ref_idx_l1 is set to 0
}

impl RandomMBRange {
    /// Returns a new RandomMBRange with variables in a modest range
    pub fn new() -> RandomMBRange {
        RandomMBRange {
            // CABAC residuals
            significant_coeff_flag: RandomBoolRange::new(0, 8, 3), // more than often it's significant
            last_significant_coeff_flag: RandomBoolRange::new(0, 8, 7), // more than often it's NOT the last significant value
            coeff_abs_level_minus1: RandomU32Range::new(0, 65535),
            coeff_sign_flag: RandomBoolRange::new(0, 1, 1),
            // CAVLC residuals
            total_coeff: RandomU32Range::new(0, 16),
            trailing_ones: RandomU32Range::new(0, 3),
            trailing_ones_sign_flag: RandomBoolRange::new(0, 1, 1),
            level_prefix: RandomU32Range::new(0, 31), // TODO: explore larger values
            level_suffix: RandomU32Range::new(0, 1000),
            total_zeros: RandomU32Range::new(0, 15),
            run_before: RandomU32Range::new(0, 14),
            // mb_pred
            prev_intra4x4_pred_mode_flag: RandomBoolRange::new(0, 1, 1),
            rem_intra4x4_pred_mode: RandomU32Range::new(0, 7), // binary is truncated to 3 bits
            prev_intra8x8_pred_mode_flag: RandomBoolRange::new(0, 1, 1),
            rem_intra8x8_pred_mode: RandomU32Range::new(0, 7), // binary is truncated to 3 bits
            intra_chroma_pred_mode: RandomU32Range::new(0, 3), // binary is truncated to 2 bits
            // mb_pred and sub_mb_pred
            ref_idx_l0: RandomU32Range::new(0, 47), // the spec says the max should be 32; these are unary encoded so too large values takes forever to encode
            ref_idx_l1: RandomU32Range::new(0, 47),
            mvd_l0: RandomI32Range::new(-100000, 100000), // spec says [-8192.5, 8192] ; the point comes from some shifts
            mvd_l1: RandomI32Range::new(-100000, 100000),
            // slice_data
            mb_skip_flag: RandomBoolRange::new(0, 8, 8), // don't skip so often
            mb_skip_run: RandomU32Range::new(0, 100000),
            mb_field_decoding_flag: RandomBoolRange::new(0, 1, 1),
            mb_i_type: RandomU32Enum::new(vec![
                0,  // I_NxN
                1,  // I_16x16_0_0_0
                2,  // I_16x16_1_0_0
                3,  // I_16x16_2_0_0
                4,  // I_16x16_3_0_0
                5,  // I_16x16_0_1_0
                6,  // I_16x16_1_1_0
                7,  // I_16x16_2_1_0
                8,  // I_16x16_3_1_0
                9,  // I_16x16_0_2_0
                10, // I_16x16_1_2_0
                11, // I_16x16_2_2_0
                12, // I_16x16_3_2_0
                13, // I_16x16_0_0_1
                14, // I_16x16_1_0_1
                15, // I_16x16_2_0_1
                16, // I_16x16_3_0_1
                17, // I_16x16_0_1_1
                18, // I_16x16_1_1_1
                19, // I_16x16_2_1_1
                20, // I_16x16_3_1_1
                21, // I_16x16_0_2_1
                22, // I_16x16_1_2_1
                23, // I_16x16_2_2_1
                24, // I_16x16_3_2_1
                25, // I_PCM
            ]),
            mb_si_type: RandomU32Enum::new(vec![
                0,  // SI,
                1,  // I_NxN,
                2,  // I_16x16_0_0_0,
                3,  // I_16x16_1_0_0,
                4,  // I_16x16_2_0_0,
                5,  // I_16x16_3_0_0,
                6,  // I_16x16_0_1_0,
                7,  // I_16x16_1_1_0,
                8,  // I_16x16_2_1_0,
                9,  // I_16x16_3_1_0,
                10, // I_16x16_0_2_0,
                11, // I_16x16_1_2_0,
                12, // I_16x16_2_2_0,
                13, // I_16x16_3_2_0,
                14, // I_16x16_0_0_1,
                15, // I_16x16_1_0_1,
                16, // I_16x16_2_0_1,
                17, // I_16x16_3_0_1,
                18, // I_16x16_0_1_1,
                19, // I_16x16_1_1_1,
                20, // I_16x16_2_1_1,
                21, // I_16x16_3_1_1,
                22, // I_16x16_0_2_1,
                23, // I_16x16_1_2_1,
                24, // I_16x16_2_2_1,
                25, // I_16x16_3_2_1,
                26, // I_PCM,
            ]),
            mb_p_type: RandomU32Enum::new(vec![
                0, // P_L016x16,
                1, // P_L0L016x8,
                2, // P_L0L08x16,
                3, // P_8x8,
                // 4,  // P_8x8ref0, // This mb_p_type is not allowed
                5,  // I_NxN,
                6,  // I_16x16_0_0_0,
                7,  // I_16x16_1_0_0,
                8,  // I_16x16_2_0_0,
                9,  // I_16x16_3_0_0,
                10, // I_16x16_0_1_0,
                11, // I_16x16_1_1_0,
                12, // I_16x16_2_1_0,
                13, // I_16x16_3_1_0,
                14, // I_16x16_0_2_0,
                15, // I_16x16_1_2_0,
                16, // I_16x16_2_2_0,
                17, // I_16x16_3_2_0,
                18, // I_16x16_0_0_1,
                19, // I_16x16_1_0_1,
                20, // I_16x16_2_0_1,
                21, // I_16x16_3_0_1,
                22, // I_16x16_0_1_1,
                23, // I_16x16_1_1_1,
                24, // I_16x16_2_1_1,
                25, // I_16x16_3_1_1,
                26, // I_16x16_0_2_1,
                27, // I_16x16_1_2_1,
                28, // I_16x16_2_2_1,
                29, // I_16x16_3_2_1,
                30, // I_PCM,
            ]),
            mb_b_type: RandomU32Enum::new(vec![
                0,  // B_Direct16x16
                1,  // B_L016x16
                2,  // B_L116x16
                3,  // B_Bi16x16
                4,  // B_L0L016x8
                5,  // B_L0L08x16
                6,  // B_L1L116x8
                7,  // B_L1L18x16
                8,  // B_L0L116x8
                9,  // B_L0L18x16
                10, // B_L1L016x8
                11, // B_L1L08x16
                12, // B_L0Bi16x8
                13, // B_L0Bi8x16
                14, // B_L1Bi16x8
                15, // B_L1Bi8x16
                16, // B_BiL016x8
                17, // B_BiL08x16
                18, // B_BiL116x8
                19, // B_BiL18x16
                20, // B_BiBi16x8
                21, // B_BiBi8x16
                22, // B_8x8
                23, // I_NxN
                24, // I_16x16_0_0_0
                25, // I_16x16_1_0_0
                26, // I_16x16_2_0_0
                27, // I_16x16_3_0_0
                28, // I_16x16_0_1_0
                29, // I_16x16_1_1_0
                30, // I_16x16_2_1_0
                31, // I_16x16_3_1_0
                32, // I_16x16_0_2_0
                33, // I_16x16_1_2_0
                34, // I_16x16_2_2_0
                35, // I_16x16_3_2_0
                36, // I_16x16_0_0_1
                37, // I_16x16_1_0_1
                38, // I_16x16_2_0_1
                39, // I_16x16_3_0_1
                40, // I_16x16_0_1_1
                41, // I_16x16_1_1_1
                42, // I_16x16_2_1_1
                43, // I_16x16_3_1_1
                44, // I_16x16_0_2_1
                45, // I_16x16_1_2_1
                46, // I_16x16_2_2_1
                47, // I_16x16_3_2_1
                48, // I_PCM
            ]),
            sub_mb_type_p: RandomU32Enum::new(vec![
                0, // P_L0_8x8
                1, // P_L0_8x4
                2, // P_L0_4x8
                3, // P_L0_4x4
            ]),
            sub_mb_type_b: RandomU32Enum::new(vec![
                0,  // B_Direct_8x8
                1,  // B_L0_8x8
                2,  // B_L1_8x8
                3,  // B_Bi_8x8
                4,  // B_L0_8x4
                5,  // B_L0_4x8
                6,  // B_L1_8x4
                7,  // B_L1_4x8
                8,  // B_Bi_8x4
                9,  // B_Bi_4x8
                10, // B_L0_4x4
                11, // B_L1_4x4
                12, // B_Bi_4x4
            ]),
            pcm_sample_luma: RandomU32Range::new(0, 255),
            pcm_sample_chroma: RandomU32Range::new(0, 255),
            transform_size_8x8_flag: RandomBoolRange::new(0, 1, 1),
            coded_block_pattern: RandomU32Range::new(0, 47), // truncated to 6 bits; for the top bits, 10 and 11 both mean 2
            mb_qp_delta: RandomI32Range::new(-26, 24),
            bias_b_p_no_residue: RandomBoolRange::new(0, 50, 1), // 49 out of 50 times, B/P slices have no residue
            bias_zero_mb_ref_idx_l0: RandomBoolRange::new(0, 20, 1), // 19 out of 20 times, ref_idx_l0 will be 0
            bias_zero_mb_ref_idx_l1: RandomBoolRange::new(0, 20, 1), // 19 out of 20 times, ref_idx_l1 will be 0
        }
    }
}

impl Default for RandomMBRange {
    fn default() -> Self {
        Self::new()
    }
}

/// Slice header syntax elements
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RandomSliceHeaderRange {
    pub first_mb_in_slice: RandomDependentU32Range, //ue(v)
    pub slice_type: RandomU32Enum,                  //ue(v)
    pub colour_plane_id: RandomU32Range,            //u(2)
    pub frame_num: RandomDependentU32Range, //u(v) -- depends on sps.log2_max_frame_num_minus4
    pub field_pic_flag: RandomBoolRange,
    pub bottom_field_flag: RandomBoolRange,
    pub idr_pic_id: RandomU32Range, //ue(v)
    pub pic_order_cnt_lsb: RandomU32Range,
    pub delta_pic_order_cnt_bottom: RandomI32Range, //se(v)
    pub delta_pic_order_cnt: RandomI32Range,        //se(v) [2]
    pub redundant_pic_cnt: RandomU32Range,          //ue(v)
    pub direct_spatial_mv_pred_flag: RandomBoolRange,
    pub num_ref_idx_active_override_flag: RandomBoolRange,
    pub num_ref_idx_l0_active_minus1: RandomDependentU32Range, //ue(v)
    pub num_ref_idx_l1_active_minus1: RandomDependentU32Range, //ue(v)
    // ref_pic_list_modification
    pub ref_pic_list_modification_flag_l0: RandomBoolRange,
    pub number_of_modifications_l0: RandomU32Range, // the number of modifications to include
    pub modification_of_pic_nums_idc_l0: RandomU32Range,
    pub abs_diff_pic_num_minus1_l0: RandomU32Range,
    pub long_term_pic_num_l0: RandomU32Range,
    pub ref_pic_list_modification_flag_l1: RandomBoolRange,
    pub number_of_modifications_l1: RandomU32Range, // the number of modifications to include
    pub modification_of_pic_nums_idc_l1: RandomU32Range,
    pub abs_diff_pic_num_minus1_l1: RandomU32Range,
    pub long_term_pic_num_l1: RandomU32Range,
    // ref_pic_list_mvc_modification
    pub abs_diff_view_idx_minus1_l0: RandomU32Range, // MVC only
    pub abs_diff_view_idx_minus1_l1: RandomU32Range, // MVC only
    // pred_weight_table
    pub luma_log2_weight_denom: RandomU32Range,
    pub chroma_log2_weight_denom: RandomU32Range,
    pub luma_weight_l0_flag: RandomBoolRange,
    pub luma_weight_l0: RandomI32Range, //len = num_ref_idx_l1_active_minus1
    pub luma_offset_l0: RandomI32Range, //len = num_ref_idx_l1_active_minus1
    pub chroma_weight_l0_flag: RandomBoolRange,
    pub chroma_weight_l0: RandomI32Range, // 2 elements each
    pub chroma_offset_l0: RandomI32Range,
    pub luma_weight_l1_flag: RandomBoolRange,
    pub luma_weight_l1: RandomI32Range, //len = num_ref_idx_l1_active_minus1
    pub luma_offset_l1: RandomI32Range, //len = num_ref_idx_l1_active_minus1
    pub chroma_weight_l1_flag: RandomBoolRange,
    pub chroma_weight_l1: RandomI32Range, // 2 elements each
    pub chroma_offset_l1: RandomI32Range, //
    // dec_ref_pic_marking
    pub no_output_of_prior_pics_flag: RandomBoolRange,
    pub long_term_reference_flag: RandomBoolRange,
    pub adaptive_ref_pic_marking_mode_flag: RandomBoolRange,
    pub number_of_mem_ops: RandomU32Range, // number of memory operations to include
    pub memory_management_control_operation: RandomU32Range,
    pub difference_of_pic_nums_minus1: RandomU32Range,
    pub long_term_pic_num: RandomU32Range,
    pub long_term_frame_idx: RandomU32Range,
    pub max_long_term_frame_idx_plus1: RandomU32Range,
    //
    pub cabac_init_idc: RandomU32Range,          //ue(v)
    pub slice_qp_delta: RandomDependentI32Range, //se(v)
    pub sp_for_switch_flag: RandomBoolRange,
    pub slice_qs_delta: RandomI32Range,                //se(v)
    pub disable_deblocking_filter_idc: RandomU32Range, //ue(v)
    pub slice_alpha_c0_offset_div2: RandomI32Range,    //se(v)
    pub slice_beta_offset_div2: RandomI32Range,        //se(v)
    pub slice_group_change_cycle: RandomU32Range,      // TODO: make dependent
    // biases in video generation
    pub bias_i_slice: RandomBoolRange, // if True, set to I slice, else sample a random slice type.
    pub bias_non_rand_frame_num: RandomBoolRange, // if True set to slice index, else randomize
    pub bias_idr_zero_frame_num: RandomBoolRange, // if True, set IDR frame_num to 0, else use slice index
    pub bias_zero_first_mb_in_slice: RandomBoolRange, // if True, first_mb_in_slice is 0, else sampled
    pub bias_slice_qp_y_top_bound: RandomBoolRange,   // if True, slice_qp_y <= 51
    pub bias_slice_qp_y_bottom_bound: RandomBoolRange, // if True, slice_qp_y >= 0
}

impl RandomSliceHeaderRange {
    pub fn new() -> RandomSliceHeaderRange {
        RandomSliceHeaderRange {
            first_mb_in_slice: RandomDependentU32Range::new(0, 65535, true), // Depends on max Frame size
            slice_type: RandomU32Enum::new(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]), // (0, 5) P slices; (1,6) B slices; (2,7) I slices; (3, 8) SP slices; (4, 9) SI slices
            colour_plane_id: RandomU32Range::new(0, 3), // u(2) --  only read if SPS allows it
            frame_num: RandomDependentU32Range::new(0, 31, true),
            field_pic_flag: RandomBoolRange::new(0, 1, 1),
            bottom_field_flag: RandomBoolRange::new(0, 1, 1),
            idr_pic_id: RandomU32Range::new(0, 65535),
            pic_order_cnt_lsb: RandomU32Range::new(0, 63),
            delta_pic_order_cnt_bottom: RandomI32Range::new(std::i32::MIN, std::i32::MAX), // se(v)
            delta_pic_order_cnt: RandomI32Range::new(std::i32::MIN, std::i32::MAX),        // se(v)
            redundant_pic_cnt: RandomU32Range::new(0, 31),
            direct_spatial_mv_pred_flag: RandomBoolRange::new(0, 1, 1),
            num_ref_idx_active_override_flag: RandomBoolRange::new(0, 1, 1),
            num_ref_idx_l0_active_minus1: RandomDependentU32Range::new(0, 32, true),
            num_ref_idx_l1_active_minus1: RandomDependentU32Range::new(0, 32, true),
            ref_pic_list_modification_flag_l0: RandomBoolRange::new(0, 1, 1),
            number_of_modifications_l0: RandomU32Range::new(0, 10),
            modification_of_pic_nums_idc_l0: RandomU32Range::new(0, 2), // [0, 2] - 3 is the stop condition
            abs_diff_pic_num_minus1_l0: RandomU32Range::new(0, 100),
            long_term_pic_num_l0: RandomU32Range::new(0, 50),
            ref_pic_list_modification_flag_l1: RandomBoolRange::new(0, 1, 1),
            number_of_modifications_l1: RandomU32Range::new(0, 10),
            modification_of_pic_nums_idc_l1: RandomU32Range::new(0, 2), // [0, 2] - 3 is the stop condition
            abs_diff_pic_num_minus1_l1: RandomU32Range::new(0, 100),
            long_term_pic_num_l1: RandomU32Range::new(0, 50),
            abs_diff_view_idx_minus1_l0: RandomU32Range::new(0, 100),
            abs_diff_view_idx_minus1_l1: RandomU32Range::new(0, 100),
            luma_log2_weight_denom: RandomU32Range::new(0, 100), // [0, 7]
            chroma_log2_weight_denom: RandomU32Range::new(0, 100), // [0, 7]
            luma_weight_l0_flag: RandomBoolRange::new(0, 1, 1),
            luma_weight_l0: RandomI32Range::new(-128, 127),
            luma_offset_l0: RandomI32Range::new(-128, 127),
            chroma_weight_l0_flag: RandomBoolRange::new(0, 1, 1),
            chroma_weight_l0: RandomI32Range::new(-128, 127),
            chroma_offset_l0: RandomI32Range::new(-128, 127),
            luma_weight_l1_flag: RandomBoolRange::new(0, 1, 1),
            luma_weight_l1: RandomI32Range::new(-128, 127),
            luma_offset_l1: RandomI32Range::new(-128, 127),
            chroma_weight_l1_flag: RandomBoolRange::new(0, 1, 1),
            chroma_weight_l1: RandomI32Range::new(-128, 127),
            chroma_offset_l1: RandomI32Range::new(-128, 127),
            no_output_of_prior_pics_flag: RandomBoolRange::new(0, 1, 1),
            long_term_reference_flag: RandomBoolRange::new(0, 1, 1),
            adaptive_ref_pic_marking_mode_flag: RandomBoolRange::new(0, 1, 1),
            number_of_mem_ops: RandomU32Range::new(0, 10),
            memory_management_control_operation: RandomU32Range::new(1, 6), // 0 is an exit operation, which we defer
            difference_of_pic_nums_minus1: RandomU32Range::new(0, 100),
            long_term_pic_num: RandomU32Range::new(0, 100),
            long_term_frame_idx: RandomU32Range::new(0, 100),
            max_long_term_frame_idx_plus1: RandomU32Range::new(0, 100),
            cabac_init_idc: RandomU32Range::new(0, 24),
            slice_qp_delta: RandomDependentI32Range::new(-26, 25, true), // [-26, 25]
            sp_for_switch_flag: RandomBoolRange::new(0, 1, 1),
            slice_qs_delta: RandomI32Range::new(-26, 25), // [-26, 25]
            disable_deblocking_filter_idc: RandomU32Range::new(0, 2),
            slice_alpha_c0_offset_div2: RandomI32Range::new(-10, 10), // [-6, 6]
            slice_beta_offset_div2: RandomI32Range::new(-10, 10),     // [-6, 6]
            slice_group_change_cycle: RandomU32Range::new(0, 16777216), // TODO: make dependent; Range is [0, Ceil(PicSizeInMapUnits\SliceGroupChangeRate)]
            bias_i_slice: RandomBoolRange::new(0, 1, 1), // 19 out of 20 times it's an I slice
            bias_non_rand_frame_num: RandomBoolRange::new(0, 20, 1), // 19 out of 20 times, it's a random frame number
            bias_idr_zero_frame_num: RandomBoolRange::new(0, 50, 1), // 49 out of 50 times, idr frame has frame_num 0
            bias_zero_first_mb_in_slice: RandomBoolRange::new(0, 50, 1), // 49 out of 50 times, first_mb_in_slice is 0
            bias_slice_qp_y_top_bound: RandomBoolRange::new(0, 100, 1), // 99 out of 100 times, slice_qp_y <= 51
            bias_slice_qp_y_bottom_bound: RandomBoolRange::new(0, 100, 1), // 99 out of 100 times, slice_qp_y >= 0
        }
    }
}

impl Default for RandomSliceHeaderRange {
    fn default() -> Self {
        Self::new()
    }
}

/// HRD syntax elements
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomHRDRange {
    pub cpb_cnt_minus1: RandomU32Range,        // ue(v)
    pub bit_rate_scale: RandomU32Range,        // u(4)
    pub cpb_size_scale: RandomU32Range,        // u(4)
    pub bit_rate_value_minus1: RandomU32Range, // ue(v)
    pub cpb_size_value_minus1: RandomU32Range, // ue(v)
    pub cbr_flag: RandomBoolRange,             // u(1)
    pub initial_cpb_removal_delay_length_minus1: RandomU32Range, // u(5)
    pub cpb_removal_delay_length_minus1: RandomU32Range, // u(5)
    pub dpb_output_delay_length_minus1: RandomU32Range, // u(5)
    pub time_offset_length: RandomU32Range,    // u(5)
}

impl RandomHRDRange {
    pub fn new() -> RandomHRDRange {
        RandomHRDRange {
            cpb_cnt_minus1: RandomU32Range::new(0, 31), // ue(v); [0, 31]
            bit_rate_scale: RandomU32Range::new(0, 15), // u(4)
            cpb_size_scale: RandomU32Range::new(0, 15), // u(4)
            bit_rate_value_minus1: RandomU32Range::new(0, std::u32::MAX), //ue(v)
            cpb_size_value_minus1: RandomU32Range::new(0, std::u32::MAX), //ue(v)
            cbr_flag: RandomBoolRange::new(0, 1, 1),
            initial_cpb_removal_delay_length_minus1: RandomU32Range::new(0, 31), // u(5)
            cpb_removal_delay_length_minus1: RandomU32Range::new(0, 31),         // u(5)
            dpb_output_delay_length_minus1: RandomU32Range::new(0, 31),          // u(5)
            time_offset_length: RandomU32Range::new(0, 31),                      // u(5)
        }
    }
}

impl Default for RandomHRDRange {
    fn default() -> Self {
        Self::new()
    }
}

/// VUI syntax elements
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomVUIRange {
    pub aspect_ratio_info_present_flag: RandomBoolRange, // u(1)
    pub aspect_ratio_idc: RandomU32Range,                // u(8)
    pub sar_width: RandomU32Range,                       // u(16)
    pub sar_height: RandomU32Range,                      // u(16)
    pub overscan_info_present_flag: RandomBoolRange,     // u(1)
    pub overscan_appropriate_flag: RandomBoolRange,      // u(1)
    pub video_signal_type_present_flag: RandomBoolRange, // u(1)
    pub video_format: RandomU32Range,                    // u(3)
    pub video_full_range_flag: RandomBoolRange,          // u(1)
    pub colour_description_present_flag: RandomBoolRange, // u(1)
    pub colour_primaries: RandomU32Range,                // u(8)
    pub transfer_characteristics: RandomU32Range,        // u(8)
    pub matrix_coefficients: RandomU32Range,             // u(8)
    pub chroma_loc_info_present_flag: RandomBoolRange,   // u(1)
    pub chroma_sample_loc_type_top_field: RandomU32Range, // ue(v)
    pub chroma_sample_loc_type_bottom_field: RandomU32Range, // ue(v)
    pub timing_info_present_flag: RandomBoolRange,       // u(1)
    pub num_units_in_tick: RandomU32Range,               // u(32)
    pub time_scale: RandomU32Range,                      // u(32)
    pub fixed_frame_rate_flag: RandomBoolRange,          // u(1)
    pub nal_hrd_parameters_present_flag: RandomBoolRange, // u(1)
    pub vcl_hrd_parameters_present_flag: RandomBoolRange, // u(1)
    pub low_delay_hrd_flag: RandomBoolRange,             // u(1)
    pub pic_struct_present_flag: RandomBoolRange,        // u(1)
    pub bitstream_restriction_flag: RandomBoolRange,     // u(1)
    pub motion_vectors_over_pic_boundaries_flag: RandomBoolRange, // u(1)
    pub max_bytes_per_pic_denom: RandomU32Range,         // ue(v)
    pub max_bits_per_mb_denom: RandomU32Range,           // ue(v)
    pub log2_max_mv_length_horizontal: RandomU32Range,   // ue(v)
    pub log2_max_mv_length_vertical: RandomU32Range,     // ue(v)
    pub max_num_reorder_frames: RandomU32Range,          // ue(v)
    pub max_dec_frame_buffering: RandomDependentU32Range, // ue(v)
    pub random_hrd_range: RandomHRDRange,
}

impl RandomVUIRange {
    pub fn new() -> RandomVUIRange {
        RandomVUIRange {
            aspect_ratio_info_present_flag: RandomBoolRange::new(0, 1, 1),
            aspect_ratio_idc: RandomU32Range::new(0, 255), // u(8) - values described in Table E-1
            sar_width: RandomU32Range::new(0, 65535),      // u(16)
            sar_height: RandomU32Range::new(0, 65535),     // u(16)
            overscan_info_present_flag: RandomBoolRange::new(0, 1, 1),
            overscan_appropriate_flag: RandomBoolRange::new(0, 1, 1),
            video_signal_type_present_flag: RandomBoolRange::new(0, 1, 1),
            video_format: RandomU32Range::new(0, 7), // u(3)
            video_full_range_flag: RandomBoolRange::new(0, 1, 1),
            colour_description_present_flag: RandomBoolRange::new(0, 1, 1),
            colour_primaries: RandomU32Range::new(0, 255), // u(8)
            transfer_characteristics: RandomU32Range::new(0, 255), // u(8)
            matrix_coefficients: RandomU32Range::new(0, 255), // u(8)
            chroma_loc_info_present_flag: RandomBoolRange::new(0, 1, 1),
            chroma_sample_loc_type_top_field: RandomU32Range::new(0, 5), // valid ranges are from 0 to 5 inclusive
            chroma_sample_loc_type_bottom_field: RandomU32Range::new(0, 5), // valid ranges are from 0 to 5 inclusive
            timing_info_present_flag: RandomBoolRange::new(0, 1, 1),
            num_units_in_tick: RandomU32Range::new(0, std::u32::MAX), // u(32)
            time_scale: RandomU32Range::new(0, std::u32::MAX),        // u(32)
            fixed_frame_rate_flag: RandomBoolRange::new(0, 1, 1),
            nal_hrd_parameters_present_flag: RandomBoolRange::new(0, 1, 1),
            vcl_hrd_parameters_present_flag: RandomBoolRange::new(0, 1, 1),
            low_delay_hrd_flag: RandomBoolRange::new(0, 1, 1),
            pic_struct_present_flag: RandomBoolRange::new(0, 1, 1),
            bitstream_restriction_flag: RandomBoolRange::new(0, 1, 1),
            motion_vectors_over_pic_boundaries_flag: RandomBoolRange::new(0, 1, 1),
            max_bytes_per_pic_denom: RandomU32Range::new(0, 1024), // valid range is from 0 to 16
            max_bits_per_mb_denom: RandomU32Range::new(0, 1024),   // valid range is from 0 to 16
            log2_max_mv_length_horizontal: RandomU32Range::new(0, 1024), // valid range is from 0 to 15
            log2_max_mv_length_vertical: RandomU32Range::new(0, 1024),
            max_num_reorder_frames: RandomU32Range::new(0, 1024), // TODO: dependent on MaxDpbFrames
            max_dec_frame_buffering: RandomDependentU32Range::new(0, 1024, false), // dependent on VUI.max_num_reorder_frames
            random_hrd_range: RandomHRDRange::new(),
        }
    }
}

impl Default for RandomVUIRange {
    fn default() -> Self {
        Self::new()
    }
}

/// SPS syntax elements
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RandomSPSRange {
    // All profiles:
    // - 44 CAVLC 4:4:4 Intra profile
    // - 66 Baseline profile
    // - 77 Main profile
    // - 83 Scalable Baseline profile (Annex G)
    // - 86 Scalable High profile (Annex G)
    // - 88 Extended profile
    // - 100 High profile
    // - 110 High 10 profile
    // - 118 Multiview High Profile
    // - 119 Multiview Field High (Found in JM version 17, not found in spec)
    // - 122 High 4:2:2
    // - 128 Stereo High Profile (Annex H)
    // - 134 MFC High Profile (Annex H)
    // - 135 MFC Depth High Profile (Annex I)
    // - 138 Multiview Depth High profile (Annex I)
    // - 139 Enhanced Multiview Depth High profile (Annex J)
    // - 144 High 4:4:4 (removed from the spec in 2006)
    // - 244 High Predictive 4:4:4
    pub profile_idc: RandomU32Enum,
    pub profile_idc_extension: RandomU32Enum,
    pub constraint_set0_flag: RandomBoolRange,
    pub constraint_set1_flag: RandomBoolRange,
    pub constraint_set2_flag: RandomBoolRange,
    pub constraint_set3_flag: RandomBoolRange,
    pub constraint_set4_flag: RandomBoolRange,
    pub constraint_set5_flag: RandomBoolRange,
    pub reserved_zero_2bits: RandomU32Range, // u(2)
    pub level_idc: RandomU32Enum,
    pub seq_parameter_set_id: RandomU32Range, // ue(v)
    pub chroma_format_idc: RandomU32Range,    // ue(v)
    pub separate_colour_plane_flag: RandomBoolRange,
    pub bit_depth_luma_minus8: RandomU32Range,   // ue(v)
    pub bit_depth_chroma_minus8: RandomU32Range, // ue(v)
    pub qpprime_y_zero_transform_bypass_flag: RandomBoolRange,
    pub seq_scaling_matrix_present_flag: RandomBoolRange,
    pub seq_scaling_list_present_flag: RandomBoolRange,
    pub delta_scale: RandomI32Range,                       // se(v)
    pub log2_max_frame_num_minus4: RandomU32Range,         // ue(v)
    pub pic_order_cnt_type: RandomU32Range,                // ue(v)
    pub log2_max_pic_order_cnt_lsb_minus4: RandomU32Range, // ue(v) - the range of this must be between 0 to 12 inclusive
    pub delta_pic_order_always_zero_flag: RandomBoolRange,
    pub offset_for_non_ref_pic: RandomI32Range, // se(v)
    pub offset_for_top_to_bottom_field: RandomI32Range, // se(v)
    pub num_ref_frames_in_pic_order_cnt_cycle: RandomU32Range, // ue(v)
    pub offset_for_ref_frame: RandomI32Range,   // se(v)
    pub max_num_ref_frames: RandomU32Range,     // ue(v)
    pub gaps_in_frame_num_value_allowed_flag: RandomBoolRange,
    pub pic_width_in_mbs_minus1: RandomU32Range, // ue(v)
    pub pic_height_in_map_units_minus1: RandomU32Range, // ue(v)
    pub frame_mbs_only_flag: RandomBoolRange,
    pub mb_adaptive_frame_field_flag: RandomBoolRange,
    pub direct_8x8_inference_flag: RandomBoolRange,
    pub frame_cropping_flag: RandomBoolRange,
    pub frame_crop_left_offset: RandomDependentU32Range, // ue(v)
    pub frame_crop_right_offset: RandomDependentU32Range, // ue(v)
    pub frame_crop_top_offset: RandomDependentU32Range,  // ue(v)
    pub frame_crop_bottom_offset: RandomDependentU32Range, // ue(v)
    pub vui_parameters_present_flag: RandomBoolRange,
    pub random_vui_range: RandomVUIRange,
    // Bias
    pub bias_same_bit_depth: RandomBoolRange, // if true, use the same bit depth
}

impl RandomSPSRange {
    pub fn new() -> RandomSPSRange {
        RandomSPSRange {
            profile_idc: RandomU32Enum::new(vec![77, 66, 100]), // Main, Baseline, and High profiles
            profile_idc_extension: RandomU32Enum::new(vec![
                77, 66, 100, 88, 118, 128, 44, 110, 122, 144, 244,
            ]), // 88 and onwards is enabled for extensions
            constraint_set0_flag: RandomBoolRange::new(0, 1, 1),
            constraint_set1_flag: RandomBoolRange::new(0, 1, 1),
            constraint_set2_flag: RandomBoolRange::new(0, 1, 1),
            constraint_set3_flag: RandomBoolRange::new(0, 1, 1),
            constraint_set4_flag: RandomBoolRange::new(0, 1, 1),
            constraint_set5_flag: RandomBoolRange::new(0, 1, 1),
            reserved_zero_2bits: RandomU32Range::new(0, 0), // should always be 0, but we can change
            level_idc: RandomU32Enum::new(vec![
                10, 11, 12, 13, 20, 21, 22, 30, 31, 32, 40, 41, 42, 50, 51, 52, 0, 9, 60, 61, 62,
            ]),
            seq_parameter_set_id: RandomU32Range::new(0, 31), // [0 - 31]
            chroma_format_idc: RandomU32Range::new(0, 3),
            separate_colour_plane_flag: RandomBoolRange::new(0, 1, 1),
            bit_depth_luma_minus8: RandomU32Range::new(0, 10), // bit depth is usually 8, anything else is non-standard
            bit_depth_chroma_minus8: RandomU32Range::new(0, 10),
            qpprime_y_zero_transform_bypass_flag: RandomBoolRange::new(0, 1, 1),
            seq_scaling_matrix_present_flag: RandomBoolRange::new(0, 1, 1),
            seq_scaling_list_present_flag: RandomBoolRange::new(0, 1, 1),
            delta_scale: RandomI32Range::new(-128, 127),
            log2_max_frame_num_minus4: RandomU32Range::new(0, 8),
            pic_order_cnt_type: RandomU32Range::new(0, 2),
            log2_max_pic_order_cnt_lsb_minus4: RandomU32Range::new(0, 12),
            delta_pic_order_always_zero_flag: RandomBoolRange::new(0, 1, 1),
            offset_for_non_ref_pic: RandomI32Range::new(std::i32::MIN, std::i32::MAX), // se(v)
            offset_for_top_to_bottom_field: RandomI32Range::new(std::i32::MIN, std::i32::MAX), // se(v)
            num_ref_frames_in_pic_order_cnt_cycle: RandomU32Range::new(0, 255), // ue(v)
            offset_for_ref_frame: RandomI32Range::new(std::i32::MIN, std::i32::MAX), // se(v)
            max_num_ref_frames: RandomU32Range::new(0, 255),
            gaps_in_frame_num_value_allowed_flag: RandomBoolRange::new(0, 1, 1),
            pic_width_in_mbs_minus1: RandomU32Range::new(0, 255),
            pic_height_in_map_units_minus1: RandomU32Range::new(0, 255),
            frame_mbs_only_flag: RandomBoolRange::new(0, 1, 1),
            mb_adaptive_frame_field_flag: RandomBoolRange::new(0, 1, 1),
            direct_8x8_inference_flag: RandomBoolRange::new(0, 1, 1),
            frame_cropping_flag: RandomBoolRange::new(0, 1, 1),
            frame_crop_left_offset: RandomDependentU32Range::new(0, std::u32::MAX, false),
            frame_crop_right_offset: RandomDependentU32Range::new(0, std::u32::MAX, false),
            frame_crop_top_offset: RandomDependentU32Range::new(0, std::u32::MAX, false),
            frame_crop_bottom_offset: RandomDependentU32Range::new(0, std::u32::MAX, false),
            vui_parameters_present_flag: RandomBoolRange::new(0, 1, 1),
            random_vui_range: RandomVUIRange::new(),
            bias_same_bit_depth: RandomBoolRange::new(0, 50, 1), // Most of the time use the same bit depth
        }
    }
}

impl Default for RandomSPSRange {
    fn default() -> Self {
        Self::new()
    }
}

/// Subset SPS syntax elements
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RandomSubsetSPSRange {
    pub random_sps_range: RandomSPSRange,
    pub random_sps_svc_range: RandomSPSSVCExtensionRange,
    pub svc_vui_parameters_present_flag: RandomBoolRange,
    pub random_svc_vui_range: RandomVUISVCParametersRange,
    pub bit_equal_to_one: RandomU32Range,
    pub random_sps_mvc_range: RandomSPSMVCExtensionRange,
    pub mvc_vui_parameters_present_flag: RandomBoolRange,
    pub random_mvc_vui_range: RandomVUIMVCParametersRange,
    pub random_sps_mvcd_range: RandomSPSMVCDExtensionRange,
    pub random_sps_3davc_range: RandomSPS3DAVCExtensionRange,
    pub num_additional_extension2_flag: RandomU32Range,
    pub additional_extension2_flag: RandomBoolRange,
}

impl RandomSubsetSPSRange {
    pub fn new() -> RandomSubsetSPSRange {
        RandomSubsetSPSRange {
            random_sps_range: RandomSPSRange::new(),
            random_sps_svc_range: RandomSPSSVCExtensionRange::new(),
            svc_vui_parameters_present_flag: RandomBoolRange::new(0, 0, 1),
            random_svc_vui_range: RandomVUISVCParametersRange::new(),
            bit_equal_to_one: RandomU32Range::new(1, 1),
            random_sps_mvc_range: RandomSPSMVCExtensionRange::new(),
            mvc_vui_parameters_present_flag: RandomBoolRange::new(0, 1, 1),
            random_mvc_vui_range: RandomVUIMVCParametersRange::new(),
            random_sps_mvcd_range: RandomSPSMVCDExtensionRange::new(),
            random_sps_3davc_range: RandomSPS3DAVCExtensionRange::new(),
            num_additional_extension2_flag: RandomU32Range::new(0, 0),
            additional_extension2_flag: RandomBoolRange::new(0, 1, 1),
        }
    }
}

impl Default for RandomSubsetSPSRange {
    fn default() -> Self {
        Self::new()
    }
}


/// SPS Extension syntax elements
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RandomSPSExtensionRange {
    pub aux_format_idc: RandomU32Range,
    pub bit_depth_aux_minus8: RandomU32Range,
    pub alpha_incr_flag: RandomBoolRange,
    pub alpha_opaque_value: RandomU32Range,
    pub alpha_transparent_value: RandomU32Range,
    pub additional_extension_flag: RandomBoolRange,
}

impl RandomSPSExtensionRange {
    pub fn new() -> RandomSPSExtensionRange {
        RandomSPSExtensionRange {
            aux_format_idc: RandomU32Range::new(0, 10), // expected range: [0, 3]
            bit_depth_aux_minus8: RandomU32Range::new(0, 10), // [0, 4]
            alpha_incr_flag: RandomBoolRange::new(0, 1, 1),
            alpha_opaque_value: RandomU32Range::new(0, 1000), // max is 2^{bit_depth_aux_minus8 + 9}
            alpha_transparent_value: RandomU32Range::new(0, 1000), // max is 2^{bit_depth_aux_minus8 + 9}
            additional_extension_flag: RandomBoolRange::new(0, 0, 1),
        }
    }
}

impl Default for RandomSPSExtensionRange {
    fn default() -> Self {
        Self::new()
    }
}

/// SPS SVC Extension syntax elements
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomSPSSVCExtensionRange {
    pub inter_layer_deblocking_filter_control_present_flag: RandomBoolRange,
    pub extended_spatial_scalability_idc: RandomU32Range,   // u(2)
    pub chroma_phase_x_plus1_flag: RandomBoolRange,
    pub chroma_phase_y_plus1: RandomU32Range,               // u(2)
    pub seq_ref_layer_chroma_phase_x_plus1_flag: RandomBoolRange,
    pub seq_ref_layer_chroma_phase_y_plus1: RandomU32Range, // u(2)
    pub seq_scaled_ref_layer_left_offset: RandomI32Range,   // se(v)
    pub seq_scaled_ref_layer_top_offset: RandomI32Range,    // se(v)
    pub seq_scaled_ref_layer_right_offset: RandomI32Range,  // se(v)
    pub seq_scaled_ref_layer_bottom_offset: RandomI32Range, // se(v)
    pub seq_tcoeff_level_prediction_flag: RandomBoolRange,
    pub adaptive_tcoeff_level_prediction_flag: RandomBoolRange,
    pub slice_header_restriction_flag: RandomBoolRange,
}

impl RandomSPSSVCExtensionRange {
    pub fn new() -> RandomSPSSVCExtensionRange {
        RandomSPSSVCExtensionRange {
            inter_layer_deblocking_filter_control_present_flag: RandomBoolRange::new(0, 1, 1),
            extended_spatial_scalability_idc: RandomU32Range::new(0, 3), // expected [0, 2]
            chroma_phase_x_plus1_flag: RandomBoolRange::new(0, 1, 1),
            chroma_phase_y_plus1: RandomU32Range::new(0, 3),    // expected [0, 2]
            seq_ref_layer_chroma_phase_x_plus1_flag: RandomBoolRange::new(0, 1, 1),
            seq_ref_layer_chroma_phase_y_plus1: RandomU32Range::new(0, 3), // expected [0, 2]
            seq_scaled_ref_layer_left_offset: RandomI32Range::new(-65536, 65536), // expected [−2^15 to 2^15 − 1]
            seq_scaled_ref_layer_top_offset: RandomI32Range::new(-65536, 65536),
            seq_scaled_ref_layer_right_offset: RandomI32Range::new(-65536, 65536),
            seq_scaled_ref_layer_bottom_offset: RandomI32Range::new(-65536, 65536),
            seq_tcoeff_level_prediction_flag: RandomBoolRange::new(0, 1, 1),
            adaptive_tcoeff_level_prediction_flag: RandomBoolRange::new(0, 1, 1),
            slice_header_restriction_flag: RandomBoolRange::new(0, 1, 1),
        }
    }
}

impl Default for RandomSPSSVCExtensionRange {
    fn default() -> Self {
        Self::new()
    }
}

/// VUI SVC Extension syntax elements
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomVUISVCParametersRange {
    pub vui_ext_num_entries_minus1: RandomU32Range, // ue(v)
    pub vui_ext_dependency_id: RandomU32Range,      // u(3)
    pub vui_ext_quality_id: RandomU32Range,         // u(4)
    pub vui_ext_temporal_id: RandomU32Range,        // u(3)
    pub vui_ext_timing_info_present_flag: RandomBoolRange,
    pub vui_ext_num_units_in_tick: RandomU32Range, // u(32)
    pub vui_ext_time_scale: RandomU32Range,        // u(32)
    pub vui_ext_fixed_frame_rate_flag: RandomBoolRange,
    pub vui_ext_nal_hrd_parameters_present_flag: RandomBoolRange,
    pub vui_ext_nal_hrd_parameters: RandomHRDRange,
    pub vui_ext_vcl_hrd_parameters_present_flag: RandomBoolRange,
    pub vui_ext_vcl_hrd_parameters: RandomHRDRange,
    pub vui_ext_low_delay_hrd_flag: RandomBoolRange,
    pub vui_ext_pic_struct_present_flag: RandomBoolRange,
}

impl RandomVUISVCParametersRange {
    pub fn new() -> RandomVUISVCParametersRange {
        RandomVUISVCParametersRange {
            vui_ext_num_entries_minus1: RandomU32Range::new(0, 2048), // expected [0, 1023]
            vui_ext_dependency_id: RandomU32Range::new(0, 7),
            vui_ext_quality_id: RandomU32Range::new(0, 15),
            vui_ext_temporal_id: RandomU32Range::new(0, 7),
            vui_ext_timing_info_present_flag: RandomBoolRange::new(0, 1, 1),
            vui_ext_num_units_in_tick: RandomU32Range::new(0, std::u32::MAX),
            vui_ext_time_scale: RandomU32Range::new(0, std::u32::MAX),
            vui_ext_fixed_frame_rate_flag: RandomBoolRange::new(0, 1, 1),
            vui_ext_nal_hrd_parameters_present_flag: RandomBoolRange::new(0, 1, 1),
            vui_ext_nal_hrd_parameters: RandomHRDRange::new(),
            vui_ext_vcl_hrd_parameters_present_flag: RandomBoolRange::new(0, 1, 1),
            vui_ext_vcl_hrd_parameters: RandomHRDRange::new(),
            vui_ext_low_delay_hrd_flag: RandomBoolRange::new(0, 1, 1),
            vui_ext_pic_struct_present_flag: RandomBoolRange::new(0, 1, 1),
        }
    }
}

impl Default for RandomVUISVCParametersRange {
    fn default() -> Self {
        Self::new()
    }
}

/// SPS MVC Extension syntax elements
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomSPSMVCExtensionRange {
    pub num_views_minus1: RandomU32Range,
    pub view_id: RandomU32Range,
    pub num_anchor_refs_l0: RandomU32Range,
    pub anchor_refs_l0: RandomU32Range,
    pub num_anchor_refs_l1: RandomU32Range,
    pub anchor_refs_l1: RandomU32Range,
    pub num_non_anchor_refs_l0: RandomU32Range,
    pub non_anchor_refs_l0: RandomU32Range,
    pub num_non_anchor_refs_l1: RandomU32Range,
    pub non_anchor_refs_l1: RandomU32Range,
    pub num_level_values_signalled_minus1: RandomU32Range,
    pub level_idc: RandomU32Range,
    pub num_applicable_ops_minus1: RandomU32Range,
    pub applicable_op_temporal_id: RandomU32Range,
    pub applicable_op_num_target_views_minus1: RandomU32Range,
    pub applicable_op_target_view_id: RandomU32Range,
    pub applicable_op_num_views_minus1: RandomU32Range,
    pub mfc_format_idc: RandomU32Range,
    pub default_grid_position_flag: RandomBoolRange,
    pub view0_grid_position_x: RandomU32Range,
    pub view0_grid_position_y: RandomU32Range,
    pub view1_grid_position_x: RandomU32Range,
    pub view1_grid_position_y: RandomU32Range,
    pub rpu_filter_enabled_flag: RandomBoolRange,
    pub rpu_field_processing_flag: RandomBoolRange,
}

impl RandomSPSMVCExtensionRange {
    pub fn new() -> RandomSPSMVCExtensionRange {
        RandomSPSMVCExtensionRange {
            num_views_minus1: RandomU32Range::new(0, 1023), //[0, 1023]
            view_id: RandomU32Range::new(0, 255),
            num_anchor_refs_l0: RandomU32Range::new(0, 255),
            anchor_refs_l0: RandomU32Range::new(0, 255),
            num_anchor_refs_l1: RandomU32Range::new(0, 255),
            anchor_refs_l1: RandomU32Range::new(0, 255),
            num_non_anchor_refs_l0: RandomU32Range::new(0, 255),
            non_anchor_refs_l0: RandomU32Range::new(0, 255),
            num_non_anchor_refs_l1: RandomU32Range::new(0, 255),
            non_anchor_refs_l1: RandomU32Range::new(0, 255),
            num_level_values_signalled_minus1: RandomU32Range::new(0, 255),
            level_idc: RandomU32Range::new(0, 255),
            num_applicable_ops_minus1: RandomU32Range::new(0, 255),
            applicable_op_temporal_id: RandomU32Range::new(0, 7), // u(3)
            applicable_op_num_target_views_minus1: RandomU32Range::new(0, 255),
            applicable_op_target_view_id: RandomU32Range::new(0, 255),
            applicable_op_num_views_minus1: RandomU32Range::new(0, 255),
            mfc_format_idc: RandomU32Range::new(0, 63), // u(6)
            default_grid_position_flag: RandomBoolRange::new(0, 1, 1),
            view0_grid_position_x: RandomU32Range::new(0, 15), // u(4)
            view0_grid_position_y: RandomU32Range::new(0, 15), // u(4)
            view1_grid_position_x: RandomU32Range::new(0, 15), // u(4)
            view1_grid_position_y: RandomU32Range::new(0, 15), // u(4)
            rpu_filter_enabled_flag: RandomBoolRange::new(0, 1, 1),
            rpu_field_processing_flag: RandomBoolRange::new(0, 1, 1),
        }
    }
}

impl Default for RandomSPSMVCExtensionRange {
    fn default() -> Self {
        Self::new()
    }
}

/// VUI MVC Extension syntax elements
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomVUIMVCParametersRange {
    pub vui_mvc_num_ops_minus1: RandomU32Range,
    pub vui_mvc_temporal_id: RandomU32Range, // u(3)
    pub vui_mvc_num_target_output_views_minus1: RandomU32Range,
    pub vui_mvc_view_id: RandomU32Range,
    pub vui_mvc_timing_info_present_flag: RandomBoolRange,
    pub vui_mvc_num_units_in_tick: RandomU32Range, // u(32)
    pub vui_mvc_time_scale: RandomU32Range,        // u(32)
    pub vui_mvc_fixed_frame_rate_flag: RandomBoolRange,
    pub vui_mvc_nal_hrd_parameters_present_flag: RandomBoolRange,
    pub vui_mvc_nal_hrd_parameters: RandomHRDRange,
    pub vui_mvc_vcl_hrd_parameters_present_flag: RandomBoolRange,
    pub vui_mvc_vcl_hrd_parameters: RandomHRDRange,
    pub vui_mvc_low_delay_hrd_flag: RandomBoolRange,
    pub vui_mvc_pic_struct_present_flag: RandomBoolRange,
}

impl RandomVUIMVCParametersRange {
    pub fn new() -> RandomVUIMVCParametersRange {
        RandomVUIMVCParametersRange {
            vui_mvc_num_ops_minus1: RandomU32Range::new(0, 1500),
            vui_mvc_temporal_id: RandomU32Range::new(0, 7), // u(3)
            vui_mvc_num_target_output_views_minus1: RandomU32Range::new(0, 10000),
            vui_mvc_view_id: RandomU32Range::new(0, 10000),
            vui_mvc_timing_info_present_flag: RandomBoolRange::new(0, 1, 1),
            vui_mvc_num_units_in_tick: RandomU32Range::new(0, std::u32::MAX), // u(32)
            vui_mvc_time_scale: RandomU32Range::new(0, std::u32::MAX),        // u(32)
            vui_mvc_fixed_frame_rate_flag: RandomBoolRange::new(0, 1, 1),
            vui_mvc_nal_hrd_parameters_present_flag: RandomBoolRange::new(0, 1, 1),
            vui_mvc_nal_hrd_parameters: RandomHRDRange::new(),
            vui_mvc_vcl_hrd_parameters_present_flag: RandomBoolRange::new(0, 1, 1),
            vui_mvc_vcl_hrd_parameters: RandomHRDRange::new(),
            vui_mvc_low_delay_hrd_flag: RandomBoolRange::new(0, 1, 1),
            vui_mvc_pic_struct_present_flag: RandomBoolRange::new(0, 1, 1),
        }
    }
}

impl Default for RandomVUIMVCParametersRange {
    fn default() -> Self {
        Self::new()
    }
}

/// SPS MVCD Extension syntax elements
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomSPSMVCDExtensionRange {}

impl RandomSPSMVCDExtensionRange {
    pub fn new() -> RandomSPSMVCDExtensionRange {
        RandomSPSMVCDExtensionRange {}
    }
}

impl Default for RandomSPSMVCDExtensionRange {
    fn default() -> Self {
        Self::new()
    }
}

/// SPS 3D-AVC Extension syntax elements
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomSPS3DAVCExtensionRange {}

impl RandomSPS3DAVCExtensionRange {
    pub fn new() -> RandomSPS3DAVCExtensionRange {
        RandomSPS3DAVCExtensionRange {}
    }
}

impl Default for RandomSPS3DAVCExtensionRange {
    fn default() -> Self {
        Self::new()
    }
}

/// SEI syntax elements
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RandomSEIRange {
    pub num_seis: RandomU32Range,
    pub payload_type: RandomU32Enum,
    pub random_buffering_period_range: RandomSEIBufferingPeriodRange, // Type 0
    pub random_pic_timing_range: RandomSEIPicTimingRange,             // Type 1
    pub random_user_data_unregistered_range: RandomSEIUserDataUnregisteredRange, // Type 5
    pub random_recovery_point_range: RandomSEIRecoveryPointRange,     // Type 6
    pub random_film_grain_char_range: RandomSEIFilmGrainCharacteristicsRange, // Type 19
}

impl RandomSEIRange {
    pub fn new() -> RandomSEIRange {
        RandomSEIRange {
            num_seis: RandomU32Range::new(1, 1),
            // Buffering period, pic timing, unregistered user data, recovery point
            payload_type: RandomU32Enum::new(vec![0, 1, 5, 6]),
            random_buffering_period_range: RandomSEIBufferingPeriodRange::new(),
            random_pic_timing_range: RandomSEIPicTimingRange::new(),
            random_user_data_unregistered_range: RandomSEIUserDataUnregisteredRange::new(),
            random_recovery_point_range: RandomSEIRecoveryPointRange::new(),
            random_film_grain_char_range: RandomSEIFilmGrainCharacteristicsRange::new(),
        }
    }
}

impl Default for RandomSEIRange {
    fn default() -> Self {
        Self::new()
    }
}

/// SEI Type 0 -- Buffering Period (Annex D.2.2)
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomSEIBufferingPeriodRange {
    pub seq_parameter_set_id: RandomDependentU32Range, // used to sample previously chosen generated SPSes
    pub initial_cpb_removal_delay: RandomU32Range, // TODO: change to dependent. proper range is determined by bit length, chosen by VUI parameters. Up to 32 bits
    pub initial_cpb_removal_delay_offset: RandomU32Range,
}

impl RandomSEIBufferingPeriodRange {
    pub fn new() -> RandomSEIBufferingPeriodRange {
        RandomSEIBufferingPeriodRange {
            seq_parameter_set_id: RandomDependentU32Range::new(0, 32, true),
            initial_cpb_removal_delay: RandomU32Range::new(0, std::u32::MAX),
            initial_cpb_removal_delay_offset: RandomU32Range::new(0, std::u32::MAX),
        }
    }
}

impl Default for RandomSEIBufferingPeriodRange {
    fn default() -> Self {
        Self::new()
    }
}

/// SEI Type 1 -- Pic Timing (Annex D.2.3)
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomSEIPicTimingRange {
    pub cpb_removal_delay: RandomDependentU32Range, // u(v)
    pub dpb_output_delay: RandomDependentU32Range,  // u(v)
    pub pic_struct: RandomU32Range,                 // u(4)
    pub clock_timestamp_flag: RandomBoolRange,
    pub ct_type: RandomU32Range, // u(2)
    pub nuit_field_based_flag: RandomBoolRange,
    pub counting_type: RandomU32Range, // u(5)
    pub full_timestamp_flag: RandomBoolRange,
    pub discontinuity_flag: RandomBoolRange,
    pub cnt_dropped_flag: RandomBoolRange,
    pub n_frames: RandomU32Range,      // u(8)
    pub seconds_value: RandomU32Range, // u(6), range: 0..59
    pub minutes_value: RandomU32Range, // u(6), range: 0..59
    pub hours_value: RandomU32Range,   // u(5), range: 0..23
    pub seconds_flag: RandomBoolRange,
    pub minutes_flag: RandomBoolRange,
    pub hours_flag: RandomBoolRange,
    pub time_offset: RandomDependentU32Range, // i(v), depends on time_offset_length
}

impl RandomSEIPicTimingRange {
    pub fn new() -> RandomSEIPicTimingRange {
        RandomSEIPicTimingRange {
            cpb_removal_delay: RandomDependentU32Range::new(0, 32, true),
            dpb_output_delay: RandomDependentU32Range::new(0, 32, true),
            pic_struct: RandomU32Range::new(0, 15),
            clock_timestamp_flag: RandomBoolRange::new(0, 1, 1),
            ct_type: RandomU32Range::new(0, 3),
            nuit_field_based_flag: RandomBoolRange::new(0, 1, 1),
            counting_type: RandomU32Range::new(0, 31), // u(5)
            full_timestamp_flag: RandomBoolRange::new(0, 1, 1),
            discontinuity_flag: RandomBoolRange::new(0, 1, 1),
            cnt_dropped_flag: RandomBoolRange::new(0, 1, 1),
            n_frames: RandomU32Range::new(0, 255),     // u(8)
            seconds_value: RandomU32Range::new(0, 63), // u(6)
            minutes_value: RandomU32Range::new(0, 63), // u(6)
            hours_value: RandomU32Range::new(0, 31),   // u(5)
            seconds_flag: RandomBoolRange::new(0, 1, 1),
            minutes_flag: RandomBoolRange::new(0, 1, 1),
            hours_flag: RandomBoolRange::new(0, 1, 1),
            time_offset: RandomDependentU32Range::new(0, 32, true),
        }
    }
}

/// SEI Type 5 -- Unregistered data with UUID APPLE1
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomSEIUnregisteredDataApple1Range {
    pub mystery_param1: RandomU32Range, // u(8)
}

impl RandomSEIUnregisteredDataApple1Range {
    pub fn new() -> RandomSEIUnregisteredDataApple1Range {
        RandomSEIUnregisteredDataApple1Range {
            mystery_param1: RandomU32Range::new(0, 255),
        }
    }
}

impl Default for RandomSEIUnregisteredDataApple1Range {
    fn default() -> Self {
        Self::new()
    }
}

/// SEI Type 5 -- Unregistered data with UUID APPLE2
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomSEIUnregisteredDataApple2Range {
    pub mystery_param1: RandomU32Range, // u(8)
    pub mystery_param2: RandomU32Range, // u(8)
    pub mystery_param3: RandomU32Range, // u(8)
    pub mystery_param4: RandomU32Range, // u(8)
    pub mystery_param5: RandomU32Range, // u(8)
    pub mystery_param6: RandomU32Range, // u(8)
    pub mystery_param7: RandomU32Range, // u(8)
    pub mystery_param8: RandomU32Range, // u(8)
}

impl RandomSEIUnregisteredDataApple2Range {
    pub fn new() -> RandomSEIUnregisteredDataApple2Range {
        RandomSEIUnregisteredDataApple2Range {
            mystery_param1: RandomU32Range::new(0, 255),
            mystery_param2: RandomU32Range::new(0, 255),
            mystery_param3: RandomU32Range::new(0, 255),
            mystery_param4: RandomU32Range::new(0, 255),
            mystery_param5: RandomU32Range::new(0, 255),
            mystery_param6: RandomU32Range::new(0, 255),
            mystery_param7: RandomU32Range::new(0, 255),
            mystery_param8: RandomU32Range::new(0, 255),
        }
    }
}

impl Default for RandomSEIUnregisteredDataApple2Range {
    fn default() -> Self {
        Self::new()
    }
}

/// SEI Type 5 -- User Data Unregistered (Annex D.2.7)
///
/// Currently only using known UUID parameters
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomSEIUserDataUnregisteredRange {
    pub uuid_iso_iec_11578: RandomU32Range, // used to sample previously chosen generated SPSes
    pub user_data_apple1: RandomSEIUnregisteredDataApple1Range,
    pub user_data_apple2: RandomSEIUnregisteredDataApple2Range,
    pub user_data_payload_length: RandomU32Range, // used to determine how many bytes to sample
}

impl RandomSEIUserDataUnregisteredRange {
    pub fn new() -> RandomSEIUserDataUnregisteredRange {
        RandomSEIUserDataUnregisteredRange {
            uuid_iso_iec_11578: RandomU32Range::new(0, KNOWN_UUIDS - 1),
            user_data_apple1: RandomSEIUnregisteredDataApple1Range::new(),
            user_data_apple2: RandomSEIUnregisteredDataApple2Range::new(),
            user_data_payload_length: RandomU32Range::new(10, 100),
        }
    }
}

impl Default for RandomSEIUserDataUnregisteredRange {
    fn default() -> Self {
        Self::new()
    }
}

/// SEI Type 6 -- Recovery Point (Annex D.2.7)
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomSEIRecoveryPointRange {
    pub recovery_frame_cnt: RandomU32Range, // ue(v)
    pub exact_match_flag: RandomBoolRange,
    pub broken_link_flag: RandomBoolRange,
    pub changing_slice_group_idc: RandomU32Range, // u(2)
}

impl RandomSEIRecoveryPointRange {
    pub fn new() -> RandomSEIRecoveryPointRange {
        RandomSEIRecoveryPointRange {
            recovery_frame_cnt: RandomU32Range::new(0, 100000),
            exact_match_flag: RandomBoolRange::new(0, 1, 1),
            broken_link_flag: RandomBoolRange::new(0, 1, 1),
            changing_slice_group_idc: RandomU32Range::new(0, 3), // u(2)
        }
    }
}

impl Default for RandomSEIRecoveryPointRange {
    fn default() -> Self {
        Self::new()
    }
}

/// SEI Type 19 -- Film Grain Characteristics (Annex D.2.21)
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomSEIFilmGrainCharacteristicsRange {
    pub film_grain_characteristics_cancel_flag: RandomBoolRange,
    pub film_grain_model_id: RandomU32Range, // u(2); look up table, only [0, 1] allowed
    pub separate_colour_description_present_flag: RandomBoolRange, // If 0, default to VUI values, described in E.2.1
    pub film_grain_bit_depth_luma_minus8: RandomU32Range,          // u(3) ; [0, 7]
    pub film_grain_bit_depth_chroma_minus8: RandomU32Range,        // u(3) ; [0, 7]
    pub film_grain_full_range_flag: RandomBoolRange,
    pub film_grain_colour_primaries: RandomU32Range, // u(8) ; lookup into table E-3; [1, 22]; other values reserved
    pub film_grain_transfer_characteristics: RandomU32Range, // u(8) ; lookup into table E-4; [1, 18]; other values reserved
    pub film_grain_matrix_coefficients: RandomU32Range, // u(8) ; lookup into table E-5; [0, 14]; other values reserved
    pub blending_mode_id: RandomU32Range,               // u(2)
    pub log2_scale_factor: RandomU32Range,              // u(4)
    pub comp_model_present_flag: RandomBoolRange,
    pub num_intensity_intervals_minus1: RandomU32Range, // u(8)
    pub num_model_values_minus1: RandomU32Range,        // u(3) [0, 5]
    pub intensity_interval_lower_bound: RandomU32Range, // u(8)
    pub intensity_interval_upper_bound: RandomU32Range, // u(8)
    pub comp_model_value: RandomI32Range, // se(v) ; bit length is film_grain_bit_depth_luma_minus8
    pub film_grain_characteristics_repetition_period: RandomU32Range, // ue(v) ; [0, 16384]
}

impl RandomSEIFilmGrainCharacteristicsRange {
    pub fn new() -> RandomSEIFilmGrainCharacteristicsRange {
        RandomSEIFilmGrainCharacteristicsRange {
            film_grain_characteristics_cancel_flag: RandomBoolRange::new(0, 1, 1),
            film_grain_model_id: RandomU32Range::new(0, 3), // u(2); look up table, only [0, 1] allowed
            separate_colour_description_present_flag: RandomBoolRange::new(0, 1, 1),
            film_grain_bit_depth_luma_minus8: RandomU32Range::new(0, 7), // u(3) ; [0, 7]
            film_grain_bit_depth_chroma_minus8: RandomU32Range::new(0, 7), // u(3) ; [0, 7]
            film_grain_full_range_flag: RandomBoolRange::new(0, 1, 1),
            film_grain_colour_primaries: RandomU32Range::new(0, 255), // u(8)
            film_grain_transfer_characteristics: RandomU32Range::new(0, 255), // u(8)
            film_grain_matrix_coefficients: RandomU32Range::new(0, 255), // u(8)
            blending_mode_id: RandomU32Range::new(0, 255),            // u(8)
            log2_scale_factor: RandomU32Range::new(0, 3),             // u(2)
            comp_model_present_flag: RandomBoolRange::new(0, 1, 1),
            num_intensity_intervals_minus1: RandomU32Range::new(0, 255), // u(8)
            num_model_values_minus1: RandomU32Range::new(0, 7),          // u(3) ; [0, 5]
            intensity_interval_lower_bound: RandomU32Range::new(0, 255), // u(8)
            intensity_interval_upper_bound: RandomU32Range::new(0, 255), // u(8)
            comp_model_value: RandomI32Range::new(std::i32::MIN, std::i32::MAX), // se(v) ; bit length is film_grain_bit_depth_luma_minus8
            film_grain_characteristics_repetition_period: RandomU32Range::new(0, std::u32::MAX), // ue(v) ; [0, 16384]
        }
    }
}

impl Default for RandomSEIFilmGrainCharacteristicsRange {
    fn default() -> Self {
        Self::new()
    }
}

/// PPS syntax elements
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomPPSRange {
    pub pic_parameter_set_id: RandomU32Range, // ue(v)
    pub entropy_coding_mode_flag: RandomBoolRange,
    pub bottom_field_pic_order_in_frame_present_flag: RandomBoolRange,
    pub num_slice_groups_minus1: RandomU32Range, // ue(v)
    pub slice_group_map_type: RandomU32Range,    // ue(v)
    pub run_length_minus1: RandomU32Range,       // ue(v)
    pub top_left: RandomU32Range,                // ue(v)
    pub bottom_right: RandomU32Range,            // ue(v)
    pub slice_group_change_direction_flag: RandomBoolRange,
    pub slice_group_change_rate_minus1: RandomU32Range, // ue(v)
    pub pic_size_in_map_units_minus1: RandomU32Range,   // ue(v)
    pub slice_group_id: RandomU32Range,                 // u(v)
    pub num_ref_idx_l0_default_active_minus1: RandomU32Range, // ue(v)
    pub num_ref_idx_l1_default_active_minus1: RandomU32Range, // ue(v)
    pub weighted_pred_flag: RandomBoolRange,
    pub weighted_bipred_idc: RandomU32Range,    // u(2)
    pub pic_init_qp_minus26: RandomI32Range,    // se(v)
    pub pic_init_qs_minus26: RandomI32Range,    // se(v)
    pub chroma_qp_index_offset: RandomI32Range, // se(v)
    pub deblocking_filter_control_present_flag: RandomBoolRange,
    pub constrained_intra_pred_flag: RandomBoolRange,
    pub redundant_pic_cnt_present_flag: RandomBoolRange,
    // more_rbsp_data()
    pub include_more_data: RandomBoolRange,
    pub transform_8x8_mode_flag: RandomBoolRange,
    pub pic_scaling_matrix_present_flag: RandomBoolRange,
    pub pic_scaling_list_present_flag: RandomBoolRange,
    pub delta_scale: RandomI32Range,
    pub second_chroma_qp_index_offset: RandomI32Range,
    // Bias
    pub bias_ignore_slice_groups: RandomBoolRange,
}

impl RandomPPSRange {
    pub fn new() -> RandomPPSRange {
        RandomPPSRange {
            pic_parameter_set_id: RandomU32Range::new(0, 512), //[0, 255]
            entropy_coding_mode_flag: RandomBoolRange::new(0, 1, 1),
            bottom_field_pic_order_in_frame_present_flag: RandomBoolRange::new(0, 1, 1),
            num_slice_groups_minus1: RandomU32Range::new(0, 0), // [0, 7]
            slice_group_map_type: RandomU32Range::new(0, 6),    // range is [0, 6]
            run_length_minus1: RandomU32Range::new(0, 16777216), // range is [0, PicSizeInMapUnits-1] for 4k by 4k
            top_left: RandomU32Range::new(0, 16777216), // requirement: top_left < bottom_right && bottom_right < PicSizeInMapUnits
            bottom_right: RandomU32Range::new(0, 16777216), // requirement: top_left < bottom_right && bottom_right < PicSizeInMapUnits
            slice_group_change_direction_flag: RandomBoolRange::new(0, 1, 1),
            slice_group_change_rate_minus1: RandomU32Range::new(0, 16777216), // should be in [0, PicSizeInMapUnits −1]
            pic_size_in_map_units_minus1: RandomU32Range::new(0, 16777216), // expected to be PicSizeInMapUnits −1
            slice_group_id: RandomU32Range::new(0, 30), // range is [0, num_slice_groups_minus1]
            num_ref_idx_l0_default_active_minus1: RandomU32Range::new(0, 1024), // max is 32
            num_ref_idx_l1_default_active_minus1: RandomU32Range::new(0, 1024), // max is 32
            weighted_pred_flag: RandomBoolRange::new(0, 1, 1),
            weighted_bipred_idc: RandomU32Range::new(0, 3), // u(2)
            pic_init_qp_minus26: RandomI32Range::new(-512, 512),
            pic_init_qs_minus26: RandomI32Range::new(-512, 512),
            chroma_qp_index_offset: RandomI32Range::new(-512, 512),
            deblocking_filter_control_present_flag: RandomBoolRange::new(0, 1, 1),
            constrained_intra_pred_flag: RandomBoolRange::new(0, 1, 1),
            redundant_pic_cnt_present_flag: RandomBoolRange::new(0, 1, 1),
            include_more_data: RandomBoolRange::new(0, 1, 1),
            transform_8x8_mode_flag: RandomBoolRange::new(0, 1, 1),
            pic_scaling_matrix_present_flag: RandomBoolRange::new(0, 1, 1),
            pic_scaling_list_present_flag: RandomBoolRange::new(0, 1, 1),
            delta_scale: RandomI32Range::new(-128, 127),
            second_chroma_qp_index_offset: RandomI32Range::new(-13, 14),
            bias_ignore_slice_groups: RandomBoolRange::new(0, 1, 1), // 90% of the time no slice groups
        }
    }
}

impl Default for RandomPPSRange {
    fn default() -> Self {
        Self::new()
    }
}

/// NALU Header SVC extension syntax elements
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomNALUHeaderSVCExtension {
    pub idr_flag: RandomBoolRange,
    pub priority_id: RandomU32Range,
    pub no_inter_layer_pred_flag: RandomBoolRange,
    pub dependency_id: RandomU32Range,
    pub quality_id: RandomU32Range,
    pub temporal_id: RandomU32Range,
    pub use_ref_base_pic_flag: RandomBoolRange,
    pub discardable_flag: RandomBoolRange,
    pub output_flag: RandomBoolRange,
    pub reserved_three_2bits: RandomU32Range,
}

impl RandomNALUHeaderSVCExtension {
    pub fn new() -> RandomNALUHeaderSVCExtension {
        RandomNALUHeaderSVCExtension {
            idr_flag: RandomBoolRange::new(0, 1, 1),
            priority_id: RandomU32Range::new(0, 255),
            no_inter_layer_pred_flag: RandomBoolRange::new(0, 1, 1),
            dependency_id: RandomU32Range::new(0, 255),
            quality_id: RandomU32Range::new(0, 255),
            temporal_id: RandomU32Range::new(0, 255),
            use_ref_base_pic_flag: RandomBoolRange::new(0, 1, 1),
            discardable_flag: RandomBoolRange::new(0, 1, 1),
            output_flag: RandomBoolRange::new(0, 1, 1),
            reserved_three_2bits: RandomU32Range::new(0, 255),
        }
    }
}

impl Default for RandomNALUHeaderSVCExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// NALU Header 3D AVC extension syntax elements
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomNALUHeader3DAVCExtension {
    pub view_idx: RandomU32Range,
    pub depth_flag: RandomBoolRange,
    pub non_idr_flag: RandomBoolRange,
    pub temporal_id: RandomU32Range,
    pub anchor_pic_flag: RandomBoolRange,
    pub inter_view_flag: RandomBoolRange,
}

impl RandomNALUHeader3DAVCExtension {
    pub fn new() -> RandomNALUHeader3DAVCExtension {
        RandomNALUHeader3DAVCExtension {
            view_idx: RandomU32Range::new(0, 255),
            depth_flag: RandomBoolRange::new(0, 1, 1),
            non_idr_flag: RandomBoolRange::new(0, 1, 1),
            temporal_id: RandomU32Range::new(0, 255),
            anchor_pic_flag: RandomBoolRange::new(0, 1, 1),
            inter_view_flag: RandomBoolRange::new(0, 1, 1),
        }
    }
}

impl Default for RandomNALUHeader3DAVCExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// NALU Header MVC extension syntax elements
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomNALUHeaderMVCExtension {
    pub non_idr_flag: RandomBoolRange,
    pub priority_id: RandomU32Range,
    pub view_id: RandomU32Range,
    pub temporal_id: RandomU32Range,
    pub anchor_pic_flag: RandomBoolRange,
    pub inter_view_flag: RandomBoolRange,
    pub reserved_one_bit: RandomBoolRange,
}

impl RandomNALUHeaderMVCExtension {
    pub fn new() -> RandomNALUHeaderMVCExtension {
        RandomNALUHeaderMVCExtension {
            non_idr_flag: RandomBoolRange::new(0, 1, 1),
            priority_id: RandomU32Range::new(0, 255),
            view_id: RandomU32Range::new(0, 255),
            temporal_id: RandomU32Range::new(0, 255),
            anchor_pic_flag: RandomBoolRange::new(0, 1, 1),
            inter_view_flag: RandomBoolRange::new(0, 1, 1),
            reserved_one_bit: RandomBoolRange::new(0, 1, 1),
        }
    }
}

impl Default for RandomNALUHeaderMVCExtension {
    fn default() -> Self {
        Self::new()
    }
}

/// NALU Header syntax elements
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RandomNALUHeader {
    pub forbidden_zero_bit: RandomU32Range,     // f(1)
    pub nal_ref_idc: RandomU32Range,            // u(2)
    pub nal_unit_type: RandomU32Enum,           // u(5)
    pub nal_unit_slice_type: RandomU32Enum,     // u(5)
    pub nal_unit_extension_type: RandomU32Enum, // u(5)
    pub nal_unit_undefined_type: RandomU32Enum, // u(5)

    // enabled for nal_unit_type 14, 20, or 21
    pub svc_extension_flag: RandomBoolRange,
    pub random_svc_extension: RandomNALUHeaderSVCExtension,
    pub avc_3d_extension_flag: RandomBoolRange,
    pub random_avc_3d_extension: RandomNALUHeader3DAVCExtension,
    pub random_mvc_extension: RandomNALUHeaderMVCExtension,
    // Bias
    pub bias_idr_nalu: RandomBoolRange, // when True, will make a slice NALU into an IDR NALU
    pub bias_slice_nalu: RandomBoolRange, // when True, make a new slice instead of other NALU types
    pub bias_undefined_nalu: RandomBoolRange, // when True, will sample an undefined NALU value
    // Extra
    pub filler_data_nalu_length: RandomU32Range, // According to the spec, filler data is not parsed so we fill it with random length bytes
    pub undefined_nalu_length: RandomU32Range, // For undefined NALUs, we throw random length number of bytes
}

impl RandomNALUHeader {
    pub fn new() -> RandomNALUHeader {
        RandomNALUHeader {
            forbidden_zero_bit: RandomU32Range::new(0, 0), // Supposed to just be 0
            nal_ref_idc: RandomU32Range::new(0, 3),        // u(2)
            nal_unit_type: RandomU32Enum::new(vec![1, 5, 6, 7, 8, 9]), // non-IDR slice, IDR slice, SEI, SPS, PPS, AUD
            nal_unit_slice_type: RandomU32Enum::new(vec![1, 5]),       // non-IDR slice, IDR slice
            nal_unit_extension_type: RandomU32Enum::new(vec![
                1,  // non-IDR slice
                5,  // IDR slice
                6,  // SEI
                7,  // SPS
                8,  // PPS
                9,  // AUD
                10, // End of Sequence
                11, // End of Stream
                12, // Filler data
                14, // Prefix NALU
                15, // Subset SPS
                20, // Slice extension
            ]),
            nal_unit_undefined_type: RandomU32Enum::new(vec![
                17, // Reserved
                18, // Reserved
                22, // Reserved
                23, // Reserved
                24, // Unspecified
                25, // Unspecified
                26, // Unspecified
                27, // Unspecified
                28, // Unspecified
                29, // Unspecified
                30, // Unspecified
                31, // Unspecified
                0,  // Should be skipped
            ]),
            svc_extension_flag: RandomBoolRange::new(0, 1, 1),
            random_svc_extension: RandomNALUHeaderSVCExtension::new(),
            avc_3d_extension_flag: RandomBoolRange::new(0, 1, 1),
            random_avc_3d_extension: RandomNALUHeader3DAVCExtension::new(),
            random_mvc_extension: RandomNALUHeaderMVCExtension::new(),
            bias_idr_nalu: RandomBoolRange::new(0, 1, 1), // 50%
            bias_slice_nalu: RandomBoolRange::new(0, 9, 1), // 90% chance of Slice NALU
            bias_undefined_nalu: RandomBoolRange::new(0, 9, 9), // 10% chance of Slice NALU
            filler_data_nalu_length: RandomU32Range::new(1, 128),
            undefined_nalu_length: RandomU32Range::new(1, 128),
        }
    }
}

impl Default for RandomNALUHeader {
    fn default() -> Self {
        Self::new()
    }
}

/// Prefix NALU syntax elements
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomPrefixNALU {
    pub store_ref_base_pic_flag: RandomBoolRange,
    pub adaptive_ref_base_pic_marking_mode_flag: RandomBoolRange,
    pub num_modifications: RandomU32Range,
    pub memory_management_base_control_operation: RandomU32Range,
    pub difference_of_base_pic_nums_minus1: RandomU32Range,
    pub long_term_base_pic_num: RandomU32Range,
    pub num_data_extensions: RandomU32Range,
    pub additional_prefix_nal_unit_extension_flag: RandomBoolRange,
    pub additional_prefix_nal_unit_extension_data_flag: RandomBoolRange,
}

impl RandomPrefixNALU {
    pub fn new() -> RandomPrefixNALU {
        RandomPrefixNALU {
            store_ref_base_pic_flag: RandomBoolRange::new(0, 1, 1),
            adaptive_ref_base_pic_marking_mode_flag: RandomBoolRange::new(0, 1, 1),
            num_modifications: RandomU32Range::new(1, 200), // 0 modifications leads to underflow
            memory_management_base_control_operation: RandomU32Range::new(1, 3), // 0 is the stop condition, so we add that in manually later
            difference_of_base_pic_nums_minus1: RandomU32Range::new(0, 100),
            long_term_base_pic_num: RandomU32Range::new(0, 100),
            num_data_extensions: RandomU32Range::new(0, 10),
            additional_prefix_nal_unit_extension_flag: RandomBoolRange::new(0, 1, 1),
            additional_prefix_nal_unit_extension_data_flag: RandomBoolRange::new(0, 1, 1),
        }
    }
}

impl Default for RandomPrefixNALU {
    fn default() -> Self {
        Self::new()
    }
}

/// Access Unit Delimiter syntax elements
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomAccessUnitDelim {
    pub primary_pic_type: RandomU32Range, // u(3)
}

impl RandomAccessUnitDelim {
    pub fn new() -> RandomAccessUnitDelim {
        RandomAccessUnitDelim {
            primary_pic_type: RandomU32Range::new(0, 7), // u(3)
        }
    }
}

impl Default for RandomAccessUnitDelim {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomRTPOptions {
    pub num_aggregation_nalus: RandomU32Range,
}

/// Overall random video properties
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct RandomizeVideo {
    pub num_nalus: RandomU32Range,
    pub enable_extensions: RandomBoolRange,
    pub enable_rtp: RandomBoolRange,
    pub mp4_width: RandomU32Range, // Used when randomizing MP4 frame size
    pub mp4_height: RandomU32Range, // Used when randomizing MP4 frame size
}

impl RandomizeVideo {
    pub fn new() -> RandomizeVideo {
        RandomizeVideo {
            num_nalus: RandomU32Range::new(4, 30),
            enable_extensions: RandomBoolRange::new(0, 1, 1),
            enable_rtp: RandomBoolRange::new(0, 1, 1),
            mp4_width: RandomU32Range::new(0, 10000),
            mp4_height: RandomU32Range::new(0, 10000),
        }
    }
}

impl Default for RandomizeVideo {
    fn default() -> Self {
        Self::new()
    }
}

/// High level struct that maintains the random ranges for all syntax elements
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RandomizeConfig {
    pub random_video_config: RandomizeVideo,
    pub random_nalu_range: RandomNALUHeader,
    pub random_access_unit_delim_range: RandomAccessUnitDelim,
    pub random_sps_range: RandomSPSRange,
    pub random_subset_sps_range: RandomSubsetSPSRange,
    pub random_sps_extension_range: RandomSPSExtensionRange,
    pub random_prefix_nalu_range: RandomPrefixNALU,
    pub random_pps_range: RandomPPSRange,
    pub random_sei_range: RandomSEIRange,
    pub random_slice_header_range: RandomSliceHeaderRange,
    pub random_mb_range: RandomMBRange,
}

impl RandomizeConfig {
    pub fn new() -> RandomizeConfig {
        RandomizeConfig {
            random_video_config: RandomizeVideo::new(),
            random_nalu_range: RandomNALUHeader::new(),
            random_access_unit_delim_range: RandomAccessUnitDelim::new(),
            random_prefix_nalu_range: RandomPrefixNALU::new(),
            random_sps_range: RandomSPSRange::new(),
            random_subset_sps_range: RandomSubsetSPSRange::new(),
            random_sps_extension_range: RandomSPSExtensionRange::new(),
            random_pps_range: RandomPPSRange::new(),
            random_sei_range: RandomSEIRange::new(),
            random_slice_header_range: RandomSliceHeaderRange::new(),
            random_mb_range: RandomMBRange::new(),
        }
    }
}

impl Default for RandomizeConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Save the default random ranges for the syntax elements
pub fn save_config() {
    // json_file will store our H264DecodedStream elements
    let mut json_file = match File::create("default.json") {
        Err(_) => panic!("couldn't create default.json"),
        Ok(file) => file,
    };

    let serialized = serde_json::to_string_pretty(&RandomizeConfig::new()).unwrap();

    match json_file.write_all(serialized.as_bytes()) {
        Err(_) => panic!("couldn't write to file default.json"),
        Ok(()) => (),
    };
}

/// Load new random ranges for the syntax elements
pub fn load_config(filename: &str) -> RandomizeConfig {
    let json_file = match File::open(filename) {
        Err(_) => panic!("couldn't open {}", filename),
        Ok(file) => file,
    };

    let reader = BufReader::new(json_file);

    let res: RandomizeConfig = match serde_json::from_reader(reader) {
        Ok(x) => x, // copy over the new result
        Err(y) => panic!("Error reading modified H264DecodedStream: {:?}", y),
    };

    res
}
