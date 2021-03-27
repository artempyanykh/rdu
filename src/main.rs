use std::{env, error::Error, fs::Metadata, io, path::PathBuf};

use clap::{crate_authors, crate_version, Clap};

use humansize::{file_size_opts, FileSize};

use tokio::fs;

use tokio_stream::wrappers::ReadDirStream;

use futures::{select, stream::FuturesUnordered, StreamExt};

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

async fn calc_space_usage(path: PathBuf) -> Result<u64, io::Error> {
    let mut meta_queue = FuturesUnordered::new();
    meta_queue.push(meta_with_path(path));

    let mut entry_queue = FuturesUnordered::new();

    let mut size = 0;

    loop {
        select! {
            resolved = meta_queue.select_next_some() => {
                let (path, meta) = resolved?;
                let file_type = meta.file_type();

                if file_type.is_symlink() {
                    // don't follow symlinks
                } else if file_type.is_dir() {
                    let entries = fs::read_dir(&path).await?;
                    let entry_stream = ReadDirStream::new(entries);
                    entry_queue.push(entry_stream.into_future());
                } else if file_type.is_file() {
                    size += meta.len();
                }
            },
            (entry, tail) = entry_queue.select_next_some() => {
                if let Some(Ok(entry)) = entry {
                    entry_queue.push(tail.into_future());
                    meta_queue.push(meta_with_path(entry.path()));
                }
            }
            complete => break,
        }
    }

    Ok(size)
}

async fn meta_with_path(path: PathBuf) -> io::Result<(PathBuf, Metadata)> {
    let meta = fs::symlink_metadata(&path).await?;
    Ok((path, meta))
}
