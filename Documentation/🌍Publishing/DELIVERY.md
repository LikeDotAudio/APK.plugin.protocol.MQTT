# Delivery — LIB.MQTT

## Build and test

```sh
cd PODS/POD.THICK/DISCOVERY.Engine/SRC
cargo test  -p apkaudio-mqtt                       # 18 tests + 1 doc test — the one check
```

Checked 2026-10-30: 18 passed + 1 doc test.

## Where it lands

| Destination | How |
|---|---|
| `node-baremetal:local` (container `Node-BareMetal`) | `NODE.BareMetal/Docker/Dockerfile` copies `LIBRARY/LIB.MQTT/SRC` into the build stage; it is compiled as a member of the plugin workspace; oaRustCore no longer depends on it. Its `config.ini` is staged to `/app/APK:DOCKERS/APK:BareMetal/LIB.MQTT/config.ini` and read at run time. |
| the plugin builder image (`DISCOVERY.Engine/Docker/Dockerfile`) | compiled as a workspace member; no plugin binary links it yet (PLUGIN.SQL adoption pending) |
| GitHub | `https://github.com/LikeDotAudio/APK.plugin.protocol.MQTT.git`, a submodule of the monorepo; pushed by `.apk.scripts/deploy_all.py` at the root |

No live URL: a library has no page of its own. Its config is visible on the bus at `APK.audio/System/Protocols/mqtt/config`.

## Not automated

- No CI runs `cargo test` for this crate; the monorepo's `.github/workflows/ci.yml` deploys the web site and verifies nothing, by design. The test is the one command above.
- A change here reaches a node only when an operator rebuilds `Node-BareMetal` (`docker compose -f PODS/POD.THICK/NODE.BareMetal/Docker/docker-compose.yml up -d --build`).

## Delivered 2026-10-31: portable file names (no `:` `|` or case twins)

- The monorepo now checks out on Windows and macOS: no tracked name holds `<>:"|?*`, a trailing dot or space, a device name or a case-only twin. Colon folders became dot folders (the POD.x convention).
- References to the renamed folders updated (`Baremetal.Manager`, `ORCHESTRATOR.hunter.VISA`). Docs/comments only, no behaviour change.

## Delivered from the 2026-10-30 audit

- **Public topic-filter grammar** — `SRC/src/router.rs` → `SRC/src/filter.rs`: `pub fn topic_matches(filter, topic) -> bool` (level-by-level, no regex, total), `validate_filter`, `TopicFilter::new` / `compile_filter` (validated; `FilterError` names the §4.7 rule), `is_valid_topic_name`, re-exported at the crate root in the default build. All seven original edge-case tests kept, plus validation tests and a doc test. PLUGIN.SQL deleting its copy: handoff filed (it needs LIB.MQTT by git, i.e. after the standalone conversion is pushed). Verified `cargo test -p apkaudio-mqtt` (engine workspace) 18 + 1 doc test pass.
- **`[mqtt]` parse and validation owned here** — `SRC/src/broker_config.rs` `BrokerConfig::from_ini` / `bundled()` / `from_payload`, with the "a non-local `host` is refused" rule (`refused_host`) and `is_local_host` (same rule as the orchestrator's `broker_is_local`). Tests `a_public_host_under_host_is_refused`, `junk_falls_back_to_the_safe_defaults`, `local_hosts`, `the_bundled_config_is_local_and_has_no_public_broker`. The orchestrator switching to it: handoff to NODE.BareMetal filed.
- **Versioned payload contract** — `CONFIG_PAYLOAD_VERSION = 1`, `PAYLOAD_KEYS`, `CONFIG_TOPIC` and `SRC/vectors/mqtt_config_payload.json` (version, topic, keys, example payload in the orchestrator's `{value:{mqtt:{…strings}}}` shape); `the_payload_contract_vector_holds` checks the vector, and that the bundled `config.ini` published that way reads back identically and still carries every key the shell reads (`host`, `ws_port`, `tcp_port`, `public_ws_port`, `public_wss_port`, `topic`).
- **The `python` feature's fate decided: retired** — `MqttRouter` / `#[pymodule] oamqttmanager_rs`, and the `pyo3`, `dashmap`, `regex` dependencies, removed (nothing mounted it since oaRustCore dropped it). Reversible: last source at commit `26e1266` (`SRC/src/router.rs`), restore procedure in ARCHITECTURE.md. Engine `Cargo.lock` refresh: handoff to DISCOVERY.Engine.
- **Tests run per change as one command** — no workflow here (the root CI verifies nothing on purpose); the check is `cargo test -p apkaudio-mqtt` in `DISCOVERY.Engine/SRC` (or `cargo test` in `SRC/`, standalone).
- **Builds from its own repository** — `SRC/Cargo.toml` names no workspace, sets `edition = "2024"` and takes `apkaudio-contracts` from APK.plugin.SDK by tag `sdk-v0.5` (the `.apk.scripts/plugin_standalone.py` shape); `.gitignore` keeps the standalone `Cargo.lock` out. The DISCOVERY.Engine workspace reaches it through the link `SRC/members/LIB.MQTT` (landed with this change). Verified: a copy of `SRC/` alone runs `cargo test --offline` green against `sdk-v0.5` (18 tests + 1 doc test); the check is now `cargo test` in `SRC/`.
- **Released `0.2.0`, tag `mqtt-v0.2.0`** — `SRC/Cargo.toml` bumped 0.1.0 → 0.2.0 (public `topic_matches`/`TopicFilter`, `BrokerConfig`, payload contract; python feature retired), annotated tag `mqtt-v0.2.0` published (owner-authorised 2026-10-31) so PLUGIN.SQL can take `apkaudio-mqtt` by tag. Verified: `git archive HEAD` of `SRC/` builds and tests alone against `sdk-v0.6`.
- **SDK repinned to `sdk-v0.7`** (b2b49e1) — `apkaudio-contracts` by `tag = "sdk-v0.7"`. Verified 2026-10-31: `git archive HEAD:SRC` alone, `cargo test` against GitHub — 18 tests + 1 doc test pass.
