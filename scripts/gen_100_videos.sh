cur_date=$(date +%s)
mkdir -p "tmp/"
output_dir="tmp/rand_100_vids_$cur_date"
tool_args="--mp4 --mp4-rand-size --safestart --rtp-replay"
generation_args="--small --ignore-edge-intra-pred --ignore-ipcm --config config/default.json"
RUST_BACKTRACE=1

echo "Saving log to $output_dir/rand_100.log"

mkdir -p $output_dir
for i in $(seq -f "%04g" 0 99); do
    cmd="./h26forge $tool_args generate $generation_args -o $output_dir/video.$cur_date.$i.264"
    echo $cmd
    echo ------------------ >> $output_dir/rand_100.log
    echo $cmd >> $output_dir/rand_100.log
    $cmd >> $output_dir/rand_100.log 2>&1
    echo ------------------ >> $output_dir/rand_100.log
done

echo "Log saved to $output_dir/rand_100.log"