use std::{
    collections::HashMap,
    env::{self, VarError},
    ffi::OsString,
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct AppData {
    #[cfg(target_os = "windows")]
    pub(crate) modio_token: Option<String>,
    pub(crate) platform: Option<BonelabPlatform>,
    pub(crate) installed_mods: HashMap<u32, InstalledMod>,
}

#[derive(Serialize, Deserialize)]
pub(crate) enum BonelabPlatform {
    Windows,
    Quest,
}

impl TryFrom<usize> for BonelabPlatform {
    type Error = anyhow::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Windows),
            1 => Ok(Self::Quest),
            other => Err(anyhow!(
                "BonelabPlatform only accepts values equal to 0 or 1, got {other}"
            )),
        }
    }
}

impl Display for BonelabPlatform {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Windows => "Windows",
                Self::Quest => "Quest",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct InstalledMod {
    pub(crate) date_updated: u64,
    pub(crate) folder: OsString,
}

impl AppData {
    #[cfg(target_os = "macos")]
    const REL_DIR_PATH: &str = "Library/Application Support/com.valentinegb.bonelab_mod_manager";
    #[cfg(target_os = "linux")]
    const REL_DIR_PATH: &str = "var/lib/bonelab_mod_manager";
    #[cfg(target_os = "windows")]
    const REL_DIR_PATH: &str = "bonelab_mod_manager";
    // TODO: add relative directory paths for Linux

    #[cfg(target_family = "unix")]
    pub(crate) fn dir_path() -> Result<PathBuf, VarError> {
        Ok(PathBuf::from(env::var("HOME")?).join(Self::REL_DIR_PATH))
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn dir_path() -> Result<PathBuf, VarError> {
        Ok(PathBuf::from(env::var("AppData")?).join(Self::REL_DIR_PATH))
    }

    fn path() -> Result<PathBuf, VarError> {
        Ok(Self::dir_path()?.join("app_data"))
    }

    async fn write_default() -> Result<Self> {
        let default = Self::default();

        default.write().await?;

        Ok(default)
    }

    pub(crate) async fn read() -> Result<Self> {
        let path = Self::path()?;

        if !fs::try_exists(&path).await? {
            return Ok(Self::write_default().await?);
        }

        let app_data = postcard::from_bytes(&fs::read(path).await?);

        if app_data
            .as_ref()
            .is_err_and(|err| *err == postcard::Error::DeserializeUnexpectedEnd)
            || app_data
                .as_ref()
                .is_err_and(|err| *err == postcard::Error::SerdeDeCustom)
        {
            return Ok(Self::write_default().await?);
        }

        Ok(app_data?)
    }

    pub(crate) async fn write(&self) -> Result<()> {
        let path = Self::path()?;

        if !fs::try_exists(&path).await? {
            fs::create_dir_all(Self::dir_path()?).await?;
        }

        fs::write(path, postcard::to_stdvec(self)?).await?;

        Ok(())
    }
}
