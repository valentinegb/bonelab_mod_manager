mod error;

use std::{
    env::{self, VarError},
    fs::{self, create_dir_all},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use tokio::io;

pub(crate) type Error = error::Error;

#[derive(Serialize, Deserialize)]
pub(crate) struct AppData {
    pub(crate) modio_token: Option<String>,
    pub(crate) installed_mods: Vec<InstalledMod>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct InstalledMod {
    pub(crate) id: u32,
    pub(crate) folder: (String, String),
    pub(crate) version: String,
}

#[cfg(target_os = "macos")]
fn dir_path() -> Result<PathBuf, VarError> {
    Ok(PathBuf::from(env::var("HOME")?)
        .join("Library/Application Support/com.valentinegb.bonelab_mod_manager"))
}

#[cfg(target_os = "linux")]
fn dir_path() -> Result<PathBuf, VarError> {
    Ok(PathBuf::from(env::var("HOME")?).join("var/lib/bonelab_mod_manager"))
}

#[cfg(target_os = "windows")]
fn dir_path() -> Result<PathBuf, VarError> {
    Ok(PathBuf::from(env::var("AppData")).join("bonelab_mod_manager"))
}

pub(crate) fn read() -> Result<AppData, Error> {
    let app_data_path = dir_path()?;

    create_dir_all(&app_data_path)?;

    let app_data_file_path = app_data_path.join("data");
    let mut app_data_bytes = fs::read(&app_data_file_path);

    if app_data_bytes
        .as_ref()
        .is_err_and(|err| err.kind() == io::ErrorKind::NotFound)
    {
        fs::write(
            &app_data_file_path,
            postcard::to_vec::<_, 2048>(&AppData {
                modio_token: None,
                installed_mods: Vec::new(),
            })?,
        )?;
        app_data_bytes = fs::read(app_data_file_path);
    }

    Ok(postcard::from_bytes(&app_data_bytes?)?)
}

pub(crate) fn write(app_data: &AppData) -> Result<(), Error> {
    let app_data_path = dir_path()?;

    create_dir_all(&app_data_path)?;

    let app_data_file_path = app_data_path.join("data");

    fs::write(&app_data_file_path, postcard::to_vec::<_, 2048>(app_data)?)?;

    Ok(())
}
