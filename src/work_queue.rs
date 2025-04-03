use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use crate::now_string;
use crate::transcoder::transcode_hevc_hvc1;

pub struct WorkQueue {
    rx: Receiver<PathBuf>,
    stop: Arc<Mutex<bool>>,
}

impl WorkQueue {
    pub fn new(stop: Arc<Mutex<bool>>, rx: Receiver<PathBuf>) -> Self {
        WorkQueue {
            rx,
            stop,
        }
    }

    pub fn forever(&mut self) {
        loop {
            if self.should_stop() { break; }
            match self.rx.recv_timeout(Duration::from_millis(500)) {
                Ok(pb) => self.work(pb),
                Err(err) => match err {
                    RecvTimeoutError::Timeout => (),
                    RecvTimeoutError::Disconnected => break,
                },
            }
        }
    }

    fn should_stop(&self) -> bool {
        let stop = self.stop.lock().unwrap();
        *stop
    }

    fn work(&self, path: PathBuf) {
        let work_path = WorkQueue::work_path(&path);
        let dst_path = WorkQueue::destination_path(&path);

        if WorkQueue::file_exists(&work_path) {
            return;
        }

        if WorkQueue::file_exists(&dst_path) {
            return;
        }

        println!("{} transcoding {:?}", now_string(), &path);
        if let Ok(_) = transcode_hevc_hvc1(Arc::clone(&self.stop), &path, &work_path) {
            let _ = fs::rename(&work_path, &dst_path);
        }
    }

    fn destination_path(src: &PathBuf) -> PathBuf {
        let mut dst = PathBuf::from(src);
        dst.set_extension("hvc1.mp4");
        dst
    }

    fn work_path(src: &PathBuf) -> PathBuf {
        let mut path = PathBuf::from(src);
        if let Some(stem) = path.file_stem() {
            let mut hidden_stem = OsString::from(".");
            hidden_stem.push(stem);
            path.set_file_name(hidden_stem);
            path.set_extension("hvc1.mp4");
        }

        path
    }

    fn file_exists(path: &PathBuf) -> bool {
        match fs::exists(path) {
            Ok(exists) => exists,
            Err(_) => false,
        }
    }
}

