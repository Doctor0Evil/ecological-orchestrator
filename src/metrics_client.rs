use crate::eol::energy::{SegmentTelemetry};
use crate::eol::types::{SegmentId, SegmentLoad};
use anyhow::Result;
use prometheus_parse::{Value, Scrape};
use std::collections::HashMap;

pub struct PrometheusTelemetry {
    /// Map segment_id -> metrics URL
    endpoints: HashMap<String, String>,
    http: reqwest::Client,
}

impl PrometheusTelemetry {
    pub fn new(endpoints: HashMap<String, String>) -> Self {
        Self {
            endpoints,
            http: reqwest::Client::new(),
        }
    }

    async fn scrape(&self, url: &str) -> Result<Scrape> {
        let text = self.http.get(url).send().await?.text().await?;
        let scrape = prometheus_parse::Scrape::parse(text.lines())?;
        Ok(scrape)
    }

    fn metric_value(scrape: &Scrape, metric: &str) -> Option<f64> {
        scrape.samples.iter().find_map(|s| {
            if s.metric == metric {
                match s.value {
                    Value::Counter(c) => Some(c),
                    Value::Gauge(g) => Some(g),
                    Value::Untyped(u) => Some(u),
                    _ => None,
                }
            } else {
                None
            }
        })
    }
}

#[async_trait::async_trait]
impl SegmentTelemetry for PrometheusTelemetry {
    async fn get_segment_load_async(&self, segment_id: &SegmentId) -> Result<SegmentLoad> {
        let url = self
            .endpoints
            .get(&segment_id.0)
            .ok_or_else(|| anyhow::anyhow!("unknown segment_id {}", segment_id.0))?;

        let scrape = self.scrape(url).await?;

        let current_flops = Self::metric_value(&scrape, "xr_flops_current").unwrap_or(0.0);
        let energy_rate_kw = Self::metric_value(&scrape, "xr_power_kw").unwrap_or(0.0);
        let thermal_margin_pct =
            Self::metric_value(&scrape, "xr_thermal_margin_pct").unwrap_or(100.0);
        let renewable_share_pct =
            Self::metric_value(&scrape, "xr_renewable_share_pct").unwrap_or(0.0);

        Ok(SegmentLoad {
            segment_id: segment_id.clone(),
            current_flops,
            energy_rate_kw,
            thermal_margin_pct,
            renewable_share_pct,
        })
    }
}
