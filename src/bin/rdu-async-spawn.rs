use clap::Clap;
use humansize::{file_size_opts, FileSize};
use rdu::Opts;
use std::{env, error::Error, io, path::PathBuf};
use tokio::fs;
use tokio::fs::DirEntry;
use async_recursion::async_recursion;

// the single threaded version is a smidge faster
// but darn close to measurment noise. But It'd
// expect it to be faster (no context switches, and
// IO bonud app...)
#[tokio::main(flavor = "current_thread")]
//#[tokio::main]
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


// use fake recursion!
// do a goroutine / kotlin-coroutine style and spin up
// a new task for each directly. A bit easier to follow (a bit.. not much).
#[async_recursion]
async fn calc_space_usage(path: PathBuf) -> Result<u64, io::Error> {
    let mut sub_dir:Vec<DirEntry> = Vec::new();
    let mut dir_files_size=0u64;

    let mut read_dir = fs::read_dir(path).await.unwrap();

    // I wish I could just iterate on this? or yield the results
    // in a stream or something? But I guess I have to await on each
    // node... so I kinda understand why (kinda).
    while let Some(entry) = read_dir.next_entry().await? {
        let f_type = entry.file_type().await?;
        if f_type.is_file() {
            dir_files_size += entry.metadata().await?.len();
        } else if f_type.is_dir() {
            sub_dir.push(entry )
        }
    }

    // I don't think this is "real" recursion. This isn't growing the stack, as the swpan and recursion macro
    // are creating a new stack frame on the heap.
    // it isn't good though. This will OMM faster tha other imples.
    let tasks = sub_dir.iter().map(|d| tokio::spawn( calc_space_usage( d.path())));

    let mut done_list = futures::future::try_join_all( tasks ).await?;

    // more mutable state cuz I'm not good enough at rust
    // to figure out why the borrow checker hates it when I iterate over Vec<Result<>>
    let mut subdirs_size = 0u64;

    while let Some(res) = done_list.pop() {
        subdirs_size += res.unwrap_or(0);
    }

    Ok(dir_files_size+subdirs_size)

}
