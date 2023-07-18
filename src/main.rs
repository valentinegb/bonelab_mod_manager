// GOALS:
// - store a manifest file that keeps track of installed mods and their version
// - get subscribed mods from mod.io
// - install mods in appropriate directory for PC, push using ADB for Quest
// - uninstall mods if unsubscribed

mod app_data;

use std::{env, panic};

use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use modio::{filter::In, mods, Modio};

const BONELAB_GAME_ID: u32 = 3809;

#[tokio::main]
async fn main() {
    let mut modio = Modio::new(env!("MODIO_API_KEY")).unwrap();
    let mut app_data = app_data::read().unwrap();

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
                .interact()
                .unwrap();

            match authentication_choice {
                0 => {
                    let email: String = Input::with_theme(&ColorfulTheme::default())
                        .with_prompt("Please enter your email")
                        .interact_text()
                        .unwrap();

                    modio
                        .auth()
                        .request_code(&email.trim())
                        .await
                        .expect("failed to request code");

                    let code: String = Password::with_theme(&ColorfulTheme::default())
                        .with_prompt("Enter the code emailed to you")
                        .interact()
                        .unwrap();
                    let credentials = modio
                        .auth()
                        .security_code(&code.trim())
                        .await
                        .expect("failed to get access token");

                    modio = modio.with_credentials(credentials.clone());
                    app_data.modio_token = Some(credentials.token.unwrap().value);

                    app_data::write(&app_data).unwrap();
                }
                1 => {
                    let token: String = Password::with_theme(&ColorfulTheme::default())
                        .with_prompt("Please enter your token")
                        .interact()
                        .unwrap();

                    modio = modio.with_credentials(token.clone());
                    app_data.modio_token = Some(token);

                    app_data::write(&app_data).unwrap();
                }
                _ => unreachable!(),
            }
        }
    }

    let subscriptions = modio
        .user()
        .subscriptions(mods::filters::GameId::_in(BONELAB_GAME_ID))
        .collect()
        .await
        .unwrap_or_else(|err| {
            if err.is_auth() {
                let mut app_data = app_data::read().unwrap();

                app_data.modio_token = None;

                app_data::write(&app_data).unwrap();
                panic!("failed to authenticate, you will have to re-login");
            } else {
                panic!("{err}");
            }
        });

    for modio_mod in subscriptions {
        dbg!(modio_mod);
    }
}
