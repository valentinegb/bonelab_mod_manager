use anyhow::Result;
use dotenvy_macro::dotenv;
use modio::{Credentials, Modio};
use tracing::{debug, error, info, instrument, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    init_tracing();

    #[cfg(debug_assertions)]
    {
        dotenv!("BONELAB_MOD_MANAGER_API_KEY");
        debug!("loaded env vars from `.env` file");
    }

    match try_main().await {
        Ok(_) => info!("executed successfully"),
        Err(error) => error!(?error, "executed unsuccessfully"),
    }
}

async fn try_main() -> Result<()> {
    authenticate().await?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            EnvFilter::try_from_default_env()
                .or_else(|_| {
                    EnvFilter::try_new(
                        #[cfg(debug_assertions)]
                        "TRACE",
                        #[cfg(not(debug_assertions))]
                        "WARN",
                    )
                })
                .unwrap(),
        )
        .init();

    debug!("tracing initialized");
}

#[instrument]
async fn authenticate() -> Result<()> {
    let modio = Modio::new(Credentials::new(env!("BONELAB_MOD_MANAGER_API_KEY")))?;

    debug!("constructed `Modio` with API key");
    warn!("not yet implemented");

    Ok(())
}
