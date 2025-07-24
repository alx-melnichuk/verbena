use actix_files::Files;
use actix_web::{get, http, web, HttpRequest, HttpResponse};
use std::{io::Error, path};
use vrb_tools::config_app;

use crate::profiles::{config_prfl, profile_controller};
use crate::streams::{config_strm, stream_controller};

const NAME_LOGO: &str = "name_logo";
const NAME_AVATAR: &str = "name_avatar";

pub fn configure() -> impl FnOnce(&mut web::ServiceConfig) {
    |config: &mut web::ServiceConfig| {
        #[rustfmt::skip]
        let alias_avatar = format!("{}/{{{}:.*}}", profile_controller::ALIAS_AVATAR_FILES_DIR,  NAME_AVATAR);
        #[rustfmt::skip]
        let alias_logo = format!("{}/{{{}:.*}}", stream_controller::ALIAS_LOGO_FILES_DIR, NAME_LOGO);

        config
            .service(Files::new("/static", "static").show_files_listing())
            .service(Files::new("/assets", "static/assets").show_files_listing())
            .service(web::resource(&alias_avatar).route(web::get().to(load_files_avatar)))
            .service(web::resource(&alias_logo).route(web::get().to(load_files_logo)))
            .service(web::resource("/{name1:.(.+).js|(.+).css}").route(web::get().to(load_files_js_css)))
            .service(web::resource("/{name2:.(.+).ico|(.+).png|(.+).svg}").route(web::get().to(load_files_images)))
            // Route returns index.html - FE app
            // .service(web::resource("/ind/{path_url:.*}").route(web::get().to(index_root)));
            .service(index_root);
    }
}

/// Loading the `index.html` file.
#[get("/ind/{path_url:.*}")]
pub async fn index_root(config_app: web::Data<config_app::ConfigApp>) -> Result<HttpResponse, Error> {
    let body_str = include_str!("../static/index.html");

    let config_app = config_app.get_ref().clone();
    let app_name = format!("<title>{}</title>", &config_app.app_name);
    let body_str = body_str.replacen("<title>APP_NAME</title>", &app_name, 1);
    #[rustfmt::skip]
    let app_domain = format!("<script>var APP_DOMAIN='{}';</script>", &config_app.app_domain );
    let body_str = body_str.replacen("<script>var APP_DOMAIN;</script>", &app_domain, 1);

    let app_backend01 = "rustc v.1.87";
    let app_backend02: Vec<&str> = vec![
        "actix = \"0.13.5\"",
        "actix-broker = \"0.4.3\"",
        "actix-cors = \"0.7.1\"",
        "actix-files = \"0.6.6\"",
        "actix-multipart = \"0.7.2\"",
        "actix-web = { version = \"4.11.0\", features = [\"openssl\"] }",
        "actix-web-actors = \"4.3.1\"",
        "argon2 = \"0.5.3\"",
        "chrono = { version = \"0.4.41\", features = [\"serde\"] }",
        "diesel = { version = \"2.2.10\", features = [\"postgres\", \"r2d2\", \"chrono\"] }",
        "diesel-derive-enum = { version = \"2.1.0\", features = [\"postgres\"] }",
        "diesel_migrations = \"2.2.0\"",
        "dotenv = \"0.15.0\"",
        "email_address = \"0.2.9\"",
        "env_logger = \"0.11.8\"",
        "futures-util = \"0.3.31\"",
        "handlebars = \"6.3.2\"",
        "image = \"0.25.6\"",
        "jsonwebtoken = \"9.3.1\"",
        "lettre = { version = \"0.11.16\", features = [\"tokio1\", \"tokio1-native-tls\"] }",
        "log = \"0.4.27\"",
        "mime = \"0.3.17\"",
        "openssl = \"0.10.72\"",
        "r2d2 = \"0.8.10\"",
        "rand = \"0.9.1\"",
        "regex = \"1.11.1\"",
        "serde = { version = \"1.0.219\", features = [\"derive\"] }",
        "serde_json = \"1.0.139\"",
        "utoipa = { version = \"5.3.1\", features = [\"chrono\", \"actix_extras\"] }",
        "utoipa-swagger-ui = { version = \"9.0.0\", features = [\"actix-web\"] }",
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

pub async fn load_files_logo(request: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    let config_strm = config_strm::ConfigStrm::init_by_env();
    let file_name = get_param(request, NAME_LOGO);
    load_file_from_dir(&config_strm.strm_logo_files_dir, &file_name).await
}

pub async fn load_files_avatar(request: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    let config_prfl = config_prfl::ConfigPrfl::init_by_env();
    let file_name = get_param(request, NAME_AVATAR);
    load_file_from_dir(&config_prfl.prfl_avatar_files_dir, &file_name).await
}

pub async fn load_files_js_css(request: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    let file_name = get_param(request, "name1");
    load_file_from_dir("static", &file_name).await
}

pub async fn load_files_images(request: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    let file_name = get_param(request, "name2");
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
