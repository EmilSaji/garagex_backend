use eyre::Report;
use garagex_backend::config::Config;
use garagex_backend::run;

#[actix_web::main]
async fn main() -> Result<(), Report> {
    // Initialize tracing/logging early
    tracing_subscriber::fmt::init();

    // Load config from env (uses dotenv internally)
    let cfg = Config::from_env();

    // Call the run() function in lib.rs which does the heavy lifting
    run(cfg).await.map_err(|e| eyre::eyre!(e))
}
