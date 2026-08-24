use std::fs;

use actix_web::web::{Data, ServiceConfig};
use vrb_db::db::DbPool2;

pub mod config_strm;
pub mod stream_controller;
pub mod stream_db;
pub mod stream_models;
pub mod stream_test_get1;
pub mod stream_test_get2;
pub mod stream_test_post_delete;
pub mod stream_test_put;

/**
 * Adding data for the controller of this library.
 *   Addendum: ConfigStrm, StreamDb.
 */
pub fn configure_data_and_server(db_pool: &DbPool2) -> impl FnOnce(&mut ServiceConfig) {
    move |config: &mut ServiceConfig| {
        // Creating a directory for upload the logo.
        let config_strm = config_strm::ConfigStrm::init_by_env();

        let logo_files_dir = config_strm.strm_logo_files_dir.clone();
        fs::create_dir_all(&logo_files_dir).expect(&format!("Error when creating the \"{}\" directory:", &logo_files_dir));

        // used: stream_controller
        let config_strm = Data::new(config_strm::ConfigStrm::init_by_env());
        config.app_data(Data::clone(&config_strm));

        // used: stream_controller
        let stream_db = Data::new(stream_db::get_stream_db_app(db_pool));
        config.app_data(Data::clone(&stream_db));

        // Add configuration of internal services.
        config.configure(stream_controller::configure());
    }
}
