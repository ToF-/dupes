
use clap::{Parser};
use std::fs;
use std::io;
use std::time::SystemTime;
use walkdir::WalkDir;

/// dupes
#[derive(Parser, Debug)]
#[command(infer_subcommands = true, infer_long_args = true, author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("."))]
    directory: String,
}

#[derive(Clone, Debug)]
struct Entry {
    file_path: String,
    file_size: u64,
    modified_time: SystemTime,
}

impl Entry {
    fn show(&self) -> String {
        format!("{:20} {}", self.file_size, self.file_path)
    }
}
type EntryList = Vec<Entry>;

fn make_entry(s: String, l: u64, t: SystemTime) -> Entry {
    return Entry {
        file_path: s.clone(),
        file_size: l,
        modified_time: t,
    }
}

fn get_entries_in_directory(dir_path: &str) -> io::Result<EntryList> {
    let mut entries: EntryList = Vec::new();
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.clone().into_path();
        let valid:bool = ! entry.file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false);
        if valid {
            if let Ok(metadata) = fs::metadata(&path) {
                let file_size = metadata.len();
                let modified_time = metadata.modified().unwrap();
                if let Some(full_name) = path.to_str() {
                    let entry_name = full_name.to_string().to_owned();
                    entries.push(make_entry(entry_name, file_size, modified_time));
                }
            }
        }
    };
    Ok(entries)
}

fn main() {
    let args = Args::parse();
    if let Ok(mut entries) = get_entries_in_directory(args.directory.as_str()) {
        entries.sort_by(|a, b| { a.file_size.cmp(&b.file_size) });
        for entry in entries.into_iter() { 
            println!("{}",entry.show());
        }
    }
}
