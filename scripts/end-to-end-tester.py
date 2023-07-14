import argparse
import time
import os
import subprocess
import hashlib
import logging
import shutil

from multiprocessing import JoinableQueue, Queue, Process, Pool

# Binary paths
h26forge_path = "../target/release/h26forge"
if os.name == 'nt':
    h26forge_path = "..\\target\\release\\h26forge.exe"
# Surprisingly, outputting JSON doesn't require much more time
h26forge_extra_flags = ["--json", "--json-no-nalu", "--strict-fmo"]

# Arguments
parser = argparse.ArgumentParser(description="Perform an end-to-end test with videos on a directory")
parser.add_argument('--testdir', help="Directory to test", required=True)
parser.add_argument('--logfilename', help="Filename to use for the log output", default="end-to-end.log")

logging.basicConfig(format='%(asctime)s %(message)s', filename="end-to-end.log", encoding='utf-8', level=logging.DEBUG)

def cleanup_old_outputs(inputvid, testdir):
    try:
        os.remove(f"{inputvid}.2_264")
    except FileNotFoundError:
        pass

    try:
        os.remove(f"{inputvid}.3_264")
    except FileNotFoundError:
        pass
    try:
        os.remove(f"{inputvid}.json")
    except FileNotFoundError:
        pass
   
    try:
        os.remove(f"{inputvid}.2_264.json")
    except FileNotFoundError:
        pass
    try:
        os.remove(f"{inputvid}.3_264.json")
    except FileNotFoundError:
        pass
    try:
        # Used by reference decoder
        testvid = os.path.join(testdir, 'test.264')
        os.remove(f"{testvid}")
    except FileNotFoundError:
        pass

def get_hash(vid):
    sha256_hash = "0"
    try:
        with open(vid, 'rb') as f:
            content = f.read()
            sha256_hash = hashlib.sha256(content).hexdigest()
    except FileNotFoundError:
        logging.error("[ERROR] Error HASHING file - not found: " + vid)
    return sha256_hash

def run_refdecoder(vid):
    # TODO: Run the reference decoder
    pass

def run_h26forge(inputvid, outputvid, use_extra, logfile):
    cmd_array = [h26forge_path]
    if use_extra:
        cmd_array.extend(h26forge_extra_flags)
    cmd_array.extend(['passthrough', '-i', inputvid, '-o', outputvid])
    logging.info(" ".join(cmd_array))

    h26forge_output = subprocess.run(cmd_array, stdout=subprocess.PIPE, stderr=subprocess.PIPE, universal_newlines=True)
    # Save the output
    with open(logfile, 'a') as f:
        f.write(h26forge_output.stdout)
        f.write(h26forge_output.stderr)
    return h26forge_output.returncode == 0

def generate_output_dirs(dir):
    output_dir = os.path.join(dir, "unable_to_decode_input")
    if not os.path.exists(output_dir):
      os.makedirs(output_dir)
   
    output_dir = os.path.join(dir, "different_passthrough")
    if not os.path.exists(output_dir):
      os.makedirs(output_dir)

    output_dir = os.path.join(dir, "unable_to_decode_passthrough")
    if not os.path.exists(output_dir):
      os.makedirs(output_dir)

    output_dir = os.path.join(dir, "incorrect_reencoding")
    if not os.path.exists(output_dir):
      os.makedirs(output_dir)

def save_unable_to_decode_input(file, dir):
    output_dir = os.path.join(dir, "unable_to_decode_input")
    shutil.copy(file,output_dir)

def save_different_passthrough(file, dir):
    output_dir = os.path.join(dir, "different_passthrough")
    shutil.copy(file,output_dir)

def save_unable_to_decode_passthrough(file, dir):
    output_dir = os.path.join(dir, "unable_to_decode_passthrough")
    shutil.copy(file,output_dir)

def save_incorrect_reencoding(file, dir):
    output_dir = os.path.join(dir, "incorrect_reencoding")
    shutil.copy(file,output_dir)

