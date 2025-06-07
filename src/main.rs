//! `nvdl` - A CLI tool for downloading and retrieving NVDA screen reader versions.
//!
//! This tool allows users to download the latest NVDA versions or retrieve
//! direct download links for specific versions (stable, alpha, beta, XP, Win7).

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
use clap::{Parser, ValueEnum};
use dialoguer::Confirm;
use nvda_url::{NvdaUrl, VersionType, WIN7_URL, XP_URL};
use reqwest::Client;
use std::{error::Error, fs::File, io::Write, process::Command};

/// Defines the command-line interface for `nvdl`.
#[derive(Parser)]
#[command(name = "nvdl", version, about)]
struct Cli {
    /// The NVDA version to retrieve (default: stable).
    #[arg(value_enum, default_value_t = Endpoint::Stable)]
    endpoint: Endpoint,
    /// Display the installer's direct download link rather than downloading it.
    #[arg(short, long)]
    url: bool,
}

/// Defines the available NVDA version types that can be retrieved.
#[derive(ValueEnum, Clone, Debug)]
enum Endpoint {
    /// Stable release version.
    Stable,
    /// Snapshot alpha version.
    Alpha,
    /// Beta release version.
    Beta,
    /// The last version compatible with Windows XP.
    Xp,
    /// The last version compatible with Windows 7.
    Win7,
}

/// Main entrypoint for the `nvdl` application.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let nvda_url = NvdaUrl::default();
    match cli.endpoint {
        Endpoint::Xp => handle_fixed_url(XP_URL, cli.url).await?,
        Endpoint::Win7 => handle_fixed_url(WIN7_URL, cli.url).await?,
        _ => {
            let version_type = match cli.endpoint {
                Endpoint::Stable => VersionType::Stable,
                Endpoint::Alpha => VersionType::Alpha,
                Endpoint::Beta => VersionType::Beta,
                _ => unreachable!(),
            };
            if cli.url {
                print_download_url(&nvda_url, version_type).await?;
            } else {
                let url = nvda_url
                    .get_url(version_type)
                    .await
                    .ok_or("Failed to retrieve download URL.")?;
                download_and_prompt(&url).await?;
            }
        }
    }
    Ok(())
}

/// Handles either downloading or printing a fixed URL (e.g. Windows XP / Windows 7).
async fn handle_fixed_url(url: &str, url_only: bool) -> Result<(), Box<dyn Error>> {
    if url_only {
        println!("{url}");
    } else {
        download_and_prompt(url).await?;
    }
    Ok(())
}

/// Fetches and prinst the download URL for a particular NVDA version type.
async fn print_download_url(
    nvda_url: &NvdaUrl,
    version_type: VersionType,
) -> Result<(), Box<dyn Error>> {
    let url = nvda_url
        .get_url(version_type)
        .await
        .ok_or("Failed to fetch the download URL.")?;
    println!("{url}");
    Ok(())
}

/// Downloads the NVDA installer from a particular URL, and asks the user if they'd like to run it if they're on Windows.
async fn download_and_prompt(url: &str) -> Result<(), Box<dyn Error>> {
    println!("Downloading...");
    let response = Client::new().get(url).send().await?.error_for_status()?;
    let content = response.bytes().await?;
    let filename = url.rsplit('/').next().unwrap_or("nvda_installer.exe");
    let mut file = File::create(filename)?;
    file.write_all(&content)?;
    println!("Downloaded {filename} to the current directory.");
    if cfg!(target_os = "windows") && confirm("Installer downloaded. Run now?", true) {
        println!("Running installer...");
        Command::new(filename).spawn()?.wait()?;
    }
    Ok(())
}

/// Prompts the user with a yes/no prompt in the terminal. Returns false on error.
fn confirm(prompt: &str, default_val: bool) -> bool {
    Confirm::new()
        .with_prompt(prompt)
        .report(false)
        .default(default_val)
        .interact()
        .unwrap_or(false)
}
