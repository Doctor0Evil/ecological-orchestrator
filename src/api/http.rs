use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use crate::eol::types::{EcologicalJobSpec, UsageWindowId, FairUseReceipt};
use crate::eol::orchestrator::EcologicalOrchestrator;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct PlanJobRequest {
    pub session_token: String,
    pub window_id: String,
    pub job: EcologicalJobSpec,
    pub expected_energy_kwh: f64,
    pub expected_carbon_kg: f64,
}

#[derive(Serialize)]
pub struct PlanJobResponse {
    pub plan: crate::eol::types::JobExecutionPlan,
}

pub fn build_router(
    orchestrator: Arc<
        EcologicalOrchestrator<
            impl crate::eol::identity::IdentityResolver,
            impl crate::eol::identity::ZoneResolver,
            impl crate::eol::quota::QuotaStore,
            impl crate::eol::energy::SegmentTelemetry,
            impl crate::eol::policy::PolicyEngine,
            impl crate::eol::logging::ImmutableLogger,
        >,
    >,
) -> Router {
    let plan_orch = orchestrator.clone();

    Router::new().route(
        "/plan_job",
        post(move |Json(req): Json<PlanJobRequest>| {
            let orch = plan_orch.clone();
            async move {
                let window = UsageWindowId(req.window_id.clone());
                let plan = orch
                    .plan_job(
                        &req.session_token,
                        window,
                        req.job,
                        req.expected_energy_kwh,
                        req.expected_carbon_kg,
                    )
                    .map_err(|e| {
                        (
                            axum::http::StatusCode::BAD_REQUEST,
                            format!("planning error: {:?}", e),
                        )
                    })?;
                Ok::<_, (axum::http::StatusCode, String)>(Json(PlanJobResponse { plan }))
            }
        }),
    )
}
