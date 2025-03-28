use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::Duration;

use crate::filters::DirEntryFilter;
use crate::transcoder::transcode_hevc_hvc1;

#[derive(Debug)]
pub struct DirectoryMonitor {
    verbose: bool,
    polling_interval: Duration,
    stop_rx: mpsc::Receiver<i16>,
    stop_tx: mpsc::Sender<i16>,
}

#[derive(Debug)]
struct ScanStats {
    pub files: usize,
    pub errors: usize,
    pub filtered: usize,
    pub skipped: usize,
    pub transcoded: usize,
}

enum ProcessResult {
    Transcoded,
    Skipped,
}

impl ScanStats {
    pub fn new() -> Self {
        ScanStats { files: 0, errors: 0, filtered: 0, skipped: 0, transcoded: 0 }
    }

    pub fn aggregate(&mut self, more: &ScanStats) {
        self.files += more.files;
        self.errors += more.errors;
        self.filtered += more.filtered;
        self.skipped += more.skipped;
        self.transcoded += more.transcoded;
    }
}

impl DirectoryMonitor {
    pub fn new(verbose: bool) -> Self {
        let (tx, rx) = mpsc::channel();
        DirectoryMonitor {
            verbose,
            polling_interval: Duration::new(10, 0),
            stop_rx: rx,
            stop_tx: tx,
        }
    }

    pub fn validate(&self, path: &str) -> bool {
        match fs::metadata(path) {
            Ok(attr) => attr.is_dir(),
            _ => false,
        }
    }

    pub fn tx(&self) -> mpsc::Sender<i16> {
        self.stop_tx.clone()
    }

    pub fn monitor(&mut self, path: &PathBuf, filters: &Vec<Box<dyn DirEntryFilter>>) -> Result<(), Box<dyn Error>> {
        loop {
            println!("Scanning {path:?}");
            let stats = self.process_directory(path, filters)?;
            println!(
                "files scanned: {} filtered: {} transcoded: {} skipped: {} errors: {}",
                stats.files,
                stats.filtered,
                stats.transcoded,
                stats.skipped,
                stats.errors);
            match self.stop_rx.recv_timeout(self.polling_interval) {
                Ok(_) => break,
                Err(e) => match e {
                    RecvTimeoutError::Timeout => continue,
                    RecvTimeoutError::Disconnected => break,
                },
            };
        }

        Ok(())
    }

    fn process_directory(&self, path: &PathBuf, filters: &Vec<Box<dyn DirEntryFilter>>) -> Result<ScanStats, Box<dyn Error>> {
        let mut stats = ScanStats::new();
        if let Ok(rd) = fs::read_dir(path) {
            let entries = rd.filter_map(|x| x.ok());
            for entry in entries {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        let rstats = self.process_directory(&entry.path(), filters)?;
                        stats.aggregate(&rstats);
                    } else {
                        stats.files += 1;
                        if filters.iter().all(|f| f.filter(&entry)) {
                            stats.filtered += 1;
                            if self.verbose { println!("{:?} passed filter check", &entry); }
                            if let Ok(result) = self.process_file(&entry) {
                                match result {
                                    ProcessResult::Skipped => { stats.skipped += 1; },
                                    ProcessResult::Transcoded => { stats.transcoded += 1; },
                                }
                            } else {
                                stats.errors += 1;
                            }
                        } else {
                            if self.verbose { println!("{:?} failed filter check", &entry); }
                        }
                    }
                }
            }
        }

        Ok(stats)
    }

    fn process_file(&self, entry: &fs::DirEntry) -> Result<ProcessResult, Box<dyn Error>> {
        let entry_path = &entry.path();
        let working_path = DirectoryMonitor::create_working_path(entry_path);
        let destination_path = DirectoryMonitor::create_destination_path(entry_path);
        if let Ok(w_exists) = fs::exists(&working_path) {
            if w_exists {
                println!("Working path already exists; skipping {:?}", entry_path);
                return Ok(ProcessResult::Skipped);
            }
        }

        if let Ok(d_exists) = fs::exists(&destination_path) {
            if d_exists {
                if self.verbose { println!("Destination path already exists; skipping {:?}", entry_path); }
                return Ok(ProcessResult::Skipped);
            }
        }

        println!("transcoding {:?} => {:?}", entry_path, &working_path);
        transcode_hevc_hvc1(&self.stop_rx, entry_path, &working_path)?;
        println!("fs::rename({:?}, {:?})", &working_path, &destination_path);
        fs::rename(&working_path, &destination_path)?;

        Ok(ProcessResult::Transcoded)
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
