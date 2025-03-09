use std::{env, path::PathBuf};
use dirmon::DirectoryMonitor;
use filters::{DirEntryFilter, MinimumSizeFilter, PathExtensionFilter};

pub mod dirmon;
pub mod direntrydb;
pub mod filters;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Missing directory argument!");
        return
    }

    let filters: Vec<Box<dyn DirEntryFilter>> = vec![
        Box::new(PathExtensionFilter::new("mkv")),
        Box::new(MinimumSizeFilter::new(1024)),
    ];
    let mut mon = DirectoryMonitor::new();
    match mon.validate(&args[1]) {
        true => {
            mon.monitor(&PathBuf::from(&args[1]), &filters);
            dbg!(mon);
        },
        false => println!("Path '{}' is not a directory.", &args[1]),
    }
}
