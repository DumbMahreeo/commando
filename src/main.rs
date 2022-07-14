use std::env::{args, self};

use database::{cdb::{create_cdb, search_in_cdb}, alpm::{parse_alpm_db, extract_alpm_db}};
use pacutils::{parse_pacman_conf, download_pacman_db};

mod database;
mod pacutils;

fn main() {
    let mut args = args();
    args.next();
    let arg = args.next().unwrap();

    let mut cdb_path = env::var("HOME").unwrap();
    cdb_path.push_str("/.local/share/commando/cdb.db");

    if arg == "--update" || arg == "-u" {
        extract_alpm_db("extracted", download_pacman_db(parse_pacman_conf()));
        create_cdb(parse_alpm_db("extracted"), cdb_path);
    } else {
        search_in_cdb(arg, cdb_path);
    }
}
