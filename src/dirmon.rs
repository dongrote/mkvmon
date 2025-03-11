use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use crate::filters::DirEntryFilter;
use crate::transcoder::transcode_hevc_hvc1;

#[derive(Debug)]
pub struct DirectoryMonitor {
}

impl DirectoryMonitor {
    pub fn new() -> Self {
        DirectoryMonitor {
        }
    }

    pub fn validate(&self, path: &str) -> bool {
        match fs::metadata(path) {
            Ok(attr) => attr.is_dir(),
            _ => false,
        }
    }

    pub fn monitor(&mut self, path: &PathBuf, filters: &Vec<Box<dyn DirEntryFilter>>) -> Result<(), Box<dyn Error>> {
        self.process_directory(path, filters)
    }

    fn process_directory(&self, path: &PathBuf, filters: &Vec<Box<dyn DirEntryFilter>>) -> Result<(), Box<dyn Error>> {
        if let Ok(rd) = fs::read_dir(path) {
            let entries = rd.filter_map(|x| x.ok());
            for entry in entries {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        self.process_directory(&entry.path(), filters)?;
                    } else {
                        if filters.iter().all(|f| f.filter(&entry)) {
                            println!("{:?} passed filter check", &entry);
                            self.process_file(&entry)?;
                        } else {
                            println!("{:?} failed filter check", &entry);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn process_file(&self, entry: &fs::DirEntry) -> Result<(), Box<dyn Error>> {
        let entry_path = &entry.path();
        let working_path = DirectoryMonitor::create_working_path(entry_path);
        let destination_path = DirectoryMonitor::create_destination_path(entry_path);
        if let Ok(w_exists) = fs::exists(&working_path) {
            if w_exists {
                println!("Working path already exists; skipping {:?}", entry_path);
                return Ok(());
            }
        }

        if let Ok(d_exists) = fs::exists(&destination_path) {
            if d_exists {
                println!("Destination path already exists; skipping {:?}", entry_path);
                return Ok(());
            }
        }

        println!("transcoding {:?} => {:?}", entry_path, &working_path);
        transcode_hevc_hvc1(entry_path, &working_path)?;
        println!("fs::rename({:?}, {:?})", &working_path, &destination_path);
        fs::rename(&working_path, &destination_path)?;

        Ok(())
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
