# APK:lib:MQTT

The MQTT bus and broker configuration, as a Rust library.

| | |
|---|---|
| **Recipe** | none — a library, not a container: it is linked into the binaries that use it |
| **Source** | `SRC/` (this repository — https://github.com/LikeDotAudio/APK.plugin.protocol.MQTT.git) — `apkaudio-mqtt` (feature `python` for oaRustCore) |
| **Used by** | the orchestrator and oaRustCore; a member of the plugin workspace |
| **Workspace** | `APK:plugins:Build/SRC/Cargo.toml` |

Its `config.ini` (`[mqtt]`) is where the two broker keys and the order between them are written down. Was `APK:BareMetal/PLUGINS/PLUGIN:protocol:MQTT`. The brokers themselves are `POD:databus/DATABUS:Broker:MQTT`.

---

*© 2026 — Written & Maintained by Anthony P. Kuzub. MIT Licence.*
