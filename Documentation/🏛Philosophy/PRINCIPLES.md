# Principles — LIB.MQTT

The principles `apkaudio-mqtt` keeps, each pointed at the code that keeps it. The ecosystem rules these come from are in the monorepo's [Bare Metal golden rules](../../../../../../Documentation/%F0%9F%8F%9BPhilosophy/Bare%20Metal/goldenrule.md) and [Doctrine](../../../../../../Documentation/%F0%9F%8F%9BPhilosophy/Doctrine/readme.Doctrine.md); they are cited here, not copied.

## Kept

1. **Owns the bus's shared rules and nothing else.** No connection, no socket, no payload decoding beyond its own config. — `SRC/src/filter.rs`, `SRC/src/broker_config.rs` headers.
2. **The spec, not the obvious regex.** MQTT 3.1.1 §4.7: `#` includes its parent level, `+` is one level and may be empty, every other character is literal. — `topic_matches` (`filter.rs`), level by level; pinned by nine tests in `filter.rs` `mod tests`.
3. **One copy of each rule, public.** `topic_matches`/`TopicFilter` and `BrokerConfig::from_ini`/`is_local_host` are the crate's public API so consumers delete their copies (PLUGIN.SQL's `topic_matches`, the orchestrator's `endpoints_from_ini`/`broker_is_local` — handoffs filed 2026-10-30).
4. **Write the reader first** (golden rule 1). `config.ini` carries no `topic_listen` / `topic_ignore` / `topic_publish`, because subscribe topics come from Python as arguments. — `SRC/config.ini:75` onward. The keys it does carry each have a named reader: `host`/`tcp_port` → `BrokerConfig::from_ini` here, today still also `endpoints_from_ini` (`NODE.BareMetal/SRC/Core/ORCHESTRATOR:hunter:VISA/src/broker/select.rs:25`); `topic` → `publish_protocol_configs` (`…/src/mqtt.rs:61`).
5. **A public broker is a decision with a name on it.** `public_host` is a separate key from `host` and ships commented out, so a bare run publishes only on the bench's own broker. — `SRC/config.ini:16–31` and `:65`; enforced in the orchestrator's `broker/select.rs`.
6. **Bake in, don't look up** (golden rule 6). The self test reads `config.ini` through `include_str!`, so it answers the same from any working directory. — `SRC/src/self_test.rs:41`; the section check is `config_declares_module` (`self_test.rs:97`).
7. **The self test asks the hard question.** It does not just call the grammar; it checks the one-level/two-level distinction and the metacharacter case an earlier version got wrong. — `self_test_filter` (`filter.rs`).
8. **The payload is a contract.** The retained `…/mqtt/config` shape is versioned (`CONFIG_PAYLOAD_VERSION`) and pinned by a checked-in vector the test reads. — `broker_config.rs`, `SRC/vectors/mqtt_config_payload.json`.

## Where the code falls short

- **Two readers until the orchestrator adopts this one.** `NODE.BareMetal`'s `broker/select.rs` still parses the file itself (handoff filed).
- **`enabled`** in `config.ini` is read by the protocol census (`NODE.BareMetal/SRC/Baremetal:Manager/protocol_census.py`) for display only; nothing switches on it.
