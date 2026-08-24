use std::fs;

use actix_web::web::{Data, ServiceConfig};
use vrb_db::db::DbPool2;

pub mod config_prfl;
pub mod profile_controller;
pub mod profile_db;
pub mod profile_models;
pub mod profile_test_delete;
pub mod profile_test_get;
pub mod profile_test_put;

/**
 * Adding data for the controller of this library.
 *   Addendum: ConfigPrfl, ProfileDb.
 *   Use: UserDb, UserRegistrDb.
 */
pub fn configure_data_and_server(db_pool: &DbPool2) -> impl FnOnce(&mut ServiceConfig) {
    move |config: &mut ServiceConfig| {
        // Creating a directory for upload the avatar.
        let config_prfl = config_prfl::ConfigPrfl::init_by_env();

        let avatar_files_dir = config_prfl.prfl_avatar_files_dir.clone();
        fs::create_dir_all(&avatar_files_dir).expect(&format!("Error when creating the \"{}\" directory:", &avatar_files_dir));

        // used: profile_controller
        let config_prfl = Data::new(config_prfl);
        config.app_data(Data::clone(&config_prfl));

        // used: profile_controller
        let profile_db = Data::new(profile_db::get_profile_db_app(db_pool));
        config.app_data(Data::clone(&profile_db));

        // Add configuration of internal services.
        config.configure(profile_controller::configure());
    }
}
