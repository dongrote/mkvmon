use std::{
    collections::HashMap,
    fs::{metadata, DirEntry},
    os::unix::fs::MetadataExt,
    path::PathBuf,
    result::Result
};

#[derive(Debug)]
pub struct DirEntryDatabase {
    entries: HashMap<PathBuf, DirEntryAttributes>,
}

#[derive(Debug)]
struct DirEntryAttributes {
    pub path: PathBuf,
    pub size: u64,
    pub compressed: Option<bool>,
    pub in_progress: Option<bool>,
}

impl DirEntryDatabase {
    pub fn new() -> Self {
        DirEntryDatabase {
            entries: HashMap::new(),
        }
    }

    pub fn upsert_direntry(&mut self, direntry: &DirEntry) -> Result<(), std::io::Error> {
        match DirEntryAttributes::new(direntry) {
            Ok(attr) => {
                self.entries.insert(direntry.path(), attr);
                Ok(())
            },
            Err(err) => Err(err),
        }
    }
}

impl DirEntryAttributes {
    pub fn new(direntry: &DirEntry) -> Result<DirEntryAttributes, std::io::Error> {
        match metadata(direntry.path()) {
            Ok(fi) => Ok(DirEntryAttributes {
                path: PathBuf::from(direntry.path()),
                size: fi.size(),
                compressed: None,
                in_progress: None,
            }),
            Err(err) => Err(err),
        }
    }
}
