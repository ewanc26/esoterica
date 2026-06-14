use anyhow::{Result, Context, anyhow};
use atrium_api::types::string::Datetime;
use atrium_api::types::Unknown;
use atrium_api::types::string::AtIdentifier;
use serde_json::json;
use bsky_sdk::BskyAgent;

pub struct AtprotoPublisher {
    agent: BskyAgent,
}

impl AtprotoPublisher {
    pub fn new(agent: BskyAgent) -> Self {
        Self { agent }
    }

    pub async fn list_publications(&self) -> Result<Vec<(String, String)>> {
        let session = self.agent.get_session().await.context("Not logged in")?;
        
        let output = self.agent.api.com.atproto.repo.list_records(
            atrium_api::com::atproto::repo::list_records::ParametersData {
                collection: "site.standard.publication".parse().map_err(|e| anyhow!("{}", e))?,
                repo: AtIdentifier::Did(session.did.clone()),
                cursor: None,
                limit: Some(50.try_into().map_err(|e| anyhow!("{}", e))?),
                reverse: None,
            }.into()
        ).await?;

        let mut pubs = Vec::new();
        for record in output.data.records {
            let val = serde_json::to_value(&record.value)?;
            if let Some(name) = val.get("name").and_then(|n| n.as_str()) {
                pubs.push((name.to_string(), record.uri.clone()));
            }
        }
        Ok(pubs)
    }

    pub async fn publish_publication(&self, name: &str, url: &str) -> Result<String> {
        let record = json!({
            "$type": "site.standard.publication",
            "name": name,
            "url": url,
            "description": "Generated via Esoterica",
            "publishedAt": Datetime::now()
        });

        let session = self.agent.get_session().await.context("Not logged in")?;
        
        let output = self.agent.api.com.atproto.repo.create_record(
            atrium_api::com::atproto::repo::create_record::InputData {
                collection: "site.standard.publication".parse().map_err(|e| anyhow!("{}", e))?,
                repo: AtIdentifier::Did(session.did.clone()),
                rkey: None, // PDS generates rkey
                record: serde_json::from_value::<Unknown>(record)?,
                swap_commit: None,
                validate: None,
            }.into()
        ).await?;
        
        Ok(output.data.uri)
    }

    pub async fn publish_dictionary(&self, lexicon: &std::collections::HashMap<String, crate::lexicon_structs::LexiconEntry>, title: &str, publication_uri: &str) -> Result<String> {
        let content = serde_json::to_string_pretty(lexicon)?;
        
        let record = json!({
            "$type": "site.standard.document",
            "title": title,
            "description": format!("Generated lexicon for {}", title),
            "publishedAt": Datetime::now(),
            "path": format!("/lexicon/{}", title.to_lowercase().replace(" ", "-")),
            "publication": { "uri": publication_uri },
            "content": {
                "$type": "site.standard.content.markdown",
                "text": format!("```json\n{}\n```", content),
                "version": "1.0"
            },
            "textContent": content
        });

        let session = self.agent.get_session().await.context("Not logged in")?;

        let output = self.agent.api.com.atproto.repo.create_record(
            atrium_api::com::atproto::repo::create_record::InputData {
                collection: "site.standard.document".parse().map_err(|e| anyhow!("{}", e))?,
                repo: AtIdentifier::Did(session.did.clone()),
                rkey: None, // PDS generates rkey
                record: serde_json::from_value::<Unknown>(record)?,
                swap_commit: None,
                validate: None,
            }.into()
        ).await?;
        
        Ok(output.data.uri)
    }
}
