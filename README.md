# DWLDUTIL
Utility for downloading files in parallel.

DWLDUtil is a library for downloading multiple files in parallel using the asynchronous [SMOL](https://crates.io/crates/smol) engine, it allows to verify the sha1 and sha512 of the files and displays a progress bar of the files.

## Usage
```rust
use dwldutil::{Downloader, DLFile, DLHashes, indicator::Silent};

// Create a new downloader, with silent indicator
let dl = Downloader::<Silent>::new()
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

## Progress Bars and indicators
in order to use progress bars or indicate in some way where the download is going, you have to use flags, these flags allow you to implement events for each downloaded file, to save time, there are already two available implementations of flags:
- indicators::indicatif::Indicatf : has to be enabled with the `indicatif_indicator` feature, it allows you to use indicatif to generate progress bars.
- indicators::Silent : does not print anything
This example shows the use of indicators:
```rust
use dwldutil::{Downloader, indicator::Silent}

let downloader = Downloader::<Silent>::new();
```

if you want to implement your own indicators, you have to create two structures, one that implements IndicatorFactory and another that implements Indicator:
```rust
use dwldutil::indicator::{Indicator, IndicateSignal, IndicatorFactory};

#[derive(Debug)]
pub struct MyIndicator;
impl IndicatorFactory for MyIndicator {
    fn create_task(name: &str, size: u64) -> impl Indicator {
        MyIndicatorChild
    }
}

pub struct MyIndicatorChild;
impl Indicator for MyIndicatorChild {
    fn effect(&mut self, position: u64) {

    }
    fn signal(&mut self, signal: IndicateSignal) {
        match signal {
            IndicateSignal::Fail(err) => {
                println!("Error downloading file {}", err);
            },
            IndicateSignal::State(stat) => {
                println!("Changing downloading state of file to {}", stat);
            },
            IndicateSignal::Success() => {
                println!("Downloading file successfull");
            }
        }
    }
}

// For use
let dl = Downloader::<MyIndicator>::new();

// if you have specific properties inside the factory, and you don't want to use the default ones use:
dl.with_indicator(MyIndicator {});

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
use dwldutil::indicator::Silent;

// Create a new downloader
let dl = Downloader::<Silent>::new()
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

## 421 Error Code
Since the last update where openssl was changed to rustls this error can occur, it is a certification error, where an invalid certificate is used for a SNI, this can be fixed with the `no_static_client` feature that creates a surf client for each download.

```toml
dwldutil = { version = "2.0.4", features = ["no_static_client"] }
```
