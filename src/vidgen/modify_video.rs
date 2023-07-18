//! Applies video transform to recovered syntax elements.

use crate::common::data_structures::H264DecodedStream;
use crate::common::data_structures::PicParameterSet;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::Command;
use std::str;

fn update_slice_dependent_vars(ds: &mut H264DecodedStream) {
    println!("\t Updating slice dependent variables");
    for i in 0..ds.slices.len() {
        // get Slice PPS
        let mut cur_pps_wrapper: Option<&PicParameterSet> = None;
        // retrieve the corresponding PPS
        for j in (0..ds.ppses.len()).rev() {
            if ds.ppses[j].pic_parameter_set_id == ds.slices[i].sh.pic_parameter_set_id {
                cur_pps_wrapper = Some(&ds.ppses[j]);
                break;
            }
        }

        let p: &PicParameterSet;
        match cur_pps_wrapper {
            Some(x) => p = x,
            _ => panic!(
                "decode_slice_header - PPS with id {} not found",
                ds.slices[i].sh.pic_parameter_set_id
            ),
        }

        // equation 7-30
        ds.slices[i].sh.slice_qp_y = 26 + ds.slices[i].sh.slice_qp_delta + p.pic_init_qp_minus26;
        ds.slices[i].sh.qp_y_prev = ds.slices[i].sh.slice_qp_y;

        // equation 7-31
        ds.slices[i].sh.qs_y = (26 + ds.slices[i].sh.slice_qs_delta + p.pic_init_qs_minus26) as u8;

        // equation 7-32
        ds.slices[i].sh.filter_offset_a = ds.slices[i].sh.slice_alpha_c0_offset_div2 << 1;

        // equation 7-33
        ds.slices[i].sh.filter_offset_b = ds.slices[i].sh.slice_beta_offset_div2 << 1;
    }
}

fn check_python_pathname() -> String {
    // macOS seems to prefer python3 to python so we check which to use
    let command1 = "python3";
    let command2 = "python";

    // First try command1
    match Command::new(command1).args(&["--version"]).output() {
        Ok(output) => {
            let output_stdout = str::from_utf8(&output.stdout).unwrap();

            if output_stdout.contains("Python 3") {
                return String::from(command1);
            }
        }
        Err(_) => {}
    };

    // Hopefully there's command2
    match Command::new(command2).args(&["--version"]).output() {
        Ok(output) => {
            let output_stdout = str::from_utf8(&output.stdout).unwrap();

            if output_stdout.contains("Python 3") {
                return String::from(command2);
            }
        }
        Err(_) => {}
    };

    panic!("Python3 not found!")
}

/// Apply a video transform to recovered syntax elements
///
/// Takes in a python file, any arguments it may take, and the decoded stream.
/// Runs the python file on the decoded stream to transform the video. It does
/// this by saving the stream as a json file, having the python script work
/// on the json and re-saving it, and opening it back up as a H264DecodedStream
/// object.
///
/// NOTE: Python may have type errors if it does not handle the stream correctly
pub fn perform_video_modification(
    modification_filename: &str,
    mod_file_arg: i32,
    ds: &mut H264DecodedStream,
) -> bool {
    // create our temporary file inside the current working directory
    let mut python_file = match File::create("temp.py") {
        Err(_) => panic!("couldn't create temp.py"),
        Ok(file) => file,
    };

    // get the contents of the python file
    let modification_code = fs::read_to_string(modification_filename).unwrap_or_else(|_| {
        panic!(
            "Couldn't read modification_code file {}",
            modification_filename
        )
    });

    let start_of_file = "
import json
import sys

# includes all the modifications along with helper scripts
sys.path.insert(1, 'transforms')

# load the json file
def load_file(fn):
    with open(fn) as json_file:
        return json.load(json_file)

# save the json file
def save_file(fn, d):
    with open(fn, 'w') as json_file:
        json.dump(d, json_file)

";
    let end_of_file = "
# pass in the json filename as the first param
fn = sys.argv[1]
d = load_file(fn)
save_file(fn, modify_video(d))
# this used to signal the script finished successfully
print('all good')
";

    let python_contents = format!("{}\n{}\n{}", start_of_file, modification_code, end_of_file);

    match python_file.write_all(python_contents.as_bytes()) {
        Err(_) => panic!("couldn't write to file temp.py"),
        Ok(()) => (),
    };

    // json_file will store our H264DecodedStream elements
    let mut json_file = match File::create("temp.json") {
        Err(_) => panic!("couldn't create temp.json"),
        Ok(file) => file,
    };

    println!("\t Creating JSON representation of video");

    let serialized = serde_json::to_string(&ds).unwrap();

    println!("\t Saving JSON to file");

    match json_file.write_all(serialized.as_bytes()) {
        Err(_) => panic!("couldn't write to file temp.json"),
        Ok(()) => (),
    };

    // close the json file for now - will open it back up later
    drop(json_file);

    println!("\t Running transformation");

    // run the python_contents locally
    let output = Command::new(check_python_pathname())
        .args(&["temp.py", "temp.json", &mod_file_arg.to_string()])
        .output()
        .expect("Failed to execute modification script");

    let output_stdout = str::from_utf8(&output.stdout).unwrap();

    // check to see if our all good message is included
    // NOTE: could be an issue if the modification script outputs 'all good'
    if output_stdout.contains("all good") {
        println!("{}", output_stdout);

        // recover the JSON and fill it into an H264DecodedStream object
        println!("\t Opening modified JSON file");
        let json_file = match File::open("temp.json") {
            Err(_) => panic!("couldn't open temp.json"),
            Ok(file) => file,
        };

        let reader = BufReader::new(json_file);

        println!("\t Parsing modified JSON file");

        let res: H264DecodedStream = match serde_json::from_reader(reader) {
            Ok(x) => x, // copy over the new result
            Err(y) => panic!("Error reading modified H264DecodedStream: {:?}", y),
        };

        println!("\t Parsing completed");

        // Overwrite our current decoded stream with the read in contents
        *ds = res.clone();

        // update slice dependent variables such as slice_qp_y
        update_slice_dependent_vars(ds);
        // TODO: communicate which NALUs have changed, and only re-encode those
    } else {
        println!("[ERROR] Failed to apply modification script");
        println!(
            "Got script stdout: {}",
            str::from_utf8(&output.stdout).unwrap()
        );
        println!(
            "Got script stderr: {}",
            str::from_utf8(&output.stderr).unwrap()
        );
        return false;
    }

    true
}
