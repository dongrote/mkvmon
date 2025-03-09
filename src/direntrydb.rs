use std::{
    collections::HashMap,
    fs::{self, metadata, DirEntry},
    os::unix::fs::MetadataExt,
    path::PathBuf,
    result::Result
};

use crate::probe::probe_file;

#[derive(Debug)]
pub struct DirEntryDatabase {
    entries: HashMap<PathBuf, DirEntryAttributes>,
}

#[derive(Debug)]
pub struct DirEntryAttributes {
    pub path: PathBuf,
    pub size: u64,
    pub compressed: bool,
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
                if let Some(file_name) = direntry.path().file_name() {
                    self.entries.insert(PathBuf::from(file_name), attr);
                }
                Ok(())
            },
            Err(err) => Err(err),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=&DirEntryAttributes> {
        self.entries.values()
    }
}

impl DirEntryAttributes {
    pub fn new(direntry: &DirEntry) -> Result<DirEntryAttributes, std::io::Error> {
        let path = direntry.path();
        match metadata(&path) {
            Ok(fi) => Ok(DirEntryAttributes {
                path: PathBuf::from(&path),
                size: fi.size(),
                compressed: DirEntryAttributes::is_compressed(&path),
                in_progress: DirEntryAttributes::is_in_progress(&path),
            }),
            Err(err) => Err(err),
        }
    }

    fn is_in_progress(_path: &PathBuf) -> Option<bool> {
        None
    }

    fn is_compressed(path: &PathBuf) -> bool {
        if let Some(stem) = path.file_stem() {
            let mut compressed_path = path.clone();
            compressed_path.set_file_name(&stem);
            compressed_path.set_extension("hvc1.mp4");
            match fs::exists(&compressed_path) {
                Ok(exists) => match exists {
                    true => match probe_file(&compressed_path) {
                        Ok(probe) => {
                            dbg!(&compressed_path, &probe);
                            probe.video_codec == "hevc" && probe.video_codec_tag == "hvc1"
                        },
                        Err(_) => false,
                    },
                    false => false,
                },
                Err(_) => false,
            }
        } else {
            false
        }
    }
}
