use std::{
    fs::{self, File},
    future::Future,
    io::{Read, Write},
    path::Path,
    pin::Pin,
    sync::Arc,
};

use indicator::{IndicateSignal, Indicator, IndicatorFactory};
use sha1::{Digest, Sha1};
use sha2::{Sha224, Sha256, Sha384, Sha512};
use smol::{Executor, io::AsyncReadExt, lock::Semaphore};
use surf::Client;
#[cfg(feature = "cas")]
pub mod cas;
pub mod indicator;

#[cfg(feature = "decompress")]
pub mod decompress;
mod redirection_middleware;

/// Struct in which all the files to be downloaded are set up
pub struct Downloader<T: IndicatorFactory> {
    pub files: Vec<DLFile>,
    pub max_concurrent_downloads: usize,
    pub max_redirections: usize,
    indicator_factory: T,
}

/// Struct in which they set the data in a file
pub struct DLFile {
    /// Size of the file in bytes
    pub size: u64,
    /// URL of the file to be downloaded
    pub url: String,
    /// Hashes of the file
    pub hashes: DLHashes,
    /// Path to save the file
    pub path: String,
    /// decompression configuration
    #[cfg(feature = "decompress")]
    pub decompression_config: Option<decompress::DLDecompressionConfig>,
    /// Event on download completion
    pub on_download: Arc<dyn Fn(String) + Send + Sync>,
    /// Unsing CAS
    #[cfg(feature = "cas")]
    pub cas: Option<cas::DLStorage>,
}
#[derive(Debug, Clone)]
pub struct DLHashes {
    pub hashes: Vec<(DLHashType, String)>,
}
impl DLHashes {
    pub fn new() -> Self {
        Self { hashes: Vec::new() }
    }
    pub fn add_hash(mut self, hash_type: DLHashType, hash_value: String) -> Self {
        self.hashes.push((hash_type, hash_value));
        self
    }
    pub fn sha1(mut self, hash: &str) -> Self {
        self.hashes.push((DLHashType::SHA1, hash.to_string()));
        self
    }
    pub fn sha256(mut self, hash: &str) -> Self {
        self.hashes.push((DLHashType::SHA256, hash.to_string()));
        self
    }
    pub fn sha384(mut self, hash: &str) -> Self {
        self.hashes.push((DLHashType::SHA384, hash.to_string()));
        self
    }
    pub fn sha512(mut self, hash: &str) -> Self {
        self.hashes.push((DLHashType::SHA512, hash.to_string()));
        self
    }
    pub fn sha224(mut self, hash: &str) -> Self {
        self.hashes.push((DLHashType::SHA224, hash.to_string()));
        self
    }
    pub fn verify_data(&self, data: &[u8]) -> bool {
        self.hashes
            .iter()
            .find(|hashed| {
                let (typ, hash) = hashed;
                typ.verify_data(data, hash)
            })
            .is_some()
    }
    pub fn verify_str(&self, data: &str) -> bool {
        self.verify_data(data.as_bytes())
    }
    pub fn verify_file(&self, path: &str) -> bool {
        let data = std::fs::read(path).unwrap();
        self.verify_data(&data)
    }
}

#[derive(Debug, Clone)]
pub enum DLHashType {
    SHA1,
    SHA256,
    SHA224,
    SHA384,
    SHA512,
}

