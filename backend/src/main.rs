use vrb_app::server_run;

// ** Funcion Main **
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Starting the web server.
    server_run().await
}
