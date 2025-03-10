use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use crate::direntrydb::DirEntryDatabase;
use crate::filters::DirEntryFilter;
use crate::transcoder::transcode_hevc_hvc1;

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

        for entry in self.db.iter().filter(|e| !e.compressed) {
            let working_path = DirectoryMonitor::create_working_path(&entry.path);
            let destination_path = DirectoryMonitor::create_destination_path(&entry.path);
            if let Ok(w_exists) = fs::exists(&working_path) {
                if w_exists {
                    println!("Working path already exists; skipping {:?}", &entry.path);
                    continue;
                }
            }

            if let Ok(d_exists) = fs::exists(&destination_path) {
                if d_exists {
                    println!("Destination path already exists; skipping {:?}", &entry.path);
                    continue;
                }
            }

            println!("transcoding {:?} => {:?}", &entry.path, &working_path);
            if let Ok(_) = transcode_hevc_hvc1(&entry.path, &working_path) {
                println!("fs::rename({:?}, {:?})", &working_path, &destination_path);
                let _ = fs::rename(&working_path, &destination_path);
            }
        }
    }

    fn create_destination_path(src: &PathBuf) -> PathBuf {
        let mut dst = PathBuf::from(src);
        dst.set_extension("hvc1.mp4");
        dst
    }

    fn create_working_path(src: &PathBuf) -> PathBuf {
        let mut path = PathBuf::from(src);
        if let Some(stem) = path.file_stem() {
            let mut hidden_stem = OsString::from(".");
            hidden_stem.push(stem);
            path.set_file_name(hidden_stem);
            path.set_extension("hvc1.mp4");
        }

        path
    }
}
