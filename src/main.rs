// TODO
// - fix email not sending
// - fix password input freezing (https://github.com/console-rs/dialoguer/issues/270)
// - actually push files to Quest headset
// - fix not being able to see both Android and Windows files (https://github.com/nickelc/modio-rs/issues/4)

mod app_data;
mod authentication;
mod installation;

use std::collections::HashMap;
use std::io;
use std::time::Duration;

use console::style;
use futures::future::try_join_all;
use indicatif::style::TemplateError;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use modio::mods::Mod;
use modio::TargetPlatform;
use modio::{filter::In, mods};
use tokio::task::JoinError;
use wrapping_error::wrapping_error;

use crate::app_data::AppData;
use crate::authentication::authenticate;
use crate::installation::install_mod;

const BONELAB_GAME_ID: u32 = 3809;

wrapping_error!(Error {
    AppData(app_data::Error),
    Authentication(authentication::Error),
    Io(io::Error),
    Modio(modio::Error),
    Join(JoinError),
    Installation(installation::Error),
    Template(TemplateError),
});

async fn check_mod(
    r#mod: Mod,
    installed_mods: &HashMap<u32, String>,
    progress: ProgressBar,
) -> Result<(), Error> {
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

async fn sync_mods() -> Result<(), Error> {
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
    main_progress.finish_with_message("done");

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = sync_mods().await {
        if let Error::Modio(err) = &err {
            if err.is_auth() {
                if let Ok(app_data) = app_data::read() {
                    if let Ok(_) = app_data::write(&AppData {
                        modio_token: None,
                        ..app_data
                    }) {
                        println!("error: authentication failed, you will need to re-login");
                        return;
                    }
                }

                println!("error: authentication failed");
                return;
            }
        }

        println!("error: {err}");
    }
}
