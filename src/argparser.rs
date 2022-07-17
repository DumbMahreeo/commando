use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None, after_help="NOTE: To disable colors set env flag NO_COLOR=1\n\n")]
pub struct Args {
    //#[clap(short, long, value_parser, help="The command to search")]
    #[clap(value_parser, help="The command to search")]
    pub command: Option<String>,

    #[clap(short, long, value_parser, help="Create or update the database")]
    pub update: bool,

    #[clap(short, long, value_parser, help="The path of the database to crate/update or search in")]
    pub path: Option<PathBuf>,
}


