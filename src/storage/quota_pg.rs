use crate::eol::quota::QuotaStore;
use crate::eol::types::{
    ActorId, CapabilityTier, ComputeEnergyAllowance, UsageSnapshot, UsageWindowId,
};
use anyhow::Result;
use deadpool_postgres::Pool;
use tokio_postgres::Row;

pub struct PgQuotaStore {
    pool: Pool,
}

impl PgQuotaStore {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    fn tier_from_i16(v: i16) -> CapabilityTier {
        match v {
            1 => CapabilityTier::Tier1,
            2 => CapabilityTier::Tier2,
            3 => CapabilityTier::Tier3,
            _ => CapabilityTier::Tier1,
        }
    }
}

impl From<Row> for ComputeEnergyAllowance {
    fn from(row: Row) -> Self {
        ComputeEnergyAllowance {
            max_flops: row.get("max_flops"),
            max_energy_kwh: row.get("max_energy_kwh"),
            max_carbon_kg: row.get("max_carbon_kg"),
            max_tier: PgQuotaStore::tier_from_i16(row.get("max_tier")),
            valid_until: row.get("valid_until"),
        }
    }
}

impl From<Row> for UsageSnapshot {
    fn from(row: Row) -> Self {
        UsageSnapshot {
            window_id: UsageWindowId(row.get("window_id")),
            flops_used: row.get("flops_used"),
            energy_kwh_used: row.get("energy_kwh_used"),
            carbon_kg_emitted: row.get("carbon_kg_emitted"),
        }
    }
}

impl QuotaStore for PgQuotaStore {
    fn get_allowance(
        &self,
        actor: &ActorId,
        window: &UsageWindowId,
    ) -> Result<ComputeEnergyAllowance> {
        let client = self.pool.get()?;
        let row = tokio::runtime::Handle::current().block_on(async {
            client
                .query_one(
                    "SELECT max_flops, max_energy_kwh, max_carbon_kg, max_tier, valid_until
                     FROM eol_allowances
                     WHERE actor_id = $1 AND window_id = $2",
                    &[&actor.0, &window.0],
                )
                .await
        })?;
        Ok(row.into())
    }

    fn get_usage(&self, actor: &ActorId, window: &UsageWindowId) -> Result<UsageSnapshot> {
        let client = self.pool.get()?;
        let row = tokio::runtime::Handle::current().block_on(async {
            client
                .query_one(
                    "SELECT window_id, flops_used, energy_kwh_used, carbon_kg_emitted
                     FROM eol_usage
                     WHERE actor_id = $1 AND window_id = $2",
                    &[&actor.0, &window.0],
                )
                .await
        })?;
        Ok(row.into())
    }

    fn reserve_quota(
        &self,
        actor: &ActorId,
        window: &UsageWindowId,
        expected_flops: f64,
        expected_energy_kwh: f64,
        expected_carbon_kg: f64,
        _tier: &CapabilityTier,
    ) -> Result<crate::eol::types::ReservationId> {
        let client = self.pool.get()?;
        let id = uuid::Uuid::new_v4();

        tokio::runtime::Handle::current().block_on(async {
            client
                .execute(
                    "INSERT INTO eol_reservations
                        (id, actor_id, window_id, expected_flops, expected_energy_kwh, expected_carbon_kg)
                     VALUES ($1, $2, $3, $4, $5, $6)",
                    &[&id, &actor.0, &window.0, &expected_flops, &expected_energy_kwh, &expected_carbon_kg],
                )
                .await
        })?;

        Ok(crate::eol::types::ReservationId(id))
    }
}
