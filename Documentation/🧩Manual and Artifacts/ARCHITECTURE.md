# Architecture — LIB.MQTT

`apkaudio-mqtt` is a Rust library (`SRC/`) in the plugin workspace `PODS/POD.THICK/DISCOVERY.Engine/SRC/Cargo.toml`. It is not a container and has no `main`.

## Using it

From Rust (default features — there are no others since 2026-10-30):

```rust
use apkaudio_mqtt::{topic_matches, TopicFilter, BrokerConfig, is_local_host};

topic_matches("APK.audio/System/+/#", "APK.audio/System/Protocols");   // true — §4.7 grammar
let f = TopicFilter::new("APK.audio/Gui/#")?;   // validated once (Err(FilterError) names the rule broken)
f.matches("APK.audio/Gui");                      // true: `#` includes its parent

let c = BrokerConfig::from_ini(&std::fs::read_to_string(path)?);  // or BrokerConfig::bundled()
c.host; c.tcp_port; c.ws_port; c.public_host; c.public_ws_port; c.public_wss_port; c.topic;
if let Some(bad) = &c.refused_host { /* say it out loud: a non-local `host` is never adopted */ }
is_local_host("192.168.1.4");                    // the "a bench-owned broker" rule
BrokerConfig::from_payload(retained_json);      // read the retained .../mqtt/config the way the shell does

let report = apkaudio_mqtt::self_test::self_test(apkaudio_contracts::self_test::Depth::Own);
```

The `python` feature (`MqttRouter`, `#[pymodule] oamqttmanager_rs`) was retired on 2026-10-30: nothing had mounted it since oaRustCore dropped it. Its last source is `SRC/src/router.rs` at commit `26e1266`; a Python host that wants it restores that file and the `pyo3`/`dashmap` feature in the same commit as its import.

## Inside

| File | Role |
|---|---|
| `SRC/src/lib.rs` | declares `filter`, `broker_config`, `self_test`; re-exports the public API |
| `SRC/src/filter.rs` | `topic_matches(filter, topic)` (level-by-level, total), `validate_filter`, `TopicFilter` (validated; `matches`, `is_wildcard`, `FromStr`), `compile_filter`, `is_valid_topic_name`, `FilterError` |
| `SRC/src/broker_config.rs` | `BrokerConfig::{from_ini, bundled, from_payload}`, `is_local_host`, `CONFIG_TOPIC`, `CONFIG_PAYLOAD_VERSION`, `PAYLOAD_KEYS` |
| `SRC/vectors/mqtt_config_payload.json` | the payload contract: version, topic, keys, an example payload — pinned by `the_payload_contract_vector_holds` |
| `SRC/src/self_test.rs` | `MODULE = "mqtt"`; checks: landing spots build, `config.ini` declares `[mqtt]`, the wildcard filter rule; `self_test_json` is the string-only door across the Core/plugin workspace boundary |
| `SRC/config.ini` | `[mqtt]` — the bench's broker decision: `host`, `tcp_port`, `ws_port`, the withdrawn `public_host`, `public_*_port`, `topic` |

## Data flow of `config.ini`

1. The NODE.BareMetal image build stages every crate `config.ini` it finds under `PODS/POD.THICK` into `/app/APK:DOCKERS/APK:BareMetal/LIB.MQTT/config.ini` (`NODE.BareMetal/Docker/Dockerfile`, the `find … -name config.ini` step).
2. The orchestrator reads it twice: `broker/select.rs` `endpoints_from_ini` picks the broker the node connects to (to be replaced by `BrokerConfig::from_ini` from this crate — handoff to NODE.BareMetal filed 2026-10-30); `mqtt.rs` `publish_protocol_configs` publishes it retained at `APK.audio/System/Protocols/mqtt/config` (with `/status`, `/host`).
3. The shell reads the websocket ports off that retained payload (`POD.THIN/OS/DataBus/config/bus-config.js`). The payload's shape is versioned here: `CONFIG_PAYLOAD_VERSION` = 1 and `SRC/vectors/mqtt_config_payload.json`.

## Self test

Run by `cargo test`. Since PLAN-3316.01 nothing on the bus answers `outgoing/selftest_run` for the libraries (`NODE.BareMetal/SRC/Core/ORCHESTRATOR:hunter:VISA/src/main.rs`, "NO BUS SELF-TEST LISTENER HERE").