def run_jsondiff(j1, j2):
    cmd_array = ["json_diff", j1, j2]
    logging.info(" ".join(cmd_array))
    jsondiff_output = subprocess.run(cmd_array, stdout=subprocess.PIPE, stderr=subprocess.PIPE, universal_newlines=True)
   
    with open(j1 + ".diff", 'w') as f:
        f.write(jsondiff_output.stdout)
        f.write(jsondiff_output.stderr)
       
    # arbitrarily set jsondiff output size to 5 -- may need to tune this later
    return len(jsondiff_output.stdout) < 5 and len(jsondiff_output.stderr) == 0

def run_test(i, inputvid, testdir, total_videos):
    # Success
    match_bitforbit = 0
    match_syntax = 0
    # H26Forge Fail
    h26forge_fail_to_decode_input = 0
    h26forge_wrong = 0
    h26forge_fail_to_decode_passthrough = 0
    h26forge_passthrough_two_diff_hash = 0
    # Reference
    reference_success_on_input_after_hforge_fails = 0
    reference_fails_on_input_after_hforge_fails = 0
    reference_fails_on_input_for_yuv_comparison = 0
    reference_fails_on_passthrough = 0
    cleanup_old_outputs(inputvid, testdir)
    logging.info(f"-- [{i}/{total_videos}] Testing {inputvid}")
    print(f"-- [{i}/{total_videos}] Testing {inputvid}")

    outputvid = inputvid + ".2_264"
    logfile = inputvid + ".h26forge_out.txt"
    h1 = get_hash(inputvid)
   
    logging.info(f"-- [{i}/{total_videos}] Passthrough 1")
    start_time = time.time()
    success = run_h26forge(inputvid, outputvid, True, logfile)
    duration = time.time() - start_time
    logging.info(f"-- [{i}/{total_videos}] Passthrough 1 time: {duration}")
    if success:
        h2 = get_hash(outputvid)
        if h1 == h2:
            logging.info(f"-- [{i}/{total_videos}] Bit for bit identical :)")
            match_bitforbit += 1
        else:
            logging.info(f"-- [{i}/{total_videos}] hash differs from original, run again with outputvid")
            outputvid2 = inputvid + ".3_264"
            logging.info(f"-- [{i}/{total_videos}] Passthrough 2")
            start_time = time.time()
            success = run_h26forge(outputvid, outputvid2, True, logfile)
            duration = time.time() - start_time
            logging.info(f"-- [{i}/{total_videos}] Passthrough 2 time: {duration}")

            if success:
                h3 = get_hash(outputvid2)
                if h2 == h3:
                    logging.info(f"-- [{i}/{total_videos}] Passthrough 1 and 2 hashes match, comparing Syntax elements by generating JSON")
                    output1json = outputvid + ".json"
                    output2json = outputvid2 + ".json"

                    # Hash the JSON outputs to compare
                    jh1 = get_hash(output1json)
                    jh2 = get_hash(output2json)

                    if jh1 == jh2:
                        logging.info(f"-- [{i}/{total_videos}] Syntax elements match between input and output :)")
                        match_syntax += 1
                    else:
                        logging.info(f"-- [{i}/{total_videos}] JSON files differ - doing a diff")
                        # If hashes don't match, then do a JSON DIFF
                        logging.info(f"-- [{i}/{total_videos}] JSON Diff")
                        start_time = time.time()
                        iszero = run_jsondiff(output1json, output2json)
                        duration = time.time() - start_time
                        logging.info(f"-- [{i}/{total_videos}] JSON Diff time: {duration}")

                        if iszero:
                            logging.info(f"-- [{i}/{total_videos}] Syntax elements match between input and output :)")
                            match_syntax += 1
                        else:
                            logging.info(f"-- [{i}/{total_videos}] Syntax elements DO NOT MATCH :(")
                            save_incorrect_reencoding(inputvid,   testdir)
                            save_incorrect_reencoding(outputvid,  testdir)
                            save_incorrect_reencoding(outputvid2, testdir)
                            save_incorrect_reencoding(logfile,    testdir)
                            h26forge_wrong += 1

                else:
                    logging.info(f"-- [{i}/{total_videos}] Passthrough 1 and 2 hashes DO NOT MATCH -- need to investigate :(")
                    save_different_passthrough(inputvid,   testdir)
                    save_different_passthrough(outputvid,  testdir)
                    save_different_passthrough(outputvid2, testdir)
                    save_different_passthrough(logfile,    testdir)
                    h26forge_passthrough_two_diff_hash += 1
            else:
                logging.info(f"-- [{i}/{total_videos}] H26Forge unable to decode generated passthrough :(")
                save_unable_to_decode_passthrough(inputvid,  testdir)
                save_unable_to_decode_passthrough(outputvid, testdir)
                save_unable_to_decode_passthrough(logfile,   testdir)
                h26forge_fail_to_decode_passthrough += 1

    else:
        logging.info(f"-- [{i}/{total_videos}] H26Forge unable to decode input. Checking if the reference decoder can decode :(")
        h26forge_fail_to_decode_input += 1

        save_unable_to_decode_input(inputvid, testdir)
        save_unable_to_decode_input(logfile,  testdir)

        start_time = time.time()
        success = run_refdecoder(inputvid)
        duration = time.time() - start_time
        logging.info(f"-- [{i}/{total_videos}] Ref decoder time: {duration}")

        if success:
            logging.info(f"-- [{i}/{total_videos}] Reference success when h26forge failed")
            reference_success_on_input_after_hforge_fails += 1
        else:
            logging.info(f"-- [{i}/{total_videos}] Reference ALSO failed")
            reference_fails_on_input_after_hforge_fails += 1
   
    cleanup_old_outputs(inputvid, testdir)

    return (match_bitforbit, match_syntax, h26forge_wrong, h26forge_fail_to_decode_input, h26forge_fail_to_decode_passthrough, h26forge_passthrough_two_diff_hash, reference_success_on_input_after_hforge_fails, reference_fails_on_input_after_hforge_fails, reference_fails_on_input_for_yuv_comparison, reference_fails_on_passthrough)

