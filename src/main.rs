mod app_data;
mod authentication;

use anyhow::Result;
use authentication::authenticate;
use console::style;

async fn try_main() -> Result<()> {
    // authenticate with mod.io
    let modio = authenticate().await?;

    println!("Successfully authenticated");

    // get subscribed mods
    // spawn a task for each mod
    // for each installed mod, check if it is subscribed
    // if not subscribed, remove

    Ok(())
}

#[tokio::main]
async fn main() {
    match try_main().await {
        Ok(_) => println!(
            "{}",
            style("Completed without any unrecoverable errors!").green()
        ),
        Err(err) => eprintln!("{}: {err:#}", style("Error").red()),
    }
}
