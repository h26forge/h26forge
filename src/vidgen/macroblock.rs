//! Macroblock syntax element randomization.

use crate::common::data_structures::CoeffToken;
use crate::common::data_structures::H264DecodedStream;
use crate::common::data_structures::MbPartPredMode;
use crate::common::data_structures::MbType;
use crate::common::data_structures::ResidualMode;
use crate::common::data_structures::SubMbType;
use crate::common::data_structures::TransformBlock;
use crate::common::data_structures::VideoParameters;
use crate::common::helper::is_slice_type;
use crate::vidgen::film::FilmState;
use crate::vidgen::generate_configurations::RandomMBRange;
use std::cmp;

/// Generate a random I macroblock type
pub fn random_i_mbtype(
    ignore_intra_pred: bool,
    ignore_ipcm: bool,
    rconfig: &RandomMBRange,
    film: &mut FilmState,
) -> MbType {
    // I16x16_[intra luma prediction mode]_[coded block pattern chroma]_[coded block pattern luma]
    // modes:
    // - 0: vertical
    // - 1: horizontal
    // - 2: DC
    // - 3: plane

    if ignore_intra_pred {
        if ignore_ipcm {
            return MbType::INxN;
        }

        match rconfig.mb_i_type.sample(film) {
            0 => MbType::INxN,
            25 => MbType::IPCM,
            _ => MbType::INxN,
        }
    } else if ignore_ipcm {
        match rconfig.mb_i_type.sample(film) {
            0 => MbType::INxN, // 8x8 or 4x4 luma intra prediction
            1 => MbType::I16x16_0_0_0,
            2 => MbType::I16x16_1_0_0,
            3 => MbType::I16x16_2_0_0,
            4 => MbType::I16x16_3_0_0,
            5 => MbType::I16x16_0_1_0,
            6 => MbType::I16x16_1_1_0,
            7 => MbType::I16x16_2_1_0,
            8 => MbType::I16x16_3_1_0,
            9 => MbType::I16x16_0_2_0,
            10 => MbType::I16x16_1_2_0,
            11 => MbType::I16x16_2_2_0,
            12 => MbType::I16x16_3_2_0,
            13 => MbType::I16x16_0_0_1,
            14 => MbType::I16x16_1_0_1,
            15 => MbType::I16x16_2_0_1,
            16 => MbType::I16x16_3_0_1,
            17 => MbType::I16x16_0_1_1,
            18 => MbType::I16x16_1_1_1,
            19 => MbType::I16x16_2_1_1,
            20 => MbType::I16x16_3_1_1,
            21 => MbType::I16x16_0_2_1,
            22 => MbType::I16x16_1_2_1,
            23 => MbType::I16x16_2_2_1,
            24 => MbType::I16x16_3_2_1,
            _ => MbType::INxN,
        }
    } else {
        match rconfig.mb_i_type.sample(film) {
            0 => MbType::INxN, // 8x8 or 4x4 luma intra prediction
            1 => MbType::I16x16_0_0_0,
            2 => MbType::I16x16_1_0_0,
            3 => MbType::I16x16_2_0_0,
            4 => MbType::I16x16_3_0_0,
            5 => MbType::I16x16_0_1_0,
            6 => MbType::I16x16_1_1_0,
            7 => MbType::I16x16_2_1_0,
            8 => MbType::I16x16_3_1_0,
            9 => MbType::I16x16_0_2_0,
            10 => MbType::I16x16_1_2_0,
            11 => MbType::I16x16_2_2_0,
            12 => MbType::I16x16_3_2_0,
            13 => MbType::I16x16_0_0_1,
            14 => MbType::I16x16_1_0_1,
            15 => MbType::I16x16_2_0_1,
            16 => MbType::I16x16_3_0_1,
            17 => MbType::I16x16_0_1_1,
            18 => MbType::I16x16_1_1_1,
            19 => MbType::I16x16_2_1_1,
            20 => MbType::I16x16_3_1_1,
            21 => MbType::I16x16_0_2_1,
            22 => MbType::I16x16_1_2_1,
            23 => MbType::I16x16_2_2_1,
            24 => MbType::I16x16_3_2_1,
            25 => MbType::IPCM,
            _ => MbType::INxN,
        }
    }
}

/// Generate a random SI macroblock type
pub fn random_si_mbtype(
    ignore_intra_pred: bool,
    ignore_ipcm: bool,
    rconfig: &RandomMBRange,
    film: &mut FilmState,
) -> MbType {
    if ignore_intra_pred {
        if ignore_ipcm {
            return MbType::INxN;
        }
        match rconfig.mb_si_type.sample(film) {
            0 => MbType::SI,
            1 => MbType::INxN,
            25 => MbType::IPCM,
            _ => MbType::IPCM,
        }
    } else {
        match rconfig.mb_si_type.sample(film) {
            0 => MbType::SI,
            1 => MbType::INxN,
            2 => MbType::I16x16_0_0_0,
            3 => MbType::I16x16_1_0_0,
            4 => MbType::I16x16_2_0_0,
            5 => MbType::I16x16_3_0_0,
            6 => MbType::I16x16_0_1_0,
            7 => MbType::I16x16_1_1_0,
            8 => MbType::I16x16_2_1_0,
            9 => MbType::I16x16_3_1_0,
            10 => MbType::I16x16_0_2_0,
            11 => MbType::I16x16_1_2_0,
            12 => MbType::I16x16_2_2_0,
            13 => MbType::I16x16_3_2_0,
            14 => MbType::I16x16_0_0_1,
            15 => MbType::I16x16_1_0_1,
            16 => MbType::I16x16_2_0_1,
            17 => MbType::I16x16_3_0_1,
            18 => MbType::I16x16_0_1_1,
            19 => MbType::I16x16_1_1_1,
            20 => MbType::I16x16_2_1_1,
            21 => MbType::I16x16_3_1_1,
            22 => MbType::I16x16_0_2_1,
            23 => MbType::I16x16_1_2_1,
            24 => MbType::I16x16_2_2_1,
            25 => MbType::I16x16_3_2_1,
            26 => MbType::IPCM,
            _ => MbType::INxN,
        }
    }
}

/// Generate a random P macroblock type
pub fn random_p_mbtype(
    ignore_intra_pred: bool,
    ignore_ipcm: bool,
    rconfig: &RandomMBRange,
    film: &mut FilmState,
) -> MbType {
    if ignore_intra_pred {
        match rconfig.mb_p_type.sample(film) {
            0 => MbType::PL016x16,
            1 => MbType::PL0L016x8,
            2 => MbType::PL0L08x16,
            3 => MbType::P8x8,
            _ => MbType::PL016x16, // high likelihood of getting this type
        }
    } else if ignore_ipcm {
        match rconfig.mb_p_type.sample(film) {
            0 => MbType::PL016x16,
            1 => MbType::PL0L016x8,
            2 => MbType::PL0L08x16,
            3 => MbType::P8x8,
            4 => MbType::P8x8ref0, // should never get picked
            5 => MbType::INxN,
            6 => MbType::I16x16_0_0_0,
            7 => MbType::I16x16_1_0_0,
            8 => MbType::I16x16_2_0_0,
            9 => MbType::I16x16_3_0_0,
            10 => MbType::I16x16_0_1_0,
            11 => MbType::I16x16_1_1_0,
            12 => MbType::I16x16_2_1_0,
            13 => MbType::I16x16_3_1_0,
            14 => MbType::I16x16_0_2_0,
            15 => MbType::I16x16_1_2_0,
            16 => MbType::I16x16_2_2_0,
            17 => MbType::I16x16_3_2_0,
            18 => MbType::I16x16_0_0_1,
            19 => MbType::I16x16_1_0_1,
            20 => MbType::I16x16_2_0_1,
            21 => MbType::I16x16_3_0_1,
            22 => MbType::I16x16_0_1_1,
            23 => MbType::I16x16_1_1_1,
            24 => MbType::I16x16_2_1_1,
            25 => MbType::I16x16_3_1_1,
            26 => MbType::I16x16_0_2_1,
            27 => MbType::I16x16_1_2_1,
            28 => MbType::I16x16_2_2_1,
            29 => MbType::I16x16_3_2_1,

            _ => MbType::PL016x16,
        }
    } else {
        match rconfig.mb_p_type.sample(film) {
            0 => MbType::PL016x16,
            1 => MbType::PL0L016x8,
            2 => MbType::PL0L08x16,
            3 => MbType::P8x8,
            4 => MbType::P8x8ref0, // should never get picked
            5 => MbType::INxN,
            6 => MbType::I16x16_0_0_0,
            7 => MbType::I16x16_1_0_0,
            8 => MbType::I16x16_2_0_0,
            9 => MbType::I16x16_3_0_0,
            10 => MbType::I16x16_0_1_0,
            11 => MbType::I16x16_1_1_0,
            12 => MbType::I16x16_2_1_0,
            13 => MbType::I16x16_3_1_0,
            14 => MbType::I16x16_0_2_0,
            15 => MbType::I16x16_1_2_0,
            16 => MbType::I16x16_2_2_0,
            17 => MbType::I16x16_3_2_0,
            18 => MbType::I16x16_0_0_1,
            19 => MbType::I16x16_1_0_1,
            20 => MbType::I16x16_2_0_1,
            21 => MbType::I16x16_3_0_1,
            22 => MbType::I16x16_0_1_1,
            23 => MbType::I16x16_1_1_1,
            24 => MbType::I16x16_2_1_1,
            25 => MbType::I16x16_3_1_1,
            26 => MbType::I16x16_0_2_1,
            27 => MbType::I16x16_1_2_1,
            28 => MbType::I16x16_2_2_1,
            29 => MbType::I16x16_3_2_1,
            30 => MbType::IPCM,

            _ => MbType::PL016x16,
        }
    }
}

