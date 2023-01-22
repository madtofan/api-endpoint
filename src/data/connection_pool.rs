use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use tracing::info;

pub type ServiceConnectionPool = Pool<Postgres>;
pub struct ServiceConnectionManager;

impl ServiceConnectionManager {
    pub async fn new_pool(
        connection_string: &str,
        run_migrations: bool,
    ) -> anyhow::Result<ServiceConnectionPool> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(connection_string)
            .await
            .context("error while initializing the database connection pool")?;

        // if run_migrations {
        //     info!("migrations enabled, running...");
        //     sqlx::migrate!()
        //         .run(&pool)
        //         .await
        //         .context("error while running database migrations")?;
        // }

        Ok(pool)
    }
}
