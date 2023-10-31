# H26Forge Release

H26Forge is domain-specific infrastructure for analyzing, generating, and manipulating syntactically correct but semantically spec-non-compliant video files.

## Usage

Generate a video: `./h26forge generate -o out.264`

Generate a video with MP4: `./h26forge --mp4 generate -o out.264`

Edit a video: `./h26forge modify -i input.264 -o output.264 -t transform.py`

Get a video's syntax elements: `./h26forge passthrough -i input.264 -o output.264 -d`

Stream a video (see [STREAMING.md](docs/STREAMING.md)): `./h26forge stream --seed 0 --webrtc-file config.txt`

## Changelog

10-31-2023: Bug fixes, rtp-dump, and streaming mode.
9-12-2023: Bug fix in video encoding.
9-11-2023: Bug fixes in video generation.