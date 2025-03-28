use std::fs;

use crate::probe::probe_file;

pub trait DirEntryFilter {
    fn filter(&self, direntry: &fs::DirEntry) -> bool;
}

pub struct PathExtensionFilter {
    extension: String,
}

pub struct MinimumSizeFilter {
    minimum_size: u64,
}

pub struct VideoCodecFilter {
    video_codec: String,
    video_codec_tag: String,
}

pub struct NotFilter {
    filter: Box<dyn DirEntryFilter>,
}

impl PathExtensionFilter {
    pub fn new(extension: &str) -> Self {
        PathExtensionFilter {
            extension: String::from(extension),
        }
    }
}

impl MinimumSizeFilter {
    pub fn new(minimum_size: u64) -> Self {
        MinimumSizeFilter {
            minimum_size,
        }
    }
}

impl VideoCodecFilter {
    pub fn new(video_codec: &str, video_codec_tag: &str) -> Self {
        VideoCodecFilter {
            video_codec: String::from(video_codec),
            video_codec_tag: String::from(video_codec_tag),
        }
    }
}

impl NotFilter {
    pub fn new(filter: Box::<dyn DirEntryFilter>) -> Self {
        NotFilter {
            filter,
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

impl DirEntryFilter for MinimumSizeFilter {
    fn filter(&self, direntry: &fs::DirEntry) -> bool {
        if let Ok(fi) = direntry.metadata() {
            fi.len() >= self.minimum_size
        } else {
            false
        }
    }
}

impl DirEntryFilter for VideoCodecFilter {
    fn filter(&self, direntry: &fs::DirEntry) -> bool {
        match probe_file(&direntry.path()) {
            Ok(metadata) => metadata.video_codec == self.video_codec && metadata.video_codec_tag == self.video_codec_tag,
            Err(_) => false,
        }
    }
}

impl DirEntryFilter for NotFilter {
    fn filter(&self, direntry: &fs::DirEntry) -> bool {
        !self.filter.filter(direntry)
    }
}
