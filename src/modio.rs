use std::io;

use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use modio::{Credentials, Modio};
use wrapping_error::wrapping_error;

use crate::app_data;

wrapping_error!(pub(super) Error {
    Modio(modio::Error),
    AppData(app_data::Error),
    Io(io::Error),
});

pub(super) async fn authenticate() -> Result<Modio, Error> {
    let modio = Modio::new(env!("MODIO_API_KEY"))?;
    let mut app_data = app_data::read()?;

    match app_data.modio_token {
        Some(token) => {
            Ok(modio.with_credentials(Credentials::with_token(env!("MODIO_API_KEY"), token)))
        }
        None => {
            println!("You are not signed in");
            let authentication_choice = Select::with_theme(&ColorfulTheme::default())
                .with_prompt(
                    "Would you like to receive a code via email or enter a manually created token?",
                )
                .item("Email me a code")
                .item("Enter a token")
                .default(0)
                .interact()?;

            match authentication_choice {
                0 => {
                    let email: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Please enter your email")
                        .interact_text()?;

                    modio.auth().request_code(&email.trim()).await?;

                    let code: String = Password::with_theme(&ColorfulTheme::default())
                        .with_prompt("Enter the code emailed to you")
                        .interact()?;
                    let credentials = modio.auth().security_code(&code.trim()).await?;

                    app_data.modio_token = Some(
                        credentials
                            .clone()
                            .token
                            .expect("credentials should be token")
                            .value,
                    );

                    app_data::write(&app_data)?;

                    Ok(modio.with_credentials(credentials))
                }
                1 => {
                    let token: String = Password::with_theme(&ColorfulTheme::default())
                        .with_prompt("Please enter your token")
                        .interact()?;

                    app_data.modio_token = Some(token.clone());

                    app_data::write(&app_data)?;

                    Ok(modio
                        .with_credentials(Credentials::with_token(env!("MODIO_API_KEY"), token)))
                }
                _ => unreachable!(),
            }
        }
    }
}
