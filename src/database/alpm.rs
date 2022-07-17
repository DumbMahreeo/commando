use std::{
    collections::HashMap,
    io::Cursor,
    process::exit,
    sync::{Arc, Mutex},
    thread,
};

use compress_tools::{ArchiveContents, ArchiveIterator};
use lazy_regex::regex;

/// Extracts libalpm file database into Vec<(desc: String, files: String)>
pub fn extract_alpm_db(data: Vec<Vec<u8>>) -> Vec<(String, String)> {
    let mut handles = Vec::with_capacity(3);
    let extracted_data = Arc::new(Mutex::new(Some(Vec::new())));

    for file in data {
        let extracted_data = extracted_data.clone();
        handles.push(thread::spawn(move || {
            let source = Cursor::new(file);

            let archive = match ArchiveIterator::from_read(source) {
                Ok(a) => a,

                Err(e) => {
                    eprintln!(
                        "[FATAL]: couldn't extract data from alpm database.\nError details: {e}"
                    );
                    exit(1);
                }
            };

            let mut start_of_entry = String::new();
            let mut desc = String::new();
            let mut files = String::new();
            for content in archive {
                use ArchiveContents::*;
                match content {
                    StartOfEntry(s) => start_of_entry = s,

                    DataChunk(data) => {
                        if let Ok(data) = String::from_utf8(data) {
                            if start_of_entry.ends_with("/desc") {
                                desc.push_str(&data);
                            } else if start_of_entry.ends_with("/files") {
                                files.push_str(&data);
                            }
                        }
                    }

                    EndOfEntry => {
                        if !desc.is_empty() && !files.is_empty() {

                            if let Ok(mut extracted_data) = extracted_data.lock() {
                                extracted_data.as_mut().unwrap().push((desc, files));
                            }

                            desc = String::new();
                            files = String::new();
                        }
                    }

                    Err(e) => {
                        eprintln!("[FATAL]: corrupted alpm database.\nError details: {e}");
                        exit(1);
                    }
                }
            }
        }));
    }

    for h in handles {
        if h.join().is_err() {
            eprintln!("[FATAL]: thread errored in extract_alpm_db");
            exit(1);
        }
    }

    let extracted_data = extracted_data.lock().unwrap().take().unwrap();
    extracted_data
}

fn read_package_name<S: AsRef<str>>(desc: S) -> String {
    let mut desc = desc.as_ref().split('\n');

    while let Some(line) = desc.next() {
        if line.trim_end() == "%NAME%" {
            return match desc.next() {
                Some(v) => v,
                None => {
                    eprintln!("[FATAL]: couldn't read package name in 'desc' file");
                    exit(1);
                }
            }
            .trim_end()
            .to_string();
        }
    }

    eprintln!("[FATAL]: couldn't find package name in 'desc' file");
    exit(1);
}

fn read_package_bins<S: AsRef<str>>(files: S) -> Vec<String> {
    let files = files.as_ref();

    let re = regex!(r"/bin/([^\s/.]+)");

    let mut commands = Vec::new();
    for caps in re.captures_iter(files) {
        let bin_name = (&caps[1]).to_string();
        if !bin_name.contains("node_modules") && !commands.contains(&bin_name) {
            commands.push(bin_name);
        }
    }

    commands
}

/// Reads raw data from extract_alpm_db and returns
/// `Vec<(Package name: String, Bin list: Vec<String>)`
pub fn parse_alpm_db(data: Vec<(String, String)>) -> HashMap<String, Vec<String>> {
    let mut parsed_data: HashMap<String, Vec<String>> = HashMap::new();
    for package in data {
        let bins = read_package_bins(package.1);
        if !bins.is_empty() {
            let package_name = read_package_name(package.0);

            for bin in bins {
                match parsed_data.get_mut(&bin) {
                    Some(v) => v.push(package_name.clone()),
                    None => {
                        parsed_data.insert(bin, vec![package_name.clone()]);
                    }
                }
            }
        }
    }

    parsed_data
}
