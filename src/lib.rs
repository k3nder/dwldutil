use std::{
    fs::{self, File},
    future::Future,
    io::{Read, Write},
    path::Path,
    pin::Pin,
    sync::Arc,
};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use sha1::Digest;
use sha2::Sha512;
use smol::{Executor, io::AsyncReadExt, lock::Semaphore};

/// Struct in which all the files to be downloaded are set up
pub struct DLBuilder {
    files: Vec<DLFile>,
}

/// Struct in which they set the data in a file
pub struct DLFile {
    /// Size of the file in bytes
    pub size: u64,
    /// URL of the file to be downloaded
    pub url: String,
    /// Hashes of the file
    pub hashes: Option<DLHashes>,
    /// Path to save the file
    pub path: String,
}
/// Hashes of the file
#[derive(Debug, Clone)]
pub struct DLHashes {
    /// SHA-1 hash of the file
    pub sha1: String,
    /// SHA-512 hash of the file
    pub sha512: String,
}

impl DLHashes {
    /// Create a new instance of DLHashes
    pub fn new(sha1: &str, sha512: &str) -> Self {
        DLHashes {
            sha1: sha1.to_string(),
            sha512: sha512.to_string(),
        }
    }
    /// Get the SHA-1 hash of the file
    pub fn sha1(&self) -> &str {
        &self.sha1
    }
    /// Get the SHA-512 hash of the file
    pub fn sha512(&self) -> &str {
        &self.sha512
    }
    /// Verify the SHA-1 hash of the file
    pub fn verify_sha1(&self, file: String, file_size: u64) -> bool {
        // open the file
        let mut file = File::open(file).unwrap();
        // initialize the hasher
        let mut hasher = sha1::Sha1::new();
        // initialize the buffer
        let mut buffer = vec![0; file_size as usize];
        // number of bytes readed
        let mut total_read = 0;

        loop {
            // read the file into the buffer
            let read = file.read(&mut buffer).unwrap();
            if read == 0 {
                // if the reader is empty, file is readed
                break;
            }
            total_read += read as u64;
            // generate the hash of the buffer
            hasher.update(&buffer[..read]);
        }
        // check if the file size matches the expected size
        // check if the hashes match
        total_read == file_size && hex::encode(hasher.finalize().to_vec()) == self.sha1
    }
    /// Verify the SHA-512 hash of the file
    pub fn verify_sha512(&self, file: String, file_size: u64) -> bool {
        // open the file
        let mut file = File::open(file).unwrap();
        // initialize the hasher
        let mut hasher = Sha512::new();
        // initialize the buffer
        let mut buffer = vec![0; file_size as usize];
        // number of bytes readed
        let mut total_read = 0;

        loop {
            // read the file into the buffer
            let read = file.read(&mut buffer).unwrap();
            if read == 0 {
                // if the reader is empty, file is readed
                break;
            }
            total_read += read as u64;
            // generate the hash of the buffer
            hasher.update(&buffer[..read]);
        }
        // check if the file size matches the expected size
        // check if the hashes match
        total_read == file_size && hex::encode(hasher.finalize().to_vec()) == self.sha512
    }
    /// Verify all the hashes of the file
    pub fn verify(&self, file: String, file_size: u64) -> bool {
        self.verify_sha1(file.clone(), file_size) && self.verify_sha512(file.clone(), file_size)
    }
}
impl DLFile {
    /// Asynchronous download of the file
    pub async fn download(&self, progress: ProgressBar) -> Result<(), String> {
        // get the values of the file
        let url = self.url.clone();
        let path = self.path.clone();
        let hashes = self.hashes.clone();
        let size = self.size;
        let path_clone = self.path.clone(); // Para el mensaje de progreso

        // make the request with SURF
        let mut response = surf::get(&url).await.expect("Failed to get response");

        // sets progress bar length
        progress.set_length(size);
        // sets progress bar message
        progress.set_message(format!("Downloading {}", path_clone));
        // if the response is successful, write the file
        if response.status().is_success() {
            // create the parent directory if it doesn't exist
            let ppath = Path::new(&path);
            if let Some(parent) = ppath.parent() {
                fs::create_dir_all(parent).unwrap();
            }

            // create the file
            let mut file = File::create(path.clone()).unwrap();

            // bytes downloaded
            let mut downloaded = 0;
            // buffer of bytes in a chunk, DEFAULT = 8KB
            let mut buffer = [0; 8192];

            // read the response body
            let mut body = response.take_body();
            loop {
                match AsyncReadExt::read(&mut body, &mut buffer).await {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        // write the chunk to the file
                        file.write_all(&buffer[..n]).unwrap();
                        downloaded += n as u64;
                        // update the progress bar
                        progress.set_position(downloaded);
                    }
                    Err(e) => return Err(e.to_string()),
                }
            }
        } else {
            // if the response isn't successful, abandon the download
            progress.abandon_with_message(format!("Error: código de estado {}", response.status()));
        }

