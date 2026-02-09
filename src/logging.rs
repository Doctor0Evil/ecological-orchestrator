use crate::types::{
    ActorId, FairUseReceipt, JobExecutionPlan, ReservationId, SegmentId, UsageWindowId,
};
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogEventType {
    JobRequested,
    QuotaReserved,
    JobStarted,
    JobCompleted,
    PolicyEvaluated,
    StabilityChecked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcologicalLogEvent {
    pub event_type: LogEventType,
    pub reservation_id: Option<ReservationId>,
    pub actor_id: Option<ActorId>,
    pub segment_id: Option<SegmentId>,
    pub window_id: Option<UsageWindowId>,
    pub metadata: serde_json::Value,
}

pub trait ImmutableLogger: Send + Sync {
    fn append(&self, event: &EcologicalLogEvent) -> Result<()>;
}

pub fn log_execution_plan<L: ImmutableLogger>(
    logger: &L,
    plan: &JobExecutionPlan,
    actor: &ActorId,
) -> Result<()> {
    let event = EcologicalLogEvent {
        event_type: LogEventType::JobStarted,
        reservation_id: Some(plan.reservation_id.clone()),
        actor_id: Some(actor.clone()),
        segment_id: Some(plan.approved_segment.clone()),
        window_id: None,
        metadata: serde_json::json!({
            "tier": format!("{:?}", plan.approved_tier),
            "stability_decision": format!("{:?}", plan.stability_decision),
        }),
    };
    logger.append(&event)
}

// Receipt generation would typically run after telemetry is reconciled.
pub fn build_receipt(
    reservation_id: ReservationId,
    actor_id: ActorId,
    segment_id: SegmentId,
    window_id: UsageWindowId,
    flops_used: f64,
    energy_kwh_used: f64,
    carbon_kg_emitted: f64,
    allowance_remaining_flops: f64,
    allowance_remaining_energy_kwh: f64,
    allowance_remaining_carbon_kg: f64,
) -> FairUseReceipt {
    FairUseReceipt {
        reservation_id,
        actor_id,
        segment_id,
        flops_used,
        energy_kwh_used,
        carbon_kg_emitted,
        window_id,
        allowance_remaining_flops,
        allowance_remaining_energy_kwh,
        allowance_remaining_carbon_kg,
        explanation: "Ecological job executed within configured FLOPs, energy, and carbon budgets; aligned with ALN ethical and stability constraints."
            .into(),
    }
}
