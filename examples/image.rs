use dwldutil::{DLFile, DLHashes, Downloader, indicator::Silent};

fn main() {
    // Create a new downloader
    let dl = Downloader::<Silent>::new()
        // add new file to downloader
        .add_file(
            DLFile::new()
                // define file properties
                .with_path("image.png")
                .with_url("https://httpbin.org/image/png")
                .with_size(8090)
                .with_hashes(DLHashes::new().sha1("379f5137831350c900e757b39e525b9db1426d53")),
        );
    // Start download
    dl.start();
}
