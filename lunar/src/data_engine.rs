use std::time::Duration;

use sea_orm::{
    ConnectOptions, Database, DatabaseConnection,
    sqlx::{self, postgres::PgConnectOptions},
};

#[cfg(not(target_arch = "wasm32"))]
use migration::{Migrator, MigratorTrait};

use crate::error::LunarError;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct DataEngine {
    database_connection: DatabaseConnection,
}

impl DataEngine {
    pub async fn new(database_url: &str) -> Result<Self, LunarError> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .map_err(|e| LunarError::DbConnectError(e.to_string()))?;

        // let mut opt = PgConnectOptions::new();
        let mut opt = ConnectOptions::new(database_url);
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(false) // disable SQLx logging
            .sqlx_logging_level(log::LevelFilter::Info); // set default Postgres schema

        let db = Database::connect(database_url)
            .await
            .map_err(|e| LunarError::DbConnectError(e.to_string()))?;

        Ok(Self {
            database_connection: db,
        })
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.database_connection
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn run_migrations(&self) -> Result<(), LunarError> {
        Migrator::up(&self.database_connection, None)
            .await
            .map_err(|e| LunarError::DbConnectError(e.to_string()))?;

        Ok(())
    }
}
