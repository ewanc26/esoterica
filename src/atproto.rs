use anyhow::{Result, Context, anyhow};
use atrium_api::types::string::Datetime;
use atrium_api::types::string::RecordKey;
use atrium_api::types::string::AtIdentifier;
use atrium_api::types::Unknown;
use serde_json::json;
use bsky_sdk::BskyAgent;

pub struct AtprotoPublisher {
    agent: BskyAgent,
}

impl AtprotoPublisher {
    pub fn new(agent: BskyAgent) -> Self {
        Self { agent }
    }

    pub async fn publish_dictionary(&self, lexicon: &std::collections::HashMap<String, String>, title: &str) -> Result<String> {
        let content = serde_json::to_string_pretty(lexicon)?;
        
        let record = json!({
            "$type": "site.standard.document",
            "title": title,
            "description": format!("Generated lexicon for {}", title),
            "publishedAt": Datetime::now(),
            "path": format!("/lexicon/{}", title.to_lowercase().replace(" ", "-")),
            "content": {
                "$type": "site.standard.content.markdown",
                "text": content,
                "version": "1.0"
            },
            "textContent": content
        });

        let session = self.agent.get_session().await.context("Not logged in")?;

        let output = self.agent.api.com.atproto.repo.put_record(
            atrium_api::com::atproto::repo::put_record::InputData {
                collection: "site.standard.document".parse().map_err(|e| anyhow!("{}", e))?,
                repo: AtIdentifier::Did(session.did.clone()),
                rkey: RecordKey::from_str(&format!("dict-{}", title).replace(" ", "-")).map_err(|e| anyhow!("{}", e))?,
                record: serde_json::from_value::<Unknown>(record)?,
                swap_record: None,
                validate: None,
                swap_commit: None,
            }.into()
        ).await?;
        
        Ok(output.data.uri)
    }
}
use std::str::FromStr;
