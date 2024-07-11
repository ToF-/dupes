use std::fs::remove_file;
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
    #[arg(short, long, default_value_t = false)]
    remove: bool,
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

#[derive(Clone, Debug)]
struct EntryPair {
    entry_a: Entry,
    entry_b: Entry,
}

impl EntryPair {
    fn show(&self) -> String {
        format!("{}\n{}\n", self.entry_a.file_path, self.entry_b.file_path)
    }
}

type EntryList = Vec<Entry>;
type EntryPairList = Vec<EntryPair>;

fn make_entry(s: String, l: u64, t: SystemTime) -> Entry {
    return Entry {
        file_path: s.clone(),
        file_size: l,
        modified_time: t,
    }
}

fn make_entry_pair(a: Entry, b: Entry) -> EntryPair {
    return EntryPair { entry_a: a, entry_b: b }
}

fn equal_content(a: Entry, b: Entry) -> bool {
    if a.file_size == 0 {
        return false;
    }
    if a.file_size != b.file_size {
        return false;
    };
    match std::fs::read(a.file_path) {
        Ok(a_bytes) => {
            match std::fs::read(b.file_path) {
                Ok(b_bytes) => {
                    for i in 0 .. a_bytes.len() {
                        if a_bytes[i] != b_bytes[i] {
                            return false
                        }
                    }
                },
                Err(err) => {
                    panic!("{}", err)
                },
            }
        },
        Err(err) => {
            panic!("{}", err)
        }
    };
    return true
}

fn duplicate_file(unsorted_entries: EntryList) -> EntryPairList {
    let mut dupes = Vec::new();
    let mut entries = unsorted_entries.clone();
    entries.sort_by(|a, b| { a.file_size.cmp(&b.file_size) });
    let mut track = make_entry(String::from("fancy sentinel entry"), std::u64::MAX, SystemTime::now());
    for entry in entries.into_iter() {
        if entry.file_size == track.file_size {
            if equal_content(track.clone(), entry.clone()) {
                dupes.push(make_entry_pair(track.clone(), entry.clone()));
            }
        };
        track = entry
    };
    dupes
}

fn get_file_entries_in_directory(dir_path: &str) -> io::Result<EntryList> {
    let mut entries: EntryList = Vec::new();
    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.clone().into_path();
        let valid:bool = ! entry.file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false)
            && entry.file_type().is_file();
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
    if let Ok(entries) = get_file_entries_in_directory(args.directory.as_str()) {
        let dupes = duplicate_file(entries);
        for pair in dupes.clone().into_iter() {
            println!("{}",pair.show());
        }
        if args.remove {
            for pair in dupes.into_iter() {
                let entry = pair.entry_b;
                println!("removing file {}", entry.file_path);
                match remove_file(entry.file_path) {
                    Ok(()) => {},
                    Err(err) => eprintln!("{}", err),
                }
            }
        }
    }
}
