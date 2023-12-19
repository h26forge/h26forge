//! WebRTC Streaming Mode
//!
//! This file is adapted from the rtp-to-webrtc WebRTC-rs example found
//! in the webrtc-rs repo: https://github.com/webrtc-rs/webrtc/tree/master/examples/examples/rtp-to-webrtc

use std::fs;
use std::io::Read;
use std::io::Write;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use std::net::TcpStream;

use crate::encoder::encoder::reencode_syntax_elements;
use crate::encoder::rtp::{
    SAFESTART_RTP_0, SAFESTART_RTP_1, SAFESTART_RTP_10, SAFESTART_RTP_2, SAFESTART_RTP_3,
    SAFESTART_RTP_4, SAFESTART_RTP_5, SAFESTART_RTP_6, SAFESTART_RTP_7, SAFESTART_RTP_8,
    SAFESTART_RTP_9,
};
use crate::vidgen::film::FilmState;
use crate::vidgen::generate_configurations::RandomizeConfig;
use crate::vidgen::vidgen::random_video;
use crate::vidgen::syntax_to_video::syntax_to_video;

use anyhow::Result;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_H264};
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::{TrackLocal, TrackLocalWriter};
use webrtc::Error;

/// encode encodes the input in base64
pub fn encode(b: &str) -> String {
    BASE64_STANDARD.encode(b)
}

/// decode decodes the input from base64
pub fn decode(s: &str) -> Result<String> {
    let b = BASE64_STANDARD.decode(s.trim())?;
    let s = String::from_utf8(b)?;
    Ok(s)
}

