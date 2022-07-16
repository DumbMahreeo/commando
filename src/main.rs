// @todo remove unused imports
use std::env::{args, self};

use database::{cdb::{create_cdb, search_in_cdb}, alpm::{parse_alpm_db, extract_alpm_db}};
use pacutils::{parse_pacman_conf, download_pacman_db};

mod database;
mod pacutils;

// @todo start using clippy
fn main() {
    let mut args = args();
    args.next();
    let arg = args.next().unwrap();

    let mut cdb_path = env::var("HOME").unwrap();
    cdb_path.push_str("/.local/share/commando/cdb.db");

    if arg == "--update" || arg == "-u" {
        println!("Downloading pacman files database");
        let pacman_db = download_pacman_db(parse_pacman_conf());
        println!("Download completed\nReading database data");
        let data = parse_alpm_db(extract_alpm_db(pacman_db));

        println!("Writing data to commando database");
        create_cdb(data, cdb_path);
        println!("All done");
    } else {
        search_in_cdb(arg, cdb_path);
    }
}


// @todo use clippy
