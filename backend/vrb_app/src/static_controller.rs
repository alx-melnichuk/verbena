use std::{fs, io::Error, path};

use actix_files::Files;
use actix_web::{HttpRequest, HttpResponse, get, http, web};
use vrb_tools::config_app;

pub fn configure(config_app: config_app::ConfigApp) -> impl FnOnce(&mut web::ServiceConfig) {
    move |config: &mut web::ServiceConfig| {
        let list = [config_app.app_dir_imgs.clone(), config_app.app_dir_static.clone()];
        for item in list {
            let path_item = path::PathBuf::from(item);
            let res = path_item.strip_prefix("./");
            let path_item = if res.is_ok() { res.unwrap().to_path_buf() } else { path_item };
            let path_item2 = path_item.clone();
            let base_prefix = path_item2.to_str().unwrap();

            // Add visibility of files from all internal directories.
            for res in fs::read_dir(path_item).unwrap() {
                let path_buf = res.unwrap().path();
                if !path_buf.is_dir() {
                    continue;
                }
                let path_buf2: path::PathBuf = path_buf.clone();
                let mount_path = path_buf2.strip_prefix(base_prefix).unwrap().to_str().unwrap();
                let serve_from = path_buf.to_str().unwrap();

                config.service(Files::new(mount_path, serve_from).show_files_listing());
            }
        }

        let path_static_files = "/{name:.(.+).js|(.+).css|(.+).ico|(.+).png|(.+).svg|(.+).html}";
        config
            .service(web::resource(path_static_files).route(web::get().to(load_static_files)))
            // Route returns index.html - FE app
            // .service(web::resource("/ind/{path_url:.*}").route(web::get().to(index_root)));
            .service(index_root);
    }
}

/// Loading the `index.html` file.
#[get("/ind/{path_url:.*}")]
pub async fn index_root(config_app: web::Data<config_app::ConfigApp>) -> Result<HttpResponse, Error> {
    let body_str = include_str!("../../static/index.html");

    let config_app = config_app.get_ref().clone();
    let app_name = format!("<title>{}</title>", &config_app.app_name);
    let body_str = body_str.replacen("<title>APP_NAME</title>", &app_name, 1);
    #[rustfmt::skip]
    let app_domain = format!("<script>var APP_DOMAIN='{}';</script>", &config_app.app_domain );
    let body_str = body_str.replacen("<script>var APP_DOMAIN;</script>", &app_domain, 1);

    let app_backend01 = "rustc v.1.97";
    let app_backend02: Vec<&str> = vec![
        "actix = \"0.13.5\"",
        "actix-broker = \"0.4.4\"",
        "actix-cors = \"0.7.1\"",
        "actix-files = \"0.6.10\"",
        "actix-multipart = \"0.8.0\"",
        "actix-web = { version = \"4.14.0\", features = [\"openssl\"] }",
        "actix-web-actors = \"4.3.1\"",
        "argon2 = \"0.5.3\"",
        "chrono = { version = \"0.4.45\", features = [\"serde\"] }",
        "dotenv = \"0.15.0\"",
        "email_address = \"0.2.9\"",
        "env_logger = \"0.11.11\"",
        "futures-util = \"0.3.33\"",
        "getrandom = \"0.4.3\"",
        "handlebars = \"6.4.3\"",
        "image = \"0.25.10\"",
        "jsonwebtoken = { version = \"11.0.0\", features = [\"aws_lc_rs\"] }",
        "lettre = { version = \"0.11.22\", features = [\"tokio1\", \"tokio1-native-tls\"] }",
        "log = \"0.4.28\"",
        "mime = \"0.3.17\"",
        "openssl = \"0.10.81\"",
        "rand = \"0.10.2\"",
        "regex = \"1.12.2\"",
        "serde = { version = \"1.0.229\", features = [\"derive\"] }",
        "serde_json = \"1.0.151\"",
        "sqlx = { version = \"0.9.0\", features = [\"chrono\", \"macros\", \"postgres\", \"runtime-tokio\"] }",
        "utoipa = { version = \"5.5.0\", features = [\"chrono\", \"actix_extras\"] }",
        "utoipa-swagger-ui = { version = \"9.0.2\", features = [\"actix-web\"] }",
        "utoipa-redoc = { version = \"6.0.0\", features = [\"actix-web\"] }",
        "utoipa-rapidoc = { version = \"6.0.0\", features = [\"actix-web\"] }",
    ];
    let app_backend03: Vec<&str> = vec!["actix-multipart-test = \"0.0.3\""];
    let app_about_s = format!(
        "<script>var APP_ABOUT={{ {},{},{} }};</script>",
        format!("'backend01':'{}'", app_backend01),
        format!("'backend02':['{}']", app_backend02.join("','")),
        format!("'backend03':['{}']", app_backend03.join("','")),
    );
    let body_str = body_str.replacen("<script>var APP_ABOUT;</script>", &app_about_s, 1);

    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(body_str))
}

pub async fn load_static_files(request: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    let file_name = get_param(request, "name");
    load_file_from_dir("static", &file_name).await
}

/// Get the value of the parameter.
fn get_param(request: HttpRequest, param_name: &str) -> String {
    let path_buf_filename: path::PathBuf = request.match_info().query(param_name).parse().unwrap();
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
