# DWLDUTIL
Utility for downloading files in parallel.

DWLDUtil is a library for downloading multiple files in parallel using the asynchronous [SMOL](https://crates.io/crates/smol) engine, it allows to verify the sha1 and sha512 of the files and displays a progress bar of the files.

## Usage
```rust
use dwldutil::{DLBuilder, DLFile, DLHashes};

// Create a new downloader
let dl = DLBuilder::new()
     // add new file to downloader
     .add_file(
     DLFile::new()
         // define file properties
         .with_path("image.png")
         .with_url("https://httpbin.org/image/png")
         .with_size(8090)
         .with_hashes(DLHashes::new()
             .sha1("379f5137831350c900e757b39e525b9db1426d53")
         )),
 );
 // Start download
dl.start();
```
*examples/image.rs*

## Downloading 10 Files at a time
to configure the downloading of multiple files at once you can use the DLStartConfig configuration, as follows
```rust
use dwldutil::DLStartConfig;
let config = DLStartConfig::new()
    .with_max_concurrent_downloads(10);
dl.start_with_config(config);
```

## Changing the progress bar style
the way to change the progress bar is like setting the maximum limit of simultaneous downloads, you use DLStartConifg, but for this setting you have to have knowledge in [indicatif](https://crates.io/crates/indicatif).
```rust
use indicatif::ProgressStyle;

let progress_style = ProgressStyle::new()
    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
    .progress_chars("#>-");

let config = DLStartConfig::new()
    .with_style(progress_style);
dl.start_with_config(config);
```
