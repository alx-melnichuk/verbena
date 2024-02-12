use actix_files::Files;
use actix_web::{http, web, HttpRequest, HttpResponse};
use std::{io::Error, path};

use crate::{settings::config_app, streams::config_slp};

pub fn configure(cfg: &mut web::ServiceConfig) {
    let config_slp = config_slp::ConfigSLP::init_by_env();
    let logo_dir = config_slp.slp_dir;
    let logos = format!("/{}", config_slp.slp_logos); // "/logos"

    cfg.service(Files::new("/static", "static").show_files_listing())
        .service(Files::new("/assets", "static/assets").show_files_listing())
        .service(Files::new(&logos, logo_dir).show_files_listing())
        .service(web::resource("/{name1:.(.+).js|(.+).css}").route(web::get().to(load_files_js_css)))
        .service(web::resource("/{name2:.(.+).ico|(.+).png}").route(web::get().to(load_files_ico_png)))
        // Route returns index.html - FE app
        .service(web::resource("/ind/{path_url:.*}").route(web::get().to(index_root)));
}

/** Loading the `index.html` file. */
pub async fn index_root() -> Result<HttpResponse, Error> {
    let body_str = include_str!("../static/index.html");
    let config_app = config_app::ConfigApp::init_by_env();

    let app_name = format!("<title>{}</title>", &config_app.app_name);
    let body_str = body_str.replacen("<title>APP_NAME</title>", &app_name, 1);
    #[rustfmt::skip]
    let app_domain = format!("<script>var APP_DOMAIN='{}/';</script>", &config_app.app_domain );
    let body_str = body_str.replacen("<script>var APP_DOMAIN;</script>", &app_domain, 1);

    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(body_str))
}

pub async fn load_files_js_css(req: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    load_file_from_dir(req, "name1", "static").await
}
pub async fn load_files_ico_png(req: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    load_file_from_dir(req, "name2", "static").await
}

/// Load from the directory a file with the name from the parameter.
async fn load_file_from_dir(req: HttpRequest, param: &str, dir: &str) -> Result<actix_files::NamedFile, Error> {
    // Get the value of the parameter.
    let path_buf_filename: path::PathBuf = req.match_info().query(param).parse().unwrap();
    let filename: &str = path_buf_filename.to_str().unwrap();

    // Normalize the directory value.
    let path_buf_dir: path::PathBuf = path::PathBuf::from(dir).iter().collect();
    let directory = path_buf_dir.to_str().unwrap();

    // Get the path to a file in a given directory.
    let path: path::PathBuf = [directory, filename].iter().collect();
    // eprintln!("## load_file_from_dir() path:\"{}\"", path.to_string_lossy());
    // Open a file in the specified directory.
    let file: actix_files::NamedFile = actix_files::NamedFile::open(path)?;

    Ok(file.use_last_modified(true))
}
