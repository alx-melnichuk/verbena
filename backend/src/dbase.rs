use diesel::{r2d2, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub type Connection = PgConnection;

// pub type DbError = Box<(dyn std::error::Error + Send + Sync)>;
//    Box<(dyn StdError + Send + Sync + 'static)>

pub type DbPool = r2d2::Pool<r2d2::ConnectionManager<Connection>>;

#[cfg(not(feature = "mockdata"))]
pub type DbPooledConnection = r2d2::PooledConnection<r2d2::ConnectionManager<Connection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn init_db_pool(db_url: &str) -> DbPool {
    log::info!("Configuring database.");
    let manager = r2d2::ConnectionManager::<Connection>::new(db_url);
    let pool = r2d2::Pool::builder().build(manager).expect("Failed to create pool.");

    pool
}

pub fn run_migration(conn: &mut PgConnection) {
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}

#[cfg(feature = "mockdata")]
use crate::users::user_orm::tests::UserOrmApp;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_orm::UserOrmApp;

pub fn get_user_orm_app(pool: DbPool) -> UserOrmApp {
    #[cfg(feature = "mockdata")]
    let user_orm: UserOrmApp = UserOrmApp::new();
    #[cfg(not(feature = "mockdata"))]
    let user_orm: UserOrmApp = UserOrmApp::new(pool);

    user_orm
}

#[cfg(feature = "mockdata")]
use crate::users::user_registr_orm::tests::UserRegistrOrmApp;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_registr_orm::UserRegistrOrmApp;

pub fn get_user_registr_orm_app(pool: DbPool) -> UserRegistrOrmApp {
    #[cfg(feature = "mockdata")]
    let user_registr_orm: UserRegistrOrmApp = UserRegistrOrmApp::new();
    #[cfg(not(feature = "mockdata"))]
    let user_registr_orm: UserRegistrOrmApp = UserRegistrOrmApp::new(pool);

    user_registr_orm
}

#[cfg(feature = "mockdata")]
use crate::sessions::session_orm::tests::SessionOrmApp;
#[cfg(not(feature = "mockdata"))]
use crate::sessions::session_orm::SessionOrmApp;

pub fn get_session_orm_app(pool: DbPool) -> SessionOrmApp {
    #[cfg(feature = "mockdata")]
    let session_orm: SessionOrmApp = SessionOrmApp::new();
    #[cfg(not(feature = "mockdata"))]
    let session_orm: SessionOrmApp = SessionOrmApp::new(pool);

    session_orm
}
