use clap::{Parser, ValueEnum};
use inquire::Confirm;
use nvda_url::{NvdaUrl, VersionType, WIN7_URL, XP_URL};
use reqwest::Client;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    fs::File,
    io::Write,
    process::Command,
};

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

#[derive(ValueEnum, Clone, Debug)]
enum Endpoint {
    Stable,
    Alpha,
    Beta,
    Xp,
    Win7,
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

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

async fn handle_fixed_url(url: &str, url_only: bool) -> Result<(), Box<dyn Error>> {
    if url_only {
        println!("{}", url);
    } else {
        download_and_prompt(url).await?;
    }
    Ok(())
}

async fn print_download_url(
    nvda_url: &NvdaUrl,
    version_type: VersionType,
) -> Result<(), Box<dyn Error>> {
    let url = nvda_url
        .get_url(version_type)
        .await
        .ok_or("Failed to fetch the download URL.")?;
    println!("{}", url);
    Ok(())
}

async fn download_and_prompt(url: &str) -> Result<(), Box<dyn Error>> {
    println!("Downloading...");
    let response = Client::new().get(url).send().await?.error_for_status()?;
    let content = response.bytes().await?;
    let filename = url.split('/').last().unwrap_or("nvda_installer.exe");
    let mut file = File::create(filename)?;
    file.write_all(&content)?;
    println!("Downloaded {} to the current directory.", filename);
    if cfg!(target_os = "windows") && inquire_yes_no("Installer downloaded. Run now?", true) {
        println!("Running installer...");
        Command::new(filename).spawn()?.wait()?;
    }
    Ok(())
}

fn inquire_yes_no(prompt: &str, default_val: bool) -> bool {
    Confirm::new(prompt)
        .with_default(default_val)
        .with_placeholder("y")
        .prompt()
        .unwrap_or(false)
}
