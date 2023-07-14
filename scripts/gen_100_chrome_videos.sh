cur_date=$(date +%s)
mkdir -p tmp/
output_dir="tmp/chrome_100_mp4_$cur_date"
tool_args="--mp4 --safestart --mp4-rand-size"
generation_args="--ignore-edge-intra-pred --ignore-ipcm --config config/chrome.json --small"
RUST_BACKTRACE=1

mkdir -p $output_dir
for i in $(seq -f "%04g" 0 99); do
    cmd="./h26forge $tool_args generate $generation_args -o $output_dir/video_$i.264"
    echo $cmd
    $cmd >> $output_dir/rand_100.log 2>&1
done

cp scripts/play_videos.html $output_dir/play_videos.html

echo "Log saved to $output_dir/rand_100.log"
echo "Open $output_dir/play_videos.html in the browser to play the videos"