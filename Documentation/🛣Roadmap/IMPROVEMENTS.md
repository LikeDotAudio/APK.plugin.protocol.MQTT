# 🛣 Improvements — LIB.MQTT (from the 2026-10-30 architecture audit)

What this repo should change to meet the standard, most valuable first. Source: [🔍Audits/ARCHITECTURE-AUDIT.md](../%F0%9F%94%8DAudits/ARCHITECTURE-AUDIT.md).
← [Roadmap](INDEX.md)

Delivered items are recorded in [🌍Publishing/DELIVERY.md](../%F0%9F%8C%8DPublishing/DELIVERY.md).

| # | Improvement | Category | Effort (S/M/L) | Evidence |
|---|---|---|---|---|
| 1 | Consume `apkaudio-contracts` as a versioned dependency (SDK tag `sdk-v0.5`) instead of `path = "../../../NODE.BareMetal/..."`, and give the crate a standalone build. **Waiting on DISCOVERY.Engine (handoff filed):** cargo refuses a member that is neither below the workspace root nor names it with `package.workspace`, so dropping `workspace =` here must land together with the engine's `members/LIB.MQTT` link; the LIB side is ready as a patch (verified: a `git archive` copy builds and tests 18+1 against `sdk-v0.5` with nothing around it) | Coupling & Cohesion | M | `SRC/Cargo.toml` lines 2 and 8 |