def print_results(results, total_videos):   
    match_bitforbit = sum([x[0] for x in results])
    match_syntax = sum([x[1] for x in results])
    match_success = match_bitforbit + match_syntax
    h26forge_wrong = sum([x[2] for x in results])
    h26forge_fail_to_decode_input = sum([x[3] for x in results])
    h26forge_fail_to_decode_passthrough = sum([x[4] for x in results])
    h26forge_passthrough_two_diff_hash = sum([x[5] for x in results])
    reference_success_on_input_after_hforge_fails = sum([x[6] for x in results])
    reference_fails_on_input_after_hforge_fails = sum([x[7] for x in results])
    reference_fails_on_input_for_yuv_comparison = sum([x[8] for x in results])
    reference_fails_on_passthrough = sum([x[9] for x in results])

    results = f"""
    ----------------------[  STATS  ]----------------------
    Total Videos:                                     {total_videos}
    Success:                                          {match_success}
    ----[   h26forge   ]----
    Match - Identical:                                {match_bitforbit}
    Match - Diff hash, same syntax:                   {match_syntax}
    Fail  - Diff syntax elem:                         {h26forge_wrong}
    Fail  - Cannot decode input:                      {h26forge_fail_to_decode_input}
    Fail  - Cannot decode Passthrough:                {h26forge_fail_to_decode_passthrough}
    Fail  - 1st != 2nd Passthrough:                   {h26forge_passthrough_two_diff_hash}
    ----[  Reference  ]----
    Success  - Can decode Input(H26Forge Fail):       {reference_success_on_input_after_hforge_fails}
    Fail  - Cannot decode Input(H26Forge Fail):       {reference_fails_on_input_after_hforge_fails}
    Fail  - Cannot decode Input(YUV):                 {reference_fails_on_input_for_yuv_comparison}
    Fail  - Cannot decode Passthrough:                {reference_fails_on_passthrough}
    ------------------------------------------------------
    """

    logging.info(results)
    print(results)
    print(f"See end-to-end.log for details")


