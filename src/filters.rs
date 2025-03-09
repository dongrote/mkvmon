use std::fs;

pub trait DirEntryFilter {
    fn filter(&self, direntry: &fs::DirEntry) -> bool;
}

pub struct PathExtensionFilter {
    extension: String,
}

impl PathExtensionFilter {
    pub fn new(extension: &str) -> Self {
        PathExtensionFilter {
            extension: String::from(extension),
        }
    }
}

impl DirEntryFilter for PathExtensionFilter {
    fn filter(&self, direntry: &fs::DirEntry) -> bool {
        if let Some(ext) = direntry.path().extension() {
            *ext == *self.extension
        } else {
            false
        }
    }
}

pub struct MinimumSizeFilter {
    minimum_size: u64,
}

impl MinimumSizeFilter {
    pub fn new(minimum_size: u64) -> Self {
        MinimumSizeFilter {
            minimum_size,
        }
    }
}

impl DirEntryFilter for MinimumSizeFilter {
    fn filter(&self, direntry: &fs::DirEntry) -> bool {
        if let Ok(fi) = direntry.metadata() {
            fi.len() >= self.minimum_size
        } else {
            false
        }
    }
}

