use std::net::SocketAddr;

use kroma_backend_core::api::server::serve;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let bind =
        std::env::var("KROMA_BACKEND_BIND").unwrap_or_else(|_| String::from("127.0.0.1:8788"));
    let addr: SocketAddr = bind.parse()?;

    serve(addr).await?;
    Ok(())
}

fn init_tracing() {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .try_init();
}
