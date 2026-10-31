# 🔍 Architecture audit — LIB.MQTT (2026-10-30)

Audited against the APK.audio multi-repo architecture standard: coupling, boundaries, duplication, contracts.
← [Audits](INDEX.md) · Improvements: [🛣Roadmap/IMPROVEMENTS.md](../%F0%9F%9B%A3Roadmap/IMPROVEMENTS.md)

## Bounded context
The MQTT 3.1.1 §4.7 topic-filter grammar plus the bench's broker decision (`[mqtt]` in `config.ini`). Rust crate `apkaudio-mqtt`, 3 source files / 477 lines, optional PyO3 shim behind feature `python`. Sampled: everything tracked (`SRC/Cargo.toml`, `SRC/config.ini`, `SRC/src/*.rs`, README, SMOKE).

## Context map
| Talks to | How (topic / endpoint / file / import) | Relationship |
|---|---|---|
| NODE.BareMetal contracts (`apkaudio-contracts`) | Cargo path dep `../../../NODE.BareMetal/SRC/contracts/rust` | Conformist (self-test report shape) |
| DISCOVERY.Engine workspace | `workspace = "../../../DISCOVERY.Engine/SRC"` | Shared build (workspace member living in another repo) |
| Orchestrator (`ORCHESTRATOR:hunter:VISA`, in NODE.BareMetal) | reads this repo's `SRC/config.ini` from disk, publishes it retained at `APK.audio/System/Protocols/mqtt/config` | Supplier of config file (file-level coupling) |
| Shell (`POD.THIN/OS/DataBus/config/bus-config.js`, `ledger/bus-ledger.js`) | the retained `.../mqtt/config` payload | Published Language (bus) |
| PLUGIN.SQL and ~46 crates that link `rumqttc` | none — they do not depend on this crate | Should be Shared Kernel; today unused |

## Scorecard
| Category | Score (0–3) | Finding |
|---|---|---|
| Coupling & Cohesion | 2 | The config's parse and validation now live here (`BrokerConfig`); still builds only inside the engine workspace with contracts by relative path (standalone patch waiting on the engine) |
| Boundary Integrity | 3 | Grammar and broker config are public, typed Rust (`topic_matches`, `TopicFilter`, `BrokerConfig::from_ini`); no socket, no PyO3 |
| Duplication Control | 2 | `topic_matches` and `is_local_host` are public; PLUGIN.SQL and the orchestrator still carry their copies until they adopt (handoffs filed) |
| Testing & Contracts | 3 | 18 tests + 1 doc test incl. a versioned payload-contract vector; one command (`cargo test -p apkaudio-mqtt`) |
| Change management | 2 | The retained payload is versioned (`CONFIG_PAYLOAD_VERSION`, vector file); the crate itself is still 0.1.0, untagged |
Score key: 3 meets the standard · 2 minor gaps · 1 significant gaps · 0 absent / violated.

## Findings
1. **Cross-repo path coupling in the manifest (open — waiting on DISCOVERY.Engine).** [`SRC/Cargo.toml`](../../SRC/Cargo.toml) line 2 makes the crate a member of `../../../DISCOVERY.Engine/SRC` and line 8 pulls `apkaudio-contracts` from `../../../NODE.BareMetal/SRC/contracts/rust`. The standalone manifest is ready and verified but must land with the engine's `members/LIB.MQTT` link.
2. **Good: the security decision is written down and defaulted safe.** `public_host` is commented out (`config.ini:65`) with the PLAN-35.04 incident evidence beside it; `host = 127.0.0.1` (`config.ini:35`); `BrokerConfig::from_ini` refuses a non-local `host` (`SRC/src/broker_config.rs`).
3. **Good: the grammar is correct and pinned.** `SRC/src/filter.rs` tests cover parent-inclusive `#`, empty `+` levels and regex metacharacters as literals.
4. **Self test bakes config in.** [`SRC/src/self_test.rs`](../../SRC/src/self_test.rs) uses `include_str!("../config.ini")` and asserts the section name matches `MODULE = "mqtt"`.
