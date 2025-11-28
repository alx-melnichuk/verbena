use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub type Connection = PgConnection;

pub type DbPool = Pool<ConnectionManager<Connection>>;

pub type DbPooledConnection = PooledConnection<ConnectionManager<Connection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn init_db_pool(db_url: &str, max_size: u32) -> DbPool {
    let manager = ConnectionManager::<Connection>::new(db_url);
    let mut builder = Pool::builder();
    if max_size > 0 {
        builder = builder.max_size(max_size);
    }
    builder.build(manager).expect("Failed to create pool.")
}

/** Execute all unapplied migrations for a given migration source */
pub fn run_migration(conn: &mut PgConnection) {
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}
