mod error;

use std::{env, io::Cursor, path::PathBuf};

use modio::lib::Url;
use zip::ZipArchive;

use self::error::Error;

pub(super) async fn install_mod(client: &reqwest::Client, url: Url) -> Result<(), Error> {
    let bytes = client.get(url).send().await?.bytes().await?;
    let mut zip = ZipArchive::new(Cursor::new(bytes))?;

    zip.extract(PathBuf::from(env::var("HOME").unwrap()).join("Downloads/Mods"))?;

    println!("successfully installed mod");

    Ok(())
}
