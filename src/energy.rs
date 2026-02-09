use crate::types::{CapabilityTier, SegmentId, SegmentLoad, StabilityDecision};
use anyhow::Result;

pub trait SegmentTelemetry: Send + Sync {
    fn get_segment_load(&self, segment_id: &SegmentId) -> Result<SegmentLoad>;
}

pub struct StabilityGuard<T: SegmentTelemetry> {
    telemetry: T,
    max_thermal_pct: f64,
    min_renewable_pct: f64,
}

impl<T: SegmentTelemetry> StabilityGuard<T> {
    pub fn new(telemetry: T, max_thermal_pct: f64, min_renewable_pct: f64) -> Self {
        Self {
            telemetry,
            max_thermal_pct,
            min_renewable_pct,
        }
    }

    pub fn check(
        &self,
        segment_id: &SegmentId,
        proposed_flops: f64,
        proposed_energy_kwh: f64,
        requested_tier: &CapabilityTier,
    ) -> Result<StabilityDecision> {
        let load = self.telemetry.get_segment_load(segment_id)?;

        if load.thermal_margin_pct < self.max_thermal_pct {
            return Ok(StabilityDecision::Throttle {
                reason: "thermal margin too low".into(),
                recommended_delay: std::time::Duration::from_secs(900),
            });
        }

        if load.renewable_share_pct < self.min_renewable_pct {
            // Try downgrading tier to reduce energy draw
            let downgraded = match requested_tier {
                CapabilityTier::Tier3 => Some(CapabilityTier::Tier2),
                CapabilityTier::Tier2 => Some(CapabilityTier::Tier1),
                CapabilityTier::Tier1 => None,
            };

            if let Some(d) = downgraded {
                return Ok(StabilityDecision::Downgrade {
                    reason: "insufficient renewable share, downgraded tier".into(),
                    downgraded_tier: d,
                });
            } else {
                return Ok(StabilityDecision::Deny {
                    reason: "insufficient renewable share for even lowest tier".into(),
                });
            }
        }

        Ok(StabilityDecision::Ok)
    }
}
