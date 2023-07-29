use std::{borrow::Cow, collections::HashMap, io::Cursor, path::Path, time::Duration};

use anyhow::{anyhow, Result};
use console::style;
use futures_util::TryStreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use modio::{mods::Mod, DownloadAction, Modio, TargetPlatform};
use zip::ZipArchive;

#[cfg(target_os = "windows")]
use crate::app_data::BonelabPlatform;
use crate::app_data::{AppData, InstalledMod};

pub(crate) async fn install_mod(
    r#mod: Mod,
    progress_bar: ProgressBar,
    modio: Modio,
    installed_mods: HashMap<u32, InstalledMod>,
) -> Result<bool> {
    match _install_mod(r#mod, progress_bar.clone(), modio, installed_mods).await {
        Ok(msg) => {
            progress_bar.set_style(ProgressStyle::with_template(&format!(
                "{} {{prefix}} - {{msg}} ({{elapsed}})",
                style("✔").green()
            ))?);
            progress_bar.finish_with_message(msg);

            Ok(true)
        }
        Err(err) => {
            progress_bar.set_style(ProgressStyle::with_template(&format!(
                "{} {{prefix}} - {{msg}}",
                style("✘").red()
            ))?);
            progress_bar.finish_with_message(format!("{}: {err:#}", style("Error").red()));

            Ok(false)
        }
    }
}

async fn _install_mod(
    r#mod: Mod,
    progress_bar: ProgressBar,
    modio: Modio,
    installed_mods: HashMap<u32, InstalledMod>,
) -> Result<impl Into<Cow<'static, str>>> {
    progress_bar.enable_steady_tick(Duration::from_millis(120));
    progress_bar.set_style(ProgressStyle::with_template(
        "{spinner:.cyan} {prefix} - {msg}",
    )?);
    progress_bar.set_prefix(format!("{} by {}", r#mod.name, r#mod.submitted_by.username));
    progress_bar.set_message("Checking");

    if let Some(installed_mod) = installed_mods.get(&r#mod.id) {
        if installed_mod.date_updated >= r#mod.date_updated {
            return Ok("Already installed");
        }
    }

    #[cfg(target_os = "windows")]
    let platform = AppData::read()
        .await?
        .platform
        .ok_or(anyhow!("Platform is not set"))?;
    #[cfg(target_os = "windows")]
    let target_platform = match platform {
        BonelabPlatform::Windows => TargetPlatform::Windows,
        BonelabPlatform::Quest => TargetPlatform::Android,
    }
    .display_name();
    #[cfg(target_family = "unix")]
    let target_platform = TargetPlatform::Android.display_name();

    let mut file_id = None;

    for platform in r#mod.platforms {
        if platform.target.display_name() == target_platform {
            file_id = Some(platform.modfile_id);
            break;
        }
    }

    #[cfg(target_os = "windows")]
    let file_id = file_id.ok_or(anyhow!("Mod does not have {platform} mod file"))?;
    #[cfg(target_family = "unix")]
    let file_id = file_id.ok_or(anyhow!("Mod does not have Quest mod file"))?;
    let downloader = modio
        .download(DownloadAction::File {
            game_id: r#mod.game_id,
            mod_id: r#mod.id,
            file_id,
        })
        .await?;

    progress_bar.set_style(ProgressStyle::with_template(
        "{spinner:.cyan} {prefix} - {msg} {wide_bar} {bytes}/{total_bytes} ({eta})",
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

    progress_bar.set_style(ProgressStyle::with_template(
        "{spinner:.cyan} {prefix} - {msg}",
    )?);
    progress_bar.set_message("Extracting");

    let mut archive = ZipArchive::new(Cursor::new(bytes))?;

    archive.extract(AppData::read().await?.mods_dir_path()?)?;

    let folder = Path::new(
        archive
            .file_names()
            .take(1)
            .collect::<Vec<&str>>()
            .first()
            .ok_or(anyhow!("Mod file archive is empty"))?,
    )
    .ancestors()
    .last()
    .ok_or(anyhow!(
        "File or directory in mod file archive does not have any ancestors"
    ))?
    .as_os_str()
    .to_os_string();
    let mut app_data = AppData::read().await?;

    // TODO: push to headset

    app_data.installed_mods.insert(
        r#mod.id,
        InstalledMod {
            date_updated: r#mod.date_updated,
            folder,
        },
    );
    app_data.write().await?;

    Ok("Installed")
}
