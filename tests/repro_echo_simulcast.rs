use anyhow::Result;
use bytes::Bytes;
use rustrtc::media::track::MediaStreamTrack;
use rustrtc::rtp::{
    RtpHeader as RustRtpHeader, RtpHeaderExtension as RustRtpHeaderExtension,
    RtpPacket as RustRtpPacket,
};
use rustrtc::{MediaKind, PeerConnection, RtcConfiguration, TransceiverDirection};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use webrtc::api::APIBuilder;
use webrtc::api::media_engine::MediaEngine;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration as WebrtcConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp::header::{Extension, Header};
use webrtc::rtp::packet::Packet;
use webrtc::rtp_transceiver::RTCRtpEncodingParameters;
use webrtc::rtp_transceiver::RTCRtpTransceiverInit;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpHeaderExtensionCapability;
use webrtc::rtp_transceiver::rtp_codec::{RTCRtpCodecCapability, RTPCodecType};
use webrtc::rtp_transceiver::rtp_transceiver_direction::RTCRtpTransceiverDirection;
use webrtc::track::track_local::TrackLocalWriter;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::util::Unmarshal;

#[tokio::test]
async fn test_echo_simulcast() -> Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();

    // 1. Create RustRTC PeerConnection (Server/Echo)
    let rust_config = RtcConfiguration::default();
    let rust_pc = PeerConnection::new(rust_config);

    // Add a transceiver to receive Video (Simulcast) and Send (Echo)
    let transceiver = rust_pc.add_transceiver(MediaKind::Video, TransceiverDirection::SendRecv);

    // 2. Create WebRTC PeerConnection (Client)
    let mut m = MediaEngine::default();
    m.register_default_codecs()?;
    m.register_header_extension(
        RTCRtpHeaderExtensionCapability {
            uri: "urn:ietf:params:rtp-hdrext:sdes:rtp-stream-id".to_owned(),
        },
        RTPCodecType::Video,
        Some(RTCRtpTransceiverDirection::Sendrecv),
    )?;
    let registry = Registry::new();
    let api = APIBuilder::new()
        .with_media_engine(m)
        .with_interceptor_registry(registry)
        .build();

    let webrtc_config = WebrtcConfiguration::default();
    let webrtc_pc = api.new_peer_connection(webrtc_config).await?;

    // Create a video track
    let codec = RTCRtpCodecCapability {
        mime_type: webrtc::api::media_engine::MIME_TYPE_VP8.to_owned(),
        clock_rate: 90000,
        channels: 0,
        ..Default::default()
    };
    let video_track = Arc::new(TrackLocalStaticRTP::new(
        codec,
        "video".to_string(),
        "webrtc_stream".to_string(),
    ));

    let _webrtc_transceiver = webrtc_pc
        .add_transceiver_from_track(
            Arc::clone(&video_track)
                as Arc<dyn webrtc::track::track_local::TrackLocal + Send + Sync>,
            Some(RTCRtpTransceiverInit {
                direction: RTCRtpTransceiverDirection::Sendrecv,
                send_encodings: vec![
                    RTCRtpEncodingParameters {
                        rid: "hi".into(),
                        ..Default::default()
                    },
                    RTCRtpEncodingParameters {
                        rid: "lo".into(),
                        ..Default::default()
                    },
                ],
            }),
        )
        .await?;

    // 3. WebRTC creates Offer
    let offer = webrtc_pc.create_offer(None).await?;
    let mut gather_complete = webrtc_pc.gathering_complete_promise().await;
    webrtc_pc.set_local_description(offer.clone()).await?;
    let _ = gather_complete.recv().await;

    let offer = webrtc_pc.local_description().await.unwrap();

    // Convert WebRTC SDP to RustRTC SDP (inject simulcast attributes if missing from Pion's SDP)
    let mut offer_sdp = offer.sdp;
    if offer_sdp.contains("m=video") && !offer_sdp.contains("a=simulcast") {
        offer_sdp = offer_sdp.replace(
            "a=sendrecv",
            "a=sendrecv\r\na=rid:hi send\r\na=rid:lo send\r\na=simulcast:send hi;lo",
        );
    }
    println!("Offer SDP:\n{}", offer_sdp);
    let rust_offer = rustrtc::SessionDescription::parse(rustrtc::SdpType::Offer, &offer_sdp)?;

    // 4. RustRTC sets Remote Description
    rust_pc.set_remote_description(rust_offer).await?;

    // 5. RustRTC creates Answer
    let answer = rust_pc.create_answer().await?;
    rust_pc.set_local_description(answer.clone())?;

    let answer_sdp = answer.to_sdp_string();
    println!("Answer SDP:\n{}", answer_sdp);
    let webrtc_answer = RTCSessionDescription::answer(answer_sdp)?;

    // 6. WebRTC sets Remote Description
    webrtc_pc.set_remote_description(webrtc_answer).await?;

    // 7. Setup Echo Logic on RustRTC
    let transceivers = rust_pc.get_transceivers();
    let t = transceivers.first().unwrap();
    let receiver = t.receiver().unwrap();

    // Simulate echo_server logic
    let simulcast_rids = receiver.get_simulcast_rids();
    println!("Simulcast RIDs: {:?}", simulcast_rids);

    let incoming_track = if !simulcast_rids.is_empty() {
        let rid = if simulcast_rids.contains(&"lo".to_string()) {
            "lo"
        } else {
            simulcast_rids.first().unwrap()
        };
        println!("Selected RID: {}", rid);
        receiver.simulcast_track(rid).unwrap()
    } else {
        receiver.track()
    };

    let (sample_source, outgoing_track, _) =
        rustrtc::media::sample_track(rustrtc::media::MediaKind::Video, 120);
    let sender = Arc::new(rustrtc::peer_connection::RtpSender::new(
        outgoing_track.clone(),
        5555,
    ));
    sender.set_params(rustrtc::peer_connection::RtpCodecParameters {
        payload_type: 96,
        clock_rate: 90000,
        channels: 0,
    });
    t.set_sender(Some(sender));

    // Echo loop
    tokio::spawn(async move {
        while let Ok(sample) = incoming_track.recv().await {
            println!("Echoing sample");
            let _ = sample_source.send(sample).await;
        }
    });

    // 8. Wait for connection
    rust_pc.wait_for_connection().await?;

    // 9. Start sending data from WebRTC (Client)
    let video_track_clone = video_track.clone();
    tokio::spawn(async move {
        let mut sequence_number = 0u16;
        loop {
            tokio::time::sleep(Duration::from_millis(33)).await;

            // Construct packet using rustrtc to ensure correct padding/alignment
            let mut header =
                RustRtpHeader::new(96, sequence_number, sequence_number as u32 * 3000, 1111);
            header.marker = false;

            // Manually construct Two-Byte Header extension block
            // ID=1, Len=2 ("lo")
            // Two-Byte Header: ID (1 byte) | Len (1 byte) | Data
            let mut ext_data = Vec::new();
            ext_data.push(1); // ID
            ext_data.push(2); // Len
            ext_data.extend_from_slice(b"lo");
            // Total 4 bytes. Aligned.

            header.extension = Some(RustRtpHeaderExtension::new(0x1000, ext_data));

            let rust_packet = RustRtpPacket::new(header, vec![0u8; 100]);
            let packet_bytes = rust_packet
                .marshal()
                .expect("Failed to marshal rust packet");

            // Unmarshal into webrtc packet
            let mut buf = Bytes::from(packet_bytes);
            let packet = Packet::unmarshal(&mut buf).expect("Failed to unmarshal webrtc packet");

            if let Err(e) = video_track_clone.write_rtp(&packet).await {
                println!("Failed to write RTP: {}", e);
                break;
            } else {
                println!("Wrote RTP packet seq={}", sequence_number);
            }
            sequence_number = sequence_number.wrapping_add(1);
        }
    });

    // 10. Verify WebRTC receives echoed packets
    // We need to attach a receiver to WebRTC PC
    // But Pion's API for receiving is via OnTrack

    let (done_tx, mut done_rx) = tokio::sync::mpsc::channel(1);

    webrtc_pc.on_track(Box::new(move |track, _, _| {
        let done_tx = done_tx.clone();
        Box::pin(async move {
            println!("WebRTC received track");
            let mut count = 0;
            while let Ok((_, _)) = track.read_rtp().await {
                count += 1;
                if count > 5 {
                    let _ = done_tx.send(()).await;
                    break;
                }
            }
        })
    }));

    // Wait for echo
    match timeout(Duration::from_secs(5), done_rx.recv()).await {
        Ok(Some(_)) => println!("Success: Echo received"),
        Ok(None) => panic!("Channel closed"),
        Err(_) => panic!("Timeout waiting for echo"),
    }

    rust_pc.close();
    webrtc_pc.close().await?;

    Ok(())
}
