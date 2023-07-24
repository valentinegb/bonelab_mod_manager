use std::{borrow::Cow, collections::HashMap, time::Duration};

use anyhow::{anyhow, Result};
use console::style;
use futures_util::TryStreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use modio::{mods::Mod, DownloadAction, Modio, TargetPlatform};

use crate::app_data::{AppData, InstalledMod};

pub(crate) async fn install_mod(
    r#mod: Mod,
    progress_bar: ProgressBar,
    modio: Modio,
    installed_mods: HashMap<u32, InstalledMod>,
) -> Result<()> {
    match _install_mod(r#mod, progress_bar.clone(), modio, installed_mods).await {
        Ok(msg) => {
            progress_bar.set_style(ProgressStyle::with_template(&format!(
                "{} {{prefix}} - {{msg}} ({{elapsed}})",
                style("✔").green()
            ))?);
            progress_bar.finish_with_message(msg);
        }
        Err(err) => {
            progress_bar.set_style(ProgressStyle::with_template(&format!(
                "{} {{prefix}} - {{msg}}",
                style("✘").red()
            ))?);
            progress_bar.finish_with_message(format!("{}: {err:#}", style("Error").red()));
        }
    }

    Ok(())
}

async fn _install_mod(
    r#mod: Mod,
    progress_bar: ProgressBar,
    modio: Modio,
    installed_mods: HashMap<u32, InstalledMod>,
) -> Result<impl Into<Cow<'static, str>>> {
    progress_bar.enable_steady_tick(Duration::from_millis(120));
    progress_bar.set_style(ProgressStyle::with_template(
        "{spinner:.blue} {prefix} - {msg}",
    )?);
    progress_bar.set_prefix(format!("{} by {}", r#mod.name, r#mod.submitted_by.username));
    progress_bar.set_message("Checking");

    if let Some(installed_mod) = installed_mods.get(&r#mod.id) {
        if installed_mod.date_updated >= r#mod.date_updated {
            return Ok("Already installed");
        }
    }

    let mut file_id = None;

    for platform in r#mod.platforms {
        if platform.target.display_name() == TargetPlatform::Android.display_name() {
            file_id = Some(platform.modfile_id);
            break;
        }
    }

    let file_id = file_id.ok_or(anyhow!("Mod does not have Android mod file"))?;
    let downloader = modio
        .download(DownloadAction::File {
            game_id: r#mod.game_id,
            mod_id: r#mod.id,
            file_id,
        })
        .await?;

    progress_bar.set_style(ProgressStyle::with_template(
        "{spinner:.blue} {prefix} - {msg} {wide_bar} {bytes}/{total_bytes} ({eta})",
    )?);
    progress_bar.set_length(downloader.content_length().ok_or(anyhow!(
        "Mod file HTTP response did not provide content length"
    ))?);
    progress_bar.set_message("Downloading");

    let mut stream = Box::pin(downloader.stream());
    let mut bytes = Vec::new();

    while let Some(chunk) = stream.try_next().await? {
        bytes.append(&mut chunk.to_vec());
        progress_bar.inc(chunk.len() as u64);
    }

    // TODO: extract and push to headset

    let mut app_data = AppData::read().await?;

    app_data.installed_mods.insert(
        r#mod.id,
        InstalledMod {
            date_updated: r#mod.date_updated,
            folder: "PLACEHOLDER".to_string(),
        },
    );
    app_data.write().await?;

    Ok("Installed")
}
