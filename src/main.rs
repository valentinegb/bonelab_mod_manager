// TODO
// - style terminal text
//   - tasks (not subtasks) bold, success green, error red, etc.
// - fix email not sending
// - fix password input freezing
// - fix failing to extract specifically my mods (maybe b/c mac?)
// - actually push files to Quest headset
// - update `installed_mods` in app data when finished installing a mod
// - fix not being able to see both Android and Windows files

mod app_data;
mod authentication;
mod installation;

use std::io;

use futures::future::try_join_all;
use indicatif::{MultiProgress, ProgressBar};
use modio::TargetPlatform;
use modio::{filter::In, mods};
use tokio::task::JoinError;
use wrapping_error::wrapping_error;

use crate::app_data::AppData;
use crate::authentication::authenticate;
use crate::installation::install_mod;

const BONELAB_GAME_ID: u32 = 3809;

wrapping_error!(Error {
    AppData(app_data::Error),
    Authentication(authentication::Error),
    Io(io::Error),
    Modio(modio::Error),
    Join(JoinError),
    Installation(installation::Error),
});

async fn sync_mods() -> Result<(), Error> {
    let modio = authenticate().await?;

    println!("getting subscribed mods...");

    let mods = modio
        .user()
        .subscriptions(mods::filters::GameId::_in(BONELAB_GAME_ID))
        .collect()
        .await?;

    println!("got subscribed mods\ngetting installed mods...");

    let installed_mods = app_data::read()?.installed_mods;

    println!("got installed mods\niterating over subscribed mods...");

    let mut tasks = Vec::new();
    let progress = MultiProgress::new();

    for r#mod in mods {
        println!("mod is {} by {}", r#mod.name, r#mod.submitted_by.username);

        match &r#mod.modfile {
            Some(mod_file) => {
                println!("mod has file");

                let mut supports_android = false;

                for platform in &mod_file.platforms {
                    if platform.target.display_name() == TargetPlatform::Android.display_name() {
                        supports_android = true;
                    }
                }

                if supports_android {
                    println!("mod supports Android");

                    match &mod_file.version {
                        Some(mod_version) => {
                            println!("mod has version");

                            if let Some(installed_mod) = installed_mods.get(&r#mod.id) {
                                println!("mod is already installed");

                                if installed_mod.version >= *mod_version {
                                    println!("mod is not newer than installed mod");
                                    continue;
                                } else {
                                    println!("mod is newer than installed mod");
                                }
                            }

                            println!("mod is not already installed");
                            tasks
                                .push(install_mod(r#mod, progress.add(ProgressBar::new_spinner())));
                        }
                        None => println!("mod does not have version"),
                    }
                } else {
                    println!("mod does not support Android");
                }
            }
            None => println!("mod does not have file"),
        }
    }

    try_join_all(tasks).await?;

    println!("iterated over subscribed mods\ndone");

    // ###################################################################

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
    }
}
