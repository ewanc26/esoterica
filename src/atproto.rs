//! ATProto publishing bindings for the Esoterica conlang generator.
//! Publishes generated lexicons as `site.standard.document` records and
//! manages `site.standard.publication` containers.

use anyhow::{anyhow, Context, Result};
use atrium_api::types::string::AtIdentifier;
use atrium_api::types::string::Datetime;
use atrium_api::types::Unknown;
use serde_json::{json, Value};
use bsky_sdk::BskyAgent;

/// Wraps a BskyAgent session for publishing conlang data to the AT Proto network.
pub struct AtprotoPublisher { agent: BskyAgent }

impl AtprotoPublisher {
    pub fn new(agent: BskyAgent) -> Self { Self { agent } }

    /// List every existing publication in the user's repo, following cursors
    /// past the first page.
    pub async fn list_publications(&self) -> Result<Vec<(String, String)>> {
        let session = self.agent.get_session().await.context("Not logged in")?;
        let mut pubs = Vec::new();
        let mut cursor = None;

        loop {
            let output = self.agent.api.com.atproto.repo.list_records(
                atrium_api::com::atproto::repo::list_records::ParametersData {
                    collection: "site.standard.publication".parse().map_err(|e| anyhow!("{}", e))?,
                    repo: AtIdentifier::Did(session.did.clone()),
                    cursor: cursor.clone(),
                    limit: Some(100.try_into().map_err(|e| anyhow!("{}", e))?),
                    reverse: None,
                }.into()
            ).await?;

            let page_len = output.data.records.len();
            for record in &output.data.records {
                let val = serde_json::to_value(&record.value)?;
                if let Some(name) = val.get("name").and_then(|n| n.as_str()) {
                    pubs.push((name.to_string(), record.uri.clone()));
                }
            }

            cursor = output.data.cursor.clone();
            // No cursor, or an empty page, means there is nothing more to fetch.
            if cursor.is_none() || page_len == 0 {
                break;
            }
        }

        Ok(pubs)
    }

    pub async fn publish_publication(&self, name: &str, url: &str) -> Result<String> {
        let record = build_publication_record(name, url, Datetime::now().as_str());
        let session = self.agent.get_session().await.context("Not logged in")?;
        let output = self.agent.api.com.atproto.repo.create_record(
            atrium_api::com::atproto::repo::create_record::InputData {
                collection: "site.standard.publication".parse().map_err(|e| anyhow!("{}", e))?,
                repo: AtIdentifier::Did(session.did.clone()),
                rkey: None,
                record: serde_json::from_value::<Unknown>(record)?,
                swap_commit: None,
                validate: None,
            }.into()
        ).await?;
        Ok(output.data.uri)
    }

    pub async fn publish_dictionary(
        &self,
        lexicon: &std::collections::BTreeMap<String, crate::lexicon_structs::LexiconEntry>,
        title: &str,
        publication_uri: &str,
    ) -> Result<String> {
        let record =
            build_dictionary_record(lexicon, title, publication_uri, Datetime::now().as_str())?;
        let session = self.agent.get_session().await.context("Not logged in")?;
        let output = self.agent.api.com.atproto.repo.create_record(
            atrium_api::com::atproto::repo::create_record::InputData {
                collection: "site.standard.document".parse().map_err(|e| anyhow!("{}", e))?,
                repo: AtIdentifier::Did(session.did.clone()),
                rkey: None,
                record: serde_json::from_value::<Unknown>(record)?,
                swap_commit: None,
                validate: None,
            }.into()
        ).await?;
        Ok(output.data.uri)
    }
}

// ── Record Construction ──────────────────────────────────────────────────
//
// Pulled out from the publishing methods so the record shape is testable
// without a live PDS session or network access.

/// Build a `site.standard.publication` record.
fn build_publication_record(name: &str, url: &str, published_at: &str) -> Value {
    json!({
        "$type": "site.standard.publication",
        "name": name,
        "url": url,
        "description": "Generated via Esoterica",
        "publishedAt": published_at,
    })
}

/// Build a `site.standard.document` record for a lexicon.
///
/// The lexicon is embedded once, as the document's markdown content; earlier
/// versions duplicated the full JSON into a separate `textContent` field,
/// roughly doubling every published record for no benefit — nothing read
/// `textContent` back out.
fn build_dictionary_record(
    lexicon: &std::collections::BTreeMap<String, crate::lexicon_structs::LexiconEntry>,
    title: &str,
    publication_uri: &str,
    published_at: &str,
) -> Result<Value> {
    let content = serde_json::to_string_pretty(lexicon)?;
    Ok(json!({
        "$type": "site.standard.document",
        "title": title,
        "description": format!("Generated lexicon for {}", title),
        "publishedAt": published_at,
        "path": format!("/lexicon/{}", slugify(title)),
        "publication": { "uri": publication_uri },
        "content": {
            "$type": "site.standard.content.markdown",
            "text": format!("```json\n{}\n```", content),
            "version": "1.0"
        },
    }))
}

