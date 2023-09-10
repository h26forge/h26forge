cur_date=$(date +%s)
mkdir -p "tmp/"
output_dir="tmp/rand_100_vids_$cur_date"
tool_args=""
generation_args="--small --ignore-edge-intra-pred --ignore-ipcm" # --config config/default.json"
RUST_BACKTRACE=1

mkdir -p $output_dir
for i in $(seq -f "%04g" 0 9999); do
    cmd="./h26forge $tool_args generate $generation_args -o $output_dir/video.$cur_date.$i.264"
    echo $cmd
    $cmd >> $output_dir/rand_100.log 2>&1
done

echo "Log saved to $output_dir/rand_100.log"