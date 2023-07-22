use std::{ffi::OsStr, path::Path};

use anyhow::{bail, Result};
use tokio::{io, process::Command};

async fn command(args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<()> {
    let output = Command::new("adb").args(args).output().await;

    if output
        .as_ref()
        .is_err_and(|err| err.kind() == io::ErrorKind::NotFound)
    {
        bail!("ADB not found, ensure that it is installed");
    }

    match std::str::from_utf8(&output?.stdout)? {
        "adb: error: failed to get feature set: no devices/emulators found\n" => bail!("No Android devices found, ensure your Quest is plugged in and you have allowed USB data transfer"),
        stdout => {
            dbg!(stdout);
            Ok(())
        }
    }
}

pub(super) async fn rm(directory: impl AsRef<Path>) -> Result<()> {
    command([
        OsStr::new("shell"),
        OsStr::new("rm"),
        OsStr::new("-r"),
        directory.as_ref().as_os_str(),
    ])
    .await
}

pub(super) async fn push(local: impl AsRef<Path>, remote: impl AsRef<Path>) -> Result<()> {
    command([
        OsStr::new("push"),
        local.as_ref().as_os_str(),
        remote.as_ref().as_os_str(),
    ])
    .await
}
