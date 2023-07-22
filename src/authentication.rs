use anyhow::{anyhow, bail, Result};
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use modio::{Credentials, Modio};

use crate::app_data::AppData;

pub(super) async fn authenticate() -> Result<Modio> {
    let mut app_data = AppData::read().await?;

    if let Some(modio_token) = app_data.modio_token {
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

            app_data.modio_token = Some(
                credentials
                    .token
                    .as_ref()
                    .ok_or(anyhow!("Credentials missing token"))?
                    .value
                    .clone(),
            );
            app_data.write().await?;

            Ok(modio.with_credentials(credentials))
        }
        1 => {
            let token = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter your token")
                .interact()?;

            app_data.modio_token = Some(token.clone());
            app_data.write().await?;

            Ok(Modio::new(Credentials::with_token(
                env!("MODIO_API_KEY"),
                token,
            ))?)
        }
        _ => bail!("Selection has index that is more than 1"),
    }
}
