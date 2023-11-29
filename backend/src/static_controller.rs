use actix_files::Files;
use actix_web::{http, web, HttpRequest, HttpResponse};
use std::io::Error;

use crate::settings::config_app;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/{filename:.(.+).js|(.+).css|(.+).ico|(.+).png}")
            .route(web::get().to(loading_js_css_files)),
    )
    .service(Files::new("/static", "static").show_files_listing())
    .service(Files::new("/assets", "static/assets").show_files_listing())
    .service(web::resource("/login").route(web::get().to(index_root)))
    .service(web::resource("/signup").route(web::get().to(index_root)))
    .service(web::resource("/forgot-password").route(web::get().to(index_root)))
    .service(web::resource("/").route(web::get().to(index_root)));
}

/** Loading the `index.html` file. */
pub async fn index_root() -> Result<HttpResponse, Error> {
    let body_str = include_str!("../static/index.html");
    let config_app = config_app::ConfigApp::init_by_env();
    let app_env = format!(
        "var appEnv = {{appRoot:'{}/', appApi:'{}/'}}",
        &config_app.app_domain, &config_app.app_domain
    );
    let body_str = body_str.replacen("var appEnv={}", &app_env.to_string(), 1);

    let app_name = format!("<title>{}</title>", &config_app.app_name);
    let body_str = body_str.replacen("<title>AppName</title>", &app_name, 1);

    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(body_str))
}

/** Loading ".js|.css|.ico|.png" files. */
pub async fn loading_js_css_files(req: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    let path_filename: std::path::PathBuf = req.match_info().query("filename").parse().unwrap();
    let path_str: &str = path_filename.to_str().unwrap();
    let path: std::path::PathBuf = ["static", path_str].iter().collect();
    // eprintln!("### path:\"{}\"", path.to_string_lossy());
    let file: actix_files::NamedFile = actix_files::NamedFile::open(path)?;
    Ok(file.use_last_modified(true))
}

// Loading static files.
