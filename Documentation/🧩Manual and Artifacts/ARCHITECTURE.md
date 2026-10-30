# Architecture — LIB.MQTT

`apkaudio-mqtt` is a Rust library (`SRC/`) in the plugin workspace `PODS/POD.THICK/DISCOVERY.Engine/SRC/Cargo.toml`. It is not a container and has no `main`.

## Using it

From Rust (default features):

```rust
// the grammar is crate-private; the public surface is the self test
let report = apkaudio_mqtt::self_test::self_test(apkaudio_contracts::self_test::Depth::Own);
let json   = apkaudio_mqtt::self_test::self_test_json("self");
```

From Python, through oaRustCore (built with the `python` feature):

```python
import oaRustCore
r = oaRustCore.mqtt_router.MqttRouter()      # also reachable as oaMQTTManager_rs
r.subscribe("APK.audio/+/status", on_status)  # callback is any Python callable
r.match_topic("APK.audio/node1/status")       # -> [on_status]
r.unsubscribe("APK.audio/+/status"); r.clear()
```

## Inside

| File | Role |
|---|---|
| `SRC/src/lib.rs` | declares `router` (always) and `self_test`; allows one pyo3 lint only under `python` |
| `SRC/src/router.rs` | `mqtt_filter_to_regex(filter)` → an anchored regex string; `MqttRouter` (feature `python`): exact filters in one `DashMap<String, Vec<callback>>`, wildcard filters compiled once into a second `DashMap<String, (Regex, Vec<callback>)>`; `match_topic` returns exact hits then every wildcard whose regex matches; `unsubscribe(filter)` drops every callback on that filter; `#[pymodule] oamqttmanager_rs` |
| `SRC/src/self_test.rs` | `MODULE = "mqtt"`; checks: landing spots build, `config.ini` declares `[mqtt]`, the wildcard filter rule; `self_test_json` is the string-only door across the Core/plugin workspace boundary |
| `SRC/config.ini` | `[mqtt]` — the bench's broker decision: `host`, `tcp_port`, `ws_port`, the withdrawn `public_host`, `public_*_port`, `topic` |

## Data flow of `config.ini`

1. The NODE.BareMetal image build stages every crate `config.ini` it finds under `PODS/POD.THICK` into `/app/APK:DOCKERS/APK:BareMetal/LIB.MQTT/config.ini` (`NODE.BareMetal/Docker/Dockerfile`, the `find … -name config.ini` step).
2. The orchestrator reads it twice: `broker/select.rs` `endpoints_from_ini` picks the broker the node connects to; `mqtt.rs` `publish_protocol_configs` publishes it retained at `APK.audio/System/Protocols/mqtt/config` (with `/status`, `/host`).
3. The shell reads the websocket ports off that retained payload (`POD.THIN/OS/DataBus/config/bus-config.js`).

## Self test

Run by `cargo test`. Since PLAN-3316.01 nothing on the bus answers `outgoing/selftest_run` for the libraries (`NODE.BareMetal/SRC/Core/ORCHESTRATOR:hunter:VISA/src/main.rs`, "NO BUS SELF-TEST LISTENER HERE").