/// Creates a WebRTC stream
#[tokio::main]
pub async fn stream(
    rconfig: RandomizeConfig,
    ignore_intra_pred: bool,
    ignore_edge_intra_pred: bool,
    ignore_ipcm: bool,
    property_empty_slice_data: bool,
    property_small_video: bool,
    include_undefined_nalus: bool,
    print_silent: bool,
    output_cut: i32,
    mut seed: u64,
    webrtc_file: &str,
    packet_delay: u64,
    server: &str,
    port: u32,
    json: &str,
) -> Result<()> {
    // Create a MediaEngine object to configure the supported codec
    let mut m = MediaEngine::default();

    m.register_default_codecs()?;

    // Create a InterceptorRegistry. This is the user configurable RTP/RTCP Pipeline.
    // This provides NACKs, RTCP Reports and other features. If you use `webrtc.NewPeerConnection`
    // this is enabled by default. If you are manually managing You MUST create a InterceptorRegistry
    // for each PeerConnection.
    let mut registry = Registry::new();

    // Use the default set of Interceptors
    registry = register_default_interceptors(registry, &mut m)?;

    // Create the API object with the MediaEngine
    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    // Prepare the configuration
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()
    };

    // Used to determine whether to generate videos or not
    // 0: still waiting
    // 1: connected
    // 2: fail or closed
    let status = Arc::new(AtomicU8::new(0));

    // Create a new RTCPeerConnection
    let peer_connection = Arc::new(api.new_peer_connection(config).await?);

    // Create Track that we send video back to browser on
    let video_track = Arc::new(TrackLocalStaticRTP::new(
        RTCRtpCodecCapability {
            mime_type: MIME_TYPE_H264.to_owned(),
            ..Default::default()
        },
        "video".to_owned(),
        "webrtc-rs".to_owned(),
    ));

    // Add this newly created track to the PeerConnection
    let rtp_sender = peer_connection
        .add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>)
        .await?;

    // Read incoming RTCP packets
    // Before these packets are returned they are processed by interceptors. For things
    // like NACK this needs to be called.
    tokio::spawn(async move {
        let mut rtcp_buf = vec![0u8; 1500];
        while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
        Result::<()>::Ok(())
    });

    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel::<()>(1);
    let (vid_tx, mut vid_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(64);

    let status1 = status.clone();
    let done_tx1 = done_tx.clone();
    // Set the handler for ICE connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection.on_ice_connection_state_change(Box::new(
        move |connection_state: RTCIceConnectionState| {
            println!("Connection State has changed {connection_state}");
            if connection_state == RTCIceConnectionState::Failed {
                let _ = done_tx1.try_send(());
                // Halt video generation
                status1.store(2, Ordering::Relaxed);
            } else if connection_state == RTCIceConnectionState::Connected {
                // Start video generation
                status1.store(1, Ordering::Relaxed);
            } else if connection_state == RTCIceConnectionState::Closed {
                // Halt video generation
                status1.store(2, Ordering::Relaxed);
            } else if connection_state == RTCIceConnectionState::Disconnected {
                // Halt video generation
                status1.store(2, Ordering::Relaxed);
            }
            Box::pin(async {})
        },
    ));

    let status2 = status.clone();

    let done_tx2 = done_tx.clone();
    // Set the handler for Peer connection state
    // This will notify you when the peer has connected/disconnected
    peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
        println!("Peer Connection State has changed: {s}");

        if s == RTCPeerConnectionState::Failed {
            // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
            // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
            // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
            println!("Peer Connection has gone to failed exiting: Done forwarding");
            let _ = done_tx2.try_send(());
            // Halt video generation
            status2.store(2, Ordering::Relaxed);
        } else if s == RTCPeerConnectionState::Connected {
            // Start video generation
            status2.store(1, Ordering::Relaxed);
        } else if s == RTCPeerConnectionState::Closed {
            // Halt video generation
            status2.store(2, Ordering::Relaxed);
        } else if s == RTCPeerConnectionState::Disconnected {
            // Halt video generation
            status2.store(2, Ordering::Relaxed);
        }

        Box::pin(async {})
    }));

    let mut encoded_desc = "".to_owned();
    if !server.is_empty() {
        let mut server_string = server.to_owned();
        server_string.push_str(":");
        server_string.push_str(&port.to_string());
        match TcpStream::connect(&server_string) {
            Ok(mut stream) => {
                let msg = b"Hello!";
                stream.write(msg).unwrap();
                println!("Awaiting SDP from server {}", server_string);
                while !encoded_desc.ends_with("\n"){
                    match stream.read_to_string(&mut encoded_desc ) {
                        Ok(_) => {},
                        Err(e) => {
                            println!("Failed to receive data: {}", e);
                        }
                    }
                }
            },
            Err(e) => {
                println!("Failed to connect: {} {}", e, server_string);
            }
        }
        encoded_desc.pop();
    } else {
        encoded_desc = fs::read_to_string(webrtc_file)?;
    }
    let desc_data = decode(&encoded_desc)?;
    let offer = serde_json::from_str::<RTCSessionDescription>(&desc_data)?;

    // Set the remote SessionDescription
    peer_connection.set_remote_description(offer).await?;

    // Create an answer
    let answer = peer_connection.create_answer(None).await?;

    // Create channel that is blocked until ICE Gathering is complete
    let mut gather_complete = peer_connection.gathering_complete_promise().await;

    // Sets the LocalDescription, and starts our UDP listeners
    peer_connection.set_local_description(answer).await?;

    // Block until ICE Gathering is complete, disabling trickle ICE
    // we do this because we only can exchange one signaling message
    // in a production application you should exchange ICE Candidates via OnICECandidate
    let _ = gather_complete.recv().await;

    // Output the answer in base64 so we can paste it in browser
    let mut b64 = "".to_owned();
    if let Some(local_desc) = peer_connection.local_description().await {
        let json_str = serde_json::to_string(&local_desc)?;
        b64 = encode(&json_str);
        println!("{b64}");
    } else {
        println!("generate local_description failed!");
    }

    if !server.is_empty() {
        b64.push('\n');
        let mut server_string = server.to_owned();
        server_string.push_str(":");
        server_string.push_str(&(port).to_string());
        match TcpStream::connect(&server_string) {
            Ok(mut stream) => {
                let msg = b64;
                stream.write(msg.as_bytes()).unwrap();
            },
            Err(e) => {
                println!("Failed to connect and send local desc: {} {}", e, server_string);
            }
        }
    }

    let done_tx3 = done_tx.clone();
    let json_filename = json.to_string();

    // Video generator
    // Generate and send RTP packets via WebRTC
    tokio::spawn(async move {
        let mut seq_num: u16 = 0x1234;
        let mut timestamp: u32 = 0x11223344;
        let mut safe_start = true;
        
        if json_filename.is_empty() {
            let mut ind = 1;
            loop {
                if status.load(Ordering::Relaxed) == 2 {
                    println!("Stopping video generation");
                    return;
                } else if status.load(Ordering::Relaxed) == 1 {
                    let mut rtp: Vec<Vec<u8>> = Vec::new();

                    if ind % 5 == 4 {
                        safe_start = true
                    }
                    if ind < 10 {
                        safe_start = true;
                    }
                    if safe_start {
                        println!("[Video {}] Generating safestart", ind);
                        rtp.push(SAFESTART_RTP_0.to_vec());
                        rtp.push(SAFESTART_RTP_1.to_vec());
                        rtp.push(SAFESTART_RTP_2.to_vec());
                        rtp.push(SAFESTART_RTP_3.to_vec());
                        rtp.push(SAFESTART_RTP_4.to_vec());
                        rtp.push(SAFESTART_RTP_5.to_vec());
                        rtp.push(SAFESTART_RTP_6.to_vec());
                        rtp.push(SAFESTART_RTP_7.to_vec());
                        rtp.push(SAFESTART_RTP_8.to_vec());
                        rtp.push(SAFESTART_RTP_9.to_vec());
                        rtp.push(SAFESTART_RTP_10.to_vec());
                        safe_start = false;
                    } else {
                        println!("[Video {}] Generating random video with seed {}", ind, seed);
                        let mut film_state = FilmState::setup_film_from_seed(seed);
                        let mut decoded_elements = random_video(
                            ignore_intra_pred,
                            ignore_edge_intra_pred,
                            ignore_ipcm,
                            property_empty_slice_data,
                            property_small_video,
                            print_silent,
                            include_undefined_nalus,
                            &rconfig,
                            &mut film_state,
                        );
                        seed = seed.wrapping_add(1);

                        // 2. Re-encode the NALUs

                        let res = reencode_syntax_elements(
                            &mut decoded_elements,
                            output_cut,
                            false,
                            print_silent,
                            true,
                        );
                        rtp = res.2;
                    }
                    println!("[Video {}] Sending {} packets", ind, rtp.len());
                    (timestamp, seq_num) = packetize_and_send(&vid_tx, rtp, timestamp, seq_num, packet_delay).await;
                    ind += 1;
                }
            }
        } else {
            loop {
                if status.load(Ordering::Relaxed) == 1 {
                    println!("[Video] Generating from video JSON");
                    let mut decoded_elements = syntax_to_video(&json_filename);
                    let res = reencode_syntax_elements(
                        &mut decoded_elements,
                        output_cut,
                        false,
                        print_silent,
                        true,
                    );
                    let rtp = res.2;
                    packetize_and_send(&vid_tx, rtp, timestamp, seq_num, packet_delay).await;
                    break;
                }
            }
            println!("All done - press ctrl-c to exit");
        }
    });

    // Video consumer
    tokio::spawn(async move {
        while let Some(pack) = vid_rx.recv().await {
            if let Err(err) = video_track.write(&pack).await {
                if Error::ErrClosedPipe == err {
                    // The peerConnection has been closed.
                } else {
                    println!("video_track write err: {err}");
                }
                let _ = done_tx3.try_send(());
                return;
            }
        }
    });

    println!("Press ctrl-c to stop");
    tokio::select! {
        _ = done_rx.recv() => {
            println!("received done signal!");
        }
        _ = tokio::signal::ctrl_c() => {
            println!();
        }
    };

    peer_connection.close().await?;

    Ok(())
}

