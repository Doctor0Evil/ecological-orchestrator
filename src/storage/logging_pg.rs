use crate::eol::logging::{EcologicalLogEvent, ImmutableLogger};
use anyhow::Result;
use deadpool_postgres::Pool;

pub struct PgImmutableLogger {
    pool: Pool,
}

impl PgImmutableLogger {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

impl ImmutableLogger for PgImmutableLogger {
    fn append(&self, event: &EcologicalLogEvent) -> Result<()> {
        let client = self.pool.get()?;
        let json = serde_json::to_value(event)?;
        tokio::runtime::Handle::current().block_on(async {
            client
                .execute(
                    "INSERT INTO eol_logs (event)
                     VALUES ($1::jsonb)",
                    &[&json],
                )
                .await
        })?;
        Ok(())
    }
}
