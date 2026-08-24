use actix_web::web::{Data, ServiceConfig};
use vrb_db::db::DbPool2;

pub mod authentication;
pub mod authentication_test;
pub mod config_jwt;
pub mod user_authent_controller;
pub mod user_authent_models;
pub mod user_authent_test;
pub mod user_db;
pub mod user_models;
pub mod user_recovery_controller;
pub mod user_recovery_db;
pub mod user_recovery_models;
pub mod user_recovery_test;
pub mod user_registr_controller;
pub mod user_registr_db;
pub mod user_registr_models;
pub mod user_registr_test;

/**
 * Adding data for the controller of this library.
 *   Addendum: ConfigJwt, UserDb, UserRegistrDb, UserRecoveryDb.
 *   Use: ConfigApp, ConfigSmtp, mailer.
 */
pub fn configure_data_and_server(db_pool: &DbPool2) -> impl FnOnce(&mut ServiceConfig) {
    move |config: &mut ServiceConfig| {
        // used: user_authent_controller, user_recovery_controller, user_registr_controller
        let config_jwt = Data::new(config_jwt::ConfigJwt::init_by_env());
        config.app_data(Data::clone(&config_jwt));

        // Create "UserDbApp".
        let user_db = Data::new(user_db::get_user_db_app(db_pool));
        config.app_data(Data::clone(&user_db));

        // used: user_registr_controller
        let user_registr_db = Data::new(user_registr_db::get_user_registr_db_app(db_pool.clone()));
        config.app_data(Data::clone(&user_registr_db));
        // used: user_recovery_controller

        let user_recovery_db = Data::new(user_recovery_db::get_user_recovery_db_app(db_pool.clone()));
        config.app_data(Data::clone(&user_recovery_db));

        // Add configuration of internal services.
        config.configure(user_recovery_controller::configure());
        config.configure(user_registr_controller::configure());
        config.configure(user_authent_controller::configure());
    }
}
