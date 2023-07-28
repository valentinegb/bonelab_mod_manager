mod app_data;
mod authentication;
mod installation;

use std::env;

use anyhow::Result;
use app_data::AppData;
use authentication::{authenticate, delete_password};
use console::{style, Key, Term};
use indicatif::{MultiProgress, ProgressBar};
use installation::install_mod;
use modio::{filter::In, mods};
use tokio::{fs::remove_dir_all, task::JoinSet};

const BONELAB_GAME_ID: u32 = 3809;

async fn try_main() -> Result<()> {
    // authenticate with mod.io
    let modio = authenticate().await?;

    // get subscribed mods
    let subscriptions = modio
        .user()
        .subscriptions(mods::filters::GameId::_in(BONELAB_GAME_ID))
        .collect()
        .await?;

    // remove installed mod if not subscribed
    let installed_mods = AppData::read().await?.installed_mods;
    let mut removed_mods = 0;

    for (installed_mod_id, installed_mod) in &installed_mods {
        if let Err(_) = subscriptions.binary_search_by(|r#mod| r#mod.id.cmp(installed_mod_id)) {
            remove_dir_all(
                AppData::dir_path()?
                    .join("Mods")
                    .join(&installed_mod.folder),
            )
            .await?;

            removed_mods += 1;
        }
    }

    match removed_mods {
        0 => (),
        1 => println!("1 installed mod was removed because it is no longer subscribed to"),
        removed_mods => println!(
            "{removed_mods} installed mods were removed because they are no longer subscribed to"
        ),
    }

    // spawn a task for each mod
    let mut set = JoinSet::new();
    let multi_progress = MultiProgress::new();

    for subscription in subscriptions {
        set.spawn(install_mod(
            subscription,
            multi_progress.add(ProgressBar::new_spinner()),
            modio.clone(),
            installed_mods.clone(),
        ));
    }

    let mut successful = 0;
    let mut unsuccessful = 0;

    while let Some(res) = set.join_next().await {
        match res?? {
            true => successful += 1,
            false => unsuccessful += 1,
        }
    }

    println!(
        "{} successful and {} unsuccessful",
        style(successful).green(),
        style(unsuccessful).red()
    );

    Ok(())
}

fn wait_to_quit() {
    let term = Term::stdout();

    term.write_line(&style("Press q to quit").bold().to_string())
        .unwrap();

    loop {
        if term.read_key().unwrap() == Key::Char('q') {
            break;
        }
    }
}

#[tokio::main]
async fn main() {
    match try_main().await {
        Ok(_) => println!(
            "{}",
            style("Completed without any unrecoverable errors!").green()
        ),
        Err(err) => {
            if let Some(err) = err.downcast_ref::<modio::Error>() {
                if err.is_auth() {
                    if let Ok(_) = delete_password().await {
                        eprintln!(
                            "{}: Authentication failed, you have been signed out",
                            style("Error").red()
                        );

                        return wait_to_quit();
                    }
                }
            }

            if let Ok(backtrace) = env::var("RUST_BACKTRACE") {
                if backtrace == "1" {
                    eprintln!("{}: {err:#}\n{}", style("Error").red(), err.backtrace());
                    return wait_to_quit();
                }
            }

            eprintln!("{}: {err:#}", style("Error").red());
        }
    }

    wait_to_quit()
}
