# Intermediate Web Server for Streaming

This server is used to relay the SDP server description between a host and target. Any web server with a publicly routable IP address should work. See the *Server flag* section in [STREAMING.md](../../docs/STREAMING.md#server-flag) for more details. Based off this [JS Fiddle](https://jsfiddle.net/z7ms3u5r/).

**NOTE: You must replace `<SERVER IP>` inside of [server.js](./server.js) before opening server.html.**

Files
- server.py: Accepts connections on port 8787 and relays SDP server descriptions via WebSockets
- server.js: Handles WebSockets and WebRTC communications
- server.html: Displays the video

Known issues:
- On some systems, server.py fails to handle Ctrl+C signals while waiting for a connection to accept. See https://stackoverflow.com/q/15189888/8169613. Task Manager, `kill`, or `Stop-Process` is your friend for now.