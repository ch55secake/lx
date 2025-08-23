use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "\x1b[1m\x1b[95mList files and directories within a directory.\x1b[0m\x1b[39m"
)]
pub(crate) struct Args {
    /// Name of the directory
    #[clap(default_value = ".")]
    pub(crate) directory: String,

    /// Show file permissions
    #[clap(short, long, action = clap::ArgAction::SetTrue)]
    pub(crate) permissions: bool,

    /// Show all items
    #[clap(short, long, action = clap::ArgAction::SetTrue)]
    pub(crate) all: bool,

    /// Filter files by pattern (supports wildcards with '*')
    #[clap(short, long, default_value = "*")]
    pub(crate) find: String,

    /// Recursively search through subdirectories
    #[clap(short, long, action = clap::ArgAction::SetTrue)]
    pub(crate) recursive: bool,
}
