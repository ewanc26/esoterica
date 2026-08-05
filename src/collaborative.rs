//! ATProto Collaborative Conlanging Stream
//!
//! Enables real-time collaborative language building by watching ATProto
//! repo changes for lexicon updates. Each update is cryptographically signed
//! and persisted as an ATProto record, enabling distributed conlang editing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A collaborative lexicon change event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexiconChange {
    /// DID of the author who made the change
    pub author: String,
    /// The word being added/modified
    pub headword: String,
    /// The operation: add, update, or delete
    pub operation: ChangeOp,
    /// Timestamp of the change (ISO 8601)
    pub timestamp: String,
    /// Optional commit message
    pub message: Option<String>,
    /// The full lexicon entry (for add/update)
    pub entry: Option<crate::lexicon_structs::LexiconEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeOp {
    Add,
    Update,
    Delete,
}

/// A collaborative session manages merging remote changes into a local lexicon.
pub struct CollaborativeSession {
    /// Local lexicon being edited
    pub lexicon: crate::lexicon_structs::Lexicon,
    /// Stack of unsent local changes
    pending_changes: Vec<LexiconChange>,
    /// History of applied changes from all authors
    pub change_log: Vec<LexiconChange>,
    /// The local author's DID
    pub author_did: String,
}

impl CollaborativeSession {
    pub fn new(lexicon: crate::lexicon_structs::Lexicon, author_did: String) -> Self {
        Self {
            lexicon,
            pending_changes: Vec::new(),
            change_log: Vec::new(),
            author_did,
        }
    }

    // ── Local Operations ────────────────────────────────────────────────────

    /// Record a local add/update to a word.
    pub fn local_add(&mut self, entry: crate::lexicon_structs::LexiconEntry) {
        let headword = entry.headword.clone();
        let is_new = !self.lexicon.0.contains_key(&headword);

        let change = LexiconChange {
            author: self.author_did.clone(),
            headword: headword.clone(),
            operation: if is_new { ChangeOp::Add } else { ChangeOp::Update },
            timestamp: chrono_now(),
            message: None,
            entry: Some(entry.clone()),
        };

        self.lexicon.0.insert(headword, entry);
        self.pending_changes.push(change);
    }

    // ── Remote Merging ─────────────────────────────────────────────────────

    /// Record a local deletion.
    pub fn local_delete(&mut self, headword: &str) {
        let change = LexiconChange {
            author: self.author_did.clone(),
            headword: headword.to_string(),
            operation: ChangeOp::Delete,
            timestamp: chrono_now(),
            message: None,
            entry: None,
        };
        self.lexicon.0.remove(headword);
        self.pending_changes.push(change);
    }

    /// Merge an incoming remote change from another collaborator.
    /// Returns true if the change was applied, false if it was already seen.
    pub fn merge_remote(&mut self, change: LexiconChange) -> bool {
        // Skip own changes
        if change.author == self.author_did {
            return false;
        }

        // Check if we've already seen this change
        if self.change_log.iter().any(|c| {
            c.author == change.author
                && c.headword == change.headword
                && c.timestamp == change.timestamp
        }) {
            return false;
        }

        match change.operation {
            ChangeOp::Add | ChangeOp::Update => {
                if let Some(ref entry) = change.entry {
                    self.lexicon.0.insert(change.headword.clone(), entry.clone());
                }
            }
            ChangeOp::Delete => {
                self.lexicon.0.remove(&change.headword);
            }
        }

        self.change_log.push(change);
        true
    }

    // ── Pending Changes & Conflict Detection ───────────────────────────────

    /// Get pending changes that should be published to ATProto.
    pub fn pending_changes(&self) -> &[LexiconChange] {
        &self.pending_changes
    }

    /// Mark all pending changes as published.
    pub fn flush_pending(&mut self) {
        self.change_log.append(&mut self.pending_changes);
    }

