use std::{
    collections::HashMap,
    env::{self, VarError},
    fs::{self, create_dir_all},
    io,
    path::PathBuf,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AppData {
    pub(crate) modio_token: Option<String>,
    pub(crate) installed_mods: HashMap<u32, (String, String)>,
}

#[cfg(target_os = "macos")]
pub(crate) fn dir_path() -> Result<PathBuf, VarError> {
    Ok(PathBuf::from(env::var("HOME")?)
        .join("Library/Application Support/com.valentinegb.bonelab_mod_manager"))
}

#[cfg(target_os = "linux")]
pub(crate) fn dir_path() -> Result<PathBuf, VarError> {
    Ok(PathBuf::from(env::var("HOME")?).join("var/lib/bonelab_mod_manager"))
}

#[cfg(target_os = "windows")]
pub(crate) fn dir_path() -> Result<PathBuf, VarError> {
    Ok(PathBuf::from(env::var("AppData")).join("bonelab_mod_manager"))
}

pub(crate) fn read() -> Result<AppData> {
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
                modio_token: Some("eyJlbmMiOiJBMTI4Q0JDLUhTMjU2IiwiYWxnIjoiUlNBLU9BRVAiLCJjdHkiOiJKV1QiLCJ6aXAiOiJERUYiLCJ4NXQiOiJzR0pJLUJhTmhlTDctUjBMejJFdlhhNlQweGs9In0.hCGxoQMmykVG2KkzUMi9_kAKjoPejRnvL4ah-wFd0Ile7VGNJ874Wy8gF5apNDFbTovAt9hXyeJOELVYMha99M1935VGbNhPIFnbtorB8KcevRICCw2IOCHrNsJUgMqrhhCH6kuUb9KIGTqCtrnzRe6KKasvzC4dhoAahbNvCLjZf3w4sDALjs1wNRz2JdKJv1xb4zCN0FLfJRcsiGR7CNrXbfi0sljMIr7s8oCE5gJumZHa0NlM3D9iuMfxxTpw5Fgxq6Gbg7CYr2XVWWlX8V1FfIN-NgNO0UDYa6EfXxu-75YhhiXIHk0PElU_ELZhhwr3YcE3yUF4Ib4YGKvmcQ.dRV2vxNCV3hHN_n0UH9_Ng.xe8cCu2jiRT-T2geJatFmqZBXG5R2LRinqiqtzP9r1sWm3dN4owW_GcBkEXRqxvmtClUtl5iwLb9cdNLQ2azQTC2sojq6R4Br3Q14Us3aqODSyYvutMuOTmfWxHjwhqJ-NNMLEnZrdpdM7hWus-mN0spysxW6JkpZOzpK_mAsM-UqjLJ1Cl_SbBhb9jZxwLBwDLNsFsFI84a5nJ3_fCk4-Ew7SEhwIrxNKHfI1hTOnWUjNr6j0IfjNfQ80aCzY7jsZMgtZ4QgDT_XvDhZpujOh-tU_tW2vvfKG0ERQxNvo6CYsQQtAUX_RNq43DjZKQmuXRKCDh0lbMQtzuD-14qTUPMFwyhfsywaDdoTV8YCZIToysCoQSumK3W34RodZ3xWqKuDoQzhcIkNcU7-aIthFANkvTDUMt4Knvl_Q43uQ8KxvtwWYplcX8wv8Y5RY36e6z30lL5t180bLArYdXWSJ8xzaPfcsMWA8fcNQJ0xiHYuG-Id6RgWXb2i6C43oi4_hpm_MZNJx-im3rIo2xBzZeqAjNhXKsoQ2EbVruXuDccm3Umn1dKKCNnHGi8p7a2onLjJYapFKNMulKNhVM5t5jCn_uEEJocBI7DsRGAv4hS5_faqLukQErrdWkhWj209XfcfjL0fXFG9KojtJhus1AGzfSymfFBY1yh9oUM0DI0jDIVwkRZjWsnWPCQUwoA-ktYW2ugmM_nwfwNMrg3bCfAfva88p_HnYGQh0lZPZKWyd3E1CTHlQMNi2TLu_H9HF9UyP-nX6CIcSwkQqcw35lFtpH5U9b14qJnFau-2H1hJuZjm-7UoJ597LGjwdyiDJerfm5U5pH7Rgm1cq9fzbeJzOj3CS31qWChq46TrRITvvRx2AtegORBVUGcMw-naKfLwBWIEdgNsR6IJmkifnwwBYr1MV1Of0Vh5uTITvnUohwTqCbfDsU1U7foG6qv9XTqbl7ruyVE29JVcCkGB-cHP6thx3yrw0-4wGu8y3kiwEKl7rWf1QyO1wHDMEz64E1GF3IME4RS2vz7S-tLtQ.Mfn1DGLOwJ4pQyJoEaIRGA".to_string()),
                installed_mods: HashMap::new(),
            })?,
        )?;
        app_data_bytes = fs::read(app_data_file_path);
    }

    Ok(postcard::from_bytes(&app_data_bytes?)?)
}

pub(crate) fn write(app_data: &AppData) -> Result<()> {
    let app_data_path = dir_path()?;

    create_dir_all(&app_data_path)?;

    let app_data_file_path = app_data_path.join("data");

    fs::write(&app_data_file_path, postcard::to_vec::<_, 2048>(app_data)?)?;

    Ok(())
}
