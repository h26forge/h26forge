##
# Out of bounds first_mb_in_slice
def oob_first_mb_in_slice(ds):
    # Slice 0 will have an OOB first_mb_in_slice
    ds["slices"][0]["sh"]["first_mb_in_slice"] = 1024

    return ds

def modify_video(ds):
    return oob_first_mb_in_slice(ds)