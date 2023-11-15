##
# all_frames_pcm
#
# Sets the first slice to be all of type intra PCM with the YUV
# values 109, 65, 190 which are the UT colors in the ITU-R 601 color space
# used by https://www.mikekohn.net/file_formats/yuv_rgb_converter.php
#
# Subsequent values are shifted my multiples of 37
#
##
def slice_all_pcm(ds):
    print("\t Setting all frames to all IPCM with increasing color")

    # UT colors converted from RGB 197, 87, 0 to YUV
    # using https://www.mikekohn.net/file_formats/yuv_rgb_converter.php
    # are 109, 65, 190

    y = 109
    u = 65
    v = 190

    from helpers import get_chroma_width_and_height

    (mb_width_c, mb_height_c) = get_chroma_width_and_height(0, ds)
    c_dims = mb_width_c * mb_height_c
    for i in range(len(ds["slices"])):
        for j in range(len(ds["slices"][i]["sd"]["macroblock_vec"])):
            ds["slices"][i]["sd"]["macroblock_vec"][j]["mb_skip_flag"] = False

            ds["slices"][i]["sd"]["macroblock_vec"][j]["mb_type"] = "IPCM"
            ds["slices"][i]["sd"]["macroblock_vec"][j]["pcm_sample_luma"] = []

            for _ in range(256):
                ds["slices"][i]["sd"]["macroblock_vec"][j]["pcm_sample_luma"].append((y + 37 * i) % 256)
            ds["slices"][i]["sd"]["macroblock_vec"][j]["pcm_sample_chroma"] = []
            for _ in range(c_dims):
                ds["slices"][i]["sd"]["macroblock_vec"][j]["pcm_sample_chroma"].append((u + 37 * i) % 256)
            for _ in range(c_dims):
                ds["slices"][i]["sd"]["macroblock_vec"][j]["pcm_sample_chroma"].append((v + 37 * i) % 256)

    return ds

def modify_video(ds):
    return slice_all_pcm(ds)