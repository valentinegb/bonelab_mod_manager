use std::{
    env::{self, VarError},
    path::PathBuf,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct AppData {
    pub(crate) modio_token: Option<String>,
}

impl AppData {
    #[cfg(target_os = "macos")]
    const REL_DIR_PATH: &str = "Library/Application Support/com.valentinegb.bonelab_mod_manager";
    // TODO: add relative directory paths for Linux and Windows

    fn dir_path() -> Result<PathBuf, VarError> {
        Ok(PathBuf::from(env::var("HOME")?).join(Self::REL_DIR_PATH))
    }

    fn path() -> Result<PathBuf, VarError> {
        Ok(Self::dir_path()?.join("app_data"))
    }

    pub(crate) async fn read() -> Result<Self> {
        let path = Self::path()?;

        if !fs::try_exists(&path).await? {
            let default = Self::default();

            default.write().await?;

            return Ok(default);
        }

        Ok(postcard::from_bytes(&fs::read(path).await?)?)
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
