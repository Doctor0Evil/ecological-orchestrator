use crate::types::{ActorId, SegmentId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorProfile {
    pub actor_id: ActorId,
    pub roles: Vec<String>,
    pub clearance_level: u8,
    pub ecological_priority_score: f32, // higher = more public-benefit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneResolution {
    pub segment_id: SegmentId,
    pub trust_level: u8,
}

pub trait IdentityResolver: Send + Sync {
    fn resolve_actor(&self, session_token: &str) -> anyhow::Result<ActorProfile>;
}

pub trait ZoneResolver: Send + Sync {
    fn resolve_zone(&self, actor: &ActorProfile) -> anyhow::Result<ZoneResolution>;
}
