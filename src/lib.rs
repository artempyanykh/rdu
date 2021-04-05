use std::path::PathBuf;

use clap::{crate_authors, crate_version, Clap};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
/// Calculate space usage of a directory tree
pub struct Opts {
    /// Directory to start from (default = current directory)
    pub dir: Option<PathBuf>,
    #[clap(short, long)]
    pub human_readable: bool,
    #[clap(short, long)]
    pub summarize: bool,
}
