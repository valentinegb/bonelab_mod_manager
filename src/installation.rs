use std::{collections::HashMap, env, fmt, io::Cursor, path::Path};

use anyhow::{anyhow, Result};
use console::Style;
use futures_util::TryStreamExt;
use indicatif::{style::TemplateError, ProgressBar, ProgressStyle};
use log::debug;
use modio::{mods::Mod, DownloadAction, Modio, TargetPlatform};
use zip::ZipArchive;

#[cfg(target_os = "windows")]
use crate::app_data::BonelabPlatform;
use crate::app_data::{AppData, InstalledMod};

#[derive(Clone, Copy)]
pub(crate) enum ModInstallationState {
    Checking,
    Downloading,
    Installing,
    Updating,
    Installed,
    Updated,
    AlreadyInstalled,
    Failed,
}

impl fmt::Display for ModInstallationState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Checking => "Checking",
                Self::Downloading => "Downloading",
                Self::Installing => "Installing",
                Self::Updating => "Updating",
                Self::Installed => "Installed",
                Self::Updated => "Updated",
                Self::AlreadyInstalled => "Already installed",
                Self::Failed => "Failed",
            }
        )
    }
}

struct ModInstallation {
    progress_bar: ProgressBar,
    state: ModInstallationState,
    bytes: u64,
    total_bytes: u64,
    name: String,
}

impl ModInstallation {
    pub(crate) fn new(name: String, progress_bar: ProgressBar) -> Result<Self, TemplateError> {
        let mut new_self = Self {
            progress_bar: progress_bar
                .with_message(name.clone())
                .with_style(Self::indeterminate_style()?),
            state: ModInstallationState::Checking,
            bytes: 0,
            total_bytes: 0,
            name,
        };

        new_self.update_state(new_self.state)?;

        debug!("create new `ModInstallation`");

        Ok(new_self)
    }

    fn update_state(&mut self, state: ModInstallationState) -> Result<(), TemplateError> {
        let doing_style = Style::new().bold().cyan();
        let done_style = Style::new().bold().green();
        let didnt_style = Style::new().bold().red();
        let state_string = state.to_string();

        self.state = state;
        debug!("set internal mod installation state");

        self.progress_bar.set_style(match state {
            ModInstallationState::Downloading => Self::bar_style()?,
            ModInstallationState::Failed => Self::error_style()?,
            _ => Self::indeterminate_style()?,
        });
        debug!("set mod installation style");

        self.progress_bar.set_prefix(match state {
            ModInstallationState::Checking
            | ModInstallationState::Downloading
            | ModInstallationState::Installing
            | ModInstallationState::Updating => doing_style.apply_to(state_string).to_string(),
            ModInstallationState::Installed
            | ModInstallationState::Updated
            | ModInstallationState::AlreadyInstalled => {
                done_style.apply_to(state_string).to_string()
            }
            ModInstallationState::Failed => didnt_style.apply_to(state_string).to_string(),
        });
        debug!("set mod installation prefix");

        match state {
            ModInstallationState::Installed
            | ModInstallationState::Updated
            | ModInstallationState::AlreadyInstalled
            | ModInstallationState::Failed => {
                self.progress_bar.finish();
                debug!("finished mod installation");
            }
            _ => (),
        }

        debug!("updated mod installation state");

        Ok(())
    }

    fn increment_bytes(&mut self, bytes: u64) {
        self.bytes += bytes;
        self.progress_bar.inc(bytes);
    }

    fn update_total_bytes(&mut self, total_bytes: u64) {
        self.total_bytes = total_bytes;
        self.progress_bar.set_length(total_bytes);
    }

    fn fail(&mut self, msg: impl fmt::Display) -> Result<(), TemplateError> {
        self.progress_bar
            .set_message(format!("{}: {msg}", self.name));
        self.update_state(ModInstallationState::Failed)?;
        debug!("failed mod installation");

        Ok(())
    }

    fn indeterminate_style() -> Result<ProgressStyle, TemplateError> {
        ProgressStyle::with_template("{prefix:>17} {wide_msg}")
    }

    fn error_style() -> Result<ProgressStyle, TemplateError> {
        ProgressStyle::with_template("{prefix:>17} {msg}")
    }

    fn bar_style() -> Result<ProgressStyle, TemplateError> {
        Ok(ProgressStyle::with_template(
            "{prefix:>17} [{bar:17}] {bytes:>11} / {total_bytes:>11}: {wide_msg}",
        )?
        .progress_chars("=> "))
    }
}

