use std::{env, path::{Path, PathBuf}, sync::{mpsc::{self, RecvTimeoutError, Sender}, Arc, Mutex}, thread, time::Duration};
use std::fs;
use filters::{PathFilter, PathExtensionFilter, NotFilter, VideoCodecFilter};
use signal_hook::{consts::{SIGINT, SIGHUP, SIGTERM}, iterator::Signals};
use notify::{self, event::{AccessKind, AccessMode}, EventKind, RecursiveMode, Watcher};
use now_str::now_string;
use work_queue::WorkQueue;

pub mod filters;
pub mod now_str;
pub mod probe;
pub mod transcoder;
pub mod work_queue;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Missing directory argument!");
        return
    }

    let filters: Vec<Box<dyn PathFilter>> = vec![
        Box::new(PathExtensionFilter::new("mkv")),
        Box::new(NotFilter::new(Box::new(VideoCodecFilter::new("hevc", "hvc1")))),
    ];

    let stop = Arc::new(Mutex::new(false));
    let notify_stop = Arc::clone(&stop);
    let (not_tx, not_rx) = mpsc::channel();
    let (wq_tx, wq_rx) = mpsc::channel();
    let mut wq = WorkQueue::new(Arc::clone(&stop), wq_rx);

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

                {
                    let mut s = stop.lock().unwrap();
                    *s = true;
                }

                break;
            }
        } else {
            println!("Error registering signal handler; Ctrl-C will not save you.");
        }
    });

    let wq_handle = thread::spawn(move || {
        wq.forever();
    });

    // update WorkQueue with filesystem events
    let watch_path = PathBuf::from(&args[1]);
    let notify_wq_tx = wq_tx.clone();
    let notify_handle = thread::spawn(move || {
        let filters: Vec<Box<dyn PathFilter>> = vec![
            Box::new(PathExtensionFilter::new("mkv")),
            Box::new(NotFilter::new(Box::new(VideoCodecFilter::new("hevc", "hvc1")))),
        ];
        if let Ok(mut watcher) = notify::recommended_watcher(not_tx) {
            if let Ok(_) = watcher.watch(&watch_path, RecursiveMode::Recursive) {
                loop {
                    let should_stop = {
                        let s = notify_stop.lock().unwrap();
                        *s
                    };

                    if should_stop {
                        println!("notify watcher received stop signal");
                        break;
                    }

                    match not_rx.recv_timeout(Duration::from_millis(100)) {
                        Ok(res) => match res {
                            Ok(event) => match event.kind {
                                EventKind::Access(ak) => match ak {
                                    AccessKind::Close(am) => match am {
                                        AccessMode::Write => {
                                            // wait until the file, opened for writing, has
                                            // finished being written because we filter on file
                                            // content and parsing a newly created, potentially
                                            // empty file could cause us to fail a filter
                                            dbg!(&event);
                                            for p in &event.paths {
                                                if filters.iter().all(|f| f.filter(Path::new(p))) {
                                                    println!("{} enqueue {:?}", now_string(), &p);
                                                    let _ = notify_wq_tx.send(p.clone());
                                                }
                                            }
                                        },
                                        _ => (),
                                    },
                                    _ => (),
                                },
                                _ => (),
                            },

                            Err(err) => println!("{} notify error: {:?}", now_string(), err),
                        }
                        Err(err) => match err {
                            RecvTimeoutError::Timeout => (),
                            RecvTimeoutError::Disconnected => {
                                println!("not_rx disconnected {:?}", err);
                                break;
                            },
                        },
                    };
                }
            }

        }
    });

    // populate WorkQueue with initial scan
    process_directory(&PathBuf::from(&args[1]), wq_tx.clone(), &filters);

    let _ = wq_handle.join();
    let _ = notify_handle.join();
}

fn process_directory(path: &PathBuf, tx: Sender<PathBuf>, filters: &Vec<Box<dyn PathFilter>>) {
    if let Ok(rd) = fs::read_dir(path) {
        let entries = rd.filter_map(|x| x.ok());
        for entry in entries {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    process_directory(&entry.path(), tx.clone(), filters);
                } else {
                    let p = entry.path();
                    if filters.iter().all(|f| f.filter(Path::new(&p))) {
                        println!("{} enqueue {:?}", now_string(), &p); 
                        let _ = tx.send(p);
                    }
                }
            }
        }
    }
}
