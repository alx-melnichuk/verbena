use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub type Connection = PgConnection;

pub type DbPool = Pool<ConnectionManager<Connection>>;

pub type DbPooledConnection = PooledConnection<ConnectionManager<Connection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn init_db_pool(db_url: &str) -> DbPool {
    let manager = ConnectionManager::<Connection>::new(db_url);
    Pool::builder().build(manager).expect("Failed to create pool.")
}

pub fn run_migration(conn: &mut PgConnection) {
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}
