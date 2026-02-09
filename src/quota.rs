use crate::types::{
    ActorId, CapabilityTier, ComputeEnergyAllowance, EcologicalJobSpec, ReservationId,
    UsageSnapshot, UsageWindowId,
};
use anyhow::Result;
use uuid::Uuid;

pub trait QuotaStore: Send + Sync {
    fn get_allowance(&self, actor: &ActorId, window: &UsageWindowId)
        -> Result<ComputeEnergyAllowance>;

    fn get_usage(&self, actor: &ActorId, window: &UsageWindowId) -> Result<UsageSnapshot>;

    fn reserve_quota(
        &self,
        actor: &ActorId,
        window: &UsageWindowId,
        expected_flops: f64,
        expected_energy_kwh: f64,
        expected_carbon_kg: f64,
        tier: &CapabilityTier,
    ) -> Result<ReservationId>;
}

pub struct QuotaService<Q: QuotaStore> {
    store: Q,
}

impl<Q: QuotaStore> QuotaService<Q> {
    pub fn new(store: Q) -> Self {
        Self { store }
    }

    pub fn check_and_reserve(
        &self,
        actor: &ActorId,
        window: &UsageWindowId,
        job: &EcologicalJobSpec,
        expected_energy_kwh: f64,
        expected_carbon_kg: f64,
    ) -> Result<ReservationId> {
        let allowance = self.store.get_allowance(actor, window)?;
        let usage = self.store.get_usage(actor, window)?;

        // Check tier
        if matches!((&job.requested_tier, &allowance.max_tier),
            (CapabilityTier::Tier3, CapabilityTier::Tier2)
            | (CapabilityTier::Tier3, CapabilityTier::Tier1)
            | (CapabilityTier::Tier2, CapabilityTier::Tier1)
        ) {
            anyhow::bail!("requested tier exceeds maximum allowed tier");
        }

        // Check FLOPs and energy/carbon
        if usage.flops_used + job.expected_flops > allowance.max_flops {
            anyhow::bail!("FLOPs allowance exceeded");
        }
        if usage.energy_kwh_used + expected_energy_kwh > allowance.max_energy_kwh {
            anyhow::bail!("energy allowance exceeded");
        }
        if usage.carbon_kg_emitted + expected_carbon_kg > allowance.max_carbon_kg {
            anyhow::bail!("carbon allowance exceeded");
        }

        let reservation_id = self.store.reserve_quota(
            actor,
            window,
            job.expected_flops,
            expected_energy_kwh,
            expected_carbon_kg,
            &job.requested_tier,
        )?;

        Ok(reservation_id)
    }
}