pub(crate) async fn install_mod(
    r#mod: Mod,
    progress_bar: ProgressBar,
    modio: Modio,
    installed_mods: HashMap<u64, InstalledMod>,
) -> Result<ModInstallationState> {
    let mut mod_installation = ModInstallation::new(r#mod.name.clone(), progress_bar)?;

    match _install_mod(r#mod, &mut mod_installation, modio, installed_mods).await {
        Ok(state) => {
            mod_installation.update_state(state)?;
            debug!("mod installation was okay");

            Ok(state)
        }
        Err(err) => {
            let mut msg = format!("{err:#}");

            if let Ok(backtrace) = env::var("RUST_BACKTRACE") {
                if backtrace == "1" {
                    msg += &format!("\n{}", err.backtrace());
                }
            }

            mod_installation.fail(msg)?;
            debug!("mod installation was notokay");

            Ok(ModInstallationState::Failed)
        }
    }
}

async fn _install_mod(
    r#mod: Mod,
    mod_installation: &mut ModInstallation,
    modio: Modio,
    installed_mods: HashMap<u64, InstalledMod>,
) -> Result<ModInstallationState> {
    let updating: bool;

    if let Some(installed_mod) = installed_mods.get(&r#mod.id.get()) {
        debug!("mod is already installed");

        if installed_mod.date_updated >= r#mod.date_updated {
            return Ok(ModInstallationState::AlreadyInstalled);
        } else {
            debug!("mod needs to be updated");
            updating = true;
        }
    } else {
        updating = false;
    }

    #[cfg(target_os = "windows")]
    let platform = AppData::read()
        .await?
        .platform
        .ok_or(anyhow!("Platform is not set"))?;
    #[cfg(target_os = "windows")]
    let target_platform = match platform {
        BonelabPlatform::Windows => TargetPlatform::Windows,
        BonelabPlatform::Quest => TargetPlatform::Android,
    }
    .display_name();
    #[cfg(target_family = "unix")]
    let target_platform = TargetPlatform::ANDROID.display_name();

    let mut file_id = None;

    for platform in r#mod.platforms {
        if platform.target.display_name() == target_platform {
            file_id = Some(platform.modfile_id);
            debug!("got mod file id");

            break;
        }
    }

    #[cfg(target_os = "windows")]
    let file_id = file_id.ok_or(anyhow!("Mod does not have {platform} mod file"))?;
    #[cfg(target_family = "unix")]
    let file_id = file_id.ok_or(anyhow!("Mod does not have Quest mod file"))?;
    let downloader = modio
        .download(DownloadAction::File {
            game_id: r#mod.game_id,
            mod_id: r#mod.id,
            file_id,
        })
        .await?;

    debug!("created mod downloader");
    mod_installation.update_state(ModInstallationState::Downloading)?;
    mod_installation.update_total_bytes(downloader.content_length().ok_or(anyhow!(
        "Mod file HTTP response did not provide content length"
    ))?);

    let mut stream = Box::pin(downloader.stream());
    let mut bytes = Vec::new();

    while let Some(chunk) = stream.try_next().await? {
        bytes.append(&mut chunk.to_vec());
        mod_installation.increment_bytes(chunk.len() as u64);
    }

    debug!("received all chunks");
    mod_installation.update_state(if updating {
        ModInstallationState::Updating
    } else {
        ModInstallationState::Installing
    })?;

    let mut archive = ZipArchive::new(Cursor::new(bytes))?;

    archive.extract(AppData::read().await?.mods_dir_path()?)?;
    debug!("extracted mod file");

    let archive_ancestors: Vec<&Path> = Path::new(
        archive
            .file_names()
            .next()
            .ok_or(anyhow!("Mod file archive is empty"))?,
    )
    .ancestors()
    .collect();
    let folder = archive_ancestors
        .get(archive_ancestors.len() - 2)
        .ok_or(anyhow!(
            "File or directory in mod file archive does not have any ancestors"
        ))?
        .as_os_str()
        .to_os_string();
    let mut app_data = AppData::read().await?;

    // TODO: push to headset

    app_data.installed_mods.insert(
        r#mod.id.get(),
        InstalledMod {
            date_updated: r#mod.date_updated,
            folder,
        },
    );
    app_data.write().await?;
    debug!("listed mod in app data as installed");

    Ok(if updating {
        ModInstallationState::Updated
    } else {
        ModInstallationState::Installed
    })
}