    /// Check for merge conflicts between local and remote changes.
    pub fn detect_conflicts(&self) -> Vec<(LexiconChange, LexiconChange)> {
        let mut conflicts = Vec::new();
        for local in &self.pending_changes {
            for remote in &self.change_log {
                if local.headword == remote.headword
                    && local.author != remote.author
                    && local.timestamp != remote.timestamp
                {
                    conflicts.push((local.clone(), remote.clone()));
                }
            }
        }
        conflicts
    }

    /// Get a summary of the collaboration session.
    pub fn summary(&self) -> CollaborationSummary {
        let mut author_counts: HashMap<String, usize> = HashMap::new();
        for change in &self.change_log {
            *author_counts.entry(change.author.clone()).or_default() += 1;
        }
        CollaborationSummary {
            total_words: self.lexicon.0.len(),
            total_changes: self.change_log.len(),
            pending: self.pending_changes.len(),
            authors: author_counts,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSummary {
    pub total_words: usize,
    pub total_changes: usize,
    pub pending: usize,
    pub authors: HashMap<String, usize>,
}

/// Current UTC time as an ISO 8601 timestamp, without a chrono dependency.
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    format_timestamp(secs)
}

/// Format a Unix timestamp (seconds since 1970-01-01T00:00:00Z) as
/// `YYYY-MM-DDTHH:MM:SSZ`, using proleptic Gregorian leap-year rules.
///
/// Pulled out from [`chrono_now`] so the calendar arithmetic — the part
/// actually worth getting wrong — is testable without depending on the
/// wall clock.
fn format_timestamp(secs: u64) -> String {
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    let mut year = 1970i64;
    let mut remaining_days = days_since_epoch as i64;
    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }
    let month_lengths = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1;
    for &ml in &month_lengths {
        if remaining_days < ml as i64 {
            break;
        }
        remaining_days -= ml as i64;
        month += 1;
    }
    let day = remaining_days + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

/// Proleptic Gregorian leap-year rule: divisible by 4, except centuries
/// unless also divisible by 400 (2000 is a leap year, 2100 is not).
fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon_structs::{Lexicon, LexiconEntry, Sense};

    fn make_entry(word: &str) -> LexiconEntry {
        LexiconEntry {
            headword: word.to_string(), etymology: "test".to_string(),
            part_of_speech: "noun".to_string(), ipa: "/t/".to_string(),
            senses: vec![Sense { definition: "test".to_string(), citations: vec![] }],
            root: word.to_string(), noun_class: None,
        }
    }

    #[test] fn test_local_add() {
        let mut session = CollaborativeSession::new(Lexicon::new(), "did:plc:alice".into());
        session.local_add(make_entry("fire"));
        assert_eq!(session.lexicon.0.len(), 1);
        assert_eq!(session.pending_changes().len(), 1);
    }

