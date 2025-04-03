use std::path::Path;
use std::fs;

use crate::probe::probe_file;

pub trait PathFilter {
    fn filter(&self, path: &Path) -> bool;
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
    filter: Box<dyn PathFilter>,
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
    pub fn new(filter: Box::<dyn PathFilter>) -> Self {
        NotFilter {
            filter,
        }
    }
}

impl PathFilter for PathExtensionFilter {
    fn filter(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            *ext == *self.extension
        } else {
            false
        }
    }
}

impl PathFilter for MinimumSizeFilter {
    fn filter(&self, path: &Path) -> bool {
        if let Ok(fi) = fs::metadata(path) {
            fi.len() >= self.minimum_size
        } else {
            false
        }
    }
}

impl PathFilter for VideoCodecFilter {
    fn filter(&self, path: &Path) -> bool {
        match probe_file(path) {
            Ok(metadata) => metadata.video_codec == self.video_codec && metadata.video_codec_tag == self.video_codec_tag,
            Err(_) => false,
        }
    }
}

impl PathFilter for NotFilter {
    fn filter(&self, path: &Path) -> bool {
        !self.filter.filter(path)
    }
}
