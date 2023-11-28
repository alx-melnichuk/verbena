use std::env;

use actix_files::Files;
use actix_web::web;

use send_email::{config_smtp, mailer};
use sessions::{config_jwt, session_orm::cfg::get_session_orm_app};
use users::{
    user_auth_controller, user_controller, user_orm::cfg::get_user_orm_app,
    user_recovery_orm::cfg::get_user_recovery_orm_app, user_registr_controller,
    user_registr_orm::cfg::get_user_registr_orm_app,
};

pub(crate) mod dbase;
pub(crate) mod errors;
pub(crate) mod extractors;
pub(crate) mod hash_tools;
pub(crate) mod schema;
pub(crate) mod send_email;
pub(crate) mod sessions;
pub mod settings;
pub(crate) mod static_controller;
pub(crate) mod tools;
pub(crate) mod users;
pub(crate) mod utils;
pub mod validators;

pub fn configure_server() -> Box<dyn Fn(&mut web::ServiceConfig)> {
    Box::new(move |cfg: &mut web::ServiceConfig| {
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not found.");
        // let domain: String = env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());

        let pool: dbase::DbPool = dbase::init_db_pool(&db_url);
        dbase::run_migration(&mut pool.get().unwrap());

        let data_config_app = web::Data::new(settings::config_app::ConfigApp::init_by_env());
        let data_config_jwt = web::Data::new(config_jwt::ConfigJwt::init_by_env());
        let config_smtp = config_smtp::ConfigSmtp::init_by_env();
        let data_config_smtp = web::Data::new(config_smtp.clone());
        // data_config_smtp.get_ref().clone()
        let data_mailer = web::Data::new(mailer::cfg::get_mailer_app(config_smtp));
        let data_user_orm = web::Data::new(get_user_orm_app(pool.clone()));
        let data_user_registr_orm = web::Data::new(get_user_registr_orm_app(pool.clone()));
        let data_session_orm = web::Data::new(get_session_orm_app(pool.clone()));
        let data_user_recovery_orm = web::Data::new(get_user_recovery_orm_app(pool.clone()));

        cfg.app_data(web::Data::clone(&data_config_app))
            .app_data(web::Data::clone(&data_config_jwt))
            .app_data(web::Data::clone(&data_config_smtp))
            .app_data(web::Data::clone(&data_mailer))
            .app_data(web::Data::clone(&data_user_orm))
            .app_data(web::Data::clone(&data_user_registr_orm))
            .app_data(web::Data::clone(&data_session_orm))
            .app_data(web::Data::clone(&data_user_recovery_orm))
            .service(Files::new("/static", "static").show_files_listing())
            .service(Files::new("/assets", "static/assets").show_files_listing())
            .configure(static_controller::configure)
            .service(
                web::scope("/api")
                    .configure(user_registr_controller::configure)
                    .configure(user_auth_controller::configure)
                    .configure(user_controller::configure),
            );
    })
}