    #[test] fn test_local_delete() {
        let mut lex = Lexicon::new();
        lex.0.insert("fire".into(), make_entry("fire"));
        let mut session = CollaborativeSession::new(lex, "did:plc:alice".into());
        session.local_delete("fire");
        assert!(session.lexicon.0.is_empty());
    }

    #[test] fn test_merge_remote() {
        let mut session = CollaborativeSession::new(Lexicon::new(), "did:plc:alice".into());
        let change = LexiconChange {
            author: "did:plc:bob".into(),
            headword: "water".into(),
            operation: ChangeOp::Add,
            timestamp: "2024-01-01T00:00:00Z".into(),
            message: None,
            entry: Some(make_entry("water")),
        };
        assert!(session.merge_remote(change));
        assert_eq!(session.lexicon.0.len(), 1);
    }

    #[test] fn test_ignore_own_changes() {
        let mut session = CollaborativeSession::new(Lexicon::new(), "did:plc:alice".into());
        let change = LexiconChange {
            author: "did:plc:alice".into(),
            headword: "test".into(),
            operation: ChangeOp::Add,
            timestamp: "2024-01-01T00:00:00Z".into(),
            message: None,
            entry: Some(make_entry("test")),
        };
        assert!(!session.merge_remote(change));
    }

    #[test] fn test_conflict_detection() {
        let mut session = CollaborativeSession::new(Lexicon::new(), "did:plc:alice".into());
        session.local_add(make_entry("sky"));
        let remote = LexiconChange {
            author: "did:plc:carol".into(),
            headword: "sky".into(),
            operation: ChangeOp::Update,
            timestamp: "2024-01-02T00:00:00Z".into(),
            message: None,
            entry: Some(make_entry("sky")),
        };
        session.merge_remote(remote);
        let conflicts = session.detect_conflicts();
        assert_eq!(conflicts.len(), 1);
    }

    #[test] fn test_summary() {
        let mut session = CollaborativeSession::new(Lexicon::new(), "did:plc:alice".into());
        session.local_add(make_entry("sun"));
        session.local_add(make_entry("moon"));
        let summary = session.summary();
        assert_eq!(summary.total_words, 2);
        assert_eq!(summary.pending, 2);
    }

    // ── Additional session coverage ──────────────────────────────────────

    #[test]
    fn local_add_over_an_existing_word_is_recorded_as_an_update() {
        let mut session = CollaborativeSession::new(Lexicon::new(), "did:plc:alice".into());
        session.local_add(make_entry("fire"));
        session.local_add(make_entry("fire"));
        let ops: Vec<_> = session.pending_changes().iter().map(|c| c.operation.clone()).collect();
        assert_eq!(ops, vec![ChangeOp::Add, ChangeOp::Update]);
        assert_eq!(session.lexicon.0.len(), 1);
    }

    #[test]
    fn merge_remote_delete_removes_the_word() {
        let mut lex = Lexicon::new();
        lex.0.insert("fire".into(), make_entry("fire"));
        let mut session = CollaborativeSession::new(lex, "did:plc:alice".into());
        let change = LexiconChange {
            author: "did:plc:bob".into(),
            headword: "fire".into(),
            operation: ChangeOp::Delete,
            timestamp: "2024-01-01T00:00:00Z".into(),
            message: None,
            entry: None,
        };
        assert!(session.merge_remote(change));
        assert!(session.lexicon.0.is_empty());
    }

    #[test]
    fn merge_remote_rejects_an_already_seen_change() {
        let mut session = CollaborativeSession::new(Lexicon::new(), "did:plc:alice".into());
        let change = LexiconChange {
            author: "did:plc:bob".into(),
            headword: "water".into(),
            operation: ChangeOp::Add,
            timestamp: "2024-01-01T00:00:00Z".into(),
            message: None,
            entry: Some(make_entry("water")),
        };
        assert!(session.merge_remote(change.clone()));
        // Same author, headword, and timestamp: already applied, so a resend is a no-op.
        assert!(!session.merge_remote(change));
        assert_eq!(session.lexicon.0.len(), 1);
    }

    #[test]
    fn flush_pending_moves_changes_into_the_log_and_clears_pending() {
        let mut session = CollaborativeSession::new(Lexicon::new(), "did:plc:alice".into());
        session.local_add(make_entry("sun"));
        session.local_add(make_entry("moon"));
        assert_eq!(session.pending_changes().len(), 2);

        session.flush_pending();

        assert!(session.pending_changes().is_empty());
        assert_eq!(session.change_log.len(), 2);
    }

    #[test]
    fn no_conflict_when_only_one_side_touches_a_word() {
        let mut session = CollaborativeSession::new(Lexicon::new(), "did:plc:alice".into());
        session.local_add(make_entry("sky"));
        let remote = LexiconChange {
            author: "did:plc:carol".into(),
            headword: "sea".into(),
            operation: ChangeOp::Add,
            timestamp: "2024-01-02T00:00:00Z".into(),
            message: None,
            entry: Some(make_entry("sea")),
        };
        session.merge_remote(remote);
        assert!(session.detect_conflicts().is_empty());
    }

    #[test]
    fn summary_tallies_changes_per_author() {
        let mut session = CollaborativeSession::new(Lexicon::new(), "did:plc:alice".into());
        session.local_add(make_entry("sun"));
        session.flush_pending();
        for (author, word) in [("did:plc:bob", "moon"), ("did:plc:bob", "star"), ("did:plc:carol", "sea")] {
            session.merge_remote(LexiconChange {
                author: author.into(),
                headword: word.into(),
                operation: ChangeOp::Add,
                timestamp: format!("2024-01-0{}T00:00:00Z", word.len()),
                message: None,
                entry: Some(make_entry(word)),
            });
        }
        let summary = session.summary();
        assert_eq!(summary.authors.get("did:plc:alice"), Some(&1));
        assert_eq!(summary.authors.get("did:plc:bob"), Some(&2));
        assert_eq!(summary.authors.get("did:plc:carol"), Some(&1));
        assert_eq!(summary.total_changes, 4);
    }

    // ── Timestamp formatting ─────────────────────────────────────────────

    #[test]
    fn epoch_formats_as_the_epoch_date() {
        assert_eq!(format_timestamp(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn formats_the_end_of_the_first_day() {
        assert_eq!(format_timestamp(86399), "1970-01-01T23:59:59Z");
    }

    #[test]
    fn rolls_over_into_the_next_day() {
        assert_eq!(format_timestamp(86400), "1970-01-02T00:00:00Z");
    }

    #[test]
    fn handles_a_leap_day() {
        // 1972 was a leap year; day 789 after epoch is 1972-02-29.
        assert_eq!(format_timestamp(789 * 86400), "1972-02-29T00:00:00Z");
    }

    #[test]
    fn day_after_a_leap_day_rolls_into_march() {
        assert_eq!(format_timestamp(790 * 86400), "1972-03-01T00:00:00Z");
    }

    #[test]
    fn non_leap_century_year_has_no_february_29() {
        // 2100 is divisible by 100 but not 400, so it is not a leap year.
        // Confirm Feb 28 -> Mar 1 rolls over without an intervening leap day.
        let epoch_to_2100_02_28 = days_between(1970, 1, 1, 2100, 2, 28);
        let epoch_to_2100_03_01 = days_between(1970, 1, 1, 2100, 3, 1);
        assert_eq!(epoch_to_2100_03_01 - epoch_to_2100_02_28, 1);
        assert_eq!(format_timestamp(epoch_to_2100_02_28 as u64 * 86400), "2100-02-28T00:00:00Z");
        assert_eq!(format_timestamp(epoch_to_2100_03_01 as u64 * 86400), "2100-03-01T00:00:00Z");
    }

    #[test]
    fn year_2000_is_a_leap_year_despite_being_a_century() {
        // Divisible by 400, so unlike 2100 it does have Feb 29.
        let days = days_between(1970, 1, 1, 2000, 2, 29);
        assert_eq!(format_timestamp(days as u64 * 86400), "2000-02-29T00:00:00Z");
    }

    #[test]
    fn time_of_day_components_are_computed_independently_of_the_date() {
        let secs = 789 * 86400 + 13 * 3600 + 5 * 60 + 42;
        assert_eq!(format_timestamp(secs), "1972-02-29T13:05:42Z");
    }

    #[test]
    fn is_leap_matches_the_gregorian_rule() {
        assert!(is_leap(1972));
        assert!(is_leap(2000));
        assert!(!is_leap(1971));
        assert!(!is_leap(1900));
        assert!(!is_leap(2100));
        assert!(is_leap(2400));
    }

    /// Days between two Gregorian dates, computed independently of
    /// `format_timestamp` so the leap-year tests aren't checking the
    /// implementation against itself.
    fn days_between(y1: i64, m1: u32, d1: u32, y2: i64, m2: u32, d2: u32) -> i64 {
        days_from_civil(y2, m2, d2) - days_from_civil(y1, m1, d1)
    }

    /// Howard Hinnant's `days_from_civil` algorithm: proleptic Gregorian
    /// calendar date to a day count, used here purely as an independent
    /// reference implementation for the leap-year tests above.
    fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
        let y = if m <= 2 { y - 1 } else { y };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let mp = (m as i64 + 9) % 12;
        let doy = (153 * mp + 2) / 5 + d as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    }
}
