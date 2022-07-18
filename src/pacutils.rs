use std::{io::Read, process::Command, str::FromStr, thread::spawn};

use lazy_regex::Regex;
use reqwest::Url;

use crate::error::CommandoError;

pub fn parse_pacman_conf() -> Result<Vec<(String, Vec<String>)>, CommandoError> {
    let output = Command::new("pacman-conf")
        .output()
        .map_err(CommandoError::NoPacmanConf)?;

    let output = String::from_utf8(output.stdout).unwrap();
    let output = output.split("\n[").skip(1);

    let name_re = Regex::new(r"(.*)]").unwrap();
    let mirror_re = Regex::new(r"Server\s?=\s?(.*)").unwrap();

    let data = output
        .map(|repo| {
            let name = &name_re.captures(repo).unwrap()[1];
            let mirrors = mirror_re
                .captures_iter(repo)
                .map(|mirror| mirror[1].to_string())
                .collect();

            (name.to_string(), mirrors)
        })
        .collect();

    Ok(data)
}

/// Returns a vector of vectors of raw database data.
/// `Vec<Vec<u8>> = Vec<AlpmFileDatabase>`
pub fn download_pacman_db(
    repos: Vec<(String, Vec<String>)>,
) -> Result<Vec<Vec<u8>>, CommandoError> {
    let mut handles = Vec::with_capacity(repos.len());
    for (mut name, mirrors) in repos {
        handles.push(spawn(move || {
            name.push_str(".files");

            for mut mirror in mirrors {
                mirror.push('/'); // might be omittable (Url::from_str can work without)

                // let mut name = repo.0.clone();
                // mirror.push('/');
                let mut mirror_url = Url::from_str(&mirror).unwrap();
                mirror_url = mirror_url.join(&name).unwrap();

                let mut res =
                    match reqwest::blocking::get(mirror_url).map(|res| res.error_for_status()) {
                        Ok(Ok(res)) => res,
                        Ok(Err(e)) | Err(e) => {
                            log::warn!("Mirror {mirror} timed out: {e}. Trying next mirror");
                            continue;
                        }
                    };

                let mut buf = Vec::new();
                res.read_to_end(&mut buf).unwrap();

                return Ok(buf);
            }

            Err(CommandoError::NoMirror { repo: name })
        }));
    }

    // let mut data = Vec::with_capacity(handles.len());
    // for handle in handles {
    //     data.push(match handle.join() {
    //         Ok(v) => v,
    //         Err(_) => {
    //             eprintln!("[FATAL]: thread errored");
    //             exit(1);
    //         }
    //     });
    // }

    handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Result<_, _>>()
}
