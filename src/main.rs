// TODO
// - fix email not sending
// - fix password input freezing (https://github.com/console-rs/dialoguer/issues/270)
// - actually push files to Quest headset
// - fix not being able to see both Android and Windows files (https://github.com/nickelc/modio-rs/issues/4)

mod adb;
mod app_data;
mod authentication;
mod installation;

use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use console::style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use futures::future::try_join_all;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use modio::mods::Mod;
use modio::TargetPlatform;
use modio::{filter::In, mods};

use crate::authentication::authenticate;
use crate::installation::install_mod;

const BONELAB_GAME_ID: u32 = 3809;
const REMOTE_BONELAB_FILES_DIR: &str = "/sdcard/Android/data/com.StressLevelZero.BONELAB/files";

async fn check_mod(
    r#mod: Mod,
    installed_mods: &HashMap<u32, String>,
    progress: ProgressBar,
) -> Result<()> {
    progress.enable_steady_tick(Duration::from_millis(120));
    progress.set_style(ProgressStyle::with_template(
        "{spinner:.blue} {prefix} - {msg}",
    )?);
    progress.set_prefix(format!("{} by {}", r#mod.name, r#mod.submitted_by.username));

    let failed_progress_style =
        ProgressStyle::with_template(&format!("{} {{prefix}} - {{msg}}", style("✘").red()))?;

    match &r#mod.modfile {
        Some(mod_file) => {
            progress.set_message("mod has file");

            let mut supports_android = false;

            for platform in &mod_file.platforms {
                if platform.target.display_name() == TargetPlatform::Android.display_name() {
                    supports_android = true;
                }
            }

            if supports_android {
                progress.set_message("mod supports Android");

                match &mod_file.version {
                    Some(mod_version) => {
                        progress.set_message("mod has version");

                        if let Some(installed_mod_version) = installed_mods.get(&r#mod.id) {
                            progress.set_message("mod is already installed");

                            if installed_mod_version >= mod_version {
                                progress.set_style(ProgressStyle::with_template(&format!(
                                    "{} {{prefix}} - {{msg}}",
                                    style("✔").green()
                                ))?);
                                progress.finish_with_message("mod is not newer than installed mod");
                                return Ok(());
                            } else {
                                progress.set_message("mod is newer than installed mod");
                            }
                        }

                        progress.set_message("mod is not already installed");
                        install_mod(r#mod, progress).await?;
                    }
                    None => {
                        progress.set_style(failed_progress_style);
                        progress.finish_with_message("mod does not have version");
                    }
                }
            } else {
                progress.set_style(failed_progress_style);
                progress.finish_with_message("mod does not support Android");
            }
        }
        None => {
            progress.set_style(failed_progress_style);
            progress.finish_with_message("mod does not have file");
        }
    }

    Ok(())
}

async fn try_main() -> Result<()> {
    let modio = authenticate().await?;
    let multi_progress = MultiProgress::new();
    let main_progress = multi_progress.add(ProgressBar::new_spinner());

    main_progress.enable_steady_tick(Duration::from_millis(120));
    main_progress.set_style(ProgressStyle::with_template("{spinner:.blue} {msg}")?);
    main_progress.set_message("getting subscribed mods");

    let mods = modio
        .user()
        .subscriptions(mods::filters::GameId::_in(BONELAB_GAME_ID))
        .collect()
        .await?;

    main_progress.set_message("getting installed mods");

    let installed_mods = app_data::read()?.installed_mods;

    main_progress.set_message("checking subscribed mods");

    let mut tasks = Vec::new();

    for r#mod in mods {
        tasks.push(check_mod(
            r#mod,
            &installed_mods,
            multi_progress.add(ProgressBar::new_spinner()),
        ));
    }

    try_join_all(tasks).await?;

    main_progress.set_style(ProgressStyle::with_template(&format!(
        "{} {{msg}}",
        style("✔").green()
    ))?);
    main_progress.finish_with_message("checked subscribed mods");

    let push_confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to push the mod files to a Quest headset?")
        .interact()?;

    if push_confirmed {
        adb::rm(REMOTE_BONELAB_FILES_DIR.to_string() + "/Mods").await?;
        adb::push(app_data::dir_path()?.join("Mods"), REMOTE_BONELAB_FILES_DIR).await?;
    }

    println!("{} Done!", style("✔").green());

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Some(err) = try_main().await.err() {
        if let Some(err) = err.downcast_ref::<modio::Error>() {
            if err.is_auth() {
                if let Ok(mut app_data) = app_data::read() {
                    app_data.modio_token = None;

                    if let Ok(_) = app_data::write(&app_data) {
                        eprintln!(
                            "{}: Authentication failed, you will need to re-login",
                            style("error").red()
                        );
                        return;
                    }
                }

                eprintln!("{}: Authentication failed", style("error").red());
                return;
            }
        }

        eprintln!("{}: {err}", style("error").red());
    }
}
