use crate::identity::{ActorProfile, IdentityResolver, ZoneResolver};
use crate::quota::QuotaService;
use crate::energy::StabilityGuard;
use crate::policy::PolicyEngine;
use crate::logging::{ImmutableLogger, EcologicalLogEvent, LogEventType};
use crate::types::*;
use anyhow::Result;

pub struct EcologicalOrchestrator<I, Z, Q, T, P, L>
where
    I: IdentityResolver,
    Z: ZoneResolver,
    Q: QuotaStore,
    T: SegmentTelemetry,
    P: PolicyEngine,
    L: ImmutableLogger,
{
    identity_resolver: I,
    zone_resolver: Z,
    quota_service: QuotaService<Q>,
    stability_guard: StabilityGuard<T>,
    policy_engine: P,
    logger: L,
}

impl<I, Z, Q, T, P, L> EcologicalOrchestrator<I, Z, Q, T, P, L>
where
    I: IdentityResolver,
    Z: ZoneResolver,
    Q: QuotaStore,
    T: SegmentTelemetry,
    P: PolicyEngine,
    L: ImmutableLogger,
{
    pub fn new(
        identity_resolver: I,
        zone_resolver: Z,
        quota_store: Q,
        stability_guard: StabilityGuard<T>,
        policy_engine: P,
        logger: L,
    ) -> Self {
        Self {
            identity_resolver,
            zone_resolver,
            quota_service: QuotaService::new(quota_store),
            stability_guard,
            policy_engine,
            logger,
        }
    }

    pub fn plan_job(
        &self,
        session_token: &str,
        window_id: UsageWindowId,
        mut job: EcologicalJobSpec,
        expected_energy_kwh: f64,
        expected_carbon_kg: f64,
    ) -> Result<JobExecutionPlan> {
        // 1. Resolve actor
        let actor = self.identity_resolver.resolve_actor(session_token)?;
        let zone = self.zone_resolver.resolve_zone(&actor)?;

        // 2. Policy evaluation
        let policy_decision = self.policy_engine.evaluate(&job)?;
        self.logger.append(&EcologicalLogEvent {
            event_type: LogEventType::PolicyEvaluated,
            reservation_id: None,
            actor_id: Some(actor.actor_id.clone()),
            segment_id: Some(zone.segment_id.clone()),
            window_id: Some(window_id.clone()),
            metadata: serde_json::json!({
                "risk_score": policy_decision.risk_score,
                "requires_human_approval": policy_decision.requires_human_approval,
                "notes": policy_decision.notes,
            }),
        })?;

        // 3. Align requested tier with allowed tiers
        if !policy_decision
            .allowed_tiers
            .iter()
            .any(|t| *t == job.requested_tier)
        {
            anyhow::bail!("requested tier not allowed by policy");
        }

        // 4. Reserve quota
        let reservation_id = self.quota_service.check_and_reserve(
            &actor.actor_id,
            &window_id,
            &job,
            expected_energy_kwh,
            expected_carbon_kg,
        )?;

        // 5. Stability guard
        let stability = self.stability_guard.check(
            &zone.segment_id,
            job.expected_flops,
            expected_energy_kwh,
            &job.requested_tier,
        )?;

        // Apply downgrade if needed
        if let StabilityDecision::Downgrade { ref downgraded_tier, .. } = stability {
            job.requested_tier = downgraded_tier.clone();
        }

        // 6. Log plan
        self.logger.append(&EcologicalLogEvent {
            event_type: LogEventType::StabilityChecked,
            reservation_id: Some(reservation_id.clone()),
            actor_id: Some(actor.actor_id.clone()),
            segment_id: Some(zone.segment_id.clone()),
            window_id: Some(window_id.clone()),
            metadata: serde_json::json!({
                "stability_decision": format!("{:?}", stability),
            }),
        })?;

        Ok(JobExecutionPlan {
            reservation_id,
            approved_segment: zone.segment_id,
            approved_tier: job.requested_tier,
            stability_decision: stability,
        })
    }
}
