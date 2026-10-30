// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! Native MQTT subscription routing: a topic in, the callbacks that want it out.
//!
//! Owns the TOPIC FILTER GRAMMAR and nothing else. It holds no connection,
//! speaks to no broker, and never decodes a payload -- callers hand it a topic
//! that has already arrived and it answers with the `Py<PyAny>` callbacks
//! registered against filters that match.
//!
//! The grammar is MQTT 3.1.1 §4.7 and it is NOT the obvious regex:
//!
//!   * `#` is multi-level AND INCLUDES THE PARENT. `sport/tennis/#` matches
//!     `sport/tennis` itself, not only its children (§4.7.1.2).
//!   * `+` is exactly one level and that level MAY BE EMPTY, so `sport/+`
//!     matches `sport/` but not `sport` (§4.7.1.3).
//!   * every other character is a literal, including the regex metacharacters
//!     an MQTT topic name is allowed to contain.
//!
//! `$`-prefixed topics are deliberately not special-cased: §4.7.2 makes that
//! the BROKER's rule about which subscriptions it will serve, and nothing that
//! is not addressed to this application reaches this router in the first place.
//!
//! Exposed to Python as `oaRustCore.mqtt_router` -- registered by string in
//! `Core/CORE:engine:RUST/lib.rs`, so `MqttRouter` and the `#[pymodule]` name below
//! are a public API and may not be renamed to suit Rust taste.

// The grammar below is plain Rust; only the delivery type around it needs
// an interpreter. PLAN-645.01.
#[cfg(feature = "python")]
use dashmap::DashMap;
#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(any(feature = "python", test))]
use regex::Regex;
#[cfg(feature = "python")]
use std::sync::Arc;

#[cfg(feature = "python")]
#[pyclass]
struct MqttRouter {
    // Exact matches: topic -> list of callbacks
    exact_matches: Arc<DashMap<String, Vec<Py<PyAny>>>>,
    // Wildcard matches: filter -> (compiled matcher, list of callbacks).
    //
    // COMPILED ONCE, at subscribe. It used to hold the pattern as a String and
    // rebuild a Regex per filter per delivered message -- in the one type whose
    // whole reason for existing is high-frequency distribution -- and swallowed
    // a compile error as "does not match", which is the failure a subscriber
    // can neither see nor be told about.
    wildcard_matches: Arc<DashMap<String, (Regex, Vec<Py<PyAny>>)>>,
}

#[cfg(feature = "python")]
#[pymethods]
impl MqttRouter {
    #[new]
    fn new() -> Self {
        MqttRouter {
            exact_matches: Arc::new(DashMap::new()),
            wildcard_matches: Arc::new(DashMap::new()),
        }
    }

    fn subscribe(&self, filter: String, callback: Py<PyAny>) {
        if filter.contains('+') || filter.contains('#') {
            let mut entry = self.wildcard_matches.entry(filter.clone()).or_insert_with(|| {
                // Infallible by construction: `mqtt_filter_to_regex` escapes
                // every literal segment and emits only its own metacharacters,
                // which is what `a_filter_full_of_regex_metacharacters_is_a_literal`
                // pins. A panic here would mean a bug in that function.
                let pattern = mqtt_filter_to_regex(&filter);
                let regex = Regex::new(&pattern)
                    .expect("mqtt_filter_to_regex emits an escaped, valid pattern");
                (regex, Vec::new())
            });
            entry.1.push(callback);
        } else {
            let mut entry = self.exact_matches.entry(filter).or_insert_with(Vec::new);
            entry.push(callback);
        }
    }

    fn unsubscribe(&self, filter: String) {
        self.exact_matches.remove(&filter);
        self.wildcard_matches.remove(&filter);
    }

    fn match_topic(&self, py: Python<'_>, topic: String) -> Vec<Py<PyAny>> {
        let mut matches = Vec::new();

        // 1. Exact match
        if let Some(callbacks) = self.exact_matches.get(&topic) {
            for cb in callbacks.iter() {
                matches.push(cb.clone_ref(py));
            }
        }

        // 2. Wildcard matches
        for r in self.wildcard_matches.iter() {
            let (regex, callbacks) = r.value();
            if regex.is_match(&topic) {
                for cb in callbacks.iter() {
                    matches.push(cb.clone_ref(py));
                }
            }
        }

        matches
    }

    fn clear(&self) {
        self.exact_matches.clear();
        self.wildcard_matches.clear();
    }
}

/// One MQTT topic filter as an anchored regular expression.
///
/// Segment by segment rather than by string substitution. The substitution
/// version escaped `.` and nothing else, so any other regex metacharacter in a
/// topic name -- MQTT allows them -- either changed the meaning of the filter
/// or failed to compile, and a failed compile was read as "matched nothing".
fn mqtt_filter_to_regex(filter: &str) -> String {
    let segments: Vec<&str> = filter.split('/').collect();
    let mut regex = String::from("^");

    for (i, segment) in segments.iter().enumerate() {
        let last = i + 1 == segments.len();
        match *segment {
            // The parent level too: `a/#` matches `a`. The `/` already written
            // for the preceding segment is taken back and made optional.
            "#" => {
                if i == 0 {
                    regex.push_str(".*");
                } else {
                    regex.pop();
                    regex.push_str("(?:/.*)?");
                }
            }
            // Exactly one level, possibly zero characters wide.
            "+" => {
                regex.push_str("[^/]*");
                if !last {
                    regex.push('/');
                }
            }
            literal => {
                regex.push_str(&regex::escape(literal));
                if !last {
                    regex.push('/');
                }
            }
        }
    }

    regex.push('$');
    regex
}

