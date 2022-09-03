use std::{process::Command, str::FromStr, rc::Rc, mem::take};

use bytes::Bytes;
use futures::{stream::FuturesUnordered, StreamExt, FutureExt};
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
pub fn download_pacman_db(mut repos: Vec<Repo>) -> Result<Vec<RawAlpmDB>, CommandoError> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut data_vec = Vec::with_capacity(repos.len());

        let futs = repos.iter_mut().map(|repo| async move {
            repo.name.push_str(".files");

            let client = reqwest::Client::new();
            let mut mirrors = take(&mut repo.mirrors).into_iter();

            while let Some(mut mirror) = mirrors.next() {
                mirror.push('/'); // might be omittable (Url::from_str can work without)

                // let mut name = repo.0.clone();
                // mirror.push('/');
                let mut mirror_url = Url::from_str(&mirror).unwrap();
                mirror_url = mirror_url.join(&repo.name).unwrap();

                let res =
                    match client.get(mirror_url).send().await.map(|res| res.error_for_status()) {
                        Ok(Ok(res)) => res,
                        Ok(Err(e)) | Err(e) => {
                            log::warn!("Mirror {mirror} timed out: {e}. Trying next mirror");
                            continue;
                        }
                    };

                return res.bytes().await.map_err(|e| CommandoError::ReceivedCorruptedAlpm(e));
            }

            Err(CommandoError::NoMirror { repo: repo.name.clone() })
        }).collect::<FuturesUnordered<_>>()
        .collect::<Vec<_>>().await;

        for data in futs {
            data_vec.push(data?)
        }

        Ok(data_vec)
    })
}
