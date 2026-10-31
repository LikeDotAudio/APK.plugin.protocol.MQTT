// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! The MQTT topic-filter grammar: does this subscription want this topic?
//!
//! THE one copy in the Rust tree (2026-10-30 audit: PLUGIN.SQL carried its
//! own; this crate's was private). It holds no connection, speaks to no broker
//! and never decodes a payload.
//!
//! The grammar is MQTT 3.1.1 §4.7 and it is NOT the obvious regex:
//!
//!   * `#` is multi-level AND INCLUDES THE PARENT. `sport/tennis/#` matches
//!     `sport/tennis` itself, not only its children (§4.7.1.2).
//!   * `+` is exactly one level and that level MAY BE EMPTY, so `sport/+`
//!     matches `sport/` but not `sport` (§4.7.1.3).
//!   * every other character is a literal, including regex metacharacters an
//!     MQTT topic name is allowed to contain.
//!
//! `$`-prefixed topics are deliberately not special-cased: §4.7.2 makes that
//! the BROKER's rule about which subscriptions it will serve.
//!
//! Level-by-level comparison, no regex: it was a regex until 2026-10-30, which
//! cost a dependency and a compile step per filter for what is a zip over
//! `split('/')`. The edge cases the regex version was fixed for are the tests
//! below, unchanged.
//!
//! ```
//! use apkaudio_mqtt::{topic_matches, TopicFilter};
//! assert!(topic_matches("APK.audio/System/+/#", "APK.audio/System/Protocols"));
//! let f = TopicFilter::new("sport/tennis/#").unwrap();   // validate once, match many
//! assert!(f.matches("sport/tennis"));
//! assert!(TopicFilter::new("a/#/b").is_err());           // `#` must be last
//! ```

/// Why a string is not a legal topic filter (MQTT 3.1.1 §4.7.1, §4.7.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterError {
    /// A filter is at least one character long.
    Empty,
    /// `#` anywhere but as the whole of the last level.
    MisplacedHash,
    /// `+` sharing a level with other characters.
    MisplacedPlus,
    /// U+0000 is forbidden in topic names and filters.
    Nul,
}

impl std::fmt::Display for FilterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            FilterError::Empty => "a topic filter cannot be empty",
            FilterError::MisplacedHash => "`#` must be the whole of the last level",
            FilterError::MisplacedPlus => "`+` must be a whole level",
            FilterError::Nul => "a topic filter cannot contain U+0000",
        })
    }
}

impl std::error::Error for FilterError {}

/// Check a filter against §4.7 without matching anything.
pub fn validate_filter(filter: &str) -> Result<(), FilterError> {
    if filter.is_empty() {
        return Err(FilterError::Empty);
    }
    if filter.contains('\0') {
        return Err(FilterError::Nul);
    }
    let levels: Vec<&str> = filter.split('/').collect();
    for (i, level) in levels.iter().enumerate() {
        if level.contains('#') && (*level != "#" || i + 1 != levels.len()) {
            return Err(FilterError::MisplacedHash);
        }
        if level.contains('+') && *level != "+" {
            return Err(FilterError::MisplacedPlus);
        }
    }
    Ok(())
}

/// Is `topic` a legal topic NAME (what a publisher may publish to)? Non-empty,
/// no wildcards, no U+0000.
pub fn is_valid_topic_name(topic: &str) -> bool {
    !topic.is_empty() && !topic.contains(['+', '#', '\0'])
}

/// Does `filter` accept `topic`? The whole of what this crate decides.
///
/// Total: an invalid filter is matched level by level as written (a `#` that
/// is not last only matches a literal `#` level), so it never panics; use
/// [`TopicFilter::new`] / [`validate_filter`] where a bad filter must be
/// refused rather than tolerated.
pub fn topic_matches(filter: &str, topic: &str) -> bool {
    let mut f = filter.split('/').peekable();
    let mut t = topic.split('/');
    loop {
        match (f.next(), t.next()) {
            // `#` as the last level takes this level, every level after it,
            // AND the parent: `a/#` matches `a` (the topic ran out right here).
            (Some("#"), _) if f.peek().is_none() => return true,
            (Some(_), None) => return false,
            (None, Some(_)) => return false,
            (None, None) => return true,
            (Some("+"), Some(_)) => {}
            (Some(level), Some(got)) => {
                if level != got {
                    return false;
                }
            }
        }
    }
}

/// A filter checked once against §4.7, for a subscription matched many times.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicFilter(String);

impl TopicFilter {
    /// Validate and keep. `Err` names the rule the filter breaks.
    pub fn new(filter: impl Into<String>) -> Result<Self, FilterError> {
        let filter = filter.into();
        validate_filter(&filter)?;
        Ok(TopicFilter(filter))
    }

    /// Does this subscription want `topic`?
    pub fn matches(&self, topic: &str) -> bool {
        topic_matches(&self.0, topic)
    }

