use std::{
    collections::HashMap,
    io::Cursor,
    thread::{self, JoinHandle},
};

use compress_tools::{ArchiveContents, ArchiveIterator};
use lazy_regex::regex;

use crate::error::CommandoError;

/// Extracts libalpm file database into Vec<(desc: String, files: String)>
pub fn extract_alpm_db(data: Vec<Vec<u8>>) -> Result<Vec<(String, String)>, CommandoError> {
    let mut handles = Vec::with_capacity(3);

    for file in data {
        // let extracted_data = extracted_data.clone();
        handles.push(thread::spawn(move || -> Result<Vec<_>, CommandoError> {
            let source = Cursor::new(file);
            let mut extracted_data = Vec::new();

            let archive = ArchiveIterator::from_read(source)
                .map_err(CommandoError::AlpmExtract)
                .unwrap();

            let mut start_of_entry = String::new();
            let mut desc = String::new();
            let mut files = String::new();
            for content in archive {
                // use ArchiveContents::*;
                match content {
                    ArchiveContents::Err(e) => return Err(CommandoError::CorruptedAlpm(e)),
                    ArchiveContents::StartOfEntry(s) => start_of_entry = s,
                    ArchiveContents::DataChunk(data) => {
                        if let Ok(data) = String::from_utf8(data) {
                            if start_of_entry.ends_with("/desc") {
                                desc.push_str(&data);
                            } else if start_of_entry.ends_with("/files") {
                                files.push_str(&data);
                            }
                        }
                    }
                    ArchiveContents::EndOfEntry => {
                        if !desc.is_empty() && !files.is_empty() {
                            // if let Ok(mut extracted_data) = extracted_data.lock() {
                            //     extracted_data.as_mut().unwrap().push((desc, files));
                            // }
                            extracted_data.push((desc, files));

                            desc = String::new();
                            files = String::new();
                        }
                    }
                }
            }

            Ok(extracted_data)
        }));
    }

    let results: Vec<_> = handles
        .into_iter()
        .map(|handle| JoinHandle::join(handle).unwrap())
        .collect::<Result<_, _>>()?;

    Ok(results.into_iter().flatten().collect())
}

fn read_package_name<S: AsRef<str>>(desc: S) -> Result<String, CommandoError> {
    let mut desc = desc.as_ref().split('\n');

    while let Some(line) = desc.next() {
        if line.trim_end() == "%NAME%" {
            let desc = desc
                .next()
                .ok_or(CommandoError::PackageNameDescRead)?
                .trim_end()
                .to_string();

            return Ok(desc);
        }
    }

    Err(CommandoError::PackageNameDescFind)
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
pub fn parse_alpm_db(
    data: Vec<(String, String)>,
) -> Result<HashMap<String, Vec<String>>, CommandoError> {
    let mut parsed_data: HashMap<String, Vec<String>> = HashMap::new();
    for (desc, files) in data {
        let bins = read_package_bins(files);
        if !bins.is_empty() {
            let package_name = read_package_name(desc)?;

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

    Ok(parsed_data)
}
