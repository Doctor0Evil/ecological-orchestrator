use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorId(pub String); // DID string

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentId(pub String); // ALN segment / xr-grid cluster

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityTier {
    Tier1,
    Tier2,
    Tier3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeEnergyAllowance {
    pub max_flops: f64,
    pub max_energy_kwh: f64,
    pub max_carbon_kg: f64,
    pub max_tier: CapabilityTier,
    pub valid_until: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageWindowId(pub String); // e.g. "2026-02-08T00Z_daily"

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSnapshot {
    pub window_id: UsageWindowId,
    pub flops_used: f64,
    pub energy_kwh_used: f64,
    pub carbon_kg_emitted: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentLoad {
    pub segment_id: SegmentId,
    pub current_flops: f64,
    pub energy_rate_kw: f64,
    pub thermal_margin_pct: f64,
    pub renewable_share_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcologicalJobSpec {
    pub actor_id: ActorId,
    pub segment_hint: Option<SegmentId>,
    pub requested_tier: CapabilityTier,
    pub expected_flops: f64,
    pub max_duration: Duration,
    pub purpose: String,      // human-readable purpose
    pub domain_tags: Vec<String>, // e.g. ["climate", "watershed", "biodiversity"]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservationId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StabilityDecision {
    Ok,
    Throttle { reason: String, recommended_delay: Duration },
    Downgrade { reason: String, downgraded_tier: CapabilityTier },
    Deny { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobExecutionPlan {
    pub reservation_id: ReservationId,
    pub approved_segment: SegmentId,
    pub approved_tier: CapabilityTier,
    pub stability_decision: StabilityDecision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairUseReceipt {
    pub reservation_id: ReservationId,
    pub actor_id: ActorId,
    pub segment_id: SegmentId,
    pub flops_used: f64,
    pub energy_kwh_used: f64,
    pub carbon_kg_emitted: f64,
    pub window_id: UsageWindowId,
    pub allowance_remaining_flops: f64,
    pub allowance_remaining_energy_kwh: f64,
    pub allowance_remaining_carbon_kg: f64,
    pub explanation: String,
}