pub async fn packetize_and_send(
    vid_tx: &tokio::sync::mpsc::Sender<Vec<u8>>,
    rtp: Vec<Vec<u8>>,
    timestamp_param: u32,
    seq_param: u16,
    packet_delay: u64,
)-> (u32, u16) {

    let mut seq_num: u16 = seq_param;
    let mut timestamp: u32 = timestamp_param;
    let ssrc: u32 = 0x77777777;
    for pack in rtp {
        let header_byte = 0x80; // version 2, no padding, no extensions, no CSRC, marker = false;
        let nal_type = pack[0] & 0x1f;
        let mut output_pack = Vec::new();
        output_pack.push(header_byte);
        let mut payload_type = 102;
        if nal_type == 5 {
            payload_type = payload_type + 0x80; // add marker
        }
        if nal_type == 1 {
            payload_type = payload_type + 0x80; // add marker
            timestamp += 3000;
        }
        if nal_type == 28 {
            let inner_nal_type = pack[1] & 0x1f;
            if inner_nal_type == 5 {
                payload_type = payload_type + 0x80; // add marker
            }
            if inner_nal_type == 1 {
                payload_type = payload_type + 0x80; // add marker
                if (pack[1] & 0x80) != 0 {
                    timestamp += 3000;
                }
            }
        }
        output_pack.push(payload_type);
        output_pack.extend(seq_num.to_be_bytes());
        seq_num += 1;
        output_pack.extend(timestamp.to_be_bytes());
        output_pack.extend(ssrc.to_be_bytes());
        output_pack.extend(pack.clone());

        let _ = vid_tx.send(output_pack).await;
        if packet_delay != 0 {
            std::thread::sleep(Duration::from_millis(packet_delay));
        }
    }
    println!("[Video] Done");
    return(timestamp, seq_num);
}
