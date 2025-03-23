use dwldutil::{DLBuilder, DLFile, DLHashes};

fn main() {
    // Create a new downloader
    let dl = DLBuilder::new()
        // add new file to downloader
        .add_file(
        DLFile::new()
            // define file properties
            .with_path("image.png")
            .with_url("https://httpbin.org/image/png")
            .with_size(8090)
            .with_hashes(DLHashes::new(
                "379f5137831350c900e757b39e525b9db1426d53",
                "08f9abaa583e6155ff1ae6967fc68daca42aba20a8a73775ec9d804eafc41265ac444c2aca7e0098635fa58b088f92d0447b1aa537dbcd162408089f04f260a0"
            )),
    );
    // Start download
    dl.start();
}
