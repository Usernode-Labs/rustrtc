use crate::media::depacketizer::{DefaultDepacketizerFactory, DepacketizerFactory};
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

/// Describes how credentials are conveyed for a given ICE server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum IceCredentialType {
    #[default]
    Password,
    Oauth,
}

/// Mirrors the W3C `RTCIceServer` dictionary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
    #[serde(default)]
    pub credential_type: IceCredentialType,
}

impl IceServer {
    pub fn new<T: Into<Vec<String>>>(urls: T) -> Self {
        Self {
            urls: urls.into(),
            username: None,
            credential: None,
            credential_type: IceCredentialType::default(),
        }
    }

    pub fn with_credential(
        mut self,
        username: impl Into<String>,
        credential: impl Into<String>,
    ) -> Self {
        self.username = Some(username.into());
        self.credential = Some(credential.into());
        self
    }

    pub fn credential_type(mut self, kind: IceCredentialType) -> Self {
        self.credential_type = kind;
        self
    }
}

impl Default for IceServer {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum IceTransportPolicy {
    #[default]
    All,
    Relay,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum BundlePolicy {
    #[default]
    Balanced,
    MaxCompat,
    MaxBundle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RtcpMuxPolicy {
    #[default]
    Require,
    Negotiate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TransportMode {
    #[default]
    WebRtc,
    Srtp,
    Rtp,
}

/// Tracks user-supplied certificate material.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CertificateConfig {
    pub pem_chain: Vec<String>,
    pub private_key_pem: Option<String>,
}

/// Configuration for audio/video codecs and parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AudioCapability {
    pub payload_type: u8,
    pub codec_name: String,
    pub clock_rate: u32,
    pub channels: u8,
    pub fmtp: Option<String>,
    pub rtcp_fbs: Vec<String>,
}

impl Default for AudioCapability {
    fn default() -> Self {
        Self {
            payload_type: 111,
            codec_name: "opus".to_string(),
            clock_rate: 48000,
            channels: 2,
            fmtp: Some("minptime=10;useinbandfec=1".to_string()),
            rtcp_fbs: vec![],
        }
    }
}

impl AudioCapability {
    pub fn opus() -> Self {
        Self::default()
    }

    pub fn pcmu() -> Self {
        Self {
            payload_type: 0,
            codec_name: "PCMU".to_string(),
            clock_rate: 8000,
            channels: 1,
            fmtp: None,
            rtcp_fbs: vec![],
        }
    }

    pub fn pcma() -> Self {
        Self {
            payload_type: 8,
            codec_name: "PCMA".to_string(),
            clock_rate: 8000,
            channels: 1,
            fmtp: None,
            rtcp_fbs: vec![],
        }
    }

    pub fn g722() -> Self {
        Self {
            payload_type: 9,
            codec_name: "G722".to_string(),
            clock_rate: 8000,
            channels: 1,
            fmtp: None,
            rtcp_fbs: vec![],
        }
    }

    pub fn g729() -> Self {
        Self {
            payload_type: 18,
            codec_name: "G729".to_string(),
            clock_rate: 8000,
            channels: 1,
            fmtp: None,
            rtcp_fbs: vec![],
        }
    }

