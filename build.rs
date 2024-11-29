//! Build script for downloading and extracting Intel Pin tool.
//! This script downloads the Pin tool archive from the official Intel website,
//! verifies the signature, and extracts the contents to a specified directory.
//!
//! To-do:
//! - Implement signature verification
//! - Add support for Windows and macOS

use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use std::{fs, io, path::Path};
use tar::Archive;

/// Constants for download URLs
const PIN_URL: &str = "https://software.intel.com/sites/landingpage/pintool/downloads/pin-external-3.31-98869-gfa6f126a8-gcc-linux.tar.gz";
// const SIG_URL: &str = "https://software.intel.com/sites/landingpage/pintool/downloads/pin-external-3.31-98869-gfa6f126a8-gcc-linux.tar.gz.sig";

/// Download a file from a given URL and save it to the specified path.
fn download_file(client: &Client, url: &str, dest: &Path) -> io::Result<()> {
    println!("Downloading: {}", url);
    let mut response = client
        .get(url)
        .header("User-Agent", "Wget/1.21.3") // Mimic wget's User-Agent or the server may block us
        .header("Accept", "*/*") // Add Accept header for compatibility
        .send()
        .expect("Failed to send request");

    if !response.status().is_success() {
        panic!("Failed to download file: {}", response.status());
    }
    let mut file = fs::File::create(dest)?;
    io::copy(&mut response, &mut file)?;
    Ok(())
}

/// Extract a tar.gz archive to the specified destination directory.
fn extract_archive(archive_path: &Path, dest: &Path) -> io::Result<()> {
    println!(
        "Extracting archive {} to {}",
        archive_path.display(),
        dest.display()
    );
    let file = fs::File::open(archive_path)?;
    let decompressor = GzDecoder::new(file);
    let mut archive = Archive::new(decompressor);
    archive.unpack(dest)?;
    Ok(())
}

fn download_pin() -> io::Result<()> {
    // Initialize HTTP client
    let client = Client::new();

    // Set up paths
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is not set");
    let pin_archive = Path::new(&out_dir).join("pin.tar.gz");
    // let pin_sig = Path::new(&out_dir).join("pin.tar.gz.sig");

    let pin_dir = Path::new(&out_dir).join("pin");

    // Ensure output directory exists
    fs::create_dir_all(&out_dir).expect("Failed to create output directory");

    // Step 1: Download files
    download_file(&client, PIN_URL, &pin_archive).expect("Failed to download PIN archive");
    // download_file(&client, SIG_URL, &pin_sig).expect("Failed to download PIN signature");

    // Step 2: Verify signature
    // To-do: Implement signature verification
    // verify_signature(&pin_archive, &pin_sig)?;

    // Step 3: Extract archive
    extract_archive(&pin_archive, &pin_dir).expect("Failed to extract archive");

    // Step 4: Delete archive and signature
    fs::remove_file(&pin_archive).expect("Failed to delete Pin archive");
    // fs::remove_file(&pin_sig).expect("Failed to delete signature");

    println!(
        "Intel Pin tool successfully prepared in {}",
        pin_dir.display()
    );

    Ok(())
}

fn main() {
    download_pin().expect("Failed to download and extract Intel Pin tool");

    // To-do:
    // - Link the Pin tool library
}