/// Generate a random B macroblock type
pub fn random_b_mbtype(
    ignore_intra_pred: bool,
    ignore_ipcm: bool,
    rconfig: &RandomMBRange,
    film: &mut FilmState,
) -> MbType {
    if ignore_intra_pred {
        match rconfig.mb_b_type.sample(film) {
            0 => MbType::BDirect16x16,
            1 => MbType::BL016x16,
            2 => MbType::BL116x16,
            3 => MbType::BBi16x16,
            4 => MbType::BL0L016x8,
            5 => MbType::BL0L08x16,
            6 => MbType::BL1L116x8,
            7 => MbType::BL1L18x16,
            8 => MbType::BL0L116x8,
            9 => MbType::BL0L18x16,
            10 => MbType::BL1L016x8,
            11 => MbType::BL1L08x16,
            12 => MbType::BL0Bi16x8,
            13 => MbType::BL0Bi8x16,
            14 => MbType::BL1Bi16x8,
            15 => MbType::BL1Bi8x16,
            16 => MbType::BBiL016x8,
            17 => MbType::BBiL08x16,
            18 => MbType::BBiL116x8,
            19 => MbType::BBiL18x16,
            20 => MbType::BBiBi16x8,
            21 => MbType::BBiBi8x16,
            22 => MbType::B8x8,
            _ => MbType::BDirect16x16,
        }
    } else if ignore_ipcm {
        // can only be I type or B type
        match rconfig.mb_b_type.sample(film) {
            0 => MbType::BDirect16x16,
            1 => MbType::BL016x16,
            2 => MbType::BL116x16,
            3 => MbType::BBi16x16,
            4 => MbType::BL0L016x8,
            5 => MbType::BL0L08x16,
            6 => MbType::BL1L116x8,
            7 => MbType::BL1L18x16,
            8 => MbType::BL0L116x8,
            9 => MbType::BL0L18x16,
            10 => MbType::BL1L016x8,
            11 => MbType::BL1L08x16,
            12 => MbType::BL0Bi16x8,
            13 => MbType::BL0Bi8x16,
            14 => MbType::BL1Bi16x8,
            15 => MbType::BL1Bi8x16,
            16 => MbType::BBiL016x8,
            17 => MbType::BBiL08x16,
            18 => MbType::BBiL116x8,
            19 => MbType::BBiL18x16,
            20 => MbType::BBiBi16x8,
            21 => MbType::BBiBi8x16,
            22 => MbType::B8x8,
            23 => MbType::INxN,
            24 => MbType::I16x16_0_0_0,
            25 => MbType::I16x16_1_0_0,
            26 => MbType::I16x16_2_0_0,
            27 => MbType::I16x16_3_0_0,
            28 => MbType::I16x16_0_1_0,
            29 => MbType::I16x16_1_1_0,
            30 => MbType::I16x16_2_1_0,
            31 => MbType::I16x16_3_1_0,
            32 => MbType::I16x16_0_2_0,
            33 => MbType::I16x16_1_2_0,
            34 => MbType::I16x16_2_2_0,
            35 => MbType::I16x16_3_2_0,
            36 => MbType::I16x16_0_0_1,
            37 => MbType::I16x16_1_0_1,
            38 => MbType::I16x16_2_0_1,
            39 => MbType::I16x16_3_0_1,
            40 => MbType::I16x16_0_1_1,
            41 => MbType::I16x16_1_1_1,
            42 => MbType::I16x16_2_1_1,
            43 => MbType::I16x16_3_1_1,
            44 => MbType::I16x16_0_2_1,
            45 => MbType::I16x16_1_2_1,
            46 => MbType::I16x16_2_2_1,
            47 => MbType::I16x16_3_2_1,
            _ => MbType::BDirect16x16,
        }
    } else {
        // can only be I type or B type
        match rconfig.mb_b_type.sample(film) {
            0 => MbType::BDirect16x16,
            1 => MbType::BL016x16,
            2 => MbType::BL116x16,
            3 => MbType::BBi16x16,
            4 => MbType::BL0L016x8,
            5 => MbType::BL0L08x16,
            6 => MbType::BL1L116x8,
            7 => MbType::BL1L18x16,
            8 => MbType::BL0L116x8,
            9 => MbType::BL0L18x16,
            10 => MbType::BL1L016x8,
            11 => MbType::BL1L08x16,
            12 => MbType::BL0Bi16x8,
            13 => MbType::BL0Bi8x16,
            14 => MbType::BL1Bi16x8,
            15 => MbType::BL1Bi8x16,
            16 => MbType::BBiL016x8,
            17 => MbType::BBiL08x16,
            18 => MbType::BBiL116x8,
            19 => MbType::BBiL18x16,
            20 => MbType::BBiBi16x8,
            21 => MbType::BBiBi8x16,
            22 => MbType::B8x8,
            23 => MbType::INxN,
            24 => MbType::I16x16_0_0_0,
            25 => MbType::I16x16_1_0_0,
            26 => MbType::I16x16_2_0_0,
            27 => MbType::I16x16_3_0_0,
            28 => MbType::I16x16_0_1_0,
            29 => MbType::I16x16_1_1_0,
            30 => MbType::I16x16_2_1_0,
            31 => MbType::I16x16_3_1_0,
            32 => MbType::I16x16_0_2_0,
            33 => MbType::I16x16_1_2_0,
            34 => MbType::I16x16_2_2_0,
            35 => MbType::I16x16_3_2_0,
            36 => MbType::I16x16_0_0_1,
            37 => MbType::I16x16_1_0_1,
            38 => MbType::I16x16_2_0_1,
            39 => MbType::I16x16_3_0_1,
            40 => MbType::I16x16_0_1_1,
            41 => MbType::I16x16_1_1_1,
            42 => MbType::I16x16_2_1_1,
            43 => MbType::I16x16_3_1_1,
            44 => MbType::I16x16_0_2_1,
            45 => MbType::I16x16_1_2_1,
            46 => MbType::I16x16_2_2_1,
            47 => MbType::I16x16_3_2_1,
            48 => MbType::IPCM,
            _ => MbType::BDirect16x16,
        }
    }
}

/// Generate a random P submacroblock type following Table 7-17
fn random_p_submbtype(rconfig: &RandomMBRange, film: &mut FilmState) -> SubMbType {
    match rconfig.sub_mb_type_p.sample(film) {
        0 => SubMbType::PL08x8,
        1 => SubMbType::PL08x4,
        2 => SubMbType::PL04x8,
        3 => SubMbType::PL04x4,

        _ => SubMbType::PL08x8,
    }
}

/// Generate a random B submacroblock type
fn random_b_submbtype(rconfig: &RandomMBRange, film: &mut FilmState) -> SubMbType {
    match rconfig.sub_mb_type_b.sample(film) {
        0 => SubMbType::BDirect8x8,
        1 => SubMbType::BL08x8,
        2 => SubMbType::BL18x8,
        3 => SubMbType::BBi8x8,
        4 => SubMbType::BL08x4,
        5 => SubMbType::BL04x8,
        6 => SubMbType::BL18x4,
        7 => SubMbType::BL14x8,
        8 => SubMbType::BBi8x4,
        9 => SubMbType::BBi4x8,
        10 => SubMbType::BL04x4,
        11 => SubMbType::BL14x4,
        12 => SubMbType::BBi4x4,

        _ => SubMbType::BDirect8x8,
    }
}

/// Generate random CAVLC coeff_token
fn randomize_coeff_token(
    max_num_ceoff: usize,
    residual_mode: ResidualMode,
    vp: &VideoParameters,
    rconfig: &RandomMBRange,
    film: &mut FilmState,
) -> CoeffToken {
    // determine our range based of off chroma_array

    let total_coeff: usize;

    if residual_mode == ResidualMode::ChromaDCLevel {
        if vp.chroma_array_type == 1 {
            // n_c == -1 so bounded from 0 to 4
            total_coeff =
                (rconfig.total_coeff.sample(film) as usize) % (cmp::min(9, max_num_ceoff));
        } else {
            // n_c == -2 so bounded from 0 to 8
            total_coeff =
                (rconfig.total_coeff.sample(film) as usize) % (cmp::min(5, max_num_ceoff));
        }
    } else {
        total_coeff = (rconfig.total_coeff.sample(film) as usize) % max_num_ceoff;
        // should be [0, 16]
    }

    let trailing_ones: usize =
        (rconfig.trailing_ones.sample(film) as usize) % (cmp::min(4, total_coeff + 1));

    CoeffToken {
        n_c: 0, // gets computed at encode time
        total_coeff,
        trailing_ones,
    }
}

/// Generate random CAVLC total_zeros
fn randomize_total_zeros(
    max_num_ceoff: usize,
    tz_vcl_index: usize,
    rconfig: &RandomMBRange,
    film: &mut FilmState,
) -> usize {
    let res: usize =
        (rconfig.total_zeros.sample(film) as usize) % (max_num_ceoff - tz_vcl_index + 1);

    res
}

/// Generate random CAVLC run_before
fn randomize_run_before(zeros_left: i32, rconfig: &RandomMBRange, film: &mut FilmState) -> usize {
    let res: usize = if zeros_left > 0 {
        (rconfig.run_before.sample(film) as usize) % (zeros_left as usize + 1)
    } else {
        0
    };

    res
}

