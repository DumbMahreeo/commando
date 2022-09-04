use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None, after_help="NOTE: To disable colors set env var NO_COLOR=1\n\nNOTE: Default database path is $HOME/.local/share/commando/cdb.db\n\n")]
pub struct Args {
    #[clap(value_parser, help = "The command to search")]
    pub command: Option<String>,

    #[clap(short, long, value_parser, help = "Create or update the database")]
    pub update: bool,

    #[clap(short, long, value_parser, help = "Print verbose output to stdout")]
    pub verbose: bool,

    #[clap(short, long, value_parser, help = "Print debug output to stderr")]
    pub debug: bool,

    #[clap(short, long, value_parser, help = "Include AUR packages in the update trough chaotic-aur repos")]
    pub aur: bool,

    #[clap(
        short,
        long,
        value_parser,
        help = "The path of the database file to create/update or search in"
    )]
    pub path: Option<PathBuf>,
}
