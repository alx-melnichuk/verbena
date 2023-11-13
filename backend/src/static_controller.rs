use actix_files::Files;
use actix_web::{http, web, HttpRequest, HttpResponse};
use std::io::Error;

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
    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html")))
}

/** Loading ".js|.css|.ico|.png" files. */
pub async fn loading_js_css_files(req: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    let path: std::path::PathBuf = req.match_info().query("filename").parse().unwrap();
    let pathstr: &str = path.to_str().unwrap();
    // let path1: String = "static".to_string() + pathstr;
    let path2: std::path::PathBuf = ["static", pathstr].iter().collect();
    // eprintln!("### path1:\"{}\"", path1);
    // eprintln!("### path2:\"{}\"", path2.to_string_lossy());
    let file: actix_files::NamedFile = actix_files::NamedFile::open(path2)?;
    Ok(file.use_last_modified(true))
}

// Loading static files.
