use std::{process::{Command, exit}, thread::{self, spawn}, io::{Read, Write}, fs::File, path::PathBuf, str::FromStr};

use lazy_regex::Regex;
use reqwest::Url;

pub fn parse_pacman_conf() -> Vec<(String, Vec<String>)> {
    match Command::new("pacman-conf").output() {
        Ok(output) => {
            let output = String::from_utf8(output.stdout).unwrap();
            let mut output = output.split("\n[").into_iter();

            output.next();
            let name_re = Regex::new(r"(.*)]").unwrap();
            let mirror_re = Regex::new(r"Server\s?=\s?(.*)").unwrap();

            let mut data = Vec::new();

            for repo in output {
                let name = &name_re.captures(repo).unwrap()[1];
                let mut mirrors = Vec::new();

                for mirror in mirror_re.captures_iter(repo) {
                    mirrors.push(mirror[1].to_string());
                }

                data.push((name.to_string(), mirrors));
            }

            return data;
        },

        Err(e) => {
            eprintln!("[FATAL]: couldn't get pacman-conf's output.\nError details: {e}");
            exit(1);
        },
    }
}

pub fn download_pacman_db(repos: Vec<(String, Vec<String>)>) -> Vec<(String, Vec<u8>)> {
    let mut handles = Vec::with_capacity(repos.len());
    for repo in repos {
        handles.push(spawn(move || {
            for mut mirror in repo.1 {
                let mut name = repo.0.clone();
                mirror.push('/');
                let mut mirror = Url::from_str(&mirror).unwrap();
                name.push_str(".files");
                mirror = mirror.join(&name).unwrap();

                if let Ok(mut response) = reqwest::blocking::get(mirror) {
                    let mut buf = Vec::new();
                    response.read_to_end(&mut buf).unwrap();
                    return (name, buf);
                };
            }
            eprintln!("[FATAL]: no working mirror found for repo '{}'.\nPerhaps check your internet connection and DNS resolver", repo.0);
            exit(1);
        }));
    }

    let mut data = Vec::with_capacity(handles.len());
    for handle in handles {
        data.push(handle.join().unwrap());
    }

    data

    // todo: proper error handling
}
