// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! `[mqtt]` in `config.ini`, read and validated HERE — the crate that owns the
//! file — instead of by its reader in another repository.
//!
//! Until the 2026-10-30 audit the orchestrator (`NODE.BareMetal/…/broker/select.rs`
//! `endpoints_from_ini`) parsed this repo's file itself and owned the rule
//! that matters most in it: **a non-local address under `host` is refused**
//! (PLAN-35.03 — that key once said `test.mosquitto.org` and put the bench's
//! retained state on a public server). Both the parse and that rule now live
//! in [`BrokerConfig::from_ini`]; a caller gets typed fields plus
//! [`BrokerConfig::refused_host`] to say out loud what was refused.
//!
//! It also pins the CONTRACT of the retained payload the orchestrator
//! publishes at `APK.audio/System/Protocols/mqtt/config` and the shell
//! (`POD.THIN/OS/DataBus/config/bus-config.js`) reads: the ini wrapped as
//! `{"value": {"mqtt": {key: "string", …}}, "path": …, "source": "backend"}`.
//! [`CONFIG_PAYLOAD_VERSION`] and `vectors/mqtt_config_payload.json` are that
//! contract; [`BrokerConfig::from_payload`] reads it the way the shell does.

use std::net::IpAddr;

/// The payload shape's version. Bump it — and the vector file — when a key the
/// shell or the orchestrator reads is renamed, removed or changes meaning.
pub const CONFIG_PAYLOAD_VERSION: u32 = 1;

/// The retained topic the orchestrator publishes this file at.
pub const CONFIG_TOPIC: &str = "APK.audio/System/Protocols/mqtt/config";

/// The keys a reader of the payload may rely on, as of [`CONFIG_PAYLOAD_VERSION`].
pub const PAYLOAD_KEYS: [&str; 9] = [
    "enabled",
    "host",
    "tcp_port",
    "ws_port",
    "public_host",
    "public_tcp_port",
    "public_ws_port",
    "public_wss_port",
    "topic",
];

/// This crate's own `config.ini`, compiled in.
pub const BUNDLED_INI: &str = include_str!("../config.ini");

pub const LOOPBACK: &str = "127.0.0.1";
pub const DEFAULT_TCP_PORT: u16 = 1883;
pub const DEFAULT_WS_PORT: u16 = 9001;
pub const DEFAULT_PUBLIC_WS_PORT: u16 = 8080;
pub const DEFAULT_PUBLIC_WSS_PORT: u16 = 8081;

/// The `[mqtt]` section, typed and validated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrokerConfig {
    pub enabled: bool,
    /// The broker THIS BENCH OWNS — always a local address (see [`is_local_host`]).
    pub host: String,
    pub tcp_port: u16,
    pub ws_port: u16,
    /// A non-local `host` the file named and this parse REFUSED. Callers must
    /// say so out loud; it is never adopted.
    pub refused_host: Option<String>,
    /// The broker to publish to first, if declared (withdrawn by default).
    pub public_host: Option<String>,
    pub public_tcp_port: u16,
    pub public_ws_port: u16,
    pub public_wss_port: u16,
    /// Where the orchestrator publishes this file (`<topic>/config`).
    pub topic: String,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        BrokerConfig {
            enabled: true,
            host: LOOPBACK.to_string(),
            tcp_port: DEFAULT_TCP_PORT,
            ws_port: DEFAULT_WS_PORT,
            refused_host: None,
            public_host: None,
            public_tcp_port: DEFAULT_TCP_PORT,
            public_ws_port: DEFAULT_PUBLIC_WS_PORT,
            public_wss_port: DEFAULT_PUBLIC_WSS_PORT,
            topic: "APK.audio/System/Protocols/mqtt".to_string(),
        }
    }
}

impl BrokerConfig {
    /// The config this crate ships (`SRC/config.ini`).
    pub fn bundled() -> Self {
        Self::from_ini(BUNDLED_INI)
    }

