use std::{
    env,
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

use clap::{crate_authors, crate_version, Clap};

use humansize::{file_size_opts, FileSize};

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
    let human_usage = usage.file_size(file_size_opts::CONVENTIONAL)?;

    println!("{}\t{}", human_usage, start_dir.display());
    Ok(())
}

fn calc_space_usage(path: &Path) -> Result<u64, io::Error> {
    let meta = fs::symlink_metadata(&path)?;
    let file_type = meta.file_type();

    let mut size = 0;

    if file_type.is_symlink() {
        // don't follow symlinks
    } else if file_type.is_dir() {
        let entries = fs::read_dir(&path)?;
        for entry in entries {
            let entry = entry?;
            size += calc_space_usage(&entry.path())?;
        }
    } else if file_type.is_file() {
        size = meta.len();
    }

    Ok(size)
}
