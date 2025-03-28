use std::{env, path::PathBuf};
use dirmon::DirectoryMonitor;
use filters::{DirEntryFilter, PathExtensionFilter, NotFilter, VideoCodecFilter};
use signal_hook::{consts::{SIGINT, SIGHUP, SIGTERM}, iterator::Signals};
use std::thread;

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

    let verbose = false;
    let mut mon = DirectoryMonitor::new(verbose);
    let tx = mon.tx();

    thread::spawn(move || {
        if let Ok(mut signals) = Signals::new(&[SIGINT, SIGHUP, SIGTERM]) {
            println!("Listening for SIGINT, SIGHUP, SIGTERM");
            for sig in signals.forever() {
                match sig {
                    SIGINT => println!("Caught SIGINT."),
                    SIGHUP => println!("Caught SIGHUP."),
                    SIGTERM => println!("Caught SIGTERM."),
                    _ => continue,
                };

                let _ = tx.send(0);
                break;
            }
        } else {
            println!("Error registering signal handler; Ctrl-C will not save you.");
        }
    });

    if mon.validate(&args[1]) {
        if let Err(e) = mon.monitor(&PathBuf::from(&args[1]), &filters) {
            println!("Error: {:?}", e);
        }
    } else {
        println!("Path '{}' is not a directory.", &args[1]);
    }
}
