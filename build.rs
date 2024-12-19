//! Build script for downloading and extracting Intel Pin tool.
//! This script downloads the Pin tool archive from the official Intel website,
//! verifies the signature, and extracts the contents to a specified directory.
//!
//! To-do:
//! - Implement signature verification
//! - Add support for Windows and macOS

use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use std::format;
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use tar::Archive;

/// Constants for download URLs
const PIN_URL: &str = "https://software.intel.com/sites/landingpage/pintool/downloads/pin-external-3.31-98869-gfa6f126a8-gcc-linux.tar.gz";
// const SIG_URL: &str = "https://software.intel.com/sites/landingpage/pintool/downloads/pin-external-3.31-98869-gfa6f126a8-gcc-linux.tar.gz.sig";

/// Download a file from a given URL and save it to the specified path.
fn download_file(client: &Client, url: &str, dest: &Path) -> io::Result<PathBuf> {
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

    Ok(dest.to_path_buf())
}

/// Extract a tar.gz archive to the specified destination directory.
fn extract_archive(archive_path: &Path, dest: &Path) -> io::Result<std::path::PathBuf> {
    let file = fs::File::open(archive_path)?;
    let decompressor = GzDecoder::new(file);
    let mut archive = Archive::new(decompressor);

    archive.unpack(dest)?;

    Ok(dest.to_path_buf())
}

fn download_pin() -> io::Result<PathBuf> {
    // Initialize HTTP client
    let client = Client::new();

    // Set up paths
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is not set");
    let out_dir = Path::new(&out_dir);
    let pin_archive = Path::new(&out_dir).join("pin.tar.gz");
    // let pin_sig = Path::new(&out_dir).join("pin.tar.gz.sig");

    // Ensure output directory exists
    fs::create_dir_all(&out_dir).expect("Failed to create output directory");

    // Step 1: Download files
    download_file(&client, PIN_URL, &pin_archive).expect("Failed to download PIN archive");
    // download_file(&client, SIG_URL, &pin_sig).expect("Failed to download PIN signature");

    // Step 2: Verify signature
    // To-do: Implement signature verification
    // verify_signature(&pin_archive, &pin_sig)?;

    // Step 3: Extract archive
    extract_archive(&pin_archive, &out_dir).expect("Failed to extract archive");

    // Step 4: Delete archive and signature
    fs::remove_file(&pin_archive).expect("Failed to delete Pin archive");
    // fs::remove_file(&pin_sig).expect("Failed to delete signature");

    // Step 5: Return path to extracted directory
    // find extracted directory with regex
    let directories = fs::read_dir(&out_dir).expect("Failed to read output directory");
    for entry in directories {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.is_dir() {
            let path_str = path.to_str().expect("Failed to convert path to string");
            if path_str.contains("pin-") {
                return Ok(path);
            }
        }
    }

    panic!("Failed to find extracted directory");
}

fn main() {
    let pin_dir = download_pin().expect("Failed to download and extract Intel Pin tool");

    let include_pin = pin_dir.join("source/include/pin");
    let include_pin_gen = include_pin.join("gen");
    let include_extra_components = pin_dir.join("extras/components/include");
    let include_xed = pin_dir.join("extras/xed-intel64/include/xed");
    let include_cxx = pin_dir.join("extras/cxx/include");
    let include_crt = pin_dir.join("extras/crt/include");
    let include_crt_arch = include_crt.join("arch-x86_64");
    let include_crt_kernel_uapi = include_crt.join("kernel/uapi");
    let include_crt_kernel_uapi_asm = include_crt_kernel_uapi.join("asm-x86");

    let pincrt_runtime = pin_dir.join("intel64/runtime/pincrt");
    let pincrt_runtime_begin = pincrt_runtime.join("crtbeginS.o");
    let pincrt_runtime_end = pincrt_runtime.join("crtendS.o");

    cxx_build::bridge("src/lib.rs")
        // CFLAGS
        .define("__PIN__", "1")
        .define("PIN_CRT", "1")
        .define("TARGET_IA32E", None)
        .define("HOST_IA32E", None)
        .define("TARGET_LINUX", None)
        .flag_if_supported("-funwind-tables")
        .flag_if_supported("-fasynchronous-unwind-tables")
        .flag_if_supported("-fomit-frame-pointer")
        .flag_if_supported("-fno-strict-aliasing")
        .flag_if_supported("-fno-exceptions")
        .flag_if_supported("-fno-rtti")
        .flag_if_supported("-fPIC")
        .flag_if_supported("-faligned-new")
        // INCLUDE
        .include(&include_pin)
        .include(&include_pin_gen)
        .include(&include_extra_components)
        .include(&include_xed)
        .include(&include_cxx)
        .include(&include_crt)
        .include(&include_crt_arch)
        .include(&include_crt_kernel_uapi)
        .include(&include_crt_kernel_uapi_asm)
        // LDFLAGS
        .flag_if_supported("-nostdlib")
        .flag_if_supported("-lc-dynamic")
        .flag_if_supported("-lm-dynamic")
        .flag_if_supported("-lstlport-dynamic")
        .flag_if_supported("-std=c++20")
        .flag_if_supported(format!("-L{}", pincrt_runtime.display()))
        // Others
        .flag_if_supported("-Wno-unused-parameter")
        // .object(pincrt_runtime_begin)
        // .object(pincrt_runtime_end)
        .compile("pin-rs");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-env-changed=OUT_DIR");
}
