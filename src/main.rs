use clap::{Parser, ValueEnum};
use inquire::Confirm;
use nvda_url::{NvdaUrl, VersionType, WIN7_URL, XP_URL};
use reqwest::Client;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    fs::File,
    process::Command,
};

#[derive(Parser)]
#[command(
    name = "nvdl",
    version,
    about = "Download or retrieve the NVDA screen reader"
)]
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
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let version_type = match cli.endpoint {
        Endpoint::Stable => VersionType::Stable,
        Endpoint::Alpha => VersionType::Alpha,
        Endpoint::Beta => VersionType::Beta,
        Endpoint::Xp => {
            if cli.url {
                println!("{}", XP_URL);
            } else {
                if let Err(e) = download_and_prompt(XP_URL).await {
                    eprintln!("Error: {}", e);
                }
            }
            return;
        }
        Endpoint::Win7 => {
            if cli.url {
                println!("{}", WIN7_URL);
            } else {
                if let Err(e) = download_and_prompt(WIN7_URL).await {
                    eprintln!("Error: {}", e);
                }
            }
            return;
        }
    };

    let nvda_url = NvdaUrl::default();

    if cli.url {
        if let Err(e) = print_download_url(&nvda_url, version_type).await {
            eprintln!("Error: {}", e);
        }
    } else {
        match get_download_url(&nvda_url, version_type).await {
            Some(download_url) => {
                if let Err(e) = download_and_prompt(&download_url).await {
                    eprintln!("Error: {}", e);
                }
            }
            None => eprintln!("Failed to retrieve download URL."),
        }
    }
}

async fn print_download_url(
    nvda_url: &NvdaUrl,
    version_type: VersionType,
) -> Result<(), Box<dyn Error>> {
    if let Some(url) = nvda_url.get_url(version_type).await {
        println!("{}", url);
        Ok(())
    } else {
        Err("Failed to fetch the download URL.".into())
    }
}

async fn get_download_url(nvda_url: &NvdaUrl, version_type: VersionType) -> Option<String> {
    nvda_url.get_url(version_type).await
}

async fn download_and_prompt(url: &str) -> Result<(), Box<dyn Error>> {
    println!("Downloading...");
    let client = Client::new();
    let response = client.get(url).send().await?;
    let content = response.bytes().await?;
    let filename = url.split('/').last().unwrap_or("nvda_installer.exe");
    let mut file = File::create(filename)?;
    std::io::copy(&mut content.as_ref(), &mut file)?;
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