/// Lowercase and hyphenate a title for use as a URL path segment.
/// Collapses runs of whitespace so "My  Conlang" doesn't produce a double hyphen.
fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon_structs::{LexiconEntry, Sense};
    use std::collections::BTreeMap;

    fn sample_lexicon() -> BTreeMap<String, LexiconEntry> {
        let mut lexicon = BTreeMap::new();
        lexicon.insert(
            "kota".to_string(),
            LexiconEntry {
                headword: "kota".to_string(),
                etymology: "test".to_string(),
                part_of_speech: "noun".to_string(),
                ipa: "/kota/".to_string(),
                senses: vec![Sense { definition: "A dwelling".to_string(), citations: vec![] }],
                root: "kota".to_string(),
                noun_class: None,
            },
        );
        lexicon
    }

    // ── Publication record ───────────────────────────────────────────────

    #[test]
    fn publication_record_has_the_expected_shape() {
        let record = build_publication_record("My Conlang", "https://example.com", "2024-01-01T00:00:00Z");
        assert_eq!(record["$type"], "site.standard.publication");
        assert_eq!(record["name"], "My Conlang");
        assert_eq!(record["url"], "https://example.com");
        assert_eq!(record["publishedAt"], "2024-01-01T00:00:00Z");
    }

    // ── Dictionary record ────────────────────────────────────────────────

    #[test]
    fn dictionary_record_has_the_expected_shape() {
        let record = build_dictionary_record(
            &sample_lexicon(),
            "My Conlang",
            "at://did:plc:abc/site.standard.publication/xyz",
            "2024-01-01T00:00:00Z",
        )
        .unwrap();
        assert_eq!(record["$type"], "site.standard.document");
        assert_eq!(record["title"], "My Conlang");
        assert_eq!(record["path"], "/lexicon/my-conlang");
        assert_eq!(
            record["publication"]["uri"],
            "at://did:plc:abc/site.standard.publication/xyz"
        );
        assert_eq!(record["content"]["$type"], "site.standard.content.markdown");
    }

    #[test]
    fn dictionary_record_embeds_the_lexicon_exactly_once() {
        // A prior version also wrote the same JSON into a `textContent` field,
        // doubling every published record's size for no reader that used it.
        let record = build_dictionary_record(&sample_lexicon(), "T", "at://x", "2024-01-01T00:00:00Z").unwrap();
        assert!(record.get("textContent").is_none());

        // "kota" appears four times within one embedding of the lexicon: the
        // map key, the "headword", "ipa", and "root" fields. A duplicated
        // embedding would double that to eight.
        let occurrences = record.to_string().matches("kota").count();
        assert_eq!(occurrences, 4, "lexicon content is not embedded exactly once");
    }

    #[test]
    fn dictionary_content_is_valid_json_inside_the_markdown_fence() {
        let record = build_dictionary_record(&sample_lexicon(), "T", "at://x", "2024-01-01T00:00:00Z").unwrap();
        let text = record["content"]["text"].as_str().unwrap();
        assert!(text.starts_with("```json\n"));
        assert!(text.ends_with("\n```"));
        let inner = text.trim_start_matches("```json\n").trim_end_matches("\n```");
        let parsed: Value = serde_json::from_str(inner).expect("embedded content is valid JSON");
        assert_eq!(parsed["kota"]["headword"], "kota");
    }

    #[test]
    fn empty_lexicon_still_produces_a_valid_record() {
        let record = build_dictionary_record(&BTreeMap::new(), "Empty", "at://x", "2024-01-01T00:00:00Z").unwrap();
        let text = record["content"]["text"].as_str().unwrap();
        let inner = text.trim_start_matches("```json\n").trim_end_matches("\n```");
        let parsed: Value = serde_json::from_str(inner).unwrap();
        assert_eq!(parsed, json!({}));
    }

    // ── Slugify ──────────────────────────────────────────────────────────

    #[test]
    fn slugify_lowercases_and_hyphenates() {
        assert_eq!(slugify("My Conlang"), "my-conlang");
    }

    #[test]
    fn slugify_collapses_repeated_whitespace() {
        assert_eq!(slugify("My   Conlang  Title"), "my-conlang-title");
    }

    #[test]
    fn slugify_trims_surrounding_whitespace() {
        assert_eq!(slugify("  My Conlang  "), "my-conlang");
    }

    #[test]
    fn slugify_handles_a_single_word() {
        assert_eq!(slugify("Esoterica"), "esoterica");
    }

    #[test]
    fn slugify_handles_an_empty_title() {
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn slugify_preserves_unicode_letters() {
        assert_eq!(slugify("Ordbók Íslenska"), "ordbók-íslenska");
    }
}
