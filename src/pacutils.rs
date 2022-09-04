use std::{env::consts::ARCH, io::Read, process::Command, str::FromStr, thread::spawn};

use lazy_regex::Regex;
use reqwest::Url;

use crate::{database::alpm::RawAlpmDB, error::CommandoError};

/// Arch repo with name and mirrorlist
pub struct Repo {
    name: String,
    mirrors: Vec<String>,
}

/// Parse pacman-conf command's output
pub fn parse_pacman_conf() -> Result<Vec<Repo>, CommandoError> {
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

            Repo {
                name: name.into(),
                mirrors,
            }
        })
        .collect();

    Ok(data)
}

/// Returns a vector of vectors of raw database data.
/// `Vec<Vec<u8>> = Vec<AlpmFileDatabase>`
pub fn download_pacman_db(
    mut repos: Vec<Repo>,
    aur: bool,
) -> Result<Vec<RawAlpmDB>, CommandoError> {
    if aur {
        repos.push(Repo {
            name: "chaotic-aur".into(),
            mirrors: vec![
                format!("https://geo-mirror.chaotic.cx/chaotic-aur/{ARCH}").into(),
                format!("https://cdn-mirror.chaotic.cx/chaotic-aur/{ARCH}").into(),
            ],
        })
    }

    let mut handles = Vec::with_capacity(repos.len());
    for mut repo in repos {
        handles.push(spawn(move || {
            repo.name.push_str(".files");

            for mut mirror in repo.mirrors {
                mirror.push('/'); // might be omittable (Url::from_str can work without)

                // let mut name = repo.0.clone();
                // mirror.push('/');
                let mut mirror_url = Url::from_str(&mirror).unwrap();
                mirror_url = mirror_url.join(&repo.name).unwrap();

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

            Err(CommandoError::NoMirror { repo: repo.name })
        }));
    }

    handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Result<_, _>>()
}
