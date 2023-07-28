use std::env;

use anyhow::{anyhow, bail, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use keyring::Entry;
use modio::{Credentials, Modio};

use crate::app_data::AppData;

pub(super) async fn get_split_password() -> Result<String> {
    let keyring_entries = AppData::read().await?.keyring_entries;

    if keyring_entries == 0 {
        bail!("There are no keyring entries");
    }

    let mut password = String::new();

    #[cfg(target_family = "unix")]
    let user = env::var("USER")?;
    #[cfg(target_os = "windows")]
    let user = env::var("USERNAME")?;

    for i in 0..keyring_entries {
        password
            .push_str(&Entry::new("bonelab_mod_manager", &format!("{user}-{i}"))?.get_password()?);
    }

    Ok(password)
}

pub(super) async fn set_split_password(password: &str) -> Result<()> {
    #[cfg(target_family = "unix")]
    let user = env::var("USER")?;
    #[cfg(target_os = "windows")]
    let user = env::var("USERNAME")?;

    #[cfg(target_os = "windows")]
    let platform_limit = 2_560;
    #[cfg(target_os = "macos")]
    let platform_limit = 16_777_110;
    #[cfg(target_os = "linux")]
    let platform_limit = 32_767;

    let mut remaining_password = password;
    let mut i = 0;

    loop {
        let entry = Entry::new("bonelab_mod_manager", &format!("{user}-{i}"))?;

        if remaining_password.len() <= platform_limit {
            dbg!(remaining_password);

            entry.set_password(remaining_password)?;

            break;
        } else {
            let (under, over) = remaining_password.split_at(platform_limit);

            dbg!(under.len(), over.len());

            entry.set_password(under)?;

            remaining_password = over;

            i += 1;
        }
    }

    let mut app_data = AppData::read().await?;

    app_data.keyring_entries = i + 1;

    app_data.write().await?;

    Ok(())
}

pub(super) async fn delete_split_password() -> Result<()> {
    #[cfg(target_family = "unix")]
    let user = env::var("USER")?;
    #[cfg(target_os = "windows")]
    let user = env::var("USERNAME")?;

    let keyring_entries = AppData::read().await?.keyring_entries;

    for i in 0..keyring_entries {
        Entry::new("bonelab_mod_manager", &format!("{user}-{i}"))?.delete_password()?;
    }

    Ok(())
}

pub(super) async fn authenticate() -> Result<Modio> {
    if let Ok(modio_token) = get_split_password().await {
        return Ok(Modio::new(Credentials::with_token(
            env!("MODIO_API_KEY"),
            modio_token,
        ))?);
    }

    println!("You are not signed in");

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("How would you like to sign in?")
        .item("Send me an email code")
        .item("Let me input my token")
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let email: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Please enter your email address")
                .interact_text()?;
            let modio = Modio::new(Credentials::new(env!("MODIO_API_KEY")))?;

            modio.auth().request_code(&email).await?;

            let code: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter the code emailed to you")
                .interact_text()?;
            let credentials = modio.auth().security_code(&code).await?;

            set_split_password(
                credentials
                    .token
                    .as_ref()
                    .ok_or(anyhow!("Credentials missing token"))?
                    .value
                    .as_ref(),
            )
            .await?;

            Ok(modio.with_credentials(credentials))
        }
        1 => {
            println!("{}: there may be a bug with a dependency (dialoguer) preventing you from entering your token. Sorry!", style("Warning").yellow());

            let token = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter your token")
                .interact()?;

            set_split_password(token.as_ref()).await?;

            Ok(Modio::new(Credentials::with_token(
                env!("MODIO_API_KEY"),
                token,
            ))?)
        }
        _ => bail!("Selection has index that is more than 1"),
    }
}
