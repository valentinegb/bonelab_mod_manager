mod app_data;
mod authentication;
mod installation;

use anyhow::Result;
use app_data::AppData;
use authentication::authenticate;
use console::style;
use futures_util::TryStreamExt;
use indicatif::{MultiProgress, ProgressBar};
use installation::install_mod;
use modio::{filter::In, mods};
use tokio::task::JoinSet;

const BONELAB_GAME_ID: u32 = 3809;

async fn try_main() -> Result<()> {
    // authenticate with mod.io
    let modio = authenticate().await?;

    // get subscribed mods
    let mut subscriptions = modio
        .user()
        .subscriptions(mods::filters::GameId::_in(BONELAB_GAME_ID))
        .iter()
        .await?;

    // spawn a task for each mod
    let installed_mods = AppData::read().await?.installed_mods;
    let mut set = JoinSet::new();
    let multi_progress = MultiProgress::new();

    while let Some(subscription) = subscriptions.try_next().await? {
        set.spawn(install_mod(
            subscription,
            multi_progress.add(ProgressBar::new_spinner()),
            modio.clone(),
            installed_mods.clone(),
        ));
    }

    while let Some(res) = set.join_next().await {
        res??;
    }

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
        Err(err) => {
            if let Some(err) = err.downcast_ref::<modio::Error>() {
                if err.is_auth() {
                    if let Ok(mut app_data) = AppData::read().await {
                        app_data.modio_token = None;

                        if let Ok(_) = app_data.write().await {
                            eprintln!(
                                "{}: Authentication failed, you have been signed out",
                                style("Error").red()
                            );

                            return;
                        }
                    }
                }
            }

            eprintln!("{}: {err:#}", style("Error").red());
        }
    }
}
