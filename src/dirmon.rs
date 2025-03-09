use std::fs;
use std::path::PathBuf;

use crate::direntrydb::DirEntryDatabase;
use crate::filters::DirEntryFilter;

#[derive(Debug)]
pub struct DirectoryMonitor {
    db: DirEntryDatabase,
}

impl DirectoryMonitor {
    pub fn new() -> Self {
        DirectoryMonitor {
            db: DirEntryDatabase::new(),
        }
    }

    pub fn validate(&self, path: &str) -> bool {
        match fs::metadata(path) {
            Ok(attr) => attr.is_dir(),
            _ => false,
        }
    }

    pub fn monitor(&mut self, path: &PathBuf, filters: &Vec<Box<dyn DirEntryFilter>>) {
        println!("Monitoring directory: {:?}", path);
        if let Ok(rd) = fs::read_dir(path) {
            let entries = rd.filter_map(|x| x.ok());
            for entry in entries {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        self.monitor(&entry.path(), filters);
                    } else {
                        if filters.iter().all(|f| f.filter(&entry)) {
                            let _ = self.db.upsert_direntry(&entry);
                        }
                    }
                }
            }
        }
    }
}
