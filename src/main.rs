use std::{env, path::PathBuf};
use dirmon::DirectoryMonitor;
use filters::{DirEntryFilter, PathExtensionFilter, NotFilter, VideoCodecFilter};

pub mod dirmon;
pub mod direntrydb;
pub mod filters;
pub mod probe;
pub mod transcoder;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Missing directory argument!");
        return
    }

    let filters: Vec<Box<dyn DirEntryFilter>> = vec![
        Box::new(PathExtensionFilter::new("mkv")),
        Box::new(NotFilter::new(Box::new(VideoCodecFilter::new("hevc", "hvc1")))),
    ];
    let mut mon = DirectoryMonitor::new();
    match mon.validate(&args[1]) {
        true => {
            let _ = mon.monitor(&PathBuf::from(&args[1]), &filters);
        },
        false => println!("Path '{}' is not a directory.", &args[1]),
    }
}
