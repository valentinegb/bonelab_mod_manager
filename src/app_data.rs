use std::{
    collections::HashMap,
    env::{self, VarError},
    ffi::OsString,
    path::PathBuf,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Serialize, Deserialize)]
pub(crate) struct AppData {
    pub(crate) modio_token: Option<String>,
    pub(crate) installed_mods: HashMap<u32, InstalledMod>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct InstalledMod {
    pub(crate) date_updated: u64,
    pub(crate) folder: OsString,
}

// PLEASE MAKE EXTRA EXTRA SURE THAT YOU DELETE THIS AND DELETE THE TOKEN BEFORE PUBLISHING
impl Default for AppData {
    fn default() -> Self {
        Self {
            modio_token: Some("eyJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiUlNBLU9BRVAiLCJjdHkiOiJKV1QiLCJ6aXAiOiJERUYiLCJ4NXQiOiJzR0pJLUJhTmhlTDctUjBMejJFdlhhNlQweGs9In0.lyHkLbzIvRop1xLTxdC8fviGnxVSPG_prx5Vs8vREXRO7XCj9jBRgLUGhQ22qlGMeiLv48b7dYGcFFBvX3kxWAK27uI7lt9yoJ378emSFzUDfLhrk5HMg9JvZQoZN9-1A92bHh3s7wOfXG_DGk-alXwEkkGGPsEMtcXcxop3-IQ9XrW65JqgeHK_SOfoaS8j6VCI4SkixcUXz7OBES2QvGiBfjLxeKbcp9CShu4yUIiKVBv2klYWk4O3N4hHlvBHKJuM_iE8HF7fPtWl8ovdiusvWNwwodB2idOlSpAYgZJp2QIwahUDFOK7GscDrfc2cr7OXD5DfGz_FdbNxO2U-Q.I2ioVFHrLMgeDQPGq0AX-A.h_e-1b5LqhAPFJ5kd_AEq9Mf766QuqpMw4DW4SIrBpdrNU6MyFv8PWi_FzUe5qh97y0_PXra15QGVo1J0hfXft2FJ7OXJkGJjYje7JEKUmf9wwXAQPD1CyVikxBwQ8U4kYy2bnJ7PnZh4nwPiLvLZ5ywN8zFj143Ia4Z6mSUYdHelXSPAUG_NBrtnBLujecPxduKQ4M78X4R2ssuWBVFVnLLCSmkfupMiOb4XOVuoWi5GWcjv_-29DW_jKbC3-FhBaTW5kBjhaD1kWmXMRsygZFY6HY1nte1Tb3vGeI1e903wTRWJR_7lqpK5hagkSv7uX5xvMrkIkcUol2afHbsduvOshWIjRiu0lbhv6K-53hlKHI1lEcuk7M4YOV344A_Stjp2Q4uQos4ZZfjY1bDqVAuxsPKFl_Sa9jNPl2nfm08P5hA1KRK0uK-0LLBmFym4UZs806ho1uIEm6XDV7iONTDie5ZnoL3aFqO0s0JA6ZUM-nfsdJ8LEPFyc21XEaPvOaEJGe7SrQFC34wclLsUc86-55KIH9DT7NLLIHtJa2N8CS5GjQ8OR2rV6KYabk3Zz2BNnUJqVH7wqmWaRwhti6HEI7THWLm6nFXJl29tLTfScAx6-WHLLStJDMtp-ek83zzV8Si9RhFzTnPAb7b3tEaw0cARjISpoRY7RbStVL1PtiJaHM3K7qhBiO9brF9XBCQAYkFe1NfHbWrQYE1RZ6lXmybG7ARXRjCXKwRR4UtRdtkGbWTQEMmJTD9azS5sKkhOq6ToBIgKsz4ur1E-Yg_urSbZ9mB_ObakY6Pfg11bkfZjHY8KrD0kS1Kdi1CLRk78T1_Fg8E98l6PV0nEe1LmKvKytinguyMwpDzzp2r3lTGxB4k0c6wK2fUNMC2yZUu26KhQm1YUUXE1PXIOq9KVapibuBouegFCTMT-A7jKUibxxk8OYkYkeGxaB6q2UGmD-iYG4Xr2VzebEF5LQ9G32Pp6tPBE9YcbxwBCDZpwPpPrkzv1hBpGdzgf1DSd2AawNzlk597N9Iw_LPT8A.YOAO5Gr0o9fgKbdIn7Tjow".to_string()),
            installed_mods: HashMap::default(),
        }
    }
}

impl AppData {
    #[cfg(target_os = "macos")]
    const REL_DIR_PATH: &str = "Library/Application Support/com.valentinegb.bonelab_mod_manager";
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
