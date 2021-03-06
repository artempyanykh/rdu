use clap::Clap;
use humansize::{file_size_opts, FileSize};
use rdu::Opts;
use std::{env, error::Error, io, path::PathBuf};
use tokio::fs;

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
    let mut meta_queue = vec![path];
    let mut size = 0;

    while let Some(path) = meta_queue.pop() {
        let meta = fs::symlink_metadata(&path).await?;
        let file_type = meta.file_type();

        if file_type.is_symlink() {
            // don't follow symlinks
        } else if file_type.is_dir() {
            let mut entries = fs::read_dir(&path).await?;
            while let Some(entry) = entries.next_entry().await? {
                meta_queue.push(entry.path());
            }
        } else if file_type.is_file() {
            size += meta.len();
        }
    }

    Ok(size)
}