/// Recover the AC residue values for later CAVLC calls to ac_resid_all_zero
///
/// Mode: 0 (i16x16aclevel); 1 (cbi16x16aclevel); 2 (cri16x16aclevel); 3 (chroma ac)
fn fill_in_ac_residue(
    ac_mode: usize,
    i_cb_cr: usize,
    transform_block_idx: usize,
    slice_idx: usize,
    macroblock_idx: usize,
    ds: &mut H264DecodedStream,
) {
    // Input for recovery
    let coeff_token: &CoeffToken;
    let trailing_ones_sign_flag: &Vec<bool>;
    let level_prefix_list: &Vec<u32>;
    let level_suffix_list: &Vec<u32>;
    let total_zeros: usize;
    let run_before_list: &Vec<usize>;

    match ac_mode {
        0 => {
            coeff_token = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .coeff_token;
            trailing_ones_sign_flag = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .trailing_ones_sign_flag;
            level_prefix_list = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .level_prefix;
            level_suffix_list = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .level_suffix;
            total_zeros = ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .total_zeros;
            run_before_list = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .run_before;
        }
        1 => {
            coeff_token = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .cb_intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .coeff_token;
            trailing_ones_sign_flag = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .cb_intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .trailing_ones_sign_flag;
            level_prefix_list = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .cb_intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .level_prefix;
            level_suffix_list = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .cb_intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .level_suffix;
            total_zeros = ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .cb_intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .total_zeros;
            run_before_list = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .cb_intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .run_before;
        }
        2 => {
            coeff_token = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .cr_intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .coeff_token;
            trailing_ones_sign_flag = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .cr_intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .trailing_ones_sign_flag;
            level_prefix_list = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .cr_intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .level_prefix;
            level_suffix_list = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .cr_intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .level_suffix;
            total_zeros = ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .cr_intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .total_zeros;
            run_before_list = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .cr_intra_16x16_ac_level_transform_blocks[transform_block_idx]
                .run_before;
        }
        _ => {
            // or 3
            coeff_token = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .chroma_ac_level_transform_blocks[i_cb_cr][transform_block_idx]
                .coeff_token;
            trailing_ones_sign_flag = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .chroma_ac_level_transform_blocks[i_cb_cr][transform_block_idx]
                .trailing_ones_sign_flag;
            level_prefix_list = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .chroma_ac_level_transform_blocks[i_cb_cr][transform_block_idx]
                .level_prefix;
            level_suffix_list = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .chroma_ac_level_transform_blocks[i_cb_cr][transform_block_idx]
                .level_suffix;
            total_zeros = ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .chroma_ac_level_transform_blocks[i_cb_cr][transform_block_idx]
                .total_zeros;
            run_before_list = &ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .chroma_ac_level_transform_blocks[i_cb_cr][transform_block_idx]
                .run_before;
        }
    }

    // Intermediate variables
    let mut level_val: Vec<i32> = Vec::new();
    let mut level_code: i32;
    let mut zeros_left: usize;
    let mut run_val: Vec<usize> = Vec::new();

    // output
    let mut coeff_level = vec![0; 16]; // 15 is usually the end idx but we add an extra for safesies

    // we copy the code from the decoder
    if coeff_token.total_coeff > 0 {
        let mut suffix_length = 0;

        if coeff_token.total_coeff > 10 && coeff_token.trailing_ones < 3 {
            suffix_length = 1;
        }

        for i in 0..coeff_token.total_coeff {
            level_val.push(0);
            if i < coeff_token.trailing_ones {
                level_val[i] = 1 - 2 * match trailing_ones_sign_flag[i] {
                    false => 0,
                    true => 1,
                };
            } else {
                let level_prefix = level_prefix_list[i];

                level_code = cmp::min(15, level_prefix as i32) << suffix_length;
                if suffix_length > 0 || level_prefix >= 14 {
                    level_code += level_suffix_list[i] as i32;
                }

                if level_prefix >= 15 && suffix_length == 0 {
                    level_code += 15;
                }
                if level_prefix >= 16 {
                    level_code += 1i32.checked_shl(level_prefix - 3).unwrap_or(0).wrapping_sub(4096);
                }
                if i == coeff_token.trailing_ones && coeff_token.trailing_ones < 3 {
                    level_code += 2;
                }
                if level_code % 2 == 0 {
                    level_val[i] = (level_code + 2) >> 1;
                } else {
                    level_val[i] = (-level_code - 1) >> 1;
                }
                if suffix_length == 0 {
                    suffix_length = 1;
                }
                if level_val[i].abs() > (3 << (suffix_length - 1)) && suffix_length < 6 {
                    suffix_length += 1;
                }
            }
        }
        if coeff_token.total_coeff < 16 {
            zeros_left = total_zeros;
        } else {
            zeros_left = 0;
        }
        for i in 0..coeff_token.total_coeff - 1 {
            // to ensure we have something at each ith position
            run_val.push(0);

            if zeros_left > 0 {
                run_val[i] = run_before_list[i];
            } // else leave at 0
            zeros_left -= run_val[i]; // conformance specifies that this should always be greater than or equal to 0
        }
        run_val.push(zeros_left);

        let mut coeff_num = 0;
        for i in (0..=coeff_token.total_coeff - 1).rev() {
            if i == coeff_token.total_coeff - 1 {
                coeff_num += run_val[i]; // ignore the +1 for the first time around
            } else {
                coeff_num += run_val[i] + 1;
            }

            coeff_level[coeff_num] = level_val[i];
        }
    }

    match ac_mode {
        0 => {
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].intra_16x16_ac_level
                [transform_block_idx] = coeff_level.clone();
        }
        1 => {
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].cb_intra_16x16_ac_level
                [transform_block_idx] = coeff_level.clone();
        }
        2 => {
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].cr_intra_16x16_ac_level
                [transform_block_idx] = coeff_level.clone();
        }
        _ => {
            // or 3
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].chroma_ac_level[i_cb_cr]
                [transform_block_idx] = coeff_level.clone();
        }
    }
}

