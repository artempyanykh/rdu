use std::{env, error::Error, path::PathBuf};

use clap::{crate_authors, crate_version, Clap};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
/// Calculate space usage of a directory tree
struct Opts {
    /// Directory to start from (default = current directory)
    dir: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();
    let start_dir = match opts.dir {
        Some(dir) => dir,
        _ => env::current_dir()?,
    };

    println!("Starting from {}", start_dir.display());
    Ok(())
}
