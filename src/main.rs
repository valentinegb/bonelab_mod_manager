mod app_data;
mod authentication;
mod installation;

use std::env;

use anyhow::Result;
use app_data::AppData;
use authentication::{authenticate, delete_password};
use console::{style, Key, Term};
#[cfg(target_os = "windows")]
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{MultiProgress, ProgressBar};
use installation::{install_mod, ModInstallationStatus};
use modio::{filter::In, mods};
use tokio::{fs::remove_dir_all, task::JoinSet};

#[cfg(target_os = "windows")]
use crate::app_data::BonelabPlatform;

const BONELAB_GAME_ID: u32 = 3809;

async fn try_main() -> Result<()> {
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

    // get subscribed mods
    let subscriptions = modio
        .user()
        .subscriptions(mods::filters::GameId::_in(BONELAB_GAME_ID))
        .collect()
        .await?;

    // remove installed mod if not subscribed
    let mut removed_mods = 0;

    for (installed_mod_id, installed_mod) in app_data.installed_mods.clone() {
        if let Err(_) = subscriptions.binary_search_by(|r#mod| r#mod.id.cmp(&installed_mod_id)) {
            remove_dir_all(app_data.mods_dir_path()?.join(&installed_mod.folder)).await?;

            app_data.installed_mods.remove(&installed_mod_id);

            removed_mods += 1;
        }
    }

    app_data.write().await?;

    // spawn a task for each mod
    let mut set = JoinSet::new();
    let multi_progress = MultiProgress::new();

    multi_progress.set_move_cursor(true);

    for subscription in subscriptions {
        set.spawn(install_mod(
            subscription,
            multi_progress.add(ProgressBar::new_spinner()),
            modio.clone(),
            app_data.installed_mods.clone(),
        ));
    }

    let mut installed = 0;
    let mut updated = 0;
    let mut already_installed = 0;
    let mut failed = 0;

    while let Some(res) = set.join_next().await {
        match res?? {
            ModInstallationStatus::Installed => installed += 1,
            ModInstallationStatus::Updated => updated += 1,
            ModInstallationStatus::AlreadyInstalled => already_installed += 1,
            ModInstallationStatus::Failed => failed += 1,
        }
    }

    println!(
        "{} installed, {} updated, {} already installed, {} removed, and {} failed",
        style(installed).green(),
        style(updated).cyan(),
        style(already_installed).bold(),
        style(removed_mods).yellow(),
        style(failed).red(),
    );

    Ok(())
}

fn wait_to_quit() {
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
    match try_main().await {
        Ok(_) => println!(
            "{}",
            style("Completed without any unrecoverable errors!").green()
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
                    eprintln!("{}: {err:#}\n{}", style("Error").red(), err.backtrace());
                    return wait_to_quit();
                }
            }

            eprintln!("{}: {err:#}", style("Error").red());
        }
    }

    wait_to_quit()
}