impl DLHashType {
    /// Función genérica que crea el hasher, actualiza con los datos y devuelve el hash en hexadecimal.
    fn compute_hash<D: Digest + Default>(data: &[u8]) -> String {
        let mut hasher = D::default();
        hasher.update(data);
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Calcula el hash usando el algoritmo seleccionado.
    pub fn compute(&self, data: &[u8]) -> String {
        match self {
            DLHashType::SHA1 => Self::compute_hash::<Sha1>(data),
            DLHashType::SHA256 => Self::compute_hash::<Sha256>(data),
            DLHashType::SHA224 => Self::compute_hash::<Sha224>(data),
            DLHashType::SHA384 => Self::compute_hash::<Sha384>(data),
            DLHashType::SHA512 => Self::compute_hash::<Sha512>(data),
        }
    }
    pub fn verify_str(&self, data: &str, hash: &str) -> bool {
        self.compute(data.to_string().as_bytes()) == hash
    }
    pub fn verify_data(&self, data: &[u8], hash: &str) -> bool {
        self.compute(data) == hash
    }
    pub fn verify_file(&self, path: &Path, hash: &str) -> bool {
        let mut buffer = Vec::new();
        File::open(path)
            .expect("Failed to open file")
            .read_to_end(&mut buffer)
            .expect("Failed to read file");
        self.verify_data(buffer.as_slice(), hash)
    }
}

fn symlink_exists(path: &Path) -> bool {
    match fs::symlink_metadata(path) {
        Ok(metadata) => metadata.file_type().is_symlink(),
        Err(_) => false,
    }
}

impl DLFile {
    /// Asynchronous download of the file
    pub async fn download(
        &self,
        mut indicator: impl Indicator,
        client: Client,
    ) -> Result<(), String> {
        // get the values of the file
        let url = self.url.clone();
        let path = self.path.clone();
        let hashes = self.hashes.clone();
        let size = self.size;
        let path_clone = self.path.clone(); // Para el mensaje de progreso

        // make the request with SURF
        let mut response = client.get(&url).await.expect("Failed to get response");

        // if the response is successful, write the file
        let path_hash: String = if response.status().is_success() {
            // create the parent directory if it doesn't exist
            let ppath = Path::new(&path);
            if let Some(parent) = ppath.parent() {
                fs::create_dir_all(parent).unwrap();
            }

            // create the file
            let (mut file, path_hash) = if self.cas.is_some() && hashes.hashes.len() > 0 {
                let hash = hashes.hashes.get(0).unwrap().clone().1;
                let storage = self.cas.as_ref().unwrap();
                if storage.find(hash.as_str()).is_some() {
                    if !symlink_exists(Path::new(path.clone().as_str())) {
                        storage.symlink(hash.as_str(), path.clone().as_str());
                    }
                    indicator.signal(IndicateSignal::Success());
                    indicator.effect(size);
                    return Ok(());
                }
                (
                    storage.new_file(hash.as_str(), path.clone().as_str()),
                    storage.path(hash.as_str()),
                )
            } else {
                (File::create(path.clone()).unwrap(), path.clone())
            };

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
                        indicator.effect(downloaded);
                    }
                    Err(e) => return Err(e.to_string()),
                }
            }
            path_hash
        } else {
            // if the response isn't successful, abandon the download
            indicator.signal(IndicateSignal::Fail(response.status().to_string()));
            String::new()
        };

        // check the hashes if they exist
        if hashes.hashes.len() > 0 && !hashes.verify_file(&path_hash) {
            // if the hash verification fails, abandon the download
            indicator.signal(IndicateSignal::Fail(format!(
                "Hash verification failed for {}",
                path_clone
            )));
            return Err("Hash verification failed".to_string());
        }

        // call the on_download event
        (self.on_download)(path_clone.clone());

        #[cfg(feature = "decompress")]
        {
            if self.decompression_config.is_some() {
                indicator.signal(IndicateSignal::State("Decompressing...".to_string()));
                let config = self.decompression_config.as_ref().unwrap();
                config.decompress(&path_clone)?;

                if config.delete_after {
                    indicator.signal(IndicateSignal::State("Cleaning up...".to_string()));
                    std::fs::remove_file(&path_clone).expect("Failed to delete file");
                }
            }
        }

        // if the hash verification succeeds, finish the download
        indicator.signal(IndicateSignal::Success());
        Ok(())
    }
    /// New instance of DLFile with default values
    pub fn new() -> Self {
        DLFile {
            path: String::new(),
            url: String::new(),
            size: 0,
            hashes: DLHashes::new(),
            #[cfg(feature = "decompress")]
            decompression_config: None,
            on_download: Arc::new(|_| {}),
            cas: None,
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
        self.hashes = hashes;
        self
    }
    /// Adds the decompression configuration of the file to instance
    #[cfg(feature = "decompress")]
    pub fn with_decompression_config(
        mut self,
        config: crate::decompress::DLDecompressionConfig,
    ) -> Self {
        self.decompression_config = Some(config);
        self
    }
    /// Adds the on_download event handler of the file to instance
    pub fn with_on_download(mut self, on_download: Arc<dyn Fn(String) + Send + Sync>) -> Self {
        self.on_download = on_download;
        self
    }
    /// Configure CAS using
    #[cfg(feature = "cas")]
    pub fn with_cas(mut self, value: cas::DLStorage) -> Self {
        self.cas = Some(value);
        self
    }
}
impl<T: IndicatorFactory> Downloader<T> {
    /// Creates a new instance of Downloader
    pub fn new() -> Self {
        Downloader {
            files: Vec::new(),
            max_concurrent_downloads: 5,
            max_redirections: 5,
            indicator_factory: Default::default(),
        }
    }
    /// Adds the files to instance
    pub fn with_files(mut self, files: Vec<DLFile>) -> Self {
        self.files.extend(files);
        self
    }
    /// Creates a new instance of Downloader with the given files
    pub fn from_files(files: Vec<DLFile>) -> Self {
        let builder = Downloader::new();
        builder.with_files(files)
    }
    /// Adds a file to the instance
    pub fn add_file(mut self, file: DLFile) -> Self {
        self.files.push(file);
        self
    }
    /// Starts the download
    pub fn start(&self) {
        // create the semaphore of the maximum concurrent downloads
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent_downloads));
        // create the executor
        let executor = Arc::new(Executor::new());
        let client = Client::new().with(redirection_middleware::RedirectMiddleware::new(
            self.max_redirections,
        ));

        // obtain the futures
        let futures: Vec<Pin<Box<dyn Future<Output = Result<(), String>>>>> = self
            .files
            .iter()
            .map(|dl_file| {
                // create the progress bar
                let mut indicator = self
                    .indicator_factory
                    .create_task(&dl_file.path, dl_file.size);
                // obtain the semaphore permit
                let semaphore = Arc::clone(&semaphore);
                let client = client.clone();
                // create the task
                let task: Pin<Box<dyn Future<Output = Result<(), String>>>> =
                    Box::pin(executor.run(async move {
                        // acquire the semaphore permit
                        let permit = semaphore.acquire().await;
                        indicator.signal(IndicateSignal::Start());
                        // download the file
                        #[cfg(feature = "no_static_client")]
                        let client = create_client(self.max_redirections);
                        dl_file.download(indicator, client.clone()).await?;
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
    /// Sets the maximum number of concurrent downloads
    pub fn with_max_concurrent_downloads(mut self, max_concurrent_downloads: usize) -> Self {
        self.max_concurrent_downloads = max_concurrent_downloads;
        self
    }
    /// Sets the maximum number of redirections
    pub fn with_max_redirections(mut self, max_redirections: usize) -> Self {
        self.max_redirections = max_redirections;
        self
    }
    /// Sets the indicator tracer
    pub fn with_indicator(mut self, indicator: T) -> Self {
        self.indicator_factory = indicator;
        self
    }
}
fn create_client(max_redirections: usize) -> Client {
    Client::new().with(redirection_middleware::RedirectMiddleware::new(
        max_redirections,
    ))
}
