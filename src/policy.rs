use crate::types::{CapabilityTier, EcologicalJobSpec};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct PolicyDecision {
    pub risk_score: f32,
    pub allowed_tiers: Vec<CapabilityTier>,
    pub requires_human_approval: bool,
    pub notes: Vec<String>,
}

pub trait PolicyEngine: Send + Sync {
    fn evaluate(&self, job: &EcologicalJobSpec) -> Result<PolicyDecision>;
}

// Example stub aligned with NIST AI RMF + HITL for critical decisions.[file:1][file:2][file:5]
pub struct SimplePolicyEngine;

impl PolicyEngine for SimplePolicyEngine {
    fn evaluate(&self, job: &EcologicalJobSpec) -> Result<PolicyDecision> {
        let mut risk = 0.1;
        let mut requires_human = false;
        let mut notes = Vec::new();

        if job.domain_tags.contains(&"geoengineering".to_string()) {
            risk = 0.9;
            requires_human = true;
            notes.push("geoengineering scenario: enforce HITL & multi-party approval".into());
        }

        if job.domain_tags.contains(&"critical_infrastructure".to_string()) {
            risk = risk.max(0.8);
            requires_human = true;
            notes.push("critical infrastructure modeling: enforce HITL".into());
        }

        Ok(PolicyDecision {
            risk_score: risk,
            allowed_tiers: vec![CapabilityTier::Tier1, CapabilityTier::Tier2, CapabilityTier::Tier3],
            requires_human_approval: requires_human,
            notes,
        })
    }
}
