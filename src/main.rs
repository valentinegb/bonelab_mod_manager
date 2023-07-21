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
mod installation;
mod modio;

use std::io;

use wrapping_error::wrapping_error;

use crate::app_data::AppData;
use crate::installation::install_mod;
use crate::modio::authenticate;

const BONELAB_GAME_ID: u32 = 3809;

wrapping_error!(Error {
    AppData(app_data::Error),
    Modio(modio::Error),
    Io(io::Error),
});

async fn sync_mods() -> Result<(), Error> {
    let modio = authenticate().await?;
    // let mods = get_subscribed_mods(modio).await?;
    // let installed_mods = app_data::read()?.installed_mods;

    // for r#mod in mods {
    //     let is_installed = true;
    //     let is_updated = false;

    //     if !is_installed || is_updated {
    //         install_mod(r#mod);
    //     }
    // }

    // let subscriptions = modio
    //     .user()
    //     .subscriptions(mods::filters::GameId::_in(BONELAB_GAME_ID))
    //     .collect()
    //     .await?;
    // let reqwest_client = reqwest::Client::new();

    // for modio_mod in subscriptions {
    //     println!(
    //         "checking subscribed mod: {} by {}",
    //         modio_mod.name, modio_mod.submitted_by.username
    //     );

    //     if modio_mod.id == 2380732 {
    //         println!("mod is... 7/11, aka hecking huge")
    //     } else if let Some(mod_file) = modio_mod.modfile {
    //         let mut approved_for_android = false;

    //         for platform in mod_file.platforms {
    //             if platform.target.display_name() == TargetPlatform::Android.display_name()
    //                 && platform.status == PlatformStatus::APPROVED
    //             {
    //                 approved_for_android = true;
    //             }
    //         }

    //         if approved_for_android {
    //             let app_data = app_data::read()?;

    //             if app_data.installed_mods.contains_key(&modio_mod.id) {
    //                 if let Some(mod_file_version) = mod_file.version {
    //                     if mod_file_version > app_data.installed_mods[&modio_mod.id].version {
    //                         println!("mod file is newer than intalled mod");

    //                         if let Err(err) =
    //                             install_mod(&reqwest_client, mod_file.download.binary_url).await
    //                         {
    //                             println!("failed to install mod: {err}");
    //                         }
    //                     } else {
    //                         println!("mod file is not newer than intalled mod");
    //                     }
    //                 } else {
    //                     println!("mod file does not have version");
    //                 }
    //             } else {
    //                 println!("mod is not installed");

    //                 if let Err(err) =
    //                     install_mod(&reqwest_client, mod_file.download.binary_url).await
    //                 {
    //                     println!("failed to install mod: {err}");
    //                 }
    //             }

    //             continue;
    //         } else {
    //             println!("mod file not approved for Android");
    //         }
    //     }

    //     println!("skipping");
    // }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = sync_mods().await {
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
