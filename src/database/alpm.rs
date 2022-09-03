use std::{
    collections::HashMap,
    io::Cursor,
    thread::{self, JoinHandle},
};

use bytes::Bytes;
use compress_tools::{ArchiveContents, ArchiveIterator};
use lazy_regex::regex;

use crate::error::CommandoError;

use super::cdb::CDBEntry;

/// Raw alpm database bytes
pub type RawAlpmDB = Bytes;

/// The representation of an extracted directory from an alpm database for a single package
pub struct PackageDir {
    /// The 'desc' file
    desc: String,

    /// The 'files' file
    files: String,
}

/// Extracts libalpm file database into a vector of packages
pub fn extract_alpm_db(data: Vec<RawAlpmDB>) -> Result<Vec<PackageDir>, CommandoError> {
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
                    ArchiveContents::Err(e) => return Err(CommandoError::ReadCorruptedAlpm(e)),
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
                            extracted_data.push(PackageDir { desc, files });

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

/// Read package name from 'desc' file
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

/// Read package binaries from 'files' file
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

/// Reads data from PackageDir and returns a list of CDB entries
pub fn parse_alpm_db(data: Vec<PackageDir>) -> Result<Vec<CDBEntry>, CommandoError> {
    let mut parsed_data: HashMap<String, Vec<String>> = HashMap::new();
    for package in data {
        let bins = read_package_bins(package.files);
        if !bins.is_empty() {
            let package_name = read_package_name(package.desc)?;

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

    let mut entries = Vec::with_capacity(parsed_data.len());

    for (command, packages) in parsed_data {
        entries.push(CDBEntry { command, packages })
    }

    Ok(entries)
}