    pub fn telephone_event() -> Self {
        Self {
            payload_type: 101,
            codec_name: "telephone-event".to_string(),
            clock_rate: 8000,
            channels: 1,
            fmtp: Some("0-16".to_string()),
            rtcp_fbs: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VideoCapability {
    pub payload_type: u8,
    pub codec_name: String,
    pub clock_rate: u32,
    pub rtcp_fbs: Vec<String>,
}

impl Default for VideoCapability {
    fn default() -> Self {
        Self {
            payload_type: 96,
            codec_name: "VP8".to_string(),
            clock_rate: 90000,
            rtcp_fbs: vec![
                "nack".to_string(),
                "nack pli".to_string(),
                "ccm fir".to_string(),
                "goog-remb".to_string(),
                "transport-cc".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApplicationCapability {
    pub sctp_port: u16,
}

impl Default for ApplicationCapability {
    fn default() -> Self {
        Self { sctp_port: 5000 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaCapabilities {
    pub audio: Vec<AudioCapability>,
    pub video: Vec<VideoCapability>,
    pub application: Option<ApplicationCapability>,
}

impl Default for MediaCapabilities {
    fn default() -> Self {
        Self {
            audio: vec![AudioCapability::opus(), AudioCapability::pcmu()],
            video: vec![VideoCapability::default()],
            application: Some(ApplicationCapability::default()),
        }
    }
}

#[derive(Clone)]
pub struct DepacketizerStrategy {
    pub factory: Arc<dyn DepacketizerFactory>,
}

impl Default for DepacketizerStrategy {
    fn default() -> Self {
        Self {
            factory: Arc::new(DefaultDepacketizerFactory),
        }
    }
}

impl Debug for DepacketizerStrategy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.factory.fmt(f)
    }
}

impl PartialEq for DepacketizerStrategy {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.factory, &other.factory)
    }
}

impl Eq for DepacketizerStrategy {}

fn default_filter_private_host_candidates() -> bool {
    true
}

/// Primary configuration for a `PeerConnection`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RtcConfiguration {
    pub ice_servers: Vec<IceServer>,
    pub ice_transport_policy: IceTransportPolicy,
    pub bundle_policy: BundlePolicy,
    pub rtcp_mux_policy: RtcpMuxPolicy,
    pub certificates: Vec<CertificateConfig>,
    pub transport_mode: TransportMode,
    pub nack_buffer_size: usize,
    pub media_capabilities: Option<MediaCapabilities>,
    pub external_ip: Option<String>,
    pub bind_ip: Option<String>,
    pub disable_ipv6: bool,
    #[serde(default = "default_filter_private_host_candidates")]
    pub filter_private_host_candidates: bool,
    pub ssrc_start: u32,
    pub stun_timeout: std::time::Duration,
    /// Timeout for the ICE nomination binding check (USE-CANDIDATE).
    /// This should be larger than `stun_timeout` to allow more retransmissions
    /// and reduce the probability of nomination failures under packet loss.
    pub nomination_timeout: std::time::Duration,
    pub ice_connection_timeout: std::time::Duration,
    /// Initial SCTP retransmission timeout (RTO).
    pub sctp_rto_initial: std::time::Duration,
    /// Minimum SCTP retransmission timeout (RTO).
    pub sctp_rto_min: std::time::Duration,
    /// Maximum SCTP retransmission timeout (RTO).
    pub sctp_rto_max: std::time::Duration,
    /// Maximum association retransmissions before declaring the peer dead.
    pub sctp_max_association_retransmits: u32,
    /// Local SCTP receiver window in bytes.
    ///
    /// Larger values allow more in-flight data (higher throughput on high-BDP
    /// links), at the cost of memory.
    pub sctp_receive_window: usize,
    /// SCTP heartbeat interval.
    pub sctp_heartbeat_interval: std::time::Duration,
    /// Maximum consecutive heartbeat failures before declaring the peer dead.
    pub sctp_max_heartbeat_failures: u32,
    /// Enable verbose SCTP diagnostics at `info` level (target: `rustrtc::sctp_diag`).
    ///
    /// Intended for troubleshooting; may be noisy.
    pub sctp_diag_enabled: bool,
    /// Disable SACK signature gating and count missing reports even on duplicate SACKs.
    ///
    /// Intended for troubleshooting; disabling gating can make loss detection more
    /// aggressive.
    pub sctp_no_sack_sig_gating: bool,
    /// Initial SCTP congestion window (cwnd) in bytes.
    ///
    /// `0` uses the library default (currently IW10 ~= 10 * 1200 bytes).
    pub sctp_initial_cwnd: usize,
    /// Maximum cwnd increase per SACK while in slow start, in bytes.
    ///
    /// This caps exponential growth when the receive path coalesces ACKs.
    /// `0` means "use the effective initial cwnd".
    pub sctp_slow_start_increase_cap: usize,
    /// When non-zero, delay SACKs in the presence of gaps (out-of-order data) by
    /// `srtt / divisor` (clamped internally).
    ///
    /// `0` disables the gap-SACK delay (SACKs are sent immediately).
    pub sctp_gap_sack_delay_srtt_divisor: u32,
    /// Maximum burst size for SCTP in number of MTU-sized packets.
    ///
    /// `0` uses a heuristic (16 packets normal, 4 in recovery) unless
    /// `sctp_unlimited_burst_non_recovery` is enabled.
    pub sctp_max_burst: usize,
    /// If true and `sctp_max_burst == 0`, disable burst limiting outside of
    /// loss recovery. This avoids adding an extra RTT for payloads that already
    /// fit within `cwnd`.
    pub sctp_unlimited_burst_non_recovery: bool,
    /// Maximum congestion window size in bytes.
    pub sctp_max_cwnd: usize,
    pub dtls_buffer_size: usize,
    pub rtp_start_port: Option<u16>,
    pub rtp_end_port: Option<u16>,
    pub enable_latching: bool,
    pub enable_ice_lite: bool,
    #[serde(skip, default)]
    pub depacketizer_strategy: DepacketizerStrategy,
}

impl Default for RtcConfiguration {
    fn default() -> Self {
        Self {
            ice_servers: Vec::new(),
            ice_transport_policy: IceTransportPolicy::default(),
            bundle_policy: BundlePolicy::default(),
            rtcp_mux_policy: RtcpMuxPolicy::default(),
            certificates: Vec::new(),
            transport_mode: TransportMode::default(),
            nack_buffer_size: 200,
            media_capabilities: None,
            external_ip: None,
            bind_ip: None,
            disable_ipv6: false,
            filter_private_host_candidates: true,
            ssrc_start: 10000,
            stun_timeout: std::time::Duration::from_secs(5),
            nomination_timeout: std::time::Duration::from_secs(10),
            ice_connection_timeout: std::time::Duration::from_secs(30),
            sctp_rto_initial: std::time::Duration::from_secs(3),
            sctp_rto_min: std::time::Duration::from_secs(1),
            sctp_rto_max: std::time::Duration::from_secs(60),
            sctp_max_association_retransmits: 20,
            sctp_receive_window: 128 * 1024, // 128KB - reduced for lower memory footprint
            sctp_heartbeat_interval: std::time::Duration::from_secs(15),
            sctp_max_heartbeat_failures: 4,
            sctp_diag_enabled: false,
            sctp_no_sack_sig_gating: false,
            sctp_initial_cwnd: 0,               // 0 = use internal default (currently IW10)
            sctp_slow_start_increase_cap: 0,    // 0 = use effective initial cwnd
            sctp_gap_sack_delay_srtt_divisor: 2, // 2 = srtt/2 (clamped internally)
            sctp_max_burst: 0, // 0 = use default heuristic
            sctp_unlimited_burst_non_recovery: true,
            sctp_max_cwnd: 256 * 1024, // 256 KB
            dtls_buffer_size: 2048,
            rtp_start_port: None,
            rtp_end_port: None,
            enable_latching: false,
            enable_ice_lite: false,
            depacketizer_strategy: DepacketizerStrategy::default(),
        }
    }
}

pub struct RtcConfigurationBuilder {
    inner: RtcConfiguration,
}

impl Default for RtcConfigurationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RtcConfigurationBuilder {
    pub fn new() -> Self {
        Self {
            inner: RtcConfiguration::default(),
        }
    }

    pub fn enable_latching(mut self, enable: bool) -> Self {
        self.inner.enable_latching = enable;
        self
    }

    pub fn enable_ice_lite(mut self, enable: bool) -> Self {
        self.inner.enable_ice_lite = enable;
        self
    }

    pub fn filter_private_host_candidates(mut self, enable: bool) -> Self {
        self.inner.filter_private_host_candidates = enable;
        self
    }

    pub fn ice_server(mut self, server: IceServer) -> Self {
        self.inner.ice_servers.push(server);
        self
    }

    pub fn ice_transport_policy(mut self, policy: IceTransportPolicy) -> Self {
        self.inner.ice_transport_policy = policy;
        self
    }

    pub fn bundle_policy(mut self, policy: BundlePolicy) -> Self {
        self.inner.bundle_policy = policy;
        self
    }

    pub fn rtcp_mux_policy(mut self, policy: RtcpMuxPolicy) -> Self {
        self.inner.rtcp_mux_policy = policy;
        self
    }

    pub fn certificate(mut self, cert: CertificateConfig) -> Self {
        self.inner.certificates.push(cert);
        self
    }

    pub fn transport_mode(mut self, mode: TransportMode) -> Self {
        self.inner.transport_mode = mode;
        self
    }

    pub fn media_capabilities(mut self, capabilities: MediaCapabilities) -> Self {
        self.inner.media_capabilities = Some(capabilities);
        self
    }

    pub fn external_ip(mut self, ip: String) -> Self {
        self.inner.external_ip = Some(ip);
        self
    }

    pub fn bind_ip(mut self, ip: String) -> Self {
        self.inner.bind_ip = Some(ip);
        self
    }

    pub fn disable_ipv6(mut self, disable: bool) -> Self {
        self.inner.disable_ipv6 = disable;
        self
    }

    pub fn ssrc_start(mut self, start: u32) -> Self {
        self.inner.ssrc_start = start;
        self
    }

    pub fn stun_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.inner.stun_timeout = timeout;
        self
    }

    pub fn nomination_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.inner.nomination_timeout = timeout;
        self
    }

    pub fn rtp_port_range(mut self, start: u16, end: u16) -> Self {
        self.inner.rtp_start_port = Some(start);
        self.inner.rtp_end_port = Some(end);
        self
    }

    pub fn dtls_buffer_size(mut self, size: usize) -> Self {
        self.inner.dtls_buffer_size = size;
        self
    }

    pub fn sctp_rto_initial(mut self, duration: std::time::Duration) -> Self {
        self.inner.sctp_rto_initial = duration;
        self
    }

    pub fn sctp_rto_min(mut self, duration: std::time::Duration) -> Self {
        self.inner.sctp_rto_min = duration;
        self
    }

    pub fn sctp_rto_max(mut self, duration: std::time::Duration) -> Self {
        self.inner.sctp_rto_max = duration;
        self
    }

    pub fn sctp_max_association_retransmits(mut self, count: u32) -> Self {
        self.inner.sctp_max_association_retransmits = count;
        self
    }

    pub fn sctp_receive_window(mut self, size: usize) -> Self {
        self.inner.sctp_receive_window = size;
        self
    }

    pub fn sctp_heartbeat_interval(mut self, duration: std::time::Duration) -> Self {
        self.inner.sctp_heartbeat_interval = duration;
        self
    }

    pub fn sctp_max_heartbeat_failures(mut self, count: u32) -> Self {
        self.inner.sctp_max_heartbeat_failures = count;
        self
    }

    /// Enable verbose SCTP diagnostics at `info` level (target: `rustrtc::sctp_diag`).
    pub fn sctp_diag_enabled(mut self, enable: bool) -> Self {
        self.inner.sctp_diag_enabled = enable;
        self
    }

    /// Disable SACK signature gating and count missing reports even on duplicate SACKs.
    pub fn sctp_no_sack_sig_gating(mut self, enable: bool) -> Self {
        self.inner.sctp_no_sack_sig_gating = enable;
        self
    }

    /// Set the initial SCTP congestion window (cwnd) in bytes.
    /// `0` means "use the library default".
    pub fn sctp_initial_cwnd(mut self, bytes: usize) -> Self {
        self.inner.sctp_initial_cwnd = bytes;
        self
    }

    /// Set the maximum cwnd increase per SACK during slow start, in bytes.
    /// `0` means "use the effective initial cwnd".
    pub fn sctp_slow_start_increase_cap(mut self, bytes: usize) -> Self {
        self.inner.sctp_slow_start_increase_cap = bytes;
        self
    }

    /// Set the divisor for gap-SACK delay: `delay = srtt / divisor` (clamped internally).
    /// `0` disables the gap-SACK delay (SACKs are sent immediately).
    pub fn sctp_gap_sack_delay_srtt_divisor(mut self, divisor: u32) -> Self {
        self.inner.sctp_gap_sack_delay_srtt_divisor = divisor;
        self
    }

    /// Set the maximum burst size for SCTP in number of MTU-sized packets.
    /// 0 means use the default heuristic (16 packets normal, 4 in recovery),
    /// unless `sctp_unlimited_burst_non_recovery` is enabled.
    /// For rate-limited TURN relays, a value of 2-4 can reduce burst-induced
    /// packet loss.
    pub fn sctp_max_burst(mut self, packets: usize) -> Self {
        self.inner.sctp_max_burst = packets;
        self
    }

    /// If true and `sctp_max_burst == 0`, disable burst limiting outside of
    /// loss recovery.
    pub fn sctp_unlimited_burst_non_recovery(mut self, enable: bool) -> Self {
        self.inner.sctp_unlimited_burst_non_recovery = enable;
        self
    }

    /// Set the maximum congestion window size in bytes.
    /// Default is 256 KB. For high-latency TURN relays, consider 512KB-1MB.
    pub fn sctp_max_cwnd(mut self, size: usize) -> Self {
        self.inner.sctp_max_cwnd = size;
        self
    }

    pub fn ice_connection_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.inner.ice_connection_timeout = timeout;
        self
    }

    pub fn build(self) -> RtcConfiguration {
        self.inner
    }
}

impl From<RtcConfigurationBuilder> for RtcConfiguration {
    fn from(builder: RtcConfigurationBuilder) -> Self {
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_rtc_configuration_defaults() {
        let config = RtcConfiguration::default();
        assert_eq!(config.ice_connection_timeout, Duration::from_secs(30));
        assert_eq!(config.sctp_rto_initial, Duration::from_secs(3));
        assert_eq!(config.sctp_rto_min, Duration::from_secs(1));
        assert_eq!(config.sctp_rto_max, Duration::from_secs(60));
        assert_eq!(config.sctp_max_association_retransmits, 20);
        assert_eq!(config.sctp_heartbeat_interval, Duration::from_secs(15));
        assert_eq!(config.sctp_max_heartbeat_failures, 4);
        assert!(!config.sctp_diag_enabled);
        assert!(!config.sctp_no_sack_sig_gating);
        assert_eq!(config.sctp_initial_cwnd, 0);
        assert_eq!(config.sctp_slow_start_increase_cap, 0);
        assert_eq!(config.sctp_gap_sack_delay_srtt_divisor, 2);
        assert_eq!(config.sctp_max_burst, 0);
        assert!(config.sctp_unlimited_burst_non_recovery);
        assert_eq!(config.sctp_max_cwnd, 256 * 1024);
    }

    #[test]
    fn test_rtc_configuration_builder() {
        let config = RtcConfigurationBuilder::new()
            .stun_timeout(Duration::from_secs(10))
            .build();
        assert_eq!(config.stun_timeout, Duration::from_secs(10));
        // Verify other defaults are still there
        assert_eq!(config.ice_connection_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_sctp_builder_methods() {
        let config = RtcConfigurationBuilder::new()
            .sctp_rto_initial(Duration::from_millis(500))
            .sctp_rto_min(Duration::from_millis(200))
            .sctp_rto_max(Duration::from_secs(10))
            .sctp_max_association_retransmits(30)
            .sctp_receive_window(512 * 1024)
            .sctp_heartbeat_interval(Duration::from_secs(10))
            .sctp_max_heartbeat_failures(8)
            .sctp_diag_enabled(true)
            .sctp_no_sack_sig_gating(true)
            .sctp_initial_cwnd(24 * 1024)
            .sctp_slow_start_increase_cap(12 * 1024)
            .sctp_gap_sack_delay_srtt_divisor(4)
            .sctp_max_burst(4)
            .sctp_unlimited_burst_non_recovery(false)
            .sctp_max_cwnd(512 * 1024)
            .ice_connection_timeout(Duration::from_secs(60))
            .build();

        assert_eq!(config.sctp_rto_initial, Duration::from_millis(500));
        assert_eq!(config.sctp_rto_min, Duration::from_millis(200));
        assert_eq!(config.sctp_rto_max, Duration::from_secs(10));
        assert_eq!(config.sctp_max_association_retransmits, 30);
        assert_eq!(config.sctp_receive_window, 512 * 1024);
        assert_eq!(config.sctp_heartbeat_interval, Duration::from_secs(10));
        assert_eq!(config.sctp_max_heartbeat_failures, 8);
        assert!(config.sctp_diag_enabled);
        assert!(config.sctp_no_sack_sig_gating);
        assert_eq!(config.sctp_initial_cwnd, 24 * 1024);
        assert_eq!(config.sctp_slow_start_increase_cap, 12 * 1024);
        assert_eq!(config.sctp_gap_sack_delay_srtt_divisor, 4);
        assert_eq!(config.sctp_max_burst, 4);
        assert!(!config.sctp_unlimited_burst_non_recovery);
        assert_eq!(config.sctp_max_cwnd, 512 * 1024);
        assert_eq!(config.ice_connection_timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_turn_optimized_config() {
        // Verify a TURN-optimized configuration can be expressed cleanly
        let config = RtcConfigurationBuilder::new()
            .sctp_rto_initial(Duration::from_millis(500))
            .sctp_rto_min(Duration::from_millis(200))
            .sctp_rto_max(Duration::from_secs(10))
            .sctp_max_association_retransmits(30)
            .sctp_max_heartbeat_failures(8)
            .sctp_max_burst(4)
            .stun_timeout(Duration::from_secs(10))
            .nomination_timeout(Duration::from_secs(20))
            .build();

        // Verify the TURN-optimized values are more aggressive than defaults
        let defaults = RtcConfiguration::default();
        assert!(config.sctp_rto_initial < defaults.sctp_rto_initial);
        assert!(config.sctp_rto_min < defaults.sctp_rto_min);
        assert!(config.sctp_rto_max < defaults.sctp_rto_max);
        assert!(config.sctp_max_association_retransmits > defaults.sctp_max_association_retransmits);
        assert!(config.sctp_max_heartbeat_failures > defaults.sctp_max_heartbeat_failures);
        assert!(config.sctp_max_burst > 0); // Explicit burst limit vs. heuristic
    }
}
