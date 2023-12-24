use tracing::{debug, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(
            #[cfg(debug_assertions)]
            Level::TRACE,
            #[cfg(not(debug_assertions))]
            Level::WARN,
        )
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    debug!("tracing initialized");
    warn!("not yet implemented");
}
