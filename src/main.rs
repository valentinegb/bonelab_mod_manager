// GOALS:
// - store a manifest file that keeps track of installed mods and their version
// - get subscribed mods from mod.io
// - install mods in appropriate directory for PC, push using ADB for Quest
// - uninstall mods if unsubscribed

use std::{env, io::stdin, path::PathBuf};

use modio::{Credentials, Modio};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, create_dir_all},
    io,
};

const BONELAB_GAME_ID: u32 = 3809;

#[derive(Serialize, Deserialize)]
struct AppData {
    modio_token: Option<String>,
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
            postcard::to_vec::<_, 32>(&AppData { modio_token: None }).unwrap(),
        )
        .await
        .unwrap();
        app_data_bytes = fs::read(app_data_file_path).await;
    }

    postcard::from_bytes(&app_data_bytes.unwrap()).unwrap()
}

#[tokio::main]
async fn main() {
    let mut modio = Modio::new(Credentials::new(env!("MODIO_API_KEY"))).unwrap();

    match read_app_data().await.modio_token {
        Some(token) => {
            modio = modio.with_credentials(token);
        }
        None => {
            println!("You are not signed in. Please enter your mod.io email:");

            let mut email = String::new();

            stdin().read_line(&mut email).unwrap();
            modio
                .auth()
                .request_code(&email.trim())
                .await
                .expect("failed to request code");
            println!("Enter the code emailed to you:");

            let mut code = String::new();

            stdin().read_line(&mut code).unwrap();

            let token = modio.auth().security_code(&code.trim()).await.unwrap();

            modio = modio.with_credentials(token);
        }
    }

    let game = modio.game(BONELAB_GAME_ID);
}
