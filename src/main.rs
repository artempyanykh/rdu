use std::{env, error::Error, fs::Metadata, io, path::PathBuf};

use clap::{crate_authors, crate_version, Clap};

use humansize::{file_size_opts, FileSize};

use tokio::{fs, runtime};

use futures::{stream::FuturesUnordered, TryStreamExt};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
/// Calculate space usage of a directory tree
struct Opts {
    /// Directory to start from (default = current directory)
    dir: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let rt = runtime::Runtime::new()?;
    rt.block_on(main_entry())
}

async fn main_entry() -> Result<(), Box<dyn Error>> {
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

async fn calc_space_usage(path: PathBuf) -> Result<u64, io::Error> {
    let mut meta_queue = FuturesUnordered::new();
    meta_queue.push(meta_with_path(path));

    let mut size = 0;

    while let Some((path, meta)) = meta_queue.try_next().await? {
        let file_type = meta.file_type();

        if file_type.is_symlink() {
            // don't follow symlinks
        } else if file_type.is_dir() {
            // This is a blocking operation!
            for entry in std::fs::read_dir(&path)? {
                let entry = entry?;
                meta_queue.push(meta_with_path(entry.path()));
            }
        } else if file_type.is_file() {
            size += meta.len();
        }
    }

    Ok(size)
}

async fn meta_with_path(path: PathBuf) -> io::Result<(PathBuf, Metadata)> {
    let meta = fs::symlink_metadata(&path).await?;
    Ok((path, meta))
}
