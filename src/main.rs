use std::{fs, thread};
use std::path::PathBuf;
use std::sync::{Arc, mpsc};
use std::sync::mpsc::Sender;
use std::time::Instant;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "C:\\")]
    directory: String,

    #[arg(short, long)]
    find: String,
}

fn main() {
    let args = Args::parse();
    let mut matches: Vec<String> = Vec::new();
    let (sender, receiver) = mpsc::channel();
    let directory = PathBuf::from(&args.directory);
    let match_on = Arc::new(args.find.to_lowercase());

    println!("Searching '{}' for '{}'", directory.to_str().unwrap().replace("\\\\?\\", ""), match_on.as_str());

    let now = Instant::now();

    find(directory, match_on, sender);

    for msg in receiver {
        matches.push(msg);
    }

    let elapsed = now.elapsed();

    for found in matches {
        println!("{}", found.replace("\\\\?\\", ""))
    }

    println!("Elapsed: {:.2?}", elapsed);
}

fn find(directory: PathBuf, match_on: Arc<String>, sender: Sender<String>) {
    if directory.is_dir() {
        let name = match directory.file_name() {
            Some(name) => {
                let Some(name) = name.to_str() else { return };
                name
            },
            None => {
                let Some(name) = directory.to_str() else { return; };

                if name == "C:\\" {
                    name
                } else {
                    return;
                }
            }
        };

        if name.to_lowercase().contains(match_on.as_str()) {
            let Some(dir_name) = directory.to_str() else { return; };

            sender.send(dir_name.to_string()).ok();
        }

        let Ok(contents) = fs::read_dir(&directory) else { return };

        for path in contents {
            let path = path.unwrap().path();
            if path.is_dir() {
                let clone_sender = sender.clone();
                let clone_match_on = Arc::clone(&match_on);
                let sub_path = PathBuf::from(path);
                thread::spawn(move || { find(sub_path, clone_match_on, clone_sender) });
            } else {
                let Some(name) = path.file_name() else { return; };
                let Some(name) = name.to_str() else { return; };
                if name.contains(match_on.as_str()) {
                    let Some(path_name) = path.to_str() else { return; };
                    sender.send(path_name.to_string()).ok();
                }
            }
        }
    } else {
        let Some(name) = directory.file_name() else { return; };
        let Some(name) = name.to_str() else { return; };
        if name.contains(match_on.as_str()) {
            sender.send(name.to_string()).unwrap();
        }
    }
}