#[cfg(feature = "python")]
#[pymodule]
pub fn oamqttmanager_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<MqttRouter>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Does this filter accept this topic? The whole of what this crate decides.
    fn matches(filter: &str, topic: &str) -> bool {
        Regex::new(&mqtt_filter_to_regex(filter))
            .expect("every filter compiles")
            .is_match(topic)
    }

    #[test]
    fn a_filter_with_no_wildcard_matches_only_itself() {
        assert!(matches("APK.audio/System/Protocols/mqtt/config", "APK.audio/System/Protocols/mqtt/config"));
        assert!(!matches("APK.audio/System/Protocols/mqtt/config", "APK.audio/System/Protocols/mqtt/status"));
        // The `.` in `APK.audio` is a literal. Escaping it was the ONE thing
        // the substitution version got right, and it is worth keeping pinned.
        assert!(!matches("APK.audio/System", "APKxaudio/System"));
    }

    #[test]
    fn a_hash_matches_the_level_it_is_attached_to() {
        // MQTT 3.1.1 §4.7.1.2, verbatim: "sport/tennis/# also matches the
        // singular sport/tennis". The substitution version emitted
        // `^sport/tennis/.*$` and missed exactly this case -- so a subscriber
        // to `<topic>/#` never saw the retained message ON `<topic>`.
        assert!(matches("sport/tennis/#", "sport/tennis"));
        assert!(matches("sport/tennis/#", "sport/tennis/player1"));
        assert!(matches("sport/tennis/#", "sport/tennis/player1/ranking"));
        assert!(!matches("sport/tennis/#", "sport/tennisball"));
        assert!(!matches("sport/tennis/#", "sport"));
    }

    #[test]
    fn a_bare_hash_matches_every_topic() {
        assert!(matches("#", "a"));
        assert!(matches("#", "a/b/c"));
        assert!(matches("#", ""));
    }

    #[test]
    fn a_plus_is_one_level_and_may_be_empty() {
        // §4.7.1.3: "sport/+ does not match sport but it does match sport/".
        assert!(matches("sport/+", "sport/"));
        assert!(!matches("sport/+", "sport"));
        assert!(matches("sport/+/player1", "sport/tennis/player1"));
        assert!(!matches("sport/+/player1", "sport/tennis/mixed/player1"));
        assert!(!matches("sport/+", "sport/tennis/player1"));
    }

    #[test]
    fn plus_and_hash_compose() {
        assert!(matches("APK.audio/System/Protocols/+/#", "APK.audio/System/Protocols/mqtt"));
        assert!(matches("APK.audio/System/Protocols/+/#", "APK.audio/System/Protocols/aes70/config"));
        assert!(!matches("APK.audio/System/Protocols/+/#", "APK.audio/System/Protocols"));
    }

    #[test]
    fn a_filter_full_of_regex_metacharacters_is_a_literal() {
        // An MQTT topic name may contain any UTF-8 but `+`, `#` and `/`. The
        // substitution version turned `a(b` into the invalid pattern `^a(b$`,
        // `Regex::new` returned Err, and `matches_mqtt` reported `false` --
        // a subscription that silently received nothing, for ever.
        assert!(matches("a(b)c/#", "a(b)c/d"));
        assert!(matches("v1.0[beta]/+", "v1.0[beta]/x"));
        assert!(matches("cost$/#", "cost$"));
        assert!(!matches("a(b)c/#", "abc/d"));
        // `*` is a literal, not "the preceding thing, repeated".
        assert!(!matches("ab*/#", "abbb/x"));
        assert!(matches("ab*/#", "ab*/x"));
    }

    #[test]
    fn the_anchors_hold_at_both_ends() {
        assert!(!matches("tennis", "sport/tennis"));
        assert!(!matches("sport", "sport/tennis"));
    }
}

/// The wildcard rule, asserted, for `crate::self_test`.
///
/// `mqtt_filter_to_regex` is where this crate is easiest to get subtly wrong --
/// its own doc comment records a substitution version that read a failed
/// compile as "matched nothing" -- so the self test asks it the questions that
/// distinguish the two wildcards rather than merely calling it.
pub(crate) fn self_test_filter() -> Result<(), String> {
    let cases: [(&str, &str, bool); 6] = [
        ("a/+/c", "a/b/c", true),
        ("a/+/c", "a/b/x/c", false),
        ("a/#", "a/b/c/d", true),
        ("a/#", "a", true),
        ("a/#", "b/c", false),
        // A topic level carrying a regex metacharacter is a topic level, not a
        // pattern. This is the case the substitution version got wrong.
        ("a/b.c", "a/bXc", false),
    ];
    for (filter, topic, want) in cases {
        let pattern = mqtt_filter_to_regex(filter);
        let compiled = regex::Regex::new(&pattern)
            .map_err(|e| format!("filter {filter:?} compiled to {pattern:?}: {e}"))?;
        if compiled.is_match(topic) != want {
            return Err(format!(
                "filter {filter:?} {} {topic:?} and should {}",
                if want { "refused" } else { "matched" },
                if want { "have matched" } else { "not have" }
            ));
        }
    }
    Ok(())
}