        // check the hashes if they exist
        if hashes.is_some() && !hashes.unwrap().verify(path.clone(), size) {
            // if the hash verification fails, abandon the download
            progress.abandon_with_message(format!("Hash verification failed for {}", path_clone));
            return Err("Hash verification failed".to_string());
        }

        // if the hash verification succeeds, finish the download
        progress.finish_with_message(format!("Done {}", path));
        Ok(())
    }
    /// New instance of DLFile with default values
    pub fn new() -> Self {
        DLFile {
            path: String::new(),
            url: String::new(),
            size: 0,
            hashes: None,
        }
    }
    /// Adds the path of the file to instance
    pub fn with_path(mut self, path: &str) -> Self {
        self.path = path.to_string();
        self
    }
    /// Adds the URL of the file to instance
    pub fn with_url(mut self, url: &str) -> Self {
        self.url = url.to_string();
        self
    }
    /// Adds the size of the file to instance
    pub fn with_size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }
    /// Adds the hashes of the file to instance
    pub fn with_hashes(mut self, hashes: DLHashes) -> Self {
        self.hashes = Some(hashes);
        self
    }
}
impl DLBuilder {
    /// Creates a new instance of DLBuilder
    pub fn new() -> Self {
        DLBuilder { files: Vec::new() }
    }
    /// Adds the files to instance
    pub fn with_files(mut self, files: Vec<DLFile>) -> Self {
        self.files.extend(files);
        self
    }
    /// Creates a new instance of DLBuilder with the given files
    pub fn from_files(files: Vec<DLFile>) -> Self {
        DLBuilder { files }
    }
    /// Adds a file to the instance
    pub fn add_file(mut self, file: DLFile) -> Self {
        self.files.push(file);
        self
    }
    /// Starts the download with the given configuration
    pub fn start_with_config(&self, config: DLStartConfig) {
        // create a new MultiProgress instance
        let m = MultiProgress::new();
        // create the semaphore of the maximum concurrent downloads
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_downloads));
        // create the executor
        let executor = Arc::new(Executor::new());

        // obtain the futures
        let futures: Vec<Pin<Box<dyn Future<Output = Result<(), String>>>>> = self
            .files
            .iter()
            .map(|dl_file| {
                // create the progress bar
                let progress = m.add(ProgressBar::new(0).with_style(config.style.clone()));
                // obtain the semaphore permit
                let semaphore = Arc::clone(&semaphore);
                // create the task
                let task: Pin<Box<dyn Future<Output = Result<(), String>>>> =
                    Box::pin(executor.run(async move {
                        // acquire the semaphore permit
                        let permit = semaphore.acquire().await;
                        // download the file
                        dl_file.download(progress).await?;
                        // release the semaphore permit
                        drop(permit);
                        Ok(())
                    }));
                task
            })
            .collect();

        // join all futures
        smol::block_on(async {
            futures::future::join_all(futures).await;
        });
    }
    /// Starts the download with default configuration
    pub fn start(&self) {
        // start the download with default configuration
        self.start_with_config(DLStartConfig::new());
    }
}

/// Configuration for download
pub struct DLStartConfig {
    pub max_concurrent_downloads: usize,
    pub style: ProgressStyle,
}
impl DLStartConfig {
    /// Creates a new configuration with default values
    pub fn new() -> Self {
        DLStartConfig {
            max_concurrent_downloads: 5,
            style: ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.green/red} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
        }
    }
    /// Sets the style of the progress bar
    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }
    /// Sets the maximum number of concurrent downloads
    pub fn with_max_concurrent_downloads(mut self, max_concurrent_downloads: usize) -> Self {
        self.max_concurrent_downloads = max_concurrent_downloads;
        self
    }
}
