use std::env;

#[cfg(target_os = "windows")]
use crate::app_data::AppData;
use anyhow::{anyhow, bail, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
#[cfg(target_family = "unix")]
use keyring::Entry;
use log::debug;
use modio::{Credentials, Modio};

#[cfg(target_family = "unix")]
async fn get_password() -> Result<String> {
    let entry = Entry::new("bonelab_mod_manager", &env::var("USER")?)?;

    Ok(entry.get_password()?)
}

#[cfg(target_os = "windows")]
async fn get_password() -> Result<String> {
    let app_data = AppData::read().await?;

    app_data
        .modio_token
        .ok_or(anyhow!("User does not have mod.io token"))
}

#[cfg(target_family = "unix")]
async fn set_password(password: &str) -> Result<()> {
    let entry = Entry::new("bonelab_mod_manager", &env::var("USER")?)?;

    Ok(entry.set_password(password)?)
}

#[cfg(target_os = "windows")]
async fn set_password(password: &str) -> Result<()> {
    let mut app_data = AppData::read().await?;

    app_data.modio_token = Some(password.to_string());

    Ok(app_data.write().await?)
}

#[cfg(target_family = "unix")]
pub(super) async fn delete_password() -> Result<()> {
    let entry = Entry::new("bonelab_mod_manager", &env::var("USER")?)?;

    Ok(entry.delete_password()?)
}

#[cfg(target_os = "windows")]
pub(super) async fn delete_password() -> Result<()> {
    let mut app_data = AppData::read().await?;

    app_data.modio_token = None;

    Ok(app_data.write().await?)
}

pub(super) async fn authenticate() -> Result<Modio> {
    if let Ok(modio_token) = get_password().await {
        debug!("got password");

        return Ok(Modio::new(Credentials::with_token(
            env!("MODIO_API_KEY"),
            modio_token,
        ))?);
    }

    debug!("could not get password");
    println!("You are not signed in");

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("How would you like to sign in?")
        .item("Send me an email code")
        .item("Let me input my token")
        .default(0)
        .interact()?;

    match selection {
        0 => {
            debug!("user selected email code");

            let email: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Please enter your email address")
                .interact_text()?;
            let modio = Modio::new(Credentials::new(env!("MODIO_API_KEY")))?;

            debug!("created mod.io client");
            modio.auth().request_code(&email).await?;
            debug!("requested code");

            let code: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter the code emailed to you")
                .interact_text()?;
            let credentials = modio.auth().security_code(&code).await?;

            debug!("got security code");
            set_password(
                credentials
                    .token
                    .as_ref()
                    .ok_or(anyhow!("Credentials missing token"))?
                    .value
                    .as_ref(),
            )
            .await?;
            debug!("set password");

            Ok(modio.with_credentials(credentials))
        }
        1 => {
            debug!("user selected input token");
            println!("{}: there may be a bug with a dependency (dialoguer) preventing you from entering your token. Sorry!", style("Warning").yellow());

            let token = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter your token")
                .interact()?;

            set_password(token.as_ref()).await?;
            debug!("set password");

            Ok(Modio::new(Credentials::with_token(
                env!("MODIO_API_KEY"),
                token,
            ))?)
        }
        _ => bail!("Selection has index that is more than 1"),
    }
}
