use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use symlink::symlink_auto;

pub struct DLStorage {
    pub path: PathBuf,
}

impl DLStorage {
    pub fn new(path: &str) -> Self {
        let path = Path::new(path);
        if !path.exists() {
            fs::create_dir_all(path).unwrap();
        }
        Self {
            path: path.to_path_buf(),
        }
    }
}

impl DLStorage {
    pub fn new_file(&self, hash: &str, file_path: &str) -> File {
        let file = self.file(hash);
        self.symlink(hash, file_path);
        file
    }
    pub fn symlink(&self, hash: &str, link: &str) -> File {
        let file = self.path(hash);
        let file = Path::new(file.as_str());
        let link = Path::new(link);

        match symlink_auto(file, link) {
            Ok(_) => {}
            Err(e) => panic!("Failed to create symlink: {}", e),
        }

        File::open(file).unwrap()
    }
    pub fn file(&self, hash: &str) -> File {
        let hash_path = self.path.join(&hash[0..2]);
        if !hash_path.exists() {
            fs::create_dir_all(&hash_path).unwrap();
        }
        File::create(hash_path.join(hash)).unwrap()
    }
    pub fn path(&self, hash: &str) -> String {
        self.path
            .join(&hash[0..2])
            .join(hash)
            .to_string_lossy()
            .into_owned()
    }
    pub fn find(&self, hash: &str) -> Option<String> {
        let hash_path = self.path.join(&hash[0..2]);
        if !hash_path.exists() {
            return None;
        }
        let file = hash_path.join(hash);
        if file.exists() {
            Some(file.to_string_lossy().into_owned())
        } else {
            None
        }
    }
}
