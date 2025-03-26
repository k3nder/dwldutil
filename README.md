# DWLDUTIL
Utility for downloading files in parallel.

DWLDUtil is a library for downloading multiple files in parallel using the asynchronous [SMOL](https://crates.io/crates/smol) engine, it allows to verify the sha1 and sha512 of the files and displays a progress bar of the files.

## Usage
```rust
use dwldutil::{Downloader, DLFile, DLHashes};

// Create a new downloader
let dl = Downloader::new()
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
let dl = dl
    .with_max_concurrent_downloads(10);
dl.start();
```

## Changing the progress bar style
the way to change the progress bar is like setting the maximum limit of simultaneous downloads, you use DLStartConifg, but for this setting you have to have knowledge in [indicatif](https://crates.io/crates/indicatif).
```rust
use indicatif::ProgressStyle;

let progress_style = ProgressStyle::new()
    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
    .progress_chars("#>-");

let dl = dl
    .with_style(progress_style);
dl.start();
```

## Descompressing Downloaded Files
if you want to decompress the downloaded file, whether it is a tar.gz or a zip file, you can do it in a simple way by activating a feature, as follows
```toml
[dependencies]
dwldutil = { version = "1.0.0", features = ["decompress", "normal_zip", "gzip"] }

```
the decompress feature is needed by both features, the normal_zip feature implements the ZIP format and the GZIP feature implements the tar.gz format.

### Decompressing zip files
```rust
use dwldutil::decompress::{ Decompressor, DLDecompressionConfig, DecompressionMethod };
use dwldutil::decompress::zip::ZipDecompressor;
use dwldutil::{Downloader, DLFile, DLHashes};

// Create a new downloader
let dl = Downloader::new()
     // add new file to downloader
     .add_file(
     DLFile::new()
         // define file properties
         .with_path("file.tar.gz")
         .with_url("https://httpbin.org/zip/file.zip")
         .with_size(8090)
         .with_hashes(DLHashes::new()
             .sha1("379f5137831350c900e757b39e525b9db1426d53")
         )),
     .with_decompression_config(DLDecompressionConfig::new(DecompressionMethod::Zip, "output_folder"))
 );
 // Start download
dl.start();
```
this will also delete the downloaded file after compressing, if you want to keep it you can use

```rust
DLDecompressionConfig::new(DecompressionMethod::Zip, "output_folder").with_delete_after(false)
```

## You have duplicate files, no problem
you can use a file storage, download the files once and use symlinks to connect everything.

the ‘cas’ feature is implemented by default

```rust
use dwldutil::{cas::DLStorage, cas::DLFile};

let storage = DLStorage::new(".objects");

let file = DLFile::new()
    .with_path("file.tar.gz")
    .with_url("https://httpbin.org/zip/file.zip")
    .with_size(8090)
    .with_hashes(DLHashes::new()
        .sha1("379f5137831350c900e757b39e525b9db1426d53")
    )
    .with_storage(storage.clone());
```

> [!WARNING]
> This needs a mandatory hash, otherwise it does not work.

## 302 Error Code
when this error occurs it usually indicates that the server is being redirected, this error has been fixed in version 1.0.0, please consider updating.

If you have already updated and the problem now appears as ‘Too many redirects’, you will have to increase the number of redirects allowed in DLStartConfig, as follows:
```rust
let dl = dl
    .with_max_redirects(10);
dl.start();
```

##
