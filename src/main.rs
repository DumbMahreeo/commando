use std::{
    env,
    fs::create_dir_all,
    path::{Path, PathBuf},
    process::exit,
};

use argparser::Args;
use clap::Parser;
use database::{
    alpm::{extract_alpm_db, parse_alpm_db},
    cdb::{create_cdb, search_in_cdb},
};
use error::CommandoError;
use log::LevelFilter;
use pacutils::{download_pacman_db, parse_pacman_conf};

mod argparser;
mod database;
mod error;
mod pacutils;
mod colors;

fn main() {
    if let Err(err) = run() {
        log::error!("{err}");
        exit(1)
    }
}

fn get_home_path() -> Result<PathBuf, CommandoError> {
    let home = env::var("HOME");
    let home = match home.as_deref() {
        Err(_) | Ok("") => Err(CommandoError::EmptyHome),
        Ok(home) => Ok(home),
    }?;

    let path = Path::new(home).join(".local/share/commando");

    create_dir_all(&path).map_err(CommandoError::CreateDatabase)?;
    Ok(path.join("cdb.db"))
}

fn run() -> Result<(), CommandoError> {
    let args = Args::parse();
    env_logger::Builder::new()
        .format_timestamp(None)
        .filter_level(
            args.debug
                .then_some(LevelFilter::Debug)
                .unwrap_or(LevelFilter::Info),
        )
        .init();

    let path = match args.path {
        Some(path) => path,
        None => get_home_path()?,
    };

    if path.is_dir() {
        return Err(CommandoError::PathIsDir);
    }

    if args.update {

        log::debug!("Downloading pacman files database");
        if args.verbose && !args.debug {
            println!("Downloading pacman files database");
        }

        let pacman_db = download_pacman_db(parse_pacman_conf()?, args.aur)?;

        log::debug!("Download completed. Reading database data");
        if args.verbose && !args.debug {
            println!("Download completed. Reading database data");
        }

        log::debug!("Extracting and parsing alpm db data");
        let data = parse_alpm_db(extract_alpm_db(pacman_db)?)?;

        log::debug!("Writing data to commando database");
        if args.verbose && !args.debug {
            println!("Writing data to commando database");
        }

        create_cdb(data, path)?;

        log::debug!("CDB file created. Update completed");
        if args.verbose && !args.debug {
            println!("All done");
        }

        return Ok(());
    }

    match args.command {
        Some(command) if command.len() <= 255 => search_in_cdb(command, path, args.verbose),
        Some(_) => Err(CommandoError::TooLong),
        None => Err(CommandoError::NoArgument),
    }
}

