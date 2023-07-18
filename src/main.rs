// GOALS:
// - store a manifest file that keeps track of installed mods and their version
// - get subscribed mods from mod.io
// - install mods in appropriate directory for PC, push using ADB for Quest
// - uninstall mods if unsubscribed

mod app_data;
mod error;

use std::env;

use app_data::AppData;
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use error::Error;
use modio::{filter::In, mods, Modio};

const BONELAB_GAME_ID: u32 = 3809;

async fn try_sync() -> Result<(), Error> {
    let mut modio = Modio::new(env!("MODIO_API_KEY"))?;
    let mut app_data = app_data::read()?;

    match app_data.modio_token {
        Some(token) => {
            modio = modio.with_credentials(token);
        }
        None => {
            println!("You are not signed in");
            let authentication_choice = Select::with_theme(&ColorfulTheme::default())
                .with_prompt(
                    "Would you like to receive a code via email or enter a manually created token?",
                )
                .item("Email me a code")
                .item("Enter a token")
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

                    modio = modio.with_credentials(credentials.clone());
                    app_data.modio_token = Some(
                        credentials
                            .token
                            .expect("credentials should be token")
                            .value,
                    );

                    app_data::write(&app_data)?;
                }
                1 => {
                    let token: String = Password::with_theme(&ColorfulTheme::default())
                        .with_prompt("Please enter your token")
                        .interact()?;

                    modio = modio.with_credentials(token.clone());
                    app_data.modio_token = Some(token);

                    app_data::write(&app_data)?;
                }
                _ => unreachable!(),
            }
        }
    }

    let subscriptions = modio
        .user()
        .subscriptions(mods::filters::GameId::_in(BONELAB_GAME_ID))
        .collect()
        .await?;

    for modio_mod in subscriptions {
        dbg!(modio_mod);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = try_sync().await {
        if let Error::Modio(err) = &err {
            if err.is_auth() {
                if let Ok(app_data) = app_data::read() {
                    if let Ok(_) = app_data::write(&AppData {
                        modio_token: None,
                        ..app_data
                    }) {
                        println!("error: authentication failed, you will need to re-login");
                        return;
                    }
                }

                println!("error: authentication failed");
                return;
            }
        }

        println!("error: {err}");
        return;
    }
}
