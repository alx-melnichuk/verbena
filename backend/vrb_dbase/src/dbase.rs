use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub type Connection = PgConnection;

pub type DbPool = Pool<ConnectionManager<Connection>>;

pub type DbPooledConnection = PooledConnection<ConnectionManager<Connection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn init_db_pool(db_url: &str) -> DbPool {
    let manager = ConnectionManager::<Connection>::new(db_url);
    let max_size = 15;
    let db_pool = Pool::builder()
        .max_size(max_size)
        .build(manager)
        .expect("Failed to create pool.");
     
    // let db_pool = Pool::builder().build(manager).expect("Failed to create pool.");
    eprintln!("db_pool.max_size(): {}", db_pool.max_size());
    db_pool
}

pub fn run_migration(conn: &mut PgConnection) {
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}
