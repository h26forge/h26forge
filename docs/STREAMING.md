# Streaming RTP to WebRTC with H26Forge

Streaming requires a target (the device that will process the generated H.264) and a host (the device H26Forge runs on). These can be the same device. Because we are establishing a peer-to-peer connection, we need to configure the target and the host for communication.

## Setting up the Target and Host
1. On the **target** device, go to this [JS Fiddle](https://jsfiddle.net/z7ms3u5r/) and copy the "Browser base64 Session Description" and send it to the host.
2. On the **host** device, save the "Browser base64 Session Description" to a file. In this example we call it `stream_config.txt`. Then run the following command:

    ```./h26forge stream --small --seed 1234 --webrtc-file stream_config.txt```.'

    This command will generate a base64 session description on the command line. Copy this and send it to the target device.
3. On the **target** device, paste the base64 into the empty "Golang base64 Session Description" box, then click "Start Session".

You should then begin to see the host generating videos and playback begin on the target.

## Video Generation Details

The seed is a random value used to generate the H.264 output and running H26Forge with the same seed again will generate the same output, for crash reproduction purposes. For better fuzzing, run H26Forge with a different seed each time.

Most flags in [GENERATION.md](GENERATION.md), including config files, will work while streaming.

If the target is experiencing SRTP decryption failure, it is likely receiving video traffic too fast. The `--packet-delay` flag can be used to slow down the RTP send rate. A delay of 50 ms works with most targets.

## Limitations

The code is hard-coded to work with `stun:stun.l.google.com:19302` and the HTML JavaScript in this [JS Fiddle](https://jsfiddle.net/z7ms3u5r/).

Future work may enable other streaming modes.
