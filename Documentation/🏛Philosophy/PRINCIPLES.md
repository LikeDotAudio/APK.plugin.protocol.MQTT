# Principles — LIB.MQTT

The principles `apkaudio-mqtt` keeps, each pointed at the code that keeps it. The ecosystem rules these come from are in the monorepo's [Bare Metal golden rules](../../../../../../Documentation/%F0%9F%8F%9BPhilosophy/Bare%20Metal/goldenrule.md) and [Doctrine](../../../../../../Documentation/%F0%9F%8F%9BPhilosophy/Doctrine/readme.Doctrine.md); they are cited here, not copied.

## Kept

1. **Owns the grammar and nothing else.** The router holds no connection, opens no socket and never decodes a payload; it answers which callbacks want a topic that has already arrived. — `SRC/src/router.rs` module header (lines 3–8); the only state is two maps in `MqttRouter` (`router.rs:38–51`).
2. **The spec, not the obvious regex.** MQTT 3.1.1 §4.7: `#` includes its parent level, `+` is one level and may be empty, every other character is literal. — `mqtt_filter_to_regex` (`router.rs:123`), which escapes each literal segment with `regex::escape`; pinned by seven tests in `router.rs` `mod tests` (from line 167).
3. **Only what needs the interpreter is gated.** The grammar compiles in the default feature set so `cargo test` and the self test can ask it questions; `MqttRouter` and the `#[pymodule]` sit behind `python`. — `SRC/src/lib.rs:5–11`, `SRC/Cargo.toml` (`regex` not optional; `python = [pyo3, dashmap]`).
4. **Write the reader first** (golden rule 1). `config.ini` carries no `topic_listen` / `topic_ignore` / `topic_publish`, because subscribe topics come from Python as arguments. — `SRC/config.ini:75` onward. The keys it does carry each have a named reader: `host`/`tcp_port` → `endpoints_from_ini` (`NODE.BareMetal/SRC/Core/ORCHESTRATOR:hunter:VISA/src/broker/select.rs:25`); `topic` → `publish_protocol_configs` (`…/src/mqtt.rs:61`).
5. **A public broker is a decision with a name on it.** `public_host` is a separate key from `host` and ships commented out, so a bare run publishes only on the bench's own broker. — `SRC/config.ini:16–31` and `:65`; enforced in the orchestrator's `broker/select.rs`.
6. **Bake in, don't look up** (golden rule 6). The self test reads `config.ini` through `include_str!`, so it answers the same from any working directory. — `SRC/src/self_test.rs:41`; the section check is `config_declares_module` (`self_test.rs:97`).
7. **The self test asks the hard question.** It does not just call the grammar; it checks the one-level/two-level distinction and the metacharacter case an earlier version got wrong. — `self_test_filter` (`router.rs:252`).

## Where the code falls short

- **A reader that nobody calls.** The PyO3 surface is not mounted: oaRustCore dropped it, because no Python under `PODS/` imported it (`NODE.BareMetal/SRC/Core/CORE:engine:RUST/lib.rs`, the "NOT MOUNTED" comment), and no Cargo.toml in the tree enables the `python` feature. A Python caller re-adds the dependency with `features = ["python"]` and its `add_submodule!` in the same commit as the import.
- **`enabled`** in `config.ini` is read by the protocol census (`NODE.BareMetal/SRC/Baremetal:Manager/protocol_census.py`) for display only; nothing switches on it.