def test_runner(val):
    i, inputvid, testdir, total_videos = val
    res = run_test(i+1, inputvid, testdir, total_videos)
    return res


def main():
    args = parser.parse_args()
    tool_name = '''

    //=================================================\\\\
    ||                                                 ||    
    ||   _      _____   ____  __                       ||
    ||  | |    / __  \\ / ___|/ _|                      ||
    ||  | |__  `' / /'/ /___| |_ ___  _ __ __ _  ___   ||
    ||  | '_ \\   / /  | ___ \\  _/ _ \\| '__/ _` |/ _ \\  ||
    ||  | | | |./ /___| \_/ | || (_) | | | (_| |  __/  ||
    ||  |_| |_|\\_____/\\_____/_| \\___/|_|  \\__, |\\___|  ||
    ||                                     __/ |       ||
    ||                                    |___/        ||
    ||                  (c) wrv                        ||  
    \\\\=================================================//

    '''
    print(tool_name)
   
    testdir = args.testdir
    logfilename = args.logfilename

    logging.basicConfig(filename=logfilename, encoding='utf-8', level=logging.DEBUG)
    logging.info(tool_name)
    print(f"[X] Testing the directory {testdir}")
    print(f"[X] Evaluate intermediate results in: ./{logfilename}")

    logging.info(testdir)

    all_files = []
    for basedir, dirs, files in os.walk(testdir):
        for filename in files:
            if filename.endswith('.264'):
                all_files.append(os.path.join(testdir, filename))
        # This break is to avoid searching recursive directories
        break
    vid_files = sorted(all_files, key = os.path.getsize)
    total_videos = len(vid_files)
    inputs = zip(range(total_videos), vid_files, [testdir]*total_videos, [total_videos]*total_videos)
   
    print(f"[X] Found {total_videos} videos")
    print(f"... Running ...")
    # Generate output dirs
    generate_output_dirs(testdir)

    # Start the test_runner
    start_time = time.time()
    results = []
    with Pool() as p:
        results = p.map(test_runner, inputs)
    duration = time.time() - start_time
    # Make sure all inputs have been consumed

    logging.info(f"[X] DONE! Took {duration} seconds")
    # Aggregate the results
    print_results(results, total_videos)

    print("Done!")


if __name__ == "__main__":
    main()


'''
Constrained Baseline, Baseline, Extended and Main profiles:
Results: 2/16/23
----------------------[  STATS  ]----------------------
Total Videos:                                     135
----[   h26forge   ]----
Match - Identical:                                73
Match - Diff hash, same syntax:                   51
Fail  - Diff syntax elem:                         0
Fail  - Cannot decode input:                      3
Fail  - Cannot decode Passthrough:                8
Fail  - 1st != 2nd Passthrough:                   0
----[  Reference  ]----
Success  - Can decode Input(H26Forge Fail):       0
Fail  - Cannot decode Input(H26Forge Fail):       3
Fail  - Cannot decode Input(YUV):                 0
Fail  - Cannot decode Passthrough:                0
------------------------------------------------------


MFC Depth High profiles
Results: 2/19/23
----------------------[  STATS  ]----------------------
Total Videos:                                     7
----[   h26forge   ]----
Match - Identical:                                0
Match - Diff hash, same syntax:                   0
Fail  - Diff syntax elem:                         0
Fail  - Cannot decode input:                      7
Fail  - Cannot decode Passthrough:                0
Fail  - 1st != 2nd Passthrough:                   0
----[  Reference  ]----
Success  - Can decode Input(H26Forge Fail):       0
Fail  - Cannot decode Input(H26Forge Fail):       7
Fail  - Cannot decode Input(YUV):                 0
Fail  - Cannot decode Passthrough:                0
------------------------------------------------------
'''