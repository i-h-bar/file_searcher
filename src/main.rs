use std::{fs, thread};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Sender;

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

    println!("Searching '{}' for '{}'", directory.to_str().unwrap().replace("\\\\?\\", ""), args.find);

    find(directory, args.find, sender);

    for msg in receiver {
        matches.push(msg);
    }

    for found in matches {
        println!("{}", found.replace("\\\\?\\", ""))
    }
}

fn find(directory: PathBuf, match_on: String, sender: Sender<String>) {
    if directory.is_dir() {
        let name = directory.file_name().unwrap().to_str().unwrap();
        if name.to_lowercase().contains(&match_on) {
            sender.send(directory.to_str().unwrap().to_string()).unwrap();
        }

        for path in fs::read_dir(directory).unwrap() {
            let path = path.unwrap().path();
            if path.is_dir() {
                let clone_sender = sender.clone();
                let clone_match_on = match_on.clone();
                let sub_path = PathBuf::from(path);
                thread::spawn(move || { find(sub_path, clone_match_on, clone_sender) });
            } else {
                let name = path.file_name().unwrap().to_str().unwrap();
                if name.contains(&match_on) {
                    sender.send(path.to_str().unwrap().to_string()).unwrap();
                }
            }
        }
    } else {
        let name = directory.file_name().unwrap().to_str().unwrap();
        if name.contains(&match_on) {
            sender.send(name.to_string()).unwrap();
        }
    }
}