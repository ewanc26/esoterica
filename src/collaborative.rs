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

    /// Get pending changes that should be published to ATProto.
    pub fn pending_changes(&self) -> &[LexiconChange] {
        &self.pending_changes
    }

    /// Mark all pending changes as published.
    pub fn flush_pending(&mut self) {
        self.change_log.extend(self.pending_changes.drain(..));
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

fn chrono_now() -> String {
    // Simple ISO 8601 timestamp without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Format: YYYY-MM-DDTHH:MM:SSZ (basic approximation)
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Approximate date calculation from Unix epoch
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

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon_structs::{Lexicon, LexiconEntry, Sense, Citation};

    fn make_entry(word: &str) -> LexiconEntry {
        LexiconEntry {
            headword: word.to_string(), etymology: "test".to_string(),
            part_of_speech: "noun".to_string(), ipa: "/t/".to_string(),
            senses: vec![Sense { definition: "test".to_string(), citations: vec![] }],
            root: word.to_string(), noun_class: None,
        }
    }

    #[test] fn test_local_add() {
        let mut session = CollaborativeSession::new(Lexicon(HashMap::new()), "did:plc:alice".into());
        session.local_add(make_entry("fire"));
        assert_eq!(session.lexicon.0.len(), 1);
        assert_eq!(session.pending_changes().len(), 1);
    }

    #[test] fn test_local_delete() {
        let mut lex = Lexicon(HashMap::new());
        lex.0.insert("fire".into(), make_entry("fire"));
        let mut session = CollaborativeSession::new(lex, "did:plc:alice".into());
        session.local_delete("fire");
        assert!(session.lexicon.0.is_empty());
    }

    #[test] fn test_merge_remote() {
        let mut session = CollaborativeSession::new(Lexicon(HashMap::new()), "did:plc:alice".into());
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
        let mut session = CollaborativeSession::new(Lexicon(HashMap::new()), "did:plc:alice".into());
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
        let mut session = CollaborativeSession::new(Lexicon(HashMap::new()), "did:plc:alice".into());
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
        let mut session = CollaborativeSession::new(Lexicon(HashMap::new()), "did:plc:alice".into());
        session.local_add(make_entry("sun"));
        session.local_add(make_entry("moon"));
        let summary = session.summary();
        assert_eq!(summary.total_words, 2);
        assert_eq!(summary.pending, 2);
    }
}
