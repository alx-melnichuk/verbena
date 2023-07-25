// use std::{convert::Infallible, io};

use actix_files::{Files, NamedFile};
// use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
// use actix_web::{
//     error, get,
//     http::{
//         header::{self, ContentType},
//         Method, StatusCode,
//     },
//     middleware, web, App, Either, HttpRequest, HttpResponse, HttpServer, Responder, Result,
// };
// use async_stream::stream;

use actix_web::{
    App, // Either, 
    Error, get, HttpRequest, HttpResponse, HttpServer, 
    http::{
        // header::{self, ContentType},
        // Method, 
        StatusCode,
    },
    middleware::Logger, Responder, Result, // web
};
use serde_json::json;

#[get("/api/healthchecker")]
async fn health_checker_handler() -> impl Responder {
    const MESSAGE: &str = "Build Simple CRUD API Actix Web";

    HttpResponse::Ok().json(json!({"status": "success","message": MESSAGE}))
}


/// simple index handler
#[get("/404")]
async fn welcome(req: HttpRequest/*, session: Session*/) -> Result<HttpResponse> {
    println!("{req:?}");

    // // session
    // let mut counter = 1;
    // if let Some(count) = session.get::<i32>("counter")? {
    //     println!("SESSION value: {count}");
    //     counter = count + 1;
    // }

    // // set counter to session
    // session.insert("counter", counter)?;

    // response
    // Ok(HttpResponse::build(StatusCode::OK)
    //     .content_type(ContentType::plaintext())
    //     .body(include_str!("../static/welcome.html")))
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/404.html")))
}  

// async fn default_handler(req_method: Method) -> Result<impl Responder> {
//     match req_method {
//         Method::GET => {
//             let file = NamedFile::open("static/404.html")?
//                 .customize()
//                 .with_status(StatusCode::NOT_FOUND);
//             Ok(Either::Left(file))
//         }
//         _ => Ok(Either::Right(HttpResponse::MethodNotAllowed().finish())),
//     }
// }

// async fn angular_index() -> Result<actix_files::NamedFile> {
//     println!("redirect to Anfular front-end");
//     Ok(service(web::resource("/something").route(web::get().to(do_something)))::NamedFile::open("../static/index.html")?)
// }
// |index|index.html

#[get("/{filename:.(.+).js|(.+).css|favicon|favicon.ico}")]
async fn index_root_js(req: HttpRequest) -> Result<actix_files::NamedFile, Error> {
    let path: std::path::PathBuf = req.match_info().query("filename").parse().unwrap();
    println!("path: '{}'", path.display());

    let pathstr  = path.to_str().unwrap();
    let pathstr2: &str = if pathstr == "favicon" { "favicon.ico" } else { pathstr };
    // let pathstr2: &str = match pathstr {  "favicon" => "favicon.ico",    "index" => "index.html",  _ => pathstr  };
    let path2: std::path::PathBuf = ["static", pathstr2].iter().collect();
    println!("path2: '{}'", path2.display());
   
    let file = actix_files::NamedFile::open(path2)?;
    Ok(file.use_last_modified(true))
}

#[get("/")] // index.html
async fn index_root_angular(req: HttpRequest) -> Result<HttpResponse> {
    println!("{req:?}");

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html")))
}  

//
// ** Funcion Main **
//
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }
    env_logger::init();

    println!("🚀 Server started successfully");
    // log::info!("starting HTTP server at http://localhost:8080");  

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            // "healthchecker"
            .service(health_checker_handler)
            // static files
            .service(Files::new("/static", "static").show_files_listing())
            // assets files
            .service(Files::new("/assets", "static/assets").show_files_listing())
            // register simple route, handle all methods
            .service(welcome)
            // // default
            // .default_service(web::to(default_handler))
            .service(index_root_js)
            // default
            .service(index_root_angular)
            // service(web::resource("/something").route(web::get().to(do_something)))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}