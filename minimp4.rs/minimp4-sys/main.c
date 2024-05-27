#define MINIMP4_IMPLEMENTATION
#include "minimp4.h"

#define VIDEO_FPS 30

static uint8_t *preload(const char *path, ssize_t *data_size)
{
    FILE *file = fopen(path, "rb");
    uint8_t *data;
    *data_size = 0;
    if (!file)
        return 0;
    if (fseek(file, 0, SEEK_END))
        exit(1);
    *data_size = (ssize_t)ftell(file);
    if (*data_size < 0)
        exit(1);
    if (fseek(file, 0, SEEK_SET))
        exit(1);
    data = (unsigned char*)malloc(*data_size);
    if (!data)
        exit(1);
    if ((ssize_t)fread(data, 1, *data_size, file) != *data_size)
        exit(1);
    fclose(file);
    return data;
}

static ssize_t get_nal_size(uint8_t *buf, ssize_t size)
{
    ssize_t pos = 3;
    while ((size - pos) > 3)
    {
        if (buf[pos] == 0 && buf[pos + 1] == 0 && buf[pos + 2] == 1)
            return pos;
        if (buf[pos] == 0 && buf[pos + 1] == 0 && buf[pos + 2] == 0 && buf[pos + 3] == 1)
            return pos;
        pos++;
    }
    return size;
}

static int write_callback(int64_t offset, const void *buffer, size_t size, void *token)
{
    FILE *f = (FILE*)token;
    fseek(f, offset, SEEK_SET);
    return fwrite(buffer, 1, size, f) != size;
}

typedef struct
{
    uint8_t *buffer;
    ssize_t size;
} INPUT_BUFFER;


int main(int argc, char **argv)
{
    // check switches
    int sequential_mode = 0;
    int fragmentation_mode = 0;
    int i;
    for(i = 1; i < argc; i++)
    {
        if (argv[i][0] != '-')
            break;
        switch (argv[i][1])
        {
        case 's': sequential_mode = 1; break;
        case 'f': fragmentation_mode = 1; break;
        default:
            printf("error: unrecognized option\n");
            return 1;
        }
    }
    if (argc <= (i + 1))
    {
        printf("Usage: minimp4 [command] [options] input output\n"
               "Options:\n"
               "    -s    - enable mux sequential mode (no seek required for writing)\n"
               "    -f    - enable mux fragmentation mode (aka fMP4)\n");
        return 0;
    }
    ssize_t h264_size;
    uint8_t *alloc_buf;
    uint8_t *buf_h264 = alloc_buf = preload(argv[i], &h264_size);
    if (!buf_h264)
    {
        printf("error: can't open h264 file\n");
        exit(1);
    }

    FILE *fout = fopen(argv[i + 1], "wb");
    if (!fout)
    {
        printf("error: can't open output file\n");
        exit(1);
    }


    int is_hevc = (0 != strstr(argv[i], "265")) || (0 != strstr(argv[i], "hevc"));

    MP4E_mux_t *mux;
    mp4_h26x_writer_t mp4wr;
    mux = MP4E_open(sequential_mode, fragmentation_mode, fout, write_callback);
    if (MP4E_STATUS_OK != mp4_h26x_write_init(&mp4wr, mux, 352, 288, is_hevc))
    {
        printf("error: mp4_h26x_write_init failed\n");
        exit(1);
    }

    while (h264_size > 0)
    {
        ssize_t nal_size = get_nal_size(buf_h264, h264_size);
        if (nal_size < 4)
        {
            buf_h264  += 1;
            h264_size -= 1;
            continue;
        }
        /*int startcode_size = 4;
        if (buf_h264[0] == 0 && buf_h264[1] == 0 && buf_h264[2] == 1)
            startcode_size = 3;
        int nal_type = buf_h264[startcode_size] & 31;
        int is_intra = (nal_type == 5);
        printf("nal size=%ld, nal_type=%d\n", nal_size, nal_type);*/

        if (MP4E_STATUS_OK != mp4_h26x_write_nal(&mp4wr, buf_h264, nal_size, 90000/VIDEO_FPS))
        {
            printf("error: mp4_h26x_write_nal failed\n");
            exit(1);
        }
        buf_h264  += nal_size;
        h264_size -= nal_size;
    }
    if (alloc_buf)
        free(alloc_buf);
    MP4E_close(mux);
    mp4_h26x_write_close(&mp4wr);
    if (fout)
        fclose(fout);
}
