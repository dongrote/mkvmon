use std::error::Error;
use std::path::PathBuf;
use std::io;
use std::process::Command;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug)]
pub struct AVProbeMetadata {
    pub video_codec: String,
    pub video_codec_tag: String,
    pub width: u64,
    pub height: u64,
}

impl AVProbeMetadata {
    pub fn empty() -> Self {
        AVProbeMetadata {
            video_codec: String::new(),
            video_codec_tag: String::new(),
            width: 0,
            height: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct FFProbeJsonOutput {
    pub streams: Vec<FFProbeJsonStream>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FFProbeJsonStream {
    pub codec_name: String,
    pub codec_tag_string: String,
    pub width: u64,
    pub height: u64,
    pub pix_fmt: String,
}

pub fn probe_file(path: &PathBuf) -> Result<AVProbeMetadata, Box<dyn Error>> { 
    let output = Command::new("ffprobe")
        .args([
            &PathBuf::from("-of"),
            &PathBuf::from("json"),
            &PathBuf::from("-show_streams"),
            &PathBuf::from("-select_streams"),
            &PathBuf::from("v:0"),
            path,
        ])
        .output()?;
    if output.status.success() {
        let utf8 = String::from_utf8(output.stdout)?;
        let deserialized = serde_json::from_str::<FFProbeJsonOutput>(&utf8)?;
        Ok(AVProbeMetadata {
            video_codec: deserialized.streams[0].codec_name.clone(),
            video_codec_tag: deserialized.streams[0].codec_tag_string.clone(),
            width: deserialized.streams[0].width,
            height: deserialized.streams[0].height,
        })
    } else {
        Err(Box::new(io::Error::new(io::ErrorKind::Other, "oh no!")))
    }
}
