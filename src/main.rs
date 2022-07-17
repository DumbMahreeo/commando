use std::{env, fs::create_dir_all, path::Path, process::exit};

use argparser::Args;
use clap::Parser;
use database::{
    alpm::{extract_alpm_db, parse_alpm_db},
    cdb::{create_cdb, search_in_cdb},
};
use pacutils::{download_pacman_db, parse_pacman_conf};

mod argparser;
mod database;
mod pacutils;

fn main() {
    let args = Args::parse();

    let mut path = args.path.unwrap_or_else(|| {
        let home = match env::var("HOME") {
            Ok(h) if h.is_empty() => {
                eprintln!("[FATAL]: please ensure that your HOME environment variable is properly set and valid UTF-8 text.\nError details: HOME env var is empty");
                exit(1);
            }

            Err(e) => {
                eprintln!("[FATAL]: please ensure that your HOME environment variable is properly set and valid UTF-8 text.\nError details: {e}");
                exit(1);
            }

            Ok(h) => h,
        };


        let path = Path::new(&home);
        path.join(".local/share/commando/cdb.db")
    });

    if path.ends_with("/") {
        if !path.exists() {
            if let Err(e) = create_dir_all(&path) {
                eprintln!(
                    "[FATAL]: couldn't create dir '{}'.\nError details: {e}",
                    path.display()
                );
                exit(1);
            }
        }

        path = path.join("cdb.db")
    } else {
        let path = path.parent().unwrap_or_else(|| Path::new("/"));

        if !path.exists() {
            if let Err(e) = create_dir_all(&path) {
                eprintln!(
                    "[FATAL]: couldn't create dir '{}'.\nError details: {e}",
                    path.display()
                );
                exit(1);
            }
        }
    }

    if args.update {
        println!("Downloading pacman files database");

        let pacman_db = download_pacman_db(parse_pacman_conf());

        println!("Download completed\nReading database data");

        let data = parse_alpm_db(extract_alpm_db(pacman_db));

        println!("Writing data to commando database");

        create_cdb(data, path);

        println!("All done");
        exit(0);
    }

    if let Some(command) = args.command {
        if command.len() <= 255 {
            search_in_cdb(command, path);
        } else {
            eprintln!("[FATAL]: <COMMAND> argument's length must be lower than 256");
            exit(1);
        }
    } else {
        eprintln!("No argument specified, please try with --help");
        exit(1)
    }
}
