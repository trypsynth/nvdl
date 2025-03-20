use clap::{Parser, ValueEnum};
use inquire::Confirm;
use reqwest::blocking::Client;
use serde_json::Value;
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    fs::File,
    io::copy,
    process::Command,
};

#[derive(Parser)]
#[command(
    name = "nvdl",
    version = "1.0",
    about = "Download or retrieve NVDA versions from the nvda.zip API"
)]
struct Cli {
    /// The NVDA version to retrieve (default: stable). If no argument is provided, downloads the stable version.
    #[arg(value_enum)]
    endpoint: Option<Endpoint>,
    /// Display the installer's direct download link rather than fetching the installer itself.
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

fn main() {
    let cli = Cli::parse();
    let endpoint = cli.endpoint.as_ref().unwrap_or(&Endpoint::Stable);
    let base_url = "https://nvda.zip";
    let json_url = format!("{}/{}.json", base_url, endpoint.to_string().to_lowercase());
    let client = Client::new();
    if !cli.url {
        match get_download_url(&client, &json_url) {
            Some(download_url) => {
                if let Err(e) = download_and_prompt(&client, &download_url) {
                    eprintln!("Error: {}", e);
                }
            }
            None => eprintln!("Failed to retrieve download URL."),
        }
    } else {
        match client.get(&json_url).send() {
            Ok(response) => {
                if let Ok(json) = response.json::<Value>() {
                    if let Some(url) = json.get("url").and_then(|v| v.as_str()) {
                        println!("{}", url);
                    } else {
                        eprintln!("Invalid JSON response: missing 'url' field.");
                    }
                } else {
                    eprintln!("Failed to parse JSON response.");
                }
            }
            Err(e) => eprintln!("Request failed: {}", e),
        }
    }
}

fn get_download_url(client: &Client, json_url: &str) -> Option<String> {
    match client.get(json_url).send() {
        Ok(response) => {
            if let Ok(json) = response.json::<Value>() {
                json.get("url")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

fn download_and_prompt(client: &Client, url: &str) -> Result<(), Box<dyn Error>> {
    println!("Downloading...");
    let response = client.get(url).send()?;
    let content = response.bytes()?;
    let filename = url.split('/').last().unwrap_or("nvda_installer.exe");
    let mut file = File::create(filename)?;
    copy(&mut content.as_ref(), &mut file)?;
    drop(file);
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
