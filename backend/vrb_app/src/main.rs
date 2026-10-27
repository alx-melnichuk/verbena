use vrb_app::server_run;

// ** Main Function **
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Starting the web server.
    server_run().await
}
