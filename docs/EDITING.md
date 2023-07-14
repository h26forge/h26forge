# Editing H.264 Syntax Elements

H26Forge can be used to edit video syntax elements with Python scripts called **transforms**. This requires having Python installed. H26Forge will first decode a video, apply the video transform, then re-encode the syntax elements. The [transforms/](../transforms/) directory contains some example video transforms.

It is helpful to work alongside the [H.264 spec](https://www.itu.int/rec/T-REC-H.264-202108-I/en) when editing videos.

## How it Works

Usage: `./h26forge modify -i input.264 -o output.264 -t transforms/slice_all_pcm.py --arg 0`.

This will first decode input.264 to create an [H264DecodedStream](../src/common/data_structures.rs#L13) object. This object will then get saved to a local file called `temp.json` that the Python script will work on. Producing this file is a `serde_json::to_string` operation.

Then the transform file will be read and inserted into a [Python wrapper](../src/vidgen/modify_video.rs#L77) to interact with the helper libraries and modify the `temp.json` file. H26Forge will create a `temp.py` file at the same location as `temp.json`, then run the command `python temp.py temp.json <arg>`, where `<arg>` is passed into H26Forge via the `--arg` argument. Python will then proceed to apply the transform to the recovered syntax elements.

Once the Python command is done, H26Forge will parse `temp.json` via a `serde_json::from_reader` operation to a new `H264DecodedStream` object, and overwrite the previously decoded ones. If the transform failed to maintain the structure of the `H264DecodedStream`, then H26Forge may fail to create the object and panic. At this point, H26Forge will then encode the modified syntax elements to produce a new bitstream.

## Format

A video transform is a Python function called `modify_video` that takes in a JSON representation of the decoded syntax elements and operates on the syntax elements. The syntax elements are a [H264DecodedStream](../src/common/data_structures.rs#L13) object, so we recommend looking at that code to determine how to access syntax elements.

The basic transform looks like this:
```python
def modify_video(ds):
  # Insert modification operations here
  return ds
```

For developer-friendliness, all the transforms included with H26Forge call a separate function with a descriptive name:
```python
def cve_2022_22675(ds):
  # Insert modification operations here
  return ds

def modify_video(ds):
  return cve_2022_22675(ds)
```

In order to receive arguments from H26Forge, we parse `sys.argv`. Note that `sys` is imported in the [Python wrapper](../src/vidgen/modify_video.rs#L77).
```python
def remove_nth_frame_residue(n, ds):
  # Insert modification operations here
  return ds

def modify_video(ds):
  n = int(sys.argv[2])
  return remove_nth_frame_residue(n, ds)
```

## Helper Functions

The helper functions are available in [transforms/helpers.py](../transforms/helpers.py).

These are the currently available helper functions:
- Update Dependent Syntax Elements
  - `set_cbp_chroma_and_luma(slice_idx, mb_idx, ds)`: This function will set the coded block pattern chroma and luma values depending on the Macroblock Type
- Get Video Properties
  - `is_slice_type(slice_num, slice_letter)`: Returns whether the the `slice_type` syntax element corresponds to a slice letter (I, P, B, SP, SI)
  - `get_chroma_width_and_height(sps_idx, ds)`: Calculates the chroma width and height depending on the `chroma_format_idc` value in an SPS
- Return new Syntax Elements
  - `new_transform_block()`: Returns a [TransformBlock](../src/common/data_structures.rs#L680) for luma/chroma residue.
  - `new_sps()`: Returns a new [SeqParameterSet](../src/common/data_structures.rs#L4840)
  - `new_vui_parameter()`: Returns a new [VUIParameters](../src/common/data_structures.rs#L3775)
  - `new_hrd_parameter()`: Returns a new [HRDParameters](../src/common/data_structures.rs#L3693)
  - `new_subset_sps()`: Returns a new [SubsetSPS](../src/common/data_structures.rs#L5295)
- Insert New Syntax Elements into the Stream
  - `add_sei_nalu(ds)`: Inserts an [SEI NALU](../src/common/data_structures.rs#L5855) into the decoded stream ds. This includes adding a new NALU header.
  - `add_sei_nalu_at_position(ds, idx)`: Inserts an [SEI NALU](../src/common/data_structures.rs#L5855) at position `idx` into the decoded stream ds.
  - `clone_and_append_existing_slice(ds, nalu_idx, slice_idx)`: Copy the Slice number `slice_idx` at NALU position `nalu_idx` and append it to the decoded stream ds.

## Example: CVE-2022-22675 PoC

To show some of the features of the video editing API, we'll walk through the [CVE-2022-22675 PoC transform](../transforms/paper_pocs/cve_2022_22675.py).

Usage: `./h26forge --mp4 --mp4-frag modify -i input_vids/SPS_PPS_SPS_I_P.264 -o poc.264 -t transforms/paper_pocs/cve_2022_22675.py`

Playing `poc.264.mp4` on iOS 15.4 or older, or macOS 12.3 or older, may lead to a device panic.

Here are some highlights from that transform:
- Importing functions from the helper library
```python
from helpers import new_vui_parameter, new_hrd_parameter, clone_and_append_existing_slice
```
- Using existing syntax elements from the stream as a part of our payload.
```python
pps_tgt_payload0 |= int(ds["ppses"][0]["weighted_pred_flag"]) << 24 # value is a byte
```
- Adding new slices to our decoded stream for each message we are writing.
```python
while len(ds["slices"]) <= len(message_snippets):
  ds = clone_and_append_existing_slice(ds, nalu_idx, slice_idx)
```

## Limitations

The biggest limitation is performance. H26Forge will create a pretty large `temp.json` file that spends a lot of time being parsed, both in Python and Rust. For this reason, it's best to limiting the editing the syntax element of relatively small files.

Another limitation is that `temp.py` has to be output in a directory that has the `transforms/` folder in it if it relies on any other transform or helper functions. 