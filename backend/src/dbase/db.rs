use diesel::{r2d2, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub type Connection = PgConnection;

// pub type DbError = Box<(dyn std::error::Error + Send + Sync)>;
//    Box<(dyn StdError + Send + Sync + 'static)>

pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<Connection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn init_db_pool(db_url: &str) -> DbPool {
    use log::info;

    info!("Configuring database...");
    let manager = r2d2::ConnectionManager::<Connection>::new(db_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    pool
}

pub fn run_migration(conn: &mut PgConnection) {
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}
