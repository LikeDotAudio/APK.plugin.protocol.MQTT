// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! `apkaudio-mqtt` (crate name; its folder is `LIBRARY/LIB.MQTT/SRC`) — the
//! bus's two shared rules, as plain Rust:
//!
//! * [`filter`] — the MQTT 3.1.1 §4.7 topic-filter grammar: [`topic_matches`],
//!   [`TopicFilter`], [`validate_filter`]. The one copy for every Rust crate
//!   that matches topics locally (PLUGIN.SQL's `capture_ignore`, …).
//! * [`broker_config`] — the `[mqtt]` section of this crate's `config.ini`,
//!   typed and validated ([`BrokerConfig::from_ini`], the "public address
//!   under `host` is refused" rule, [`is_local_host`]), and the versioned
//!   contract of the retained `APK.audio/System/Protocols/mqtt/config` payload.
//!
//! No MQTT client, no socket, no runtime. The `python` feature (a PyO3
//! `MqttRouter` that nothing had mounted since oaRustCore stopped) was retired
//! on 2026-10-30; its last source is in this repository's history at commit
//! `26e1266` (`SRC/src/router.rs`) should a Python host ever want it back.

pub mod broker_config;
pub mod filter;

pub use broker_config::{is_local_host, BrokerConfig, CONFIG_PAYLOAD_VERSION, CONFIG_TOPIC};
pub use filter::{compile_filter, is_valid_topic_name, topic_matches, validate_filter, FilterError, TopicFilter};

/// This crate's own self test — what it can prove about itself with nothing
/// plugged in. Run by `cargo test`; nothing answers `outgoing/selftest_run`
/// for it on the bus since PLAN-3316.01 (see `self_test.rs`). The report shape
/// is `apkaudio_contracts::self_test`'s. PLAN-459.01.
pub mod self_test;
