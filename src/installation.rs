use std::{
    env::{self, VarError},
    io::Cursor,
    path::PathBuf,
};

use futures::StreamExt;
use indicatif::{style::TemplateError, ProgressBar, ProgressStyle};
use modio::mods::Mod;
use reqwest::get;
use wrapping_error::wrapping_error;
use zip::{result::ZipError, ZipArchive};

wrapping_error!(pub(super) Error {
    Reqwest(reqwest::Error),
    Zip(ZipError),
    Var(VarError),
    Template(TemplateError),
});

pub(super) async fn install_mod(r#mod: Mod, progress: ProgressBar) -> Result<(), Error> {
    progress.set_message(format!(
        "downloading {} by {}",
        r#mod.name, r#mod.submitted_by.username,
    ));

    let response = get(r#mod
        .modfile
        .expect("mod should have file")
        .download
        .binary_url)
    .await?;

    progress.set_length(
        response
            .content_length()
            .expect("content length should be provided"),
    );
    progress.set_style(ProgressStyle::with_template(
        "{msg} {wide_bar} {bytes}/{total_bytes} ({eta})",
    )?);

    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();

    while let Some(item) = stream.next().await {
        for byte in item? {
            bytes.push(byte);
            progress.set_position(bytes.len() as u64);
        }
    }

    progress.set_style(ProgressStyle::with_template("{msg} {spinner}")?);
    progress.set_message(format!(
        "downloaded. extracting {} by {}...",
        r#mod.name, r#mod.submitted_by.username,
    ));

    ZipArchive::new(Cursor::new(&bytes))?
        .extract(PathBuf::from(env::var("HOME")?).join("Downloads/Mods"))?;

    progress.set_style(ProgressStyle::with_template("{msg} ({elapsed})")?);
    progress.finish_with_message(format!(
        "extracted {} by {}",
        r#mod.name, r#mod.submitted_by.username,
    ));

    Ok(())

    // ##############################################################

    // let bytes = client.get(url).send().await?.bytes().await?;
    // let mut zip = ZipArchive::new(Cursor::new(bytes))?;

    // zip.extract(PathBuf::from(env::var("HOME").unwrap()).join("Downloads/Mods"))?;

    // println!("successfully installed mod");

    // Ok(())
}
