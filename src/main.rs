// TODO
// - style terminal text
//   - tasks (not subtasks) bold, success green, error red, etc.
// - modularize code into tasks
// - show progress bars for downloading and extracting
// - fix email not sending
// - fix password input freezing
// - fix failing to extract specifically my mods (maybe b/c mac?)
// - actually push files to Quest headset
// - update `installed_mods` in app data when finished installing a mod
// - fix not being able to see both Android and Windows files

mod app_data;
mod error;
mod installation;

use std::env;

use app_data::AppData;
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use error::Error;
use modio::{files::PlatformStatus, filter::In, mods, Credentials, Modio, TargetPlatform};

use crate::installation::install_mod;

const BONELAB_GAME_ID: u32 = 3809;

async fn try_sync() -> Result<(), Error> {
    let mut modio = Modio::new(env!("MODIO_API_KEY"))?;
    let mut app_data = app_data::read()?;

    match app_data.modio_token {
        Some(token) => {
            modio = modio.with_credentials(Credentials::with_token(env!("MODIO_API_KEY"), token));
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

                    modio = modio.with_credentials(Credentials::with_token(
                        env!("MODIO_API_KEY"),
                        token.clone(),
                    ));
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
    let reqwest_client = reqwest::Client::new();

    for modio_mod in subscriptions {
        println!(
            "checking subscribed mod: {} by {}",
            modio_mod.name, modio_mod.submitted_by.username
        );

        if modio_mod.id == 2380732 {
            println!("mod is... 7/11, aka hecking huge")
        } else if let Some(mod_file) = modio_mod.modfile {
            let mut approved_for_android = false;

            for platform in mod_file.platforms {
                if platform.target.display_name() == TargetPlatform::Android.display_name()
                    && platform.status == PlatformStatus::APPROVED
                {
                    approved_for_android = true;
                }
            }

            if approved_for_android {
                let app_data = app_data::read()?;

                if app_data.installed_mods.contains_key(&modio_mod.id) {
                    if let Some(mod_file_version) = mod_file.version {
                        if mod_file_version > app_data.installed_mods[&modio_mod.id].version {
                            println!("mod file is newer than intalled mod");

                            if let Err(err) =
                                install_mod(&reqwest_client, mod_file.download.binary_url).await
                            {
                                println!("failed to install mod: {err}");
                            }
                        } else {
                            println!("mod file is not newer than intalled mod");
                        }
                    } else {
                        println!("mod file does not have version");
                    }
                } else {
                    println!("mod is not installed");

                    if let Err(err) =
                        install_mod(&reqwest_client, mod_file.download.binary_url).await
                    {
                        println!("failed to install mod: {err}");
                    }
                }

                continue;
            } else {
                println!("mod file not approved for Android");
            }
        }

        println!("skipping");
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
