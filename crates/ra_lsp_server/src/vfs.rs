use std::{
    fs,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

use crate::thread_watcher::{ThreadWatcher, Worker};

#[derive(Debug)]
pub struct FileEvent {
    pub path: PathBuf,
    pub kind: FileEventKind,
}

#[derive(Debug)]
pub enum FileEventKind {
    Add(String),
}

pub fn roots_loader() -> (Worker<PathBuf, (PathBuf, Vec<FileEvent>)>, ThreadWatcher) {
    Worker::<PathBuf, (PathBuf, Vec<FileEvent>)>::spawn(
        "roots loader",
        128,
        |input_receiver, output_sender| {
            input_receiver
                .map(|path| {
                    log::debug!("loading {} ...", path.as_path().display());
                    let events = load_root(path.as_path());
                    log::debug!("... loaded {}", path.as_path().display());
                    (path, events)
                })
                .for_each(|it| output_sender.send(it))
        },
    )
}

fn load_root(path: &Path) -> Vec<FileEvent> {
    let mut res = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                log::warn!("watcher error: {}", e);
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|os| os.to_str()) != Some("rs") {
            continue;
        }
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) => {
                log::warn!("watcher error: {}", e);
                continue;
            }
        };
        res.push(FileEvent {
            path: path.to_owned(),
            kind: FileEventKind::Add(text),
        })
    }
    res
}
