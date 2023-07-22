use std::time::Duration;

use anyhow::{bail, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use modio::mods::Mod;
use tokio::time::sleep;

pub(crate) async fn install_mod(r#mod: Mod, progress_bar: ProgressBar) -> Result<()> {
    if let Some(err) = _install_mod(r#mod, progress_bar.clone()).await.err() {
        progress_bar.set_style(ProgressStyle::with_template(&format!(
            "{} {{prefix}} - {{msg}}",
            style("✘").red()
        ))?);
        progress_bar.finish_with_message(format!("{}: {err:#}", style("Error").red()));
    }

    Ok(())
}

async fn _install_mod(r#mod: Mod, progress_bar: ProgressBar) -> Result<()> {
    progress_bar.enable_steady_tick(Duration::from_millis(120));
    progress_bar.set_style(ProgressStyle::with_template(
        "{spinner:.blue} {prefix} - {msg}",
    )?);
    progress_bar.set_prefix(format!("{} by {}", r#mod.name, r#mod.submitted_by.username));
    progress_bar.set_message("Nothing, yet");

    sleep(Duration::from_secs(5)).await;

    bail!("This is a test error");

    Ok(())
}
