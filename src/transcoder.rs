use std::error::Error;
use std::io;
use std::path::PathBuf;
use std::process::Command;

pub fn transcode_hevc_hvc1(src: &PathBuf, dst: &PathBuf) -> Result<(), Box<dyn Error>> {
    let output = Command::new("ffmpeg")
        .args([
            &PathBuf::from("-i"),
            &PathBuf::from(src),
            &PathBuf::from("-c:v"),
            &PathBuf::from("libx265"),
            &PathBuf::from("-crf"),
            &PathBuf::from("25"),
            &PathBuf::from("-preset"),
            &PathBuf::from("slow"),
            &PathBuf::from("-tag:v"),
            &PathBuf::from("hvc1"),
            &PathBuf::from("-c:a"),
            &PathBuf::from("copy"),
            &PathBuf::from("-c:s"),
            &PathBuf::from("copy"),
            &PathBuf::from(dst),
        ])
        .output()?;
    match output.status.success() {
        true => Ok(()),
        false => Err(Box::new(io::Error::new(io::ErrorKind::Other, "transcode error"))),
    }
}
