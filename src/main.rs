mod app_data;
mod authentication;
mod installation;

use std::env;

use anyhow::{bail, Result};
use app_data::AppData;
use authentication::{authenticate, delete_password};
use console::{style, Key, Term};
#[cfg(target_os = "windows")]
use dialoguer::{theme::ColorfulTheme, Select};
#[cfg(debug_assertions)]
use env_logger::Env;
use indicatif::{MultiProgress, ProgressBar};
use installation::{install_mod, ModInstallationState};
use log::debug;
use modio::{filter::In, mods};
use tokio::{
    fs::{remove_dir_all, remove_file},
    io,
    task::JoinSet,
};

#[cfg(target_os = "windows")]
use crate::app_data::BonelabPlatform;

const BONELAB_GAME_ID: u32 = 3809;

async fn try_main() -> Result<()> {
    debug!("entering `try_main()`");

    // authenticate with mod.io
    let modio = authenticate().await?;

    // choose platform
    let mut app_data = AppData::read().await?;

    #[cfg(target_os = "windows")]
    if let None = app_data.platform {
        let select = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Which platform do you play Bonelab on?")
            .item("Windows")
            .item("Quest")
            .default(0)
            .interact()?;
        let platform = BonelabPlatform::try_from(select)?;

        app_data.platform = Some(platform);
    }

    debug!("platform chosen");

    // get subscribed mods
    let mut subscriptions = modio
        .user()
        .subscriptions(mods::filters::GameId::_in(BONELAB_GAME_ID))
        .collect()
        .await?;

    debug!("got subscribed mods");

    // remove installed mod if not subscribed
    let mut removed_mods = 0;

    for (installed_mod_id, installed_mod) in app_data.installed_mods.clone() {
        if let Err(_) = subscriptions.binary_search_by(|r#mod| r#mod.id.cmp(&installed_mod_id)) {
            let installed_mod_path = app_data.mods_dir_path()?.join(&installed_mod.folder);

            debug!(
                "removing installed mod with id `{}` and folder \"{}\"",
                installed_mod_id,
                installed_mod_path.display(),
            );

            let maybe_err;

            // If it's a code mod, attempt to remove file instead of directory
            if installed_mod_path.ends_with(".dll") {
                maybe_err = remove_file(installed_mod_path).await.err();
            } else {
                maybe_err = remove_dir_all(installed_mod_path).await.err();
            }

            if let Some(err) = maybe_err {
                if err.kind() != io::ErrorKind::NotFound {
                    bail!(err);
                }
            }

            app_data.installed_mods.remove(&installed_mod_id);

            removed_mods += 1;
        }
    }

    app_data.write().await?;
    debug!("removed unsubscribed mods");

    // spawn a task for each mod
    let mut set = JoinSet::new();
    let multi_progress = MultiProgress::new();
    let concurrent_downloads: u8 =
        if let Ok(concurrent_downloads) = env::var("BMM_CONCURRENT_DOWNLOADS") {
            concurrent_downloads.parse()?
        } else {
            4
        };

    for _ in 0..concurrent_downloads {
        if let Some(subscription) = subscriptions.pop() {
            debug!("spawning task for \"{}\"", subscription.name);
            set.spawn(install_mod(
                subscription,
                multi_progress.add(ProgressBar::new_spinner()),
                modio.clone(),
                app_data.installed_mods.clone(),
            ));
            debug!("spawned task");
        } else {
            debug!("no more subscriptions");
            break;
        }
    }

    let mut installed = 0;
    let mut updated = 0;
    let mut already_installed = 0;
    let mut failed = 0;

    while let Some(res) = set.join_next().await {
        match res?? {
            ModInstallationState::Installed => installed += 1,
            ModInstallationState::Updated => updated += 1,
            ModInstallationState::AlreadyInstalled => already_installed += 1,
            ModInstallationState::Failed => failed += 1,
            _ => unreachable!(),
        }

        debug!("task finished, another concurrent download can be made");

        if let Some(subscription) = subscriptions.pop() {
            debug!("spawning task for \"{}\"", subscription.name);
            set.spawn(install_mod(
                subscription,
                multi_progress.add(ProgressBar::new_spinner()),
                modio.clone(),
                app_data.installed_mods.clone(),
            ));
            debug!("spawned task");
        }
    }

    println!(
        "\n\n{} installed, {} updated, {} already installed, {} removed, and {} failed\n",
        style(installed).bold().green(),
        style(updated).bold().cyan(),
        style(already_installed).bold(),
        style(removed_mods).bold().yellow(),
        style(failed).bold().red(),
    );
    debug!("all done!");

    Ok(())
}

fn wait_to_quit() {
    debug!("waiting to quit");

    let term = Term::stdout();

    term.write_line(&style("Press q to quit").bold().to_string())
        .unwrap();

    loop {
        if term.read_key().unwrap() == Key::Char('q') {
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    #[cfg(not(debug_assertions))]
    env_logger::init();

    debug!("`env_logger` initialized");

    match try_main().await {
        Ok(_) => println!(
            "{}",
            style("Completed without any unrecoverable errors!")
                .bold()
                .green()
        ),
        Err(err) => {
            if let Some(err) = err.downcast_ref::<modio::Error>() {
                if err.is_auth() {
                    if let Ok(_) = delete_password().await {
                        eprintln!(
                            "{}: Authentication failed, you have been signed out",
                            style("Error").red()
                        );

                        return wait_to_quit();
                    }
                }
            }

            if let Ok(backtrace) = env::var("RUST_BACKTRACE") {
                if backtrace == "1" {
                    eprintln!(
                        "{}: {err:#}\n{}",
                        style("Error").bold().red(),
                        err.backtrace()
                    );
                    return wait_to_quit();
                }
            }

            eprintln!("{}: {err:#}", style("Error").red());
        }
    }

    wait_to_quit();

    debug!("quitting");
}
