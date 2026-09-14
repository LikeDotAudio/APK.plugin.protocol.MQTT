// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! `apkaudio-mqtt` (crate name; its folder is `Protocol-mqtt`) — MQTT manager helpers.
//!
//! **This crate is a PyO3 extension shim around a pure grammar.** The topic
//! filter grammar in `router` compiles in the default feature set, because
//! `self_test` asks it questions and `self_test` is never gated; only the
//! `MqttRouter` delivery type and its `#[pymodule]` sit behind the `python`
//! feature, which is NOT enabled by default. Build with `--features python` to
//! get the Python-facing half. Until PLAN-645.01 the whole module was gated and
//! this crate could not compile in its own default configuration at all.
//!
//! (The `cargo new` template `add()` that used to sit here has been removed: a
//! function asserting 2 + 2 as the crate's public entry point misrepresented the
//! crate as unimplemented.)
//! ONE lint is allowed crate-wide and it is not this crate's code:
//! `unsafe_op_in_unsafe_fn` fires eight times inside pyo3's `#[pymethods]`
//! expansion in `router`. Named rather than blanketed, for the reason
//! `Protocol-aes70/src/lib.rs` gives at length: a crate-wide `allow` list is
//! how an unused import and four elided lifetimes hid there for a year.
#![cfg_attr(feature = "python", allow(unsafe_op_in_unsafe_fn))]

pub mod router;

/// This crate's own self test — what it can prove about itself with nothing
/// plugged in. Reached from the bus through `outgoing/selftest_run`; see
/// `apkaudio_contracts::self_test` for the two inputs and the withdrawal the
/// first of them performs. PLAN-459.01.
pub mod self_test;