    /// Whether the filter has any wildcard (an exact filter can be a map key).
    pub fn is_wildcard(&self) -> bool {
        self.0.contains(['+', '#'])
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TopicFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for TopicFilter {
    type Err = FilterError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TopicFilter::new(s)
    }
}

/// [`TopicFilter::new`], by the name the audit asked for.
pub fn compile_filter(filter: &str) -> Result<TopicFilter, FilterError> {
    TopicFilter::new(filter)
}

#[cfg(test)]
mod tests {
    use super::topic_matches as matches;
    use super::*;

    #[test]
    fn a_filter_with_no_wildcard_matches_only_itself() {
        assert!(matches("APK.audio/System/Protocols/mqtt/config", "APK.audio/System/Protocols/mqtt/config"));
        assert!(!matches("APK.audio/System/Protocols/mqtt/config", "APK.audio/System/Protocols/mqtt/status"));
        // The `.` in `APK.audio` is a literal.
        assert!(!matches("APK.audio/System", "APKxaudio/System"));
    }

    #[test]
    fn a_hash_matches_the_level_it_is_attached_to() {
        // MQTT 3.1.1 §4.7.1.2, verbatim: "sport/tennis/# also matches the
        // singular sport/tennis" — a subscriber to `<topic>/#` must see the
        // retained message ON `<topic>`.
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
        assert!(matches("#", "/leading"));
    }

    #[test]
    fn a_plus_is_one_level_and_may_be_empty() {
        // §4.7.1.3: "sport/+ does not match sport but it does match sport/".
        assert!(matches("sport/+", "sport/"));
        assert!(!matches("sport/+", "sport"));
        assert!(matches("sport/+/player1", "sport/tennis/player1"));
        assert!(!matches("sport/+/player1", "sport/tennis/mixed/player1"));
        assert!(!matches("sport/+", "sport/tennis/player1"));
        assert!(matches("+/+", "/finance"));
        assert!(matches("/+", "/finance"));
        assert!(!matches("+", "/finance"));
    }

    #[test]
    fn plus_and_hash_compose() {
        assert!(matches("APK.audio/System/Protocols/+/#", "APK.audio/System/Protocols/mqtt"));
        assert!(matches("APK.audio/System/Protocols/+/#", "APK.audio/System/Protocols/aes70/config"));
        assert!(!matches("APK.audio/System/Protocols/+/#", "APK.audio/System/Protocols"));
    }

    #[test]
    fn a_filter_full_of_regex_metacharacters_is_a_literal() {
        assert!(matches("a(b)c/#", "a(b)c/d"));
        assert!(matches("v1.0[beta]/+", "v1.0[beta]/x"));
        assert!(matches("cost$/#", "cost$"));
        assert!(!matches("a(b)c/#", "abc/d"));
        assert!(!matches("ab*/#", "abbb/x"));
        assert!(matches("ab*/#", "ab*/x"));
    }

    #[test]
    fn the_anchors_hold_at_both_ends() {
        assert!(!matches("tennis", "sport/tennis"));
        assert!(!matches("sport", "sport/tennis"));
        assert!(!matches("sport/tennis", "sport/tennis/"));
    }

    #[test]
    fn invalid_filters_are_refused_by_the_typed_door() {
        assert_eq!(TopicFilter::new(""), Err(FilterError::Empty));
        assert_eq!(TopicFilter::new("a/#/b"), Err(FilterError::MisplacedHash));
        assert_eq!(TopicFilter::new("a/b#"), Err(FilterError::MisplacedHash));
        assert_eq!(TopicFilter::new("a/b+/c"), Err(FilterError::MisplacedPlus));
        assert_eq!(TopicFilter::new("a\0b"), Err(FilterError::Nul));
        let f: TopicFilter = "APK.audio/+/x/#".parse().unwrap();
        assert!(f.is_wildcard() && f.matches("APK.audio/q/x"));
        assert!(!TopicFilter::new("a/b").unwrap().is_wildcard());
        assert!(compile_filter("#").unwrap().matches("anything/at/all"));
        // The total function tolerates what the typed door refuses.
        assert!(!matches("a/#/b", "a/x/b"));
        assert!(matches("a/#/b", "a/#/b"));
    }

    #[test]
    fn topic_names_carry_no_wildcards() {
        assert!(is_valid_topic_name("APK.audio/System"));
        assert!(!is_valid_topic_name("APK.audio/+"));
        assert!(!is_valid_topic_name(""));
    }
}

/// The wildcard rule, asserted, for `crate::self_test`: the questions that
/// distinguish the two wildcards, rather than merely calling the matcher.
pub(crate) fn self_test_filter() -> Result<(), String> {
    let cases: [(&str, &str, bool); 6] = [
        ("a/+/c", "a/b/c", true),
        ("a/+/c", "a/b/x/c", false),
        ("a/#", "a/b/c/d", true),
        ("a/#", "a", true),
        ("a/#", "b/c", false),
        // A topic level carrying a regex metacharacter is a topic level, not a
        // pattern.
        ("a/b.c", "a/bXc", false),
    ];
    for (filter, topic, want) in cases {
        if topic_matches(filter, topic) != want {
            return Err(format!(
                "filter {filter:?} {} {topic:?} and should {}",
                if want { "refused" } else { "matched" },
                if want { "have matched" } else { "not have" }
            ));
        }
    }
    Ok(())
}
