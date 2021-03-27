use std::{
    env,
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

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

    let usage = calc_space_usage(&start_dir)?;

    println!("{}\t{}", usage, start_dir.display());
    Ok(())
}

fn calc_space_usage(path: &Path) -> Result<u64, io::Error> {
    let meta = fs::metadata(&path)?;

    let mut size = 0;

    if meta.is_dir() {
        let entries = fs::read_dir(&path)?;
        for entry in entries {
            let entry = entry?;
            size += calc_space_usage(&entry.path())?;
        }
    } else {
        size = meta.len();
    }

    Ok(size)
}