/// Generate a random Macroblock residual data
pub fn randomize_residual(
    slice_idx: usize,
    macroblock_idx: usize,
    vp: &VideoParameters,
    rconfig: &RandomMBRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    // Luma elements

    // i16x16DCLevel is length 16
    if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].mb_part_pred_mode(0)
        == MbPartPredMode::Intra16x16
    {
        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
            .intra_16x16_dc_level_transform_blocks
            .available = true;

        if vp.entropy_coding_mode_flag {
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_dc_level_transform_blocks
                .coded_block_flag = true;
            // reset existing values
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_dc_level_transform_blocks
                .significant_coeff_flag = Vec::new();
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_dc_level_transform_blocks
                .last_significant_coeff_flag = Vec::new();
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_dc_level_transform_blocks
                .coeff_sign_flag = Vec::new();
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_dc_level_transform_blocks
                .coeff_abs_level_minus1 = Vec::new();

            for _ in 0..16 {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .intra_16x16_dc_level_transform_blocks
                    .significant_coeff_flag
                    .push(rconfig.significant_coeff_flag.sample(film));
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .intra_16x16_dc_level_transform_blocks
                    .last_significant_coeff_flag
                    .push(rconfig.last_significant_coeff_flag.sample(film));
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .intra_16x16_dc_level_transform_blocks
                    .coeff_sign_flag
                    .push(rconfig.coeff_sign_flag.sample(film));

                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .intra_16x16_dc_level_transform_blocks
                    .coeff_abs_level_minus1
                    .push(rconfig.coeff_abs_level_minus1.sample(film));
            }
        } else {
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_dc_level_transform_blocks
                .coeff_token =
                randomize_coeff_token(16, ResidualMode::Intra16x16DCLevel, vp, rconfig, film);
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_dc_level_transform_blocks
                .total_zeros = randomize_total_zeros(
                16,
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .intra_16x16_dc_level_transform_blocks
                    .coeff_token
                    .total_coeff,
                rconfig,
                film,
            );

            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_dc_level_transform_blocks
                .trailing_ones_sign_flag = Vec::new();
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_dc_level_transform_blocks
                .level_prefix = Vec::new();
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_dc_level_transform_blocks
                .level_suffix = Vec::new();
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_dc_level_transform_blocks
                .run_before = Vec::new();

            let mut zeros_left: i32 = ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .intra_16x16_dc_level_transform_blocks
                .total_zeros as i32;
            let mut run_val: usize = 0;
            for _ in 0..16 {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .intra_16x16_dc_level_transform_blocks
                    .trailing_ones_sign_flag
                    .push(rconfig.trailing_ones_sign_flag.sample(film));
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .intra_16x16_dc_level_transform_blocks
                    .level_prefix
                    .push(rconfig.level_prefix.sample(film));
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .intra_16x16_dc_level_transform_blocks
                    .level_suffix
                    .push(rconfig.level_suffix.sample(film));
                if zeros_left > 0 {
                    run_val = randomize_run_before(zeros_left, rconfig, film);
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .intra_16x16_dc_level_transform_blocks
                        .run_before
                        .push(run_val);
                } else {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .intra_16x16_dc_level_transform_blocks
                        .run_before
                        .push(0);
                }
                zeros_left -= run_val as i32;
            }
        }
    }

    // i16x16ACLevel: there are 16 arrays of length 16
    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].intra_16x16_ac_level_transform_blocks =
        Vec::new();
    // level4x4: there are 16 arrays of length 16
    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].luma_level_4x4_transform_blocks =
        Vec::new();
    // level8x8: there are 4 arrays of length 64
    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].luma_level_8x8_transform_blocks =
        Vec::new();

    for i_8x8 in 0..4 {
        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
            .luma_level_8x8_transform_blocks
            .push(TransformBlock::new());

        if !ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].transform_size_8x8_flag
            || !vp.entropy_coding_mode_flag
        {
            for i_4x4 in 0..4 {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .intra_16x16_ac_level_transform_blocks
                    .push(TransformBlock::new());
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .luma_level_4x4_transform_blocks
                    .push(TransformBlock::new());
                // prepare the AC values
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .intra_16x16_ac_level
                    .push(Vec::new());
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .luma_level_4x4
                    .push(Vec::new());
                if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].coded_block_pattern_luma
                    & (1 << i_8x8)
                    > 0
                {
                    let i = i_8x8 * 4 + i_4x4;

                    if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].mb_part_pred_mode(0)
                        == MbPartPredMode::Intra16x16
                    {
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .intra_16x16_ac_level_transform_blocks[i]
                            .available = true;
                        if vp.entropy_coding_mode_flag {
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .intra_16x16_ac_level_transform_blocks[i]
                                .coded_block_flag = true;

                            // reset existing values
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .intra_16x16_ac_level_transform_blocks[i]
                                .significant_coeff_flag = Vec::new();
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .intra_16x16_ac_level_transform_blocks[i]
                                .last_significant_coeff_flag = Vec::new();
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .intra_16x16_ac_level_transform_blocks[i]
                                .coeff_sign_flag = Vec::new();
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .intra_16x16_ac_level_transform_blocks[i]
                                .coeff_abs_level_minus1 = Vec::new();

                            for _ in 0..16 {
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .intra_16x16_ac_level_transform_blocks[i]
                                    .significant_coeff_flag
                                    .push(rconfig.significant_coeff_flag.sample(film));
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .intra_16x16_ac_level_transform_blocks[i]
                                    .last_significant_coeff_flag
                                    .push(rconfig.last_significant_coeff_flag.sample(film));
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .intra_16x16_ac_level_transform_blocks[i]
                                    .coeff_sign_flag
                                    .push(rconfig.coeff_sign_flag.sample(film));

                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .intra_16x16_ac_level_transform_blocks[i]
                                    .coeff_abs_level_minus1
                                    .push(rconfig.coeff_abs_level_minus1.sample(film));
                            }
                        } else {
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .intra_16x16_ac_level_transform_blocks[i]
                                .coeff_token = randomize_coeff_token(
                                15,
                                ResidualMode::Intra16x16ACLevel,
                                vp,
                                rconfig,
                                film,
                            );
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .intra_16x16_ac_level_transform_blocks[i]
                                .total_zeros = randomize_total_zeros(
                                15,
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .intra_16x16_ac_level_transform_blocks[i]
                                    .coeff_token
                                    .total_coeff,
                                rconfig,
                                film,
                            );

                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .intra_16x16_ac_level_transform_blocks[i]
                                .trailing_ones_sign_flag = Vec::new();
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .intra_16x16_ac_level_transform_blocks[i]
                                .level_prefix = Vec::new();
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .intra_16x16_ac_level_transform_blocks[i]
                                .level_suffix = Vec::new();
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .intra_16x16_ac_level_transform_blocks[i]
                                .run_before = Vec::new();

                            let mut zeros_left: i32 =
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .intra_16x16_ac_level_transform_blocks[i]
                                    .total_zeros as i32;
                            let mut run_val: usize = 0;
                            for _ in 0..16 {
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .intra_16x16_ac_level_transform_blocks[i]
                                    .trailing_ones_sign_flag
                                    .push(rconfig.trailing_ones_sign_flag.sample(film));
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .intra_16x16_ac_level_transform_blocks[i]
                                    .level_prefix
                                    .push(rconfig.level_prefix.sample(film));
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .intra_16x16_ac_level_transform_blocks[i]
                                    .level_suffix
                                    .push(rconfig.level_suffix.sample(film));
                                if zeros_left > 0 {
                                    run_val = randomize_run_before(zeros_left, rconfig, film);
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .intra_16x16_ac_level_transform_blocks[i]
                                        .run_before
                                        .push(run_val);
                                } else {
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .intra_16x16_ac_level_transform_blocks[i]
                                        .run_before
                                        .push(0);
                                }
                                zeros_left -= run_val as i32;
                            }

                            // Fill in i16x16ACLevel
                            fill_in_ac_residue(0, 0, i, slice_idx, macroblock_idx, ds);
                        }
                        // Need to copy it it to luma4x4 because it turns out these should originally be the same?
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .luma_level_4x4_transform_blocks[i] =
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .intra_16x16_ac_level_transform_blocks[i]
                                .clone();
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].luma_level_4x4 =
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .intra_16x16_ac_level
                                .clone();
                    } else {
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .luma_level_4x4_transform_blocks[i]
                            .available = true;
                        if vp.entropy_coding_mode_flag {
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .luma_level_4x4_transform_blocks[i]
                                .coded_block_flag = true;
                            // reset existing values
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .luma_level_4x4_transform_blocks[i]
                                .significant_coeff_flag = Vec::new();
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .luma_level_4x4_transform_blocks[i]
                                .last_significant_coeff_flag = Vec::new();
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .luma_level_4x4_transform_blocks[i]
                                .coeff_sign_flag = Vec::new();
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .luma_level_4x4_transform_blocks[i]
                                .coeff_abs_level_minus1 = Vec::new();

                            for _ in 0..16 {
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .luma_level_4x4_transform_blocks[i]
                                    .significant_coeff_flag
                                    .push(rconfig.significant_coeff_flag.sample(film));
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .luma_level_4x4_transform_blocks[i]
                                    .last_significant_coeff_flag
                                    .push(rconfig.last_significant_coeff_flag.sample(film));
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .luma_level_4x4_transform_blocks[i]
                                    .coeff_sign_flag
                                    .push(rconfig.coeff_sign_flag.sample(film));

                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .luma_level_4x4_transform_blocks[i]
                                    .coeff_abs_level_minus1
                                    .push(rconfig.coeff_abs_level_minus1.sample(film));
                            }
                        } else {
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .luma_level_4x4_transform_blocks[i]
                                .coeff_token = randomize_coeff_token(
                                16,
                                ResidualMode::LumaLevel4x4,
                                vp,
                                rconfig,
                                film,
                            );
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .luma_level_4x4_transform_blocks[i]
                                .total_zeros = randomize_total_zeros(
                                16,
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .luma_level_4x4_transform_blocks[i]
                                    .coeff_token
                                    .total_coeff,
                                rconfig,
                                film,
                            );

                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .luma_level_4x4_transform_blocks[i]
                                .trailing_ones_sign_flag = Vec::new();
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .luma_level_4x4_transform_blocks[i]
                                .level_prefix = Vec::new();
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .luma_level_4x4_transform_blocks[i]
                                .level_suffix = Vec::new();
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .luma_level_4x4_transform_blocks[i]
                                .run_before = Vec::new();

                            let mut zeros_left: i32 =
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .luma_level_4x4_transform_blocks[i]
                                    .total_zeros as i32;
                            let mut run_val: usize = 0;
                            for _ in 0..16 {
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .luma_level_4x4_transform_blocks[i]
                                    .trailing_ones_sign_flag
                                    .push(rconfig.trailing_ones_sign_flag.sample(film));
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .luma_level_4x4_transform_blocks[i]
                                    .level_prefix
                                    .push(rconfig.level_prefix.sample(film));
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .luma_level_4x4_transform_blocks[i]
                                    .level_suffix
                                    .push(rconfig.level_suffix.sample(film));
                                if zeros_left > 0 {
                                    run_val = randomize_run_before(zeros_left, rconfig, film);
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .luma_level_4x4_transform_blocks[i]
                                        .run_before
                                        .push(run_val);
                                } else {
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .luma_level_4x4_transform_blocks[i]
                                        .run_before
                                        .push(0);
                                }
                                zeros_left -= run_val as i32;
                            }
                        }
                        // Need to copy it it to luma4x4 because it turns out these should originally be the same?
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .intra_16x16_ac_level_transform_blocks[i] =
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .luma_level_4x4_transform_blocks[i]
                                .clone();
                    }
                }
            }
        } else if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].coded_block_pattern_luma
            & (1 << i_8x8)
            > 0
        {
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .luma_level_8x8_transform_blocks[i_8x8]
                .available = true;

            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .luma_level_8x8_transform_blocks[i_8x8]
                .coded_block_flag = true;
            // reset existing values
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .luma_level_8x8_transform_blocks[i_8x8]
                .significant_coeff_flag = Vec::new();
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .luma_level_8x8_transform_blocks[i_8x8]
                .last_significant_coeff_flag = Vec::new();
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .luma_level_8x8_transform_blocks[i_8x8]
                .coeff_sign_flag = Vec::new();
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .luma_level_8x8_transform_blocks[i_8x8]
                .coeff_abs_level_minus1 = Vec::new();

            for _ in 0..64 {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .luma_level_8x8_transform_blocks[i_8x8]
                    .significant_coeff_flag
                    .push(rconfig.significant_coeff_flag.sample(film));
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .luma_level_8x8_transform_blocks[i_8x8]
                    .last_significant_coeff_flag
                    .push(rconfig.last_significant_coeff_flag.sample(film));
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .luma_level_8x8_transform_blocks[i_8x8]
                    .coeff_sign_flag
                    .push(rconfig.coeff_sign_flag.sample(film));
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .luma_level_8x8_transform_blocks[i_8x8]
                    .coeff_abs_level_minus1
                    .push(rconfig.coeff_abs_level_minus1.sample(film));
            }
        }
    }

    // YUV420 and YUV422 Chroma values
    if vp.chroma_array_type == 1 || vp.chroma_array_type == 2 {
        // chromaDCLevel there are 2 arrays of length 16
        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].chroma_dc_level_transform_blocks =
            Vec::new();
        // chromaACLevel there are 2 arrays, each with 16 arrays of length 16
        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].chroma_ac_level_transform_blocks =
            Vec::new();
        for i_cb_cr in 0..2 {
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .chroma_dc_level_transform_blocks
                .push(TransformBlock::new());
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .chroma_ac_level
                .push(Vec::new());

            // ChromaDC values
            if (ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].coded_block_pattern_chroma
                & 3)
                > 0
            {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .chroma_dc_level_transform_blocks[i_cb_cr]
                    .available = true;
                if vp.entropy_coding_mode_flag {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .chroma_dc_level_transform_blocks[i_cb_cr]
                        .coded_block_flag = true;
                    // reset existing values
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .chroma_dc_level_transform_blocks[i_cb_cr]
                        .significant_coeff_flag = Vec::new();
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .chroma_dc_level_transform_blocks[i_cb_cr]
                        .last_significant_coeff_flag = Vec::new();
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .chroma_dc_level_transform_blocks[i_cb_cr]
                        .coeff_sign_flag = Vec::new();
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .chroma_dc_level_transform_blocks[i_cb_cr]
                        .coeff_abs_level_minus1 = Vec::new();

                    for _ in 0..16 {
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_dc_level_transform_blocks[i_cb_cr]
                            .significant_coeff_flag
                            .push(rconfig.significant_coeff_flag.sample(film));
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_dc_level_transform_blocks[i_cb_cr]
                            .last_significant_coeff_flag
                            .push(rconfig.last_significant_coeff_flag.sample(film));
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_dc_level_transform_blocks[i_cb_cr]
                            .coeff_sign_flag
                            .push(rconfig.coeff_sign_flag.sample(film));
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_dc_level_transform_blocks[i_cb_cr]
                            .coeff_abs_level_minus1
                            .push(rconfig.coeff_abs_level_minus1.sample(film));
                    }
                } else {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .chroma_dc_level_transform_blocks[i_cb_cr]
                        .coeff_token = randomize_coeff_token(
                        4 * ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].num_c8x8,
                        ResidualMode::ChromaDCLevel,
                        vp,
                        rconfig,
                        film,
                    );
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .chroma_dc_level_transform_blocks[i_cb_cr]
                        .total_zeros = randomize_total_zeros(
                        4 * ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].num_c8x8,
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_dc_level_transform_blocks[i_cb_cr]
                            .coeff_token
                            .total_coeff,
                        rconfig,
                        film,
                    );

                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .chroma_dc_level_transform_blocks[i_cb_cr]
                        .trailing_ones_sign_flag = Vec::new();
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .chroma_dc_level_transform_blocks[i_cb_cr]
                        .level_prefix = Vec::new();
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .chroma_dc_level_transform_blocks[i_cb_cr]
                        .level_suffix = Vec::new();
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .chroma_dc_level_transform_blocks[i_cb_cr]
                        .run_before = Vec::new();

                    let mut zeros_left: i32 = ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .chroma_dc_level_transform_blocks[i_cb_cr]
                        .total_zeros as i32;
                    let mut run_val: usize = 0;
                    for _ in 0..16 {
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_dc_level_transform_blocks[i_cb_cr]
                            .trailing_ones_sign_flag
                            .push(rconfig.trailing_ones_sign_flag.sample(film));
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_dc_level_transform_blocks[i_cb_cr]
                            .level_prefix
                            .push(rconfig.level_prefix.sample(film));
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_dc_level_transform_blocks[i_cb_cr]
                            .level_suffix
                            .push(rconfig.level_suffix.sample(film));
                        if zeros_left > 0 {
                            run_val = randomize_run_before(zeros_left, rconfig, film);
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .chroma_dc_level_transform_blocks[i_cb_cr]
                                .run_before
                                .push(run_val);
                        } else {
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .chroma_dc_level_transform_blocks[i_cb_cr]
                                .run_before
                                .push(0);
                        }
                        zeros_left -= run_val as i32;
                    }
                }
            }

            // ChromaAC values
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .chroma_ac_level_transform_blocks
                .push(Vec::new());
            for j in 0..16 {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .chroma_ac_level_transform_blocks[i_cb_cr]
                    .push(TransformBlock::new());
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].chroma_ac_level[i_cb_cr]
                    .push(Vec::new());

                if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].coded_block_pattern_chroma
                    & 2
                    > 1
                {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .chroma_ac_level_transform_blocks[i_cb_cr][j]
                        .available = true;
                    if vp.entropy_coding_mode_flag {
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_ac_level_transform_blocks[i_cb_cr][j]
                            .coded_block_flag = true;

                        // reset existing values
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_ac_level_transform_blocks[i_cb_cr][j]
                            .significant_coeff_flag = Vec::new();
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_ac_level_transform_blocks[i_cb_cr][j]
                            .last_significant_coeff_flag = Vec::new();
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_ac_level_transform_blocks[i_cb_cr][j]
                            .coeff_sign_flag = Vec::new();
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_ac_level_transform_blocks[i_cb_cr][j]
                            .coeff_abs_level_minus1 = Vec::new();

                        for _ in 0..16 {
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .chroma_ac_level_transform_blocks[i_cb_cr][j]
                                .significant_coeff_flag
                                .push(rconfig.significant_coeff_flag.sample(film));
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .chroma_ac_level_transform_blocks[i_cb_cr][j]
                                .last_significant_coeff_flag
                                .push(rconfig.last_significant_coeff_flag.sample(film));
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .chroma_ac_level_transform_blocks[i_cb_cr][j]
                                .coeff_sign_flag
                                .push(rconfig.coeff_sign_flag.sample(film));
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .chroma_ac_level_transform_blocks[i_cb_cr][j]
                                .coeff_abs_level_minus1
                                .push(rconfig.coeff_abs_level_minus1.sample(film));
                        }
                    } else {
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_ac_level_transform_blocks[i_cb_cr][j]
                            .coeff_token = randomize_coeff_token(
                            15,
                            ResidualMode::ChromaACLevel,
                            vp,
                            rconfig,
                            film,
                        );
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_ac_level_transform_blocks[i_cb_cr][j]
                            .total_zeros = randomize_total_zeros(
                            15,
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .chroma_ac_level_transform_blocks[i_cb_cr][j]
                                .coeff_token
                                .total_coeff,
                            rconfig,
                            film,
                        );
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_ac_level_transform_blocks[i_cb_cr][j]
                            .trailing_ones_sign_flag = Vec::new();
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_ac_level_transform_blocks[i_cb_cr][j]
                            .level_prefix = Vec::new();
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_ac_level_transform_blocks[i_cb_cr][j]
                            .level_suffix = Vec::new();
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .chroma_ac_level_transform_blocks[i_cb_cr][j]
                            .run_before = Vec::new();

                        let mut zeros_left: i32 = ds.slices[slice_idx].sd.macroblock_vec
                            [macroblock_idx]
                            .chroma_ac_level_transform_blocks[i_cb_cr][j]
                            .total_zeros as i32;
                        let mut run_val: usize = 0;
                        for _ in 0..16 {
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .chroma_ac_level_transform_blocks[i_cb_cr][j]
                                .trailing_ones_sign_flag
                                .push(rconfig.trailing_ones_sign_flag.sample(film));
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .chroma_ac_level_transform_blocks[i_cb_cr][j]
                                .level_prefix
                                .push(rconfig.level_prefix.sample(film));
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .chroma_ac_level_transform_blocks[i_cb_cr][j]
                                .level_suffix
                                .push(rconfig.level_suffix.sample(film));
                            if zeros_left > 0 {
                                run_val = randomize_run_before(zeros_left, rconfig, film);
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .chroma_ac_level_transform_blocks[i_cb_cr][j]
                                    .run_before
                                    .push(run_val);
                            } else {
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .chroma_ac_level_transform_blocks[i_cb_cr][j]
                                    .run_before
                                    .push(0);
                            }
                            zeros_left -= run_val as i32;
                        }
                        // Fill in ChromaAC
                        fill_in_ac_residue(3, i_cb_cr, j, slice_idx, macroblock_idx, ds);
                    }
                }
            }
        }
    } else if vp.chroma_array_type == 3 {
        // CbIntra16x16DCLevel is length 16
        if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].mb_part_pred_mode(0)
            == MbPartPredMode::Intra16x16
        {
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .cb_intra_16x16_dc_level_transform_blocks
                .available = true;
            if vp.entropy_coding_mode_flag {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_intra_16x16_dc_level_transform_blocks
                    .coded_block_flag = true;

                // reset existing values
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_intra_16x16_dc_level_transform_blocks
                    .significant_coeff_flag = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_intra_16x16_dc_level_transform_blocks
                    .last_significant_coeff_flag = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_intra_16x16_dc_level_transform_blocks
                    .coeff_sign_flag = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_intra_16x16_dc_level_transform_blocks
                    .coeff_abs_level_minus1 = Vec::new();

                for _ in 0..16 {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cb_intra_16x16_dc_level_transform_blocks
                        .significant_coeff_flag
                        .push(rconfig.significant_coeff_flag.sample(film));
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cb_intra_16x16_dc_level_transform_blocks
                        .last_significant_coeff_flag
                        .push(rconfig.last_significant_coeff_flag.sample(film));
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cb_intra_16x16_dc_level_transform_blocks
                        .coeff_sign_flag
                        .push(rconfig.coeff_sign_flag.sample(film));
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cb_intra_16x16_dc_level_transform_blocks
                        .coeff_abs_level_minus1
                        .push(rconfig.coeff_abs_level_minus1.sample(film));
                }
            } else {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_intra_16x16_dc_level_transform_blocks
                    .coeff_token =
                    randomize_coeff_token(16, ResidualMode::CbIntra16x16DCLevel, vp, rconfig, film);
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_intra_16x16_dc_level_transform_blocks
                    .total_zeros = randomize_total_zeros(
                    16,
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cb_intra_16x16_dc_level_transform_blocks
                        .coeff_token
                        .total_coeff,
                    rconfig,
                    film,
                );
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_intra_16x16_dc_level_transform_blocks
                    .trailing_ones_sign_flag = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_intra_16x16_dc_level_transform_blocks
                    .level_prefix = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_intra_16x16_dc_level_transform_blocks
                    .level_suffix = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_intra_16x16_dc_level_transform_blocks
                    .run_before = Vec::new();

                let mut zeros_left: i32 = ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_intra_16x16_dc_level_transform_blocks
                    .total_zeros as i32;
                let mut run_val: usize = 0;
                for _ in 0..16 {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cb_intra_16x16_dc_level_transform_blocks
                        .trailing_ones_sign_flag
                        .push(rconfig.trailing_ones_sign_flag.sample(film));
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cb_intra_16x16_dc_level_transform_blocks
                        .level_prefix
                        .push(rconfig.level_prefix.sample(film));
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cb_intra_16x16_dc_level_transform_blocks
                        .level_suffix
                        .push(rconfig.level_suffix.sample(film));
                    if zeros_left > 0 {
                        run_val = randomize_run_before(zeros_left, rconfig, film);
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .cb_intra_16x16_dc_level_transform_blocks
                            .run_before
                            .push(run_val);
                    } else {
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .cb_intra_16x16_dc_level_transform_blocks
                            .run_before
                            .push(0);
                    }
                    zeros_left -= run_val as i32;
                }
            }
        }

        // CbIntra16x16ACLevel: there are 16 arrays of length 16
        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
            .cb_intra_16x16_ac_level_transform_blocks = Vec::new();
        // CbLevel4x4: there are 16 arrays of length 16
        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].cb_level_4x4_transform_blocks =
            Vec::new();
        // CbLevel8x8: there are 4 arrays of length 64
        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].cb_level_8x8_transform_blocks =
            Vec::new();

        for i_8x8 in 0..4 {
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .cb_level_8x8_transform_blocks
                .push(TransformBlock::new());

            if !ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].transform_size_8x8_flag
                || !vp.entropy_coding_mode_flag
            {
                for i_4x4 in 0..4 {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cb_intra_16x16_ac_level_transform_blocks
                        .push(TransformBlock::new());
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cb_level_4x4_transform_blocks
                        .push(TransformBlock::new());
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cb_intra_16x16_ac_level
                        .push(Vec::new());

                    if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .coded_block_pattern_luma
                        & (1 << i_8x8)
                        > 0
                    {
                        let i = i_8x8 * 4 + i_4x4;

                        if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .mb_part_pred_mode(0)
                            == MbPartPredMode::Intra16x16
                        {
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .cb_intra_16x16_ac_level_transform_blocks[i]
                                .available = true;

                            if vp.entropy_coding_mode_flag {
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_intra_16x16_ac_level_transform_blocks[i]
                                    .coded_block_flag = true;

                                // reset existing values
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_intra_16x16_ac_level_transform_blocks[i]
                                    .significant_coeff_flag = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_intra_16x16_ac_level_transform_blocks[i]
                                    .last_significant_coeff_flag = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_intra_16x16_ac_level_transform_blocks[i]
                                    .coeff_sign_flag = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_intra_16x16_ac_level_transform_blocks[i]
                                    .coeff_abs_level_minus1 = Vec::new();

                                for _ in 0..16 {
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_intra_16x16_ac_level_transform_blocks[i]
                                        .significant_coeff_flag
                                        .push(rconfig.significant_coeff_flag.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_intra_16x16_ac_level_transform_blocks[i]
                                        .last_significant_coeff_flag
                                        .push(rconfig.last_significant_coeff_flag.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_intra_16x16_ac_level_transform_blocks[i]
                                        .coeff_sign_flag
                                        .push(rconfig.coeff_sign_flag.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_intra_16x16_ac_level_transform_blocks[i]
                                        .coeff_abs_level_minus1
                                        .push(rconfig.coeff_abs_level_minus1.sample(film));
                                }
                            } else {
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_intra_16x16_ac_level_transform_blocks[i]
                                    .coeff_token = randomize_coeff_token(
                                    15,
                                    ResidualMode::CbIntra16x16ACLevel,
                                    vp,
                                    rconfig,
                                    film,
                                );
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_intra_16x16_ac_level_transform_blocks[i]
                                    .total_zeros = randomize_total_zeros(
                                    15,
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_intra_16x16_ac_level_transform_blocks[i]
                                        .coeff_token
                                        .total_coeff,
                                    rconfig,
                                    film,
                                );
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_intra_16x16_ac_level_transform_blocks[i]
                                    .trailing_ones_sign_flag = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_intra_16x16_ac_level_transform_blocks[i]
                                    .level_prefix = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_intra_16x16_ac_level_transform_blocks[i]
                                    .level_suffix = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_intra_16x16_ac_level_transform_blocks[i]
                                    .run_before = Vec::new();

                                let mut zeros_left: i32 =
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_intra_16x16_ac_level_transform_blocks[i]
                                        .total_zeros as i32;
                                let mut run_val: usize = 0;
                                for _ in 0..16 {
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_intra_16x16_ac_level_transform_blocks[i]
                                        .trailing_ones_sign_flag
                                        .push(rconfig.trailing_ones_sign_flag.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_intra_16x16_ac_level_transform_blocks[i]
                                        .level_prefix
                                        .push(rconfig.level_prefix.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_intra_16x16_ac_level_transform_blocks[i]
                                        .level_suffix
                                        .push(rconfig.level_suffix.sample(film));
                                    if zeros_left > 0 {
                                        run_val = randomize_run_before(zeros_left, rconfig, film);
                                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                            .cb_intra_16x16_ac_level_transform_blocks[i]
                                            .run_before
                                            .push(run_val);
                                    } else {
                                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                            .cb_intra_16x16_ac_level_transform_blocks[i]
                                            .run_before
                                            .push(0);
                                    }
                                    zeros_left -= run_val as i32;
                                }
                                // Fill in CbIntra16x16ACLevel
                                fill_in_ac_residue(1, 0, i, slice_idx, macroblock_idx, ds);
                            }
                        } else {
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .cb_level_4x4_transform_blocks[i]
                                .available = true;

                            if vp.entropy_coding_mode_flag {
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_level_4x4_transform_blocks[i]
                                    .coded_block_flag = true;
                                // reset existing values
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_level_4x4_transform_blocks[i]
                                    .significant_coeff_flag = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_level_4x4_transform_blocks[i]
                                    .last_significant_coeff_flag = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_level_4x4_transform_blocks[i]
                                    .coeff_sign_flag = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_level_4x4_transform_blocks[i]
                                    .coeff_abs_level_minus1 = Vec::new();

                                for _ in 0..16 {
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_level_4x4_transform_blocks[i]
                                        .significant_coeff_flag
                                        .push(rconfig.significant_coeff_flag.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_level_4x4_transform_blocks[i]
                                        .last_significant_coeff_flag
                                        .push(rconfig.last_significant_coeff_flag.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_level_4x4_transform_blocks[i]
                                        .coeff_sign_flag
                                        .push(rconfig.coeff_sign_flag.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_level_4x4_transform_blocks[i]
                                        .coeff_abs_level_minus1
                                        .push(rconfig.coeff_abs_level_minus1.sample(film));
                                }
                            } else {
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_level_4x4_transform_blocks[i]
                                    .coeff_token = randomize_coeff_token(
                                    16,
                                    ResidualMode::CbLevel4x4,
                                    vp,
                                    rconfig,
                                    film,
                                );
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_level_4x4_transform_blocks[i]
                                    .total_zeros = randomize_total_zeros(
                                    16,
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_level_4x4_transform_blocks[i]
                                        .coeff_token
                                        .total_coeff,
                                    rconfig,
                                    film,
                                );
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_level_4x4_transform_blocks[i]
                                    .trailing_ones_sign_flag = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_level_4x4_transform_blocks[i]
                                    .level_prefix = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_level_4x4_transform_blocks[i]
                                    .level_suffix = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cb_level_4x4_transform_blocks[i]
                                    .run_before = Vec::new();

                                let mut zeros_left: i32 =
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_level_4x4_transform_blocks[i]
                                        .total_zeros as i32;
                                let mut run_val: usize = 0;
                                for _ in 0..16 {
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_level_4x4_transform_blocks[i]
                                        .trailing_ones_sign_flag
                                        .push(rconfig.trailing_ones_sign_flag.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_level_4x4_transform_blocks[i]
                                        .level_prefix
                                        .push(rconfig.level_prefix.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cb_level_4x4_transform_blocks[i]
                                        .level_suffix
                                        .push(rconfig.level_suffix.sample(film));
                                    if zeros_left > 0 {
                                        run_val = randomize_run_before(zeros_left, rconfig, film);
                                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                            .cb_level_4x4_transform_blocks[i]
                                            .run_before
                                            .push(run_val);
                                    } else {
                                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                            .cb_level_4x4_transform_blocks[i]
                                            .run_before
                                            .push(0);
                                    }
                                    zeros_left -= run_val as i32;
                                }
                            }
                        }
                    }
                }
            } else if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .coded_block_pattern_luma
                & (1 << i_8x8)
                > 0
            {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_level_8x8_transform_blocks[i_8x8]
                    .available = true;
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_level_8x8_transform_blocks[i_8x8]
                    .coded_block_flag = true;
                // reset existing values
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_level_8x8_transform_blocks[i_8x8]
                    .significant_coeff_flag = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_level_8x8_transform_blocks[i_8x8]
                    .last_significant_coeff_flag = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_level_8x8_transform_blocks[i_8x8]
                    .coeff_sign_flag = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cb_level_8x8_transform_blocks[i_8x8]
                    .coeff_abs_level_minus1 = Vec::new();

                for _ in 0..64 {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cb_level_8x8_transform_blocks[i_8x8]
                        .significant_coeff_flag
                        .push(rconfig.significant_coeff_flag.sample(film));
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cb_level_8x8_transform_blocks[i_8x8]
                        .last_significant_coeff_flag
                        .push(rconfig.last_significant_coeff_flag.sample(film));
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cb_level_8x8_transform_blocks[i_8x8]
                        .coeff_sign_flag
                        .push(rconfig.coeff_sign_flag.sample(film));
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cb_level_8x8_transform_blocks[i_8x8]
                        .coeff_abs_level_minus1
                        .push(rconfig.coeff_abs_level_minus1.sample(film));
                }
            }
        }

        // CrIntra16x16DCLevel is length 16
        if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].mb_part_pred_mode(0)
            == MbPartPredMode::Intra16x16
        {
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .cr_intra_16x16_dc_level_transform_blocks
                .available = true;

            if vp.entropy_coding_mode_flag {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_intra_16x16_dc_level_transform_blocks
                    .coded_block_flag = true;

                // reset existing values
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_intra_16x16_dc_level_transform_blocks
                    .significant_coeff_flag = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_intra_16x16_dc_level_transform_blocks
                    .last_significant_coeff_flag = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_intra_16x16_dc_level_transform_blocks
                    .coeff_sign_flag = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_intra_16x16_dc_level_transform_blocks
                    .coeff_abs_level_minus1 = Vec::new();

                for _ in 0..16 {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cr_intra_16x16_dc_level_transform_blocks
                        .significant_coeff_flag
                        .push(rconfig.significant_coeff_flag.sample(film));
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cr_intra_16x16_dc_level_transform_blocks
                        .last_significant_coeff_flag
                        .push(rconfig.last_significant_coeff_flag.sample(film));
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cr_intra_16x16_dc_level_transform_blocks
                        .coeff_sign_flag
                        .push(rconfig.coeff_sign_flag.sample(film));
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cr_intra_16x16_dc_level_transform_blocks
                        .coeff_abs_level_minus1
                        .push(rconfig.coeff_abs_level_minus1.sample(film));
                }
            } else {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_intra_16x16_dc_level_transform_blocks
                    .coeff_token =
                    randomize_coeff_token(16, ResidualMode::CrIntra16x16DCLevel, vp, rconfig, film);
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_intra_16x16_dc_level_transform_blocks
                    .total_zeros = randomize_total_zeros(
                    16,
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cr_intra_16x16_dc_level_transform_blocks
                        .coeff_token
                        .total_coeff,
                    rconfig,
                    film,
                );
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_intra_16x16_dc_level_transform_blocks
                    .trailing_ones_sign_flag = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_intra_16x16_dc_level_transform_blocks
                    .level_prefix = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_intra_16x16_dc_level_transform_blocks
                    .level_suffix = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_intra_16x16_dc_level_transform_blocks
                    .run_before = Vec::new();

                let mut zeros_left: i32 = ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_intra_16x16_dc_level_transform_blocks
                    .total_zeros as i32;
                let mut run_val: usize = 0;
                for _ in 0..16 {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cr_intra_16x16_dc_level_transform_blocks
                        .trailing_ones_sign_flag
                        .push(rconfig.trailing_ones_sign_flag.sample(film));
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cr_intra_16x16_dc_level_transform_blocks
                        .level_prefix
                        .push(rconfig.level_prefix.sample(film));
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cr_intra_16x16_dc_level_transform_blocks
                        .level_suffix
                        .push(rconfig.level_suffix.sample(film));
                    if zeros_left > 0 {
                        run_val = randomize_run_before(zeros_left, rconfig, film);
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .cr_intra_16x16_dc_level_transform_blocks
                            .run_before
                            .push(run_val);
                    } else {
                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .cr_intra_16x16_dc_level_transform_blocks
                            .run_before
                            .push(0);
                    }
                    zeros_left -= run_val as i32;
                }
            }
        }

        // CrIntra16x16ACLevel: there are 16 arrays of length 16
        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
            .cr_intra_16x16_ac_level_transform_blocks = Vec::new();
        // CrLevel4x4: there are 16 arrays of length 16
        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].cr_level_4x4_transform_blocks =
            Vec::new();
        // CrLevel8x8: there are 4 arrays of length 64
        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].cr_level_8x8_transform_blocks =
            Vec::new();

        for i_8x8 in 0..4 {
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .cr_level_8x8_transform_blocks
                .push(TransformBlock::new());

            if !ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].transform_size_8x8_flag
                || !vp.entropy_coding_mode_flag
            {
                for i_4x4 in 0..4 {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cr_intra_16x16_ac_level_transform_blocks
                        .push(TransformBlock::new());
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cr_level_4x4_transform_blocks
                        .push(TransformBlock::new());
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cr_intra_16x16_ac_level
                        .push(Vec::new());

                    if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .coded_block_pattern_luma
                        & (1 << i_8x8)
                        > 0
                    {
                        let i = i_8x8 * 4 + i_4x4;

                        if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                            .mb_part_pred_mode(0)
                            == MbPartPredMode::Intra16x16
                        {
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .cr_intra_16x16_ac_level_transform_blocks[i]
                                .available = true;

                            if vp.entropy_coding_mode_flag {
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_intra_16x16_ac_level_transform_blocks[i]
                                    .coded_block_flag = true;

                                // reset existing values
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_intra_16x16_ac_level_transform_blocks[i]
                                    .significant_coeff_flag = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_intra_16x16_ac_level_transform_blocks[i]
                                    .last_significant_coeff_flag = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_intra_16x16_ac_level_transform_blocks[i]
                                    .coeff_sign_flag = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_intra_16x16_ac_level_transform_blocks[i]
                                    .coeff_abs_level_minus1 = Vec::new();

                                for _ in 0..16 {
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_intra_16x16_ac_level_transform_blocks[i]
                                        .significant_coeff_flag
                                        .push(rconfig.significant_coeff_flag.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_intra_16x16_ac_level_transform_blocks[i]
                                        .last_significant_coeff_flag
                                        .push(rconfig.last_significant_coeff_flag.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_intra_16x16_ac_level_transform_blocks[i]
                                        .coeff_sign_flag
                                        .push(rconfig.coeff_sign_flag.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_intra_16x16_ac_level_transform_blocks[i]
                                        .coeff_abs_level_minus1
                                        .push(rconfig.coeff_abs_level_minus1.sample(film));
                                }
                            } else {
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_intra_16x16_ac_level_transform_blocks[i]
                                    .coeff_token = randomize_coeff_token(
                                    15,
                                    ResidualMode::CrIntra16x16ACLevel,
                                    vp,
                                    rconfig,
                                    film,
                                );
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_intra_16x16_ac_level_transform_blocks[i]
                                    .total_zeros = randomize_total_zeros(
                                    15,
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_intra_16x16_ac_level_transform_blocks[i]
                                        .coeff_token
                                        .total_coeff,
                                    rconfig,
                                    film,
                                );
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_intra_16x16_ac_level_transform_blocks[i]
                                    .trailing_ones_sign_flag = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_intra_16x16_ac_level_transform_blocks[i]
                                    .level_prefix = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_intra_16x16_ac_level_transform_blocks[i]
                                    .level_suffix = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_intra_16x16_ac_level_transform_blocks[i]
                                    .run_before = Vec::new();

                                let mut zeros_left: i32 =
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_intra_16x16_ac_level_transform_blocks[i]
                                        .total_zeros as i32;
                                let mut run_val: usize = 0;
                                for _ in 0..16 {
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_intra_16x16_ac_level_transform_blocks[i]
                                        .trailing_ones_sign_flag
                                        .push(rconfig.trailing_ones_sign_flag.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_intra_16x16_ac_level_transform_blocks[i]
                                        .level_prefix
                                        .push(rconfig.level_prefix.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_intra_16x16_ac_level_transform_blocks[i]
                                        .level_suffix
                                        .push(rconfig.level_suffix.sample(film));
                                    if zeros_left > 0 {
                                        run_val = randomize_run_before(zeros_left, rconfig, film);
                                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                            .cr_intra_16x16_ac_level_transform_blocks[i]
                                            .run_before
                                            .push(run_val);
                                    } else {
                                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                            .cr_intra_16x16_ac_level_transform_blocks[i]
                                            .run_before
                                            .push(0);
                                    }
                                    zeros_left -= run_val as i32;
                                }
                                // Fill in CrIntra16x16ACLevel
                                fill_in_ac_residue(2, 0, i, slice_idx, macroblock_idx, ds);
                            }
                        } else {
                            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                .cr_level_4x4_transform_blocks[i]
                                .available = true;
                            if vp.entropy_coding_mode_flag {
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_level_4x4_transform_blocks[i]
                                    .coded_block_flag = true;
                                // reset existing values
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_level_4x4_transform_blocks[i]
                                    .significant_coeff_flag = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_level_4x4_transform_blocks[i]
                                    .last_significant_coeff_flag = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_level_4x4_transform_blocks[i]
                                    .coeff_sign_flag = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_level_4x4_transform_blocks[i]
                                    .coeff_abs_level_minus1 = Vec::new();

                                for _ in 0..16 {
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_level_4x4_transform_blocks[i]
                                        .significant_coeff_flag
                                        .push(rconfig.significant_coeff_flag.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_level_4x4_transform_blocks[i]
                                        .last_significant_coeff_flag
                                        .push(rconfig.last_significant_coeff_flag.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_level_4x4_transform_blocks[i]
                                        .coeff_sign_flag
                                        .push(rconfig.coeff_sign_flag.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_level_4x4_transform_blocks[i]
                                        .coeff_abs_level_minus1
                                        .push(rconfig.coeff_abs_level_minus1.sample(film));
                                }
                            } else {
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_level_4x4_transform_blocks[i]
                                    .coeff_token = randomize_coeff_token(
                                    16,
                                    ResidualMode::CrLevel4x4,
                                    vp,
                                    rconfig,
                                    film,
                                );
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_level_4x4_transform_blocks[i]
                                    .total_zeros = randomize_total_zeros(
                                    16,
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_level_4x4_transform_blocks[i]
                                        .coeff_token
                                        .total_coeff,
                                    rconfig,
                                    film,
                                );
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_level_4x4_transform_blocks[i]
                                    .trailing_ones_sign_flag = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_level_4x4_transform_blocks[i]
                                    .level_prefix = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_level_4x4_transform_blocks[i]
                                    .level_suffix = Vec::new();
                                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                    .cr_level_4x4_transform_blocks[i]
                                    .run_before = Vec::new();

                                let mut zeros_left: i32 =
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_level_4x4_transform_blocks[i]
                                        .total_zeros as i32;
                                let mut run_val: usize = 0;
                                for _ in 0..16 {
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_level_4x4_transform_blocks[i]
                                        .trailing_ones_sign_flag
                                        .push(rconfig.trailing_ones_sign_flag.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_level_4x4_transform_blocks[i]
                                        .level_prefix
                                        .push(rconfig.level_prefix.sample(film));
                                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                        .cr_level_4x4_transform_blocks[i]
                                        .level_suffix
                                        .push(rconfig.level_suffix.sample(film));
                                    if zeros_left > 0 {
                                        run_val = randomize_run_before(zeros_left, rconfig, film);
                                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                            .cr_level_4x4_transform_blocks[i]
                                            .run_before
                                            .push(run_val);
                                    } else {
                                        ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                                            .cr_level_4x4_transform_blocks[i]
                                            .run_before
                                            .push(0);
                                    }
                                    zeros_left -= run_val as i32;
                                }
                            }
                        }
                    }
                }
            } else if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .coded_block_pattern_luma
                & (1 << i_8x8)
                > 0
            {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_level_8x8_transform_blocks[i_8x8]
                    .available = true;

                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_level_8x8_transform_blocks[i_8x8]
                    .coded_block_flag = true;
                // reset existing values
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_level_8x8_transform_blocks[i_8x8]
                    .significant_coeff_flag = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_level_8x8_transform_blocks[i_8x8]
                    .last_significant_coeff_flag = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_level_8x8_transform_blocks[i_8x8]
                    .coeff_sign_flag = Vec::new();
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .cr_level_8x8_transform_blocks[i_8x8]
                    .coeff_abs_level_minus1 = Vec::new();

                for _ in 0..64 {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cr_level_8x8_transform_blocks[i_8x8]
                        .significant_coeff_flag
                        .push(rconfig.significant_coeff_flag.sample(film));
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cr_level_8x8_transform_blocks[i_8x8]
                        .last_significant_coeff_flag
                        .push(rconfig.last_significant_coeff_flag.sample(film));
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cr_level_8x8_transform_blocks[i_8x8]
                        .coeff_sign_flag
                        .push(rconfig.coeff_sign_flag.sample(film));
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .cr_level_8x8_transform_blocks[i_8x8]
                        .coeff_abs_level_minus1
                        .push(rconfig.coeff_abs_level_minus1.sample(film));
                }
            }
        }
    }
}

/// Generate a random Macroblock prediction data
pub fn randomize_mb_pred(
    slice_idx: usize,
    vp: &VideoParameters,
    ignore_edge_intra_pred: bool,
    ignore_intra_pred: bool,
    macroblock_idx: usize,
    rconfig: &RandomMBRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    let mppm = ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].mb_part_pred_mode(0);

    if mppm == MbPartPredMode::Intra4x4
        || mppm == MbPartPredMode::Intra8x8
        || mppm == MbPartPredMode::Intra16x16
    {
        let mut ignore_intra_pred_flag = ignore_intra_pred;

        if ignore_edge_intra_pred {
            let x_d = (macroblock_idx as u32) % vp.pic_width_in_mbs;
            let y_d = (macroblock_idx as u32) / vp.pic_width_in_mbs;

            ignore_intra_pred_flag |= x_d == 0 || y_d == 0;
        }

        if mppm == MbPartPredMode::Intra4x4 {
            for luma_4x4_blk_idx in 0..16 {
                if ignore_intra_pred_flag {
                    // if currently ignore intra pred, then we just say we'll use the previous value
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .prev_intra4x4_pred_mode_flag[luma_4x4_blk_idx] = true;
                } else {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .prev_intra4x4_pred_mode_flag[luma_4x4_blk_idx] =
                        rconfig.prev_intra4x4_pred_mode_flag.sample(film);
                }

                if !ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .prev_intra4x4_pred_mode_flag[luma_4x4_blk_idx]
                {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].rem_intra4x4_pred_mode
                        [luma_4x4_blk_idx] = rconfig.rem_intra4x4_pred_mode.sample(film)
                }
            }
        }

        if mppm == MbPartPredMode::Intra8x8 {
            for luma_8x8_blk_idx in 0..4 {
                if ignore_intra_pred_flag {
                    // if currently ignore intra pred, then we just say we'll use the previous value
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .prev_intra8x8_pred_mode_flag[luma_8x8_blk_idx] = true;
                } else {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                        .prev_intra8x8_pred_mode_flag[luma_8x8_blk_idx] =
                        rconfig.prev_intra8x8_pred_mode_flag.sample(film);
                }
                if !ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .prev_intra8x8_pred_mode_flag[luma_8x8_blk_idx]
                {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].rem_intra8x8_pred_mode
                        [luma_8x8_blk_idx] = rconfig.rem_intra8x8_pred_mode.sample(film)
                }
            }
        }

        // we do slice_idx+2 to use the params associated with this slice; the first two are SPS and PPS video_params
        if vp.chroma_array_type == 1 || vp.chroma_array_type == 2 {
            if ignore_intra_pred_flag {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].intra_chroma_pred_mode = 0;
            // DC mode is used whenever ignoring intra prediction
            } else {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].intra_chroma_pred_mode =
                    rconfig.intra_chroma_pred_mode.sample(film) as u8;
            }
        }
    } else if mppm != MbPartPredMode::Direct {
        let mb_num_partions = ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].num_mb_part();
        for mb_part_idx in 0..mb_num_partions {
            if (ds.slices[slice_idx].sh.num_ref_idx_l0_active_minus1 > 0
                || ds.slices[slice_idx].sd.mb_field_decoding_flag[macroblock_idx]
                    != ds.slices[slice_idx].sh.field_pic_flag)
                && ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .mb_part_pred_mode(mb_part_idx)
                    != MbPartPredMode::PredL1
            {
                // these are unary encoded, so too large values take forever to generate
                // 384 is a magic number in the JM reference decoder that causes it to crash - any ref_idx greater than it
                // and it will not decode the rest of the file correctly

                // Bias towards 0 due to long unary encoding
                if rconfig.bias_zero_mb_ref_idx_l0.sample(film) {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].ref_idx_l0
                        [mb_part_idx] = 0;
                } else {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].ref_idx_l0
                        [mb_part_idx] = rconfig.ref_idx_l0.sample(film);
                }
            }

            if (ds.slices[slice_idx].sh.num_ref_idx_l1_active_minus1 > 0
                || ds.slices[slice_idx].sd.mb_field_decoding_flag[macroblock_idx]
                    != ds.slices[slice_idx].sh.field_pic_flag)
                && ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                    .mb_part_pred_mode(mb_part_idx)
                    != MbPartPredMode::PredL0
            {
                // these are unary encoded, so too large values take forever to generate
                // 384 is a magic number in the JM reference decoder that causes it to crash - any ref_idx greater than it
                // and it will not decode the rest of the file correctly

                // Bias towards 0 due to long unary encoding
                if rconfig.bias_zero_mb_ref_idx_l1.sample(film) {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].ref_idx_l1
                        [mb_part_idx] = 0;
                } else {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].ref_idx_l1[mb_part_idx] =
                        rconfig.ref_idx_l1.sample(film)
                }
            }

            if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].mb_part_pred_mode(mb_part_idx)
                != MbPartPredMode::PredL1
            {
                for comp_idx in 0..2 {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].mvd_l0[mb_part_idx][0]
                        [comp_idx] = rconfig.mvd_l0.sample(film)
                }
            }

            if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].mb_part_pred_mode(mb_part_idx)
                != MbPartPredMode::PredL0
            {
                for comp_idx in 0..2 {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].mvd_l1[mb_part_idx][0]
                        [comp_idx] = rconfig.mvd_l1.sample(film)
                }
            }
        }
    }
}

