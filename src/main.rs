// GOALS:
// - store a manifest file that keeps track of installed mods and their version
// - get subscribed mods from mod.io
// - install mods in appropriate directory for PC, push using ADB for Quest
// - uninstall mods if unsubscribed

use std::{env, path::PathBuf};

use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use modio::{filter::In, mods, Modio};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, create_dir_all},
    io,
};

const BONELAB_GAME_ID: u32 = 3809;

#[derive(Serialize, Deserialize)]
struct AppData {
    modio_token: Option<String>,
    installed_mods: Vec<InstalledMod>,
}

#[derive(Serialize, Deserialize)]
struct InstalledMod {
    id: u32,
    folder: (String, String),
    version: String,
}

#[cfg(target_os = "macos")]
fn app_data_path() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap())
        .join("Library/Application Support/com.valentinegb.bonelab_mod_manager")
}

#[cfg(target_os = "linux")]
fn app_data_path() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap()).join("var/lib/bonelab_mod_manager")
}

#[cfg(target_os = "windows")]
fn app_data_path() -> PathBuf {
    PathBuf::from(env::var("AppData").unwrap()).join("bonelab_mod_manager")
}

async fn read_app_data() -> AppData {
    let app_data_path = app_data_path();

    create_dir_all(&app_data_path).await.unwrap();

    let app_data_file_path = app_data_path.join("data");
    let mut app_data_bytes = fs::read(&app_data_file_path).await;

    if app_data_bytes
        .as_ref()
        .is_err_and(|err| err.kind() == io::ErrorKind::NotFound)
    {
        fs::write(
            &app_data_file_path,
            postcard::to_vec::<_, 2048>(&AppData {
                modio_token: None,
                installed_mods: Vec::new(),
            })
            .unwrap(),
        )
        .await
        .unwrap();
        app_data_bytes = fs::read(app_data_file_path).await;
    }

    postcard::from_bytes(&app_data_bytes.unwrap()).unwrap()
}

async fn write_app_data(app_data: &AppData) {
    let app_data_path = app_data_path();

    create_dir_all(&app_data_path).await.unwrap();

    let app_data_file_path = app_data_path.join("data");

    fs::write(
        &app_data_file_path,
        postcard::to_vec::<_, 2048>(app_data).unwrap(),
    )
    .await
    .unwrap();
}

#[tokio::main]
async fn main() {
    let mut modio = Modio::new(env!("MODIO_API_KEY")).unwrap();
    let app_data = read_app_data().await;

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

                    write_app_data(&AppData {
                        modio_token: Some(credentials.token.unwrap().value),
                        ..app_data
                    })
                    .await;
                }
                1 => {
                    let token: String = Password::with_theme(&ColorfulTheme::default())
                        .with_prompt("Please enter your token")
                        .interact()
                        .unwrap();

                    modio = modio.with_credentials(token.clone());

                    write_app_data(&AppData {
                        modio_token: Some(token),
                        ..app_data
                    })
                    .await;
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
        .expect("failed to get subscriptions");

    for modio_mod in subscriptions {
        dbg!(modio_mod);
    }
}
