use std::error::Error;
use std::io;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub fn transcode_hevc_hvc1(stop: Arc<Mutex<bool>>, src: &PathBuf, dst: &PathBuf) -> Result<(), Box<dyn Error>> {
    let mut retval = Ok(());
    let mut cleanup_dst = false;
    let mut child = Command::new("ffmpeg")
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
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    loop {
        if let Ok(res) = child.try_wait() {
            if let Some(status) = res {
                if !status.success() {
                    retval = Err(Box::new(io::Error::new(io::ErrorKind::Other, "transcode error")));
                    cleanup_dst = true;
                }
                break;
            } else {
                let abort = {
                    let s = stop.lock().unwrap();
                    *s
                };

                if abort {
                    let _ = child.kill();
                    cleanup_dst = true;
                }
            }
        } else {
            println!("child.try_wait() failed!");
            break;
        }

        thread::sleep(Duration::from_millis(100));
    }

    if cleanup_dst {
        let _ = fs::remove_file(dst);
    }

    Ok(retval?)
}