    /// Parse the `[mqtt]` section of an ini text. Missing keys, unparsable
    /// ports and a missing section all fall back to the loopback defaults —
    /// the safe broker — and a non-local `host` is refused into
    /// [`BrokerConfig::refused_host`].
    pub fn from_ini(text: &str) -> Self {
        Self::from_pairs(section_pairs(text, "mqtt"))
    }

    /// Read the retained payload the way the shell does: `value.mqtt`, else
    /// `value`, else the document itself; values may be strings or numbers.
    pub fn from_payload(json: &str) -> Option<Self> {
        let doc: serde_json::Value = serde_json::from_str(json).ok()?;
        let value = doc.get("value").filter(|v| v.is_object()).unwrap_or(&doc);
        let section = value.get("mqtt").filter(|v| v.is_object()).unwrap_or(value);
        let pairs = section
            .as_object()?
            .iter()
            .map(|(k, v)| {
                let text = match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                (k.clone(), text)
            })
            .collect();
        Some(Self::from_pairs(pairs))
    }

    fn from_pairs(pairs: Vec<(String, String)>) -> Self {
        let mut c = BrokerConfig::default();
        let get = |k: &str| {
            pairs
                .iter()
                .rev()
                .find(|(key, _)| key.eq_ignore_ascii_case(k))
                .map(|(_, v)| v.trim())
                .filter(|v| !v.is_empty())
        };
        let port = |k: &str, fallback: u16| {
            get(k).and_then(|p| p.parse::<u16>().ok()).filter(|p| *p != 0).unwrap_or(fallback)
        };
        if let Some(e) = get("enabled") {
            c.enabled = !matches!(e.to_ascii_lowercase().as_str(), "false" | "0" | "no" | "off");
        }
        if let Some(host) = get("host") {
            if is_local_host(host) {
                c.host = host.to_string();
            } else {
                c.refused_host = Some(host.to_string());
            }
        }
        c.tcp_port = port("tcp_port", DEFAULT_TCP_PORT);
        c.ws_port = port("ws_port", DEFAULT_WS_PORT);
        c.public_host = get("public_host").map(str::to_string);
        c.public_tcp_port = port("public_tcp_port", DEFAULT_TCP_PORT);
        c.public_ws_port = port("public_ws_port", DEFAULT_PUBLIC_WS_PORT);
        c.public_wss_port = port("public_wss_port", DEFAULT_PUBLIC_WSS_PORT);
        if let Some(t) = get("topic") {
            c.topic = t.to_string();
        }
        c
    }
}

/// Is this broker address one the bench itself owns?
///
/// Loopback, RFC1918, link-local, IPv6 ULA, `localhost`, the compose service
/// name `broker`, and `.local` / `.localhost` names. A hostname that is none of
/// these is a broker nobody here can vouch for. The same rule as the
/// orchestrator's `mqtt.rs` `broker_is_local`, which should call this one.
pub fn is_local_host(host: &str) -> bool {
    let host = host.trim().trim_start_matches('[').trim_end_matches(']');
    if host.eq_ignore_ascii_case("localhost")
        || host.eq_ignore_ascii_case("broker")
        || host.eq_ignore_ascii_case("localhost.localdomain")
        || host.ends_with(".local")
        || host.ends_with(".localhost")
    {
        return true;
    }
    match host.parse::<IpAddr>() {
        Ok(IpAddr::V4(v4)) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
        Ok(IpAddr::V6(v6)) => {
            v6.is_loopback()
                || (v6.segments()[0] & 0xffc0) == 0xfe80
                || (v6.segments()[0] & 0xfe00) == 0xfc00
        }
        Err(_) => false,
    }
}

