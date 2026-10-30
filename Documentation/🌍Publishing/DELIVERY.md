# Delivery — LIB.MQTT

## Build and test

```sh
cd PODS/POD.THICK/DISCOVERY.Engine/SRC
cargo test  -p apkaudio-mqtt                       # 11 tests (7 grammar, 4 self test)
cargo build -p apkaudio-mqtt --features python     # the PyO3 half
```

Checked 2026-10-30: 11 passed; the `python` build finishes.

## Where it lands

| Destination | How |
|---|---|
| `node-baremetal:local` (container `Node-BareMetal`) | `NODE.BareMetal/Docker/Dockerfile` copies `LIBRARY/LIB.MQTT/SRC` into the build stage; it is compiled as a member of the plugin workspace; oaRustCore no longer depends on it. Its `config.ini` is staged to `/app/APK:DOCKERS/APK:BareMetal/LIB.MQTT/config.ini` and read at run time. |
| the plugin builder image (`DISCOVERY.Engine/Docker/Dockerfile`) | compiled as a workspace member without `python`; no plugin binary uses it |
| GitHub | `https://github.com/LikeDotAudio/APK.plugin.protocol.MQTT.git`, a submodule of the monorepo; pushed by `.apk.scripts/deploy_all.py` at the root |

No live URL: a library has no page of its own. Its config is visible on the bus at `APK.audio/System/Protocols/mqtt/config`.

## Not automated

- No CI runs `cargo test` for this crate; the monorepo's `.github/workflows/ci.yml` deploys the web site and verifies nothing, by design. The test is an operator's command.
- A change here reaches a node only when an operator rebuilds `Node-BareMetal` (`docker compose -f PODS/POD.THICK/NODE.BareMetal/Docker/docker-compose.yml up -d --build`).
