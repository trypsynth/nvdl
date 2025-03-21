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

fn main() {
    let cli = Cli::parse();
    let base_url = "https://nvda.zip";
    let json_url = format!("{}/{}.json", base_url, cli.endpoint);
    let client = Client::new();
    if cli.url {
        if let Err(e) = print_download_url(&client, &json_url) {
            eprintln!("Error: {}", e);
        }
    } else {
        match get_download_url(&client, &json_url) {
            Some(download_url) => {
                if let Err(e) = download_and_prompt(&client, &download_url) {
                    eprintln!("Error: {}", e);
                }
            }
            None => eprintln!("Failed to retrieve download URL."),
        }
    }
}

fn print_download_url(client: &Client, json_url: &str) -> Result<(), Box<dyn Error>> {
    let json: Value = client.get(json_url).send()?.json()?;
    if let Some(url) = json.get("url").and_then(|v| v.as_str()) {
        println!("{}", url);
        Ok(())
    } else {
        Err("Invalid JSON response: missing 'url' field.".into())
    }
}

fn get_download_url(client: &Client, json_url: &str) -> Option<String> {
    client
        .get(json_url)
        .send()
        .ok()?
        .json::<Value>()
        .ok()?
        .get("url")?
        .as_str()
        .map(str::to_string)
}

fn download_and_prompt(client: &Client, url: &str) -> Result<(), Box<dyn Error>> {
    println!("Downloading...");
    let response = client.get(url).send()?;
    let content = response.bytes()?;
    let filename = url.split('/').last().unwrap_or("nvda_installer.exe");
    let mut file = File::create(filename)?;
    copy(&mut content.as_ref(), &mut file)?;
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