/// `key = value` pairs of one `[section]`, comments (`#`, `;`) and blank lines
/// skipped. Section names compare case-insensitively.
fn section_pairs(text: &str, wanted: &str) -> Vec<(String, String)> {
    let mut inside = false;
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            inside = name.trim().eq_ignore_ascii_case(wanted);
            continue;
        }
        if inside {
            if let Some((k, v)) = line.split_once('=') {
                out.push((k.trim().to_string(), v.trim().to_string()));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const VECTOR: &str = include_str!("../vectors/mqtt_config_payload.json");

    #[test]
    fn the_bundled_config_is_local_and_has_no_public_broker() {
        let c = BrokerConfig::bundled();
        assert_eq!(c.host, "127.0.0.1");
        assert_eq!((c.tcp_port, c.ws_port), (1883, 9001));
        assert_eq!(c.refused_host, None);
        // Withdrawn 2026-08-29 (PLAN-35.04): commented out, so None.
        assert_eq!(c.public_host, None);
        assert_eq!((c.public_ws_port, c.public_wss_port), (8080, 8081));
        assert_eq!(c.topic, "APK.audio/System/Protocols/mqtt");
        assert_eq!(format!("{}/config", c.topic), CONFIG_TOPIC);
    }

    /// PLAN-35.03: a public address under `host` is refused, not adopted.
    #[test]
    fn a_public_host_under_host_is_refused() {
        let c = BrokerConfig::from_ini("[mqtt]\nhost = test.mosquitto.org\ntcp_port = 1884\n");
        assert_eq!(c.host, "127.0.0.1");
        assert_eq!(c.refused_host.as_deref(), Some("test.mosquitto.org"));
        assert_eq!(c.tcp_port, 1884);
        let c = BrokerConfig::from_ini("[MQTT]\nhost = broker\n");
        assert_eq!((c.host.as_str(), c.refused_host), ("broker", None));
    }

    #[test]
    fn junk_falls_back_to_the_safe_defaults() {
        let c = BrokerConfig::from_ini("[other]\nhost = 10.0.0.1\n[mqtt]\ntcp_port = banana\nws_port = 0\nenabled = false\n");
        assert_eq!(c.host, "127.0.0.1", "another section's key leaked");
        assert_eq!((c.tcp_port, c.ws_port, c.enabled), (1883, 9001, false));
        assert_eq!(BrokerConfig::from_ini(""), BrokerConfig::default());
    }

    #[test]
    fn local_hosts() {
        for h in ["127.0.0.1", "10.1.2.3", "192.168.0.9", "172.16.0.1", "169.254.1.1", "::1", "[fe80::1]", "fd00::5", "localhost", "broker", "bench.local"] {
            assert!(is_local_host(h), "{h}");
        }
        for h in ["test.mosquitto.org", "8.8.8.8", "44.44.44.1", "2001:db8::1", ""] {
            assert!(!is_local_host(h), "{h}");
        }
    }

    /// The consumer-driven contract: the payload the orchestrator publishes
    /// and the shell reads, pinned by a checked-in vector.
    #[test]
    fn the_payload_contract_vector_holds() {
        let v: serde_json::Value = serde_json::from_str(VECTOR).expect("vector parses");
        assert_eq!(v["version"], CONFIG_PAYLOAD_VERSION, "bump the vector with the constant");
        assert_eq!(v["topic"], CONFIG_TOPIC);
        let keys: Vec<&str> = v["keys"].as_array().unwrap().iter().map(|k| k.as_str().unwrap()).collect();
        assert_eq!(keys, PAYLOAD_KEYS);

        let example = v["example"].to_string();
        let c = BrokerConfig::from_payload(&example).expect("example reads");
        assert_eq!((c.ws_port, c.public_ws_port, c.public_wss_port), (9001, 8080, 8081));
        assert_eq!(c.host, "127.0.0.1");

        // What the orchestrator would publish from the bundled ini — every
        // value a string, under value.mqtt — reads back to the same config,
        // and every key the shell reads is present in it.
        let mut section = serde_json::Map::new();
        for (k, val) in section_pairs(BUNDLED_INI, "mqtt") {
            section.insert(k, serde_json::Value::String(val));
        }
        for k in ["host", "ws_port", "tcp_port", "public_ws_port", "public_wss_port", "topic"] {
            assert!(section.contains_key(k), "config.ini lost `{k}`, which the payload contract promises");
        }
        let published = serde_json::json!({"value": {"mqtt": section}, "path": "x/config.ini", "source": "backend"});
        assert_eq!(BrokerConfig::from_payload(&published.to_string()), Some(BrokerConfig::bundled()));
    }
}
