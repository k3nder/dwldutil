/// Decompressor trait for decompressing files.
pub trait Decompressor {
    fn decompress(file: &str, path: &str) -> Result<(), String>;
}

/// Methods for decompressing files.
pub enum DecompressionMethod {
    #[cfg(feature = "gzip")]
    TarGzip,
    #[cfg(feature = "normal_zip")]
    Zip,
}

impl DecompressionMethod {
    /// Decompresses a file using the specified method.
    pub fn decompress(&self, file: &str, path: &str) -> Result<(), String> {
        match self {
            #[cfg(feature = "gzip")]
            DecompressionMethod::TarGzip => gzip::TarGzipDecompressor::decompress(file, path),
            #[cfg(feature = "normal_zip")]
            DecompressionMethod::Zip => zip::ZipDecompressor::decompress(file, path),
            _ => return Ok(()),
        }
    }
}

/// Decompress tar.gz files.
#[cfg(feature = "gzip")]
mod gzip {
    use std::fs::File;

    use super::Decompressor;
    use flate2::read::GzDecoder;
    use tar::Archive;
    pub struct TarGzipDecompressor;

    /// Decompressor for tar.gz file.
    impl Decompressor for TarGzipDecompressor {
        fn decompress(file: &str, path: &str) -> Result<(), String> {
            let tar_gz = File::open(file).expect("Failed to open archive");
            let tar = GzDecoder::new(tar_gz);
            let mut archive = Archive::new(tar);
            archive.unpack(path).expect("Failed to extract archive");
            Ok(())
        }
    }
}

/// Decompressor for zip file.
#[cfg(feature = "normal_zip")]
mod zip {
    use std::fs::{File, create_dir_all};
    use std::io;
    use std::path::Path;
    use zip::ZipArchive;

    use super::Decompressor;

    pub struct ZipDecompressor;

    /// Decompressor for zip file.
    impl Decompressor for ZipDecompressor {
        fn decompress(file: &str, path: &str) -> Result<(), String> {
            let file = File::open(file).expect("Failed to open archive");
            let mut archive = ZipArchive::new(file).expect("Failed to open archive");

            // Asegurarse que el directorio de destino existe
            create_dir_all(path).expect("Failed to create directory");

            for i in 0..archive.len() {
                let mut file = archive.by_index(i).expect("Failed to extract file");
                let outpath = Path::new(path).join(file.mangled_name());

                if file.name().ends_with('/') {
                    create_dir_all(&outpath).expect("Failed to create directory");
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            create_dir_all(p).expect("Failed to create directory");
                        }
                    }
                    let mut outfile = File::create(&outpath).expect("Failed to create file");
                    io::copy(&mut file, &mut outfile).expect("Failed to copy file");
                }
            }

            Ok(())
        }
    }
}
/// Decompressor Configuration
pub struct DLDecompressionConfig {
    /// Decompression method
    pub method: DecompressionMethod,
    /// Output directory
    pub output: String,
    /// Delete file after decompression
    pub delete_after: bool,
}
impl DLDecompressionConfig {
    /// Create a new decompression configuration
    pub fn new(method: DecompressionMethod, output: &str) -> Self {
        DLDecompressionConfig {
            method,
            output: output.to_string(),
            delete_after: true,
        }
    }
    /// Set the decompression method
    pub fn with_method(mut self, method: DecompressionMethod) -> Self {
        self.method = method;
        self
    }
    /// Set the output directory
    pub fn with_output(mut self, output: String) -> Self {
        self.output = output;
        self
    }
    /// Set whether to delete the file after decompression
    pub fn with_delete_after(mut self, delete_after: bool) -> Self {
        self.delete_after = delete_after;
        self
    }
    /// Set delete after to true
    pub fn delete_after(mut self) -> Self {
        self.delete_after = true;
        self
    }
    /// Decompress a file
    pub fn decompress(&self, file: &str) -> Result<(), String> {
        self.method.decompress(file, &self.output)?;
        Ok(())
    }
}
