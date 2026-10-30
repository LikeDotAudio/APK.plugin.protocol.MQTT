# APK:lib:MQTT

The MQTT bus and broker configuration, and the MQTT topic-filter grammar, as a Rust library.

| | |
|---|---|
| **Recipe** | none — a library, not a container: it is linked into the binaries that use it |
| **Source** | `SRC/` (this repository — https://github.com/LikeDotAudio/APK.plugin.protocol.MQTT.git) — crate `apkaudio-mqtt` (opt-in feature `python`; nothing enables it) |
| **Used by** | no crate depends on it; oaRustCore no longer mounts `mqtt_router`. Its `config.ini` is read by the orchestrator and, off the bus, by the shell; a member of the plugin workspace |
| **Workspace** | `PODS/POD.THICK/DISCOVERY.Engine/SRC/Cargo.toml` (shared `target/` at the repository root) |
| **Family / Category** | LIBRARY — bus library; links no MQTT client, opens no socket, binds no port |

Was `APK:BareMetal/PLUGINS/PLUGIN:protocol:MQTT`. The brokers themselves are `PODS/POD.databus/DATABUS.Broker.MQTT`.


**Wiki:** [Documentation/README.md](Documentation/README.md) — principles, architecture and delivery.

## What it does

| File | What is in it |
|---|---|
| `SRC/src/router.rs` | the MQTT 3.1.1 §4.7 filter grammar (`#` includes its parent level, `+` is one level and may be empty, everything else literal) compiled to an anchored regex; `MqttRouter` (feature `python`) — `subscribe(filter, callback)`, `unsubscribe(filter)`, `match_topic(topic)` → the callbacks that want it, `clear()`. No connection, no payload decoding |
| `SRC/src/self_test.rs` | the self test, module name `mqtt` |
| `SRC/config.ini` | `[mqtt]` — the broker keys below, and `topic` |

## The config is the broker decision

`SRC/config.ini` is where the bench's two brokers and the order between them are written down; its comments carry the history (PLAN-35.03, PLAN-35.04) and should be read before changing a key.

| Key | Read by | Meaning |
|---|---|---|
| `host`, `tcp_port` | orchestrator, `ORCHESTRATOR:hunter:VISA/src/broker/select.rs` (`endpoints_from_ini`) | the broker this bench owns; a public address here is refused out loud |
| `public_host`, `public_tcp_port` | the same | the broker to publish to first — **withdrawn** (commented out) since 2026-08-29, so the orchestrator stays local unless `--public-host` is typed |
| `ws_port`, `public_ws_port`, `public_wss_port` | the shell, `POD.THIN/OS/DataBus/config/bus-config.js`, from the retained payload | the websocket ports the browser connects to |
| `topic` | orchestrator, `ORCHESTRATOR:hunter:VISA/src/mqtt.rs` | where this file is published |
| `enabled` | nothing found | rides along in the payload |

The orchestrator publishes it retained at `APK.audio/System/Protocols/mqtt/config` (with `/status` = `online` and, on a broker the bench owns, `/host`).

## Build and test

```sh
cd PODS/POD.THICK/DISCOVERY.Engine/SRC
cargo test -p apkaudio-mqtt                     # 11 tests: 7 on the filter grammar, 4 on the self test
cargo build -p apkaudio-mqtt --features python  # the PyO3 half
```

Checked 2026-10-30: `cargo test -p apkaudio-mqtt` — 11 passed.

## Philosophy

- **Owns the grammar and nothing else.** The router holds no connection and never decodes a payload; it answers which callbacks want a topic that has already arrived (`SRC/src/router.rs`, header).
- **The spec, not the obvious regex.** `#` matching its parent and an empty `+` level are written as tests, and regex metacharacters in a filter are literals (`SRC/src/router.rs`, `mod tests`).
- **A public broker is a decision with a name on it.** Nothing local may quietly become public: `public_host` is a separate key, withdrawn by default, and the orchestrator says which broker it chose on every boot (`SRC/config.ini`; enforced in the orchestrator's `broker/select.rs` and `broker/choice.rs`).
- **Write the reader first.** No `topic_listen` / `topic_ignore` / `topic_publish` keys, because the subscribe topics come from Python as arguments, never from this file (`SRC/config.ini`, closing comment).
- **Only what needs the interpreter is gated.** The grammar compiles in the default feature set; `MqttRouter` and the `#[pymodule]` sit behind `python` (`SRC/src/lib.rs`, `SRC/Cargo.toml`).
- **Bake in, don't look up.** The self test reads `config.ini` through `include_str!` and checks the section still names the module (`SRC/src/self_test.rs`).

---

*© 2026 — Written & Maintained by Anthony P. Kuzub. MIT Licence.*

<!-- wiki:smoke -->
**Smoke test** — ✅ PASS 1/1 since 2026-10-30 12:51 EDT — [results](Documentation/%F0%9F%A7%A9Manual%20and%20Artifacts/SMOKE.md)
<!-- /wiki:smoke -->
