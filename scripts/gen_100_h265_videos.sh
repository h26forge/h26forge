cur_date=$(date +%s)
mkdir -p "tmp/"
output_dir="tmp/rand_100_vids_$cur_date"
tool_args="--mp4 --mp4-rand-size"
generation_args="--randcode"
RUST_BACKTRACE=1

echo "Saving log to $output_dir/rand_100.log"

mkdir -p $output_dir
for i in $(seq -f "%04g" 0 99); do
    cmd="target/debug/h26forge $tool_args experimental $generation_args -o $output_dir/video.$cur_date.$i.265"
    echo $cmd
    echo ------------------ >> $output_dir/rand_100.log
    echo $cmd >> $output_dir/rand_100.log
    $cmd >> $output_dir/rand_100.log 2>&1
    echo ------------------ >> $output_dir/rand_100.log
done

echo "Log saved to $output_dir/rand_100.log"