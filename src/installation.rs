use std::{io::Cursor, path::Path};

use anyhow::Result;
use console::style;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use modio::mods::Mod;
use reqwest::get;
use zip::ZipArchive;

use crate::{adb, app_data};

const REMOTE_BONELAB_FILES_DIR: &str = "/sdcard/Android/data/com.StressLevelZero.BONELAB/files";

pub(super) async fn install_mod(r#mod: Mod, progress: ProgressBar) -> Result<()> {
    if let Err(err) = try_install_mod(r#mod, progress.clone()).await {
        progress.set_style(ProgressStyle::with_template(&format!(
            "{} {{prefix}} - {{msg}}",
            style("✘").red()
        ))?);
        progress.finish_with_message(err.to_string());
    }

    Ok(())
}

async fn try_install_mod(r#mod: Mod, progress: ProgressBar) -> Result<()> {
    progress.set_message("downloading");

    let mod_file = r#mod.modfile.expect("Mod should have file");
    let response = get(mod_file.download.binary_url).await?;

    progress.set_length(
        response
            .content_length()
            .expect("Content length should be provided"),
    );
    progress.set_style(ProgressStyle::with_template(
        "{spinner:.blue} {prefix} - {msg} {wide_bar} {bytes}/{total_bytes} ({eta})",
    )?);

    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();

    while let Some(item) = stream.next().await {
        for byte in item? {
            bytes.push(byte);
            progress.set_position(bytes.len() as u64);
        }
    }

    progress.set_style(ProgressStyle::with_template(
        "{spinner:.blue} {prefix} - {msg}",
    )?);
    progress.set_message("extracting");

    let mut zip = ZipArchive::new(Cursor::new(&bytes))?;

    zip.extract(app_data::dir_path()?.join("Mods"))?;

    let folder = Path::new(
        zip.file_names()
            .next()
            .expect("Mod file should contain one folder"),
    )
    .ancestors()
    .last()
    .expect("Path should have at least one ancestor")
    .to_str()
    .expect("Path should be representable with str")
    .to_string();

    progress.set_message("pushing");

    adb::rm(REMOTE_BONELAB_FILES_DIR.to_string() + "/Mods" + &folder).await?;
    adb::push(
        app_data::dir_path()?.join("Mods").join(&folder),
        REMOTE_BONELAB_FILES_DIR.to_string() + "/Mods",
    )
    .await?;

    progress.set_style(ProgressStyle::with_template(&format!(
        "{} {{prefix}} - {{msg}} ({{elapsed}})",
        style("✔").green()
    ))?);
    progress.finish_with_message("pushed");

    let mut app_data = app_data::read()?;

    app_data.installed_mods.insert(
        r#mod.id,
        (
            mod_file.version.expect("Mod file should have version"),
            folder,
        ),
    );

    app_data::write(&app_data)?;

    Ok(())
}
