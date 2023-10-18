# Streaming RTP with H26Forge

## Streaming Set-up

Streaming requires a target (the device that will process the generated H264) and a host (the device h26forge runs on). These can be the same device.

On the host, set-up webrtc-rs. First, get the source:

```git clone https://github.com/webrtc-rs/webrtc.git```

The needed code supports VP8. To change it to H264, open webrtc/examples/examples/rtp-to-webrtc/rtp-to-webrtc.rs, and change 'VP8' to 'H264' on lines 8 and 103 (i.e. "use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_VP8};" becomes "use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_H264};". **WARNING:** do not skip this step

Build the needed example using:

```cargo build --example rtp-to-webrtc```

Go to this [JS fiddler](https://jsfiddle.net/z7ms3u5r/) on the target device, and copy the base64 session description. Then, on the host, run:

```echo $BROWSER_SDP | ./target/debug/examples/rtp-to-webrtc```

where BROWSER_SDP is the base64 text copied from the target device.

This command will generate a base64 session description on the commandline. Copy this into the empty box on the target device, then click "Start Session".

The target is now ready to receive H264 packets.

## Running H26forge

To stream to the above set-up, run:

```./h26forge  stream --small --seed 1234 --port 5004````

The seed is a random value used to generate the H264 output and running H26forge with the same seed again will generate the same output, for crash reproduction purposes. For better fuzzing, run H264 with a different seed each time

Most flags in [GENERATION.md](GENERATION.md), including config files, will work while streaming.
