use actix_files::Files;
use actix_web::{http, web, HttpRequest, HttpResponse};
use std::{io::Error, path};

use crate::{
    settings::config_app,
    streams::{config_strm, stream_controller},
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    let logo = stream_controller::ALIAS_LOGO_FILES;
    let alias_logo = format!("/{}/{}", logo, "{name_logo:.*}");

    cfg.service(Files::new("/static", "static").show_files_listing())
        .service(Files::new("/assets", "static/assets").show_files_listing())
        .service(web::resource(&alias_logo).route(web::get().to(load_files_logo)))
        .service(web::resource("/{name1:.(.+).js|(.+).css}").route(web::get().to(load_files_js_css)))
        .service(web::resource("/{name2:.(.+).ico|(.+).png|(.+).svg}").route(web::get().to(load_files_images)))
        // Route returns index.html - FE app
        .service(web::resource("/ind/{path_url:.*}").route(web::get().to(index_root)));
}

/// Loading the `index.html` file.
pub async fn index_root() -> Result<HttpResponse, Error> {
    let body_str = include_str!("../static/index.html");
    let config_app = config_app::ConfigApp::init_by_env();

    let app_name = format!("<title>{}</title>", &config_app.app_name);
    let body_str = body_str.replacen("<title>APP_NAME</title>", &app_name, 1);
    #[rustfmt::skip]
    let app_domain = format!("<script>var APP_DOMAIN='{}';</script>", &config_app.app_domain );
    let body_str = body_str.replacen("<script>var APP_DOMAIN;</script>", &app_domain, 1);

    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(body_str))
}

pub async fn load_files_logo(req: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    let config_strm = config_strm::ConfigStrm::init_by_env();
    load_file_from_dir(&config_strm.strm_logo_files_dir, &get_param(req, "name_logo")).await
}

pub async fn load_files_js_css(req: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    load_file_from_dir("static", &get_param(req, "name1")).await
}

pub async fn load_files_images(req: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    load_file_from_dir("static", &get_param(req, "name2")).await
}

/// Get the value of the parameter.
fn get_param(req: HttpRequest, param_name: &str) -> String {
    let path_buf_filename: path::PathBuf = req.match_info().query(param_name).parse().unwrap();
    path_buf_filename.to_str().unwrap().to_string()
}

/// Load from the directory a file with the name from the parameter.
async fn load_file_from_dir(dir: &str, file_name: &str) -> Result<actix_files::NamedFile, Error> {
    // Normalize the directory value.
    let path_buf_dir: path::PathBuf = path::PathBuf::from(dir).iter().collect();
    let directory = path_buf_dir.to_str().unwrap();

    // Get the path to a file in a given directory.
    let path_buf: path::PathBuf = [directory, file_name].iter().collect();
    // #[rustfmt::skip]
    // eprintln!("load_file_from_dir(dir: '{}', file_name: '{}') exists({})={}", dir, file_name,
    // &path_buf.to_string_lossy().into_owned(), path_buf.as_path().exists());
    // Open a file in the specified directory.
    let file: actix_files::NamedFile = actix_files::NamedFile::open(path_buf)?;

    Ok(file.use_last_modified(true))
}
