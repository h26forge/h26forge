#define MINIMP4_IMPLEMENTATION
#ifdef _WIN32
#include <sys/types.h>
#include <stddef.h>
#endif
#include "minimp4.h"

static size_t get_nal_size(const uint8_t *buf, size_t size)
{
    size_t pos = 3;
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

void write_mp4(mp4_h26x_writer_t *mp4wr, int fps, const uint8_t *data, size_t data_size)
{
    while (data_size > 0)
    {
        size_t nal_size = get_nal_size(data, data_size);
        if (nal_size < 4)
        {
            data += 1;
            data_size -= 1;
            continue;
        }
        mp4_h26x_write_nal(mp4wr, data, nal_size, 90000 / fps);
        data += nal_size;
        data_size -= nal_size;
    }
}
