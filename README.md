# nvdl - NVDA Downloader
`nvdl` is a simple command-line tool that allows users to download or retrieve the latest NVDA screen reader versions from the `https://nvda.zip` API. 

## Features
- Download the latest NVDA versions (Stable, Alpha, Beta, XP, Win7).
- Retrieve the direct download link instead of downloading the installer.
- Optionally prompt the user to run the installer after download.

## Installation
Ensure you have Rust installed, then build the tool with:

```sh
cargo install --path .
```

## Usage

### Download the latest stable version:
```sh
nvdl
```

### Download a specific NVDA version:
```sh
nvdl alpha
nvdl beta
nvdl xp
nvdl win7
```

### Get the direct download URL instead of downloading:
```sh
nvdl --url
nvdl beta --url
```

### Behavior on Windows
- If run on Windows, `nvdl` will prompt the user to run the installer after downloading.

## API Endpoints Used

- `/stable.json` ? Retrieves the latest stable version.
- `/alpha.json` ? Retrieves the latest alpha version.
- `/beta.json` ? Retrieves the latest beta version.
- `/xp.json` ? Retrieves the last version for Windows XP.
- `/win7.json` ? Retrieves the last version for Windows 7 SP1 and Windows 8.0.

## License
This project is open-source and available under the Zlib License. See the LICENSE file for more details.
