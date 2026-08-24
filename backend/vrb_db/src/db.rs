use std::time::Duration;

use sqlx::{
    self, PgPool,
    migrate::{MigrateError, Migrator},
    postgres::PgPoolOptions,
};

pub type DbPool2 = PgPool;

pub async fn init_db_pool(
    database_url: &str,
    max_conn: u32,
    min_conn: u32,
    max_lifetime_sec: u64,
    idle_sec: u64,
) -> Result<DbPool2, sqlx::Error> {
    let mut pool_options = PgPoolOptions::new();

    if max_conn > 0 {
        // Set the maximum number of connections that this pool should maintain.
        pool_options = pool_options.max_connections(max_conn);
    }
    if min_conn > 0 {
        // Set the minimum number of connections to maintain at all times.
        pool_options = pool_options.min_connections(min_conn);
    }
    if max_lifetime_sec > 0 {
        // Set the maximum lifetime of individual connections.
        pool_options = pool_options.max_lifetime(Duration::from_secs(max_lifetime_sec));
    }
    if idle_sec > 0 {
        // Set a maximum idle duration for individual connections.
        pool_options = pool_options.idle_timeout(Duration::from_secs(idle_sec));
    }

    pool_options.connect(&database_url).await
}

/** Execute all unapplied migrations for a given migration source */
pub async fn run_migration_db(db_pool: &DbPool2) -> Result<(), MigrateError> {
    let migration_db = Migrator::new(std::path::Path::new("./migrations")).await?;
    migration_db.run(db_pool).await
}
