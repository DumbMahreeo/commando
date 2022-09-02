use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandoError {
    #[error(
        "Please ensure that your HOME environment variable is properly set and valid UTF-8 text"
    )]
    EmptyHome,

    #[error("Couldn't create database directory at path '{0}'")]
    CreateDatabase(std::io::Error),

    #[error("Path must be a file, not a directory")]
    PathIsDir,

    #[error("Couldn't create CDB file at path '{0}': {1}")]
    CdbCreation(String, std::io::Error),

    #[error("Cannot open CDB file: {0}")]
    CdbOpen(std::io::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unrecognized file format (wrong magic number). Expected CDB file")]
    BadMagic,

    #[error("Wrong CDB version, please update your database. Expected CDB version {expected}, got {got}")]
    BadVersion { expected: u32, got: u32 },

    #[error("Couldn't extract data from alpm database: {0}")]
    AlpmExtract(compress_tools::Error),

    #[error("Corrupted alpm database: {0}")]
    CorruptedAlpm(compress_tools::Error),

    #[error("No argument specified, please try with --help")]
    NoArgument,

    #[error("<COMMAND> argument's length must be lower than 256")]
    TooLong,

    #[error("Couldn't get pacman-conf's output: {0}")]
    NoPacmanConf(std::io::Error),

    #[error("No working mirror found for repo '{repo}'")]
    NoMirror { repo: String },

    #[error("Couldn't read package name in 'desc' file")]
    PackageNameDescRead,

    #[error("Couldn't find package name in 'desc' file")]
    PackageNameDescFind,
}
