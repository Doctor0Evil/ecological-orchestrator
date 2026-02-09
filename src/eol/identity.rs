use crate::eol::types::{ActorId, SegmentId};
use crate::eol::identity::{ActorProfile, ZoneResolution, IdentityResolver, ZoneResolver};
use anyhow::Result;

// Example: resolve DIDs from an internal service or map session tokens to DIDs.
pub struct HttpIdentityResolver {
    client: reqwest::Client,
    base_url: String,
}

impl HttpIdentityResolver {
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }
}

impl IdentityResolver for HttpIdentityResolver {
    fn resolve_actor(&self, session_token: &str) -> Result<ActorProfile> {
        let url = format!("{}/resolve_actor", self.base_url);
        let resp: serde_json::Value = tokio::runtime::Handle::current().block_on(async {
            self.client
                .get(&url)
                .header("X-Session-Token", session_token)
                .send()
                .await?
                .json()
                .await
        })?;
        // Map JSON into ActorProfile
        Ok(ActorProfile {
            actor_id: ActorId(resp["actor_id"].as_str().unwrap_or_default().to_string()),
            roles: vec!["researcher".into()], // TODO: parse from JSON
            clearance_level: resp["clearance"].as_u64().unwrap_or(1) as u8,
            ecological_priority_score: resp["eco_priority"].as_f64().unwrap_or(0.5) as f32,
        })
    }
}

pub struct StaticZoneResolver;

impl ZoneResolver for StaticZoneResolver {
    fn resolve_zone(&self, actor: &ActorProfile) -> Result<ZoneResolution> {
        // Map actor roles / DIDs to predefined ALN segments.
        // Replace this with a real config or DB lookup.
        let segment = if actor.roles.contains(&"climate_lab".to_string()) {
            SegmentId("segment_climate_hpc".into())
        } else {
            SegmentId("segment_general_research".into())
        };

        Ok(ZoneResolution {
            segment_id: segment,
            trust_level: 3,
        })
    }
}
