use std::env;

use anyhow::{anyhow, bail, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use keyring::Entry;
use modio::{Credentials, Modio};

pub(super) async fn authenticate() -> Result<Modio> {
    #[cfg(target_family = "unix")]
    let user = env::var("USER");
    #[cfg(target_os = "windows")]
    let user = env::var("USERNAME");
    let entry = Entry::new("bonelab_mod_manager", &user?)?;

    if let Ok(modio_token) = entry.get_password() {
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

            entry.set_password(
                credentials
                    .token
                    .as_ref()
                    .ok_or(anyhow!("Credentials missing token"))?
                    .value
                    .as_ref(),
            )?;

            Ok(modio.with_credentials(credentials))
        }
        1 => {
            println!("{}: there may be a bug with a dependency (dialoguer) preventing you from entering your token. Sorry!", style("Warning").yellow());

            let token = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter your token")
                .interact()?;

            entry.set_password(token.as_ref())?;

            Ok(Modio::new(Credentials::with_token(
                env!("MODIO_API_KEY"),
                token,
            ))?)
        }
        _ => bail!("Selection has index that is more than 1"),
    }
}
