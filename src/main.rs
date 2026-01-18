//! `nvdl` - A CLI tool for downloading and retrieving NVDA screen reader versions.
//!
//! This tool allows users to download the latest NVDA versions or retrieve
//! direct download links for specific versions (stable, alpha, beta, XP, Win7).

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use dialoguer::Confirm;
use nvda_url::{NvdaUrl, VersionType, WIN7_HASH, WIN7_URL, XP_HASH, XP_URL};
use reqwest::Client;
use sha1::{Digest, Sha1};
use std::{env::current_dir, fs::File, io::Write, process::Command};

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
	/// Display the installer's hash rather than downloading it.
	#[arg(short, long)]
	checksum: bool,
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

impl Endpoint {
	const fn as_version_type(&self) -> Option<VersionType> {
		match self {
			Self::Stable => Some(VersionType::Stable),
			Self::Alpha => Some(VersionType::Alpha),
			Self::Beta => Some(VersionType::Beta),
			_ => None,
		}
	}

	const fn as_fixed_version(&self) -> Option<(&'static str, &'static str)> {
		match self {
			Self::Xp => Some((XP_URL, XP_HASH)),
			Self::Win7 => Some((WIN7_URL, WIN7_HASH)),
			_ => None,
		}
	}
}

/// Main entrypoint for the `nvdl` application.
#[tokio::main]
async fn main() -> Result<()> {
	let cli = Cli::parse();
	let nvda_url = NvdaUrl::default();
	if let Some((url, hash)) = cli.endpoint.as_fixed_version() {
		handle_metadata(url, hash, cli.url, cli.checksum).await?;
	} else if let Some(version_type) = cli.endpoint.as_version_type() {
		let (url, hash) = nvda_url.get_details(version_type).await.context("Failed to retrieve download URL.")?;
		handle_metadata(&url, &hash, cli.url, cli.checksum).await?;
	}
	Ok(())
}

/// Handles either downloading NVDA or printing the download URL and/or hash.
async fn handle_metadata(url: &str, hash: &str, print_url: bool, print_hash: bool) -> Result<()> {
	if print_url && print_hash {
		println!("{url} ({hash})");
	} else if print_url {
		println!("{url}");
	} else if print_hash {
		println!("{hash}");
	} else {
		download_and_prompt(url, hash).await?;
	}
	Ok(())
}

/// Downloads the NVDA installer from a particular URL, and asks the user if they'd like to run it if they're on Windows.
async fn download_and_prompt(url: &str, hash: &str) -> Result<()> {
	let mut expected_hash = [0u8; 20];
	let compare_hashes = match base16ct::mixed::decode(hash, &mut expected_hash) {
		Err(_) => {
			if !confirm("The server returned an invalid hash. Download anyway?", false) {
				return Ok(());
			}
			false
		}
		Ok(_) => true,
	};
	println!("Downloading...");
	let response = Client::new().get(url).send().await?.error_for_status()?;
	let content = response.bytes().await?;
	let actual_hash = Sha1::digest(&content);
	if compare_hashes && actual_hash.as_slice() != expected_hash && !confirm("Hashes do not match. Save anyway?", false)
	{
		return Ok(());
	}
	let filename = url.rsplit('/').next().filter(|s| !s.is_empty()).unwrap_or("nvda_installer.exe");
	let mut file = File::create(filename)?;
	file.write_all(&content)?;
	file.sync_data()?;
	drop(file);
	println!("Downloaded {filename} to the current directory.");
	if cfg!(target_os = "windows") && confirm("Installer downloaded. Run now?", true) {
		println!("Running installer...");
		Command::new(current_dir()?.join(filename)).spawn()?.wait()?;
	}
	Ok(())
}

/// Prompts the user with a yes/no prompt in the terminal. Returns false on error.
fn confirm(prompt: &str, default_val: bool) -> bool {
	Confirm::new().with_prompt(prompt).report(false).default(default_val).interact().unwrap_or(false)
}