/// Generate a random sub macroblock prediction data
pub fn randomize_sub_mb_pred(
    slice_idx: usize,
    macroblock_idx: usize,
    rconfig: &RandomMBRange,
    ds: &mut H264DecodedStream,
    film: &mut FilmState,
) {
    for mb_part_idx in 0..4 {
        if is_slice_type(ds.slices[slice_idx].sh.slice_type, "B") {
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].sub_mb_type[mb_part_idx] =
                random_b_submbtype(rconfig, film);
        } else {
            ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].sub_mb_type[mb_part_idx] =
                random_p_submbtype(rconfig, film);
        }

        if (ds.slices[slice_idx].sh.num_ref_idx_l0_active_minus1 > 0
            || ds.slices[slice_idx].sd.mb_field_decoding_flag[macroblock_idx]
                != ds.slices[slice_idx].sh.field_pic_flag)
            && ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].mb_type != MbType::P8x8ref0
            && ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].sub_mb_type[mb_part_idx]
                != SubMbType::BDirect8x8
            && ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .sub_mb_part_pred_mode(mb_part_idx)
                != MbPartPredMode::PredL1
        {
            // these are unary encoded, so too large values take forever to generate
            // 384 is a magic number in the JM reference decoder that causes it to crash - any ref_idx greater than it
            // and it will not decode the rest of the file correctly

            // Bias towards 0 due to long unary encoding
            if rconfig.bias_zero_mb_ref_idx_l0.sample(film) {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].ref_idx_l0[mb_part_idx] = 0;
            } else {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].ref_idx_l0[mb_part_idx] =
                    rconfig.ref_idx_l0.sample(film);
            }
        }

        if (ds.slices[slice_idx].sh.num_ref_idx_l1_active_minus1 > 0
            || ds.slices[slice_idx].sd.mb_field_decoding_flag[macroblock_idx]
                != ds.slices[slice_idx].sh.field_pic_flag)
            && ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].sub_mb_type[mb_part_idx]
                != SubMbType::BDirect8x8
            && ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .sub_mb_part_pred_mode(mb_part_idx)
                != MbPartPredMode::PredL0
        {
            // these are unary encoded, so too large values take forever to generate
            // 384 is a magic number in the JM reference decoder that causes it to crash - any ref_idx greater than it
            // and it will not decode the rest of the file correctly

            // Bias towards 0 due to long unary encoding
            if rconfig.bias_zero_mb_ref_idx_l1.sample(film) {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].ref_idx_l1[mb_part_idx] = 0
            } else {
                ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].ref_idx_l1[mb_part_idx] =
                    rconfig.ref_idx_l1.sample(film);
            }
        }

        if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].sub_mb_type[mb_part_idx]
            != SubMbType::BDirect8x8
            && ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .sub_mb_part_pred_mode(mb_part_idx)
                != MbPartPredMode::PredL1
        {
            for sub_mb_part_idx in 0..ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .num_sub_mb_part(mb_part_idx)
            {
                for comp_idx in 0..2 {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].mvd_l0[mb_part_idx]
                        [sub_mb_part_idx][comp_idx] = rconfig.mvd_l0.sample(film)
                }
            }
        }

        if ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].sub_mb_type[mb_part_idx]
            != SubMbType::BDirect8x8
            && ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .sub_mb_part_pred_mode(mb_part_idx)
                != MbPartPredMode::PredL0
        {
            for sub_mb_part_idx in 0..ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx]
                .num_sub_mb_part(mb_part_idx)
            {
                for comp_idx in 0..2 {
                    ds.slices[slice_idx].sd.macroblock_vec[macroblock_idx].mvd_l1[mb_part_idx]
                        [sub_mb_part_idx][comp_idx] = rconfig.mvd_l1.sample(film)
                }
            }
        }
    }
}
