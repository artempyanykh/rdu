use std::{env, error::Error, future::Future, io, path::PathBuf, pin::Pin};

use clap::{crate_authors, crate_version, Clap};

use humansize::{file_size_opts, FileSize};

use tokio::fs;

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
/// Calculate space usage of a directory tree
struct Opts {
    /// Directory to start from (default = current directory)
    dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::parse();
    let start_dir = match opts.dir {
        Some(dir) => dir,
        _ => env::current_dir()?,
    };

    let usage = calc_space_usage(start_dir.clone()).await?;
    let human_usage = usage.file_size(file_size_opts::CONVENTIONAL)?;

    println!("{}\t{}", human_usage, start_dir.display());
    Ok(())
}

fn calc_space_usage(path: PathBuf) -> Pin<Box<dyn Future<Output = Result<u64, io::Error>>>> {
    Pin::from(Box::new(async move {
        let meta = fs::symlink_metadata(&path).await?;
        let file_type = meta.file_type();

        let mut size = 0;

        if file_type.is_symlink() {
            // don't follow symlinks
        } else if file_type.is_dir() {
            let mut entries = fs::read_dir(&path).await?;
            while let Some(entry) = entries.next_entry().await? {
                let entry = entry;
                size += calc_space_usage(entry.path()).await?;
            }
        } else if file_type.is_file() {
            size = meta.len();
        }

        Ok(size)
    }))
}
