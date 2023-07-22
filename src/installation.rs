use std::{env::VarError, io::Cursor};

use console::style;
use futures::StreamExt;
use indicatif::{style::TemplateError, ProgressBar, ProgressStyle};
use modio::mods::Mod;
use reqwest::get;
use wrapping_error::wrapping_error;
use zip::{result::ZipError, ZipArchive};

use crate::app_data;

wrapping_error!(pub(super) Error {
    Reqwest(reqwest::Error),
    Zip(ZipError),
    Var(VarError),
    Template(TemplateError),
    AppData(app_data::Error),
});

pub(super) async fn install_mod(r#mod: Mod, progress: ProgressBar) -> Result<(), Error> {
    progress.set_message("downloading");

    let mod_file = r#mod.modfile.expect("mod should have file");
    let response = get(mod_file.download.binary_url).await?;

    progress.set_length(
        response
            .content_length()
            .expect("content length should be provided"),
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

    ZipArchive::new(Cursor::new(&bytes))?.extract(app_data::dir_path()?.join("Mods"))?;

    progress.set_style(ProgressStyle::with_template(&format!(
        "{} {{prefix}} - {{msg}} ({{elapsed}})",
        style("✔").green()
    ))?);
    progress.finish_with_message("extracted");

    let mut app_data = app_data::read()?;

    app_data.installed_mods.insert(
        r#mod.id,
        mod_file.version.expect("mod file should have version"),
    );

    app_data::write(&app_data)?;

    Ok(())
}
