# rdu

`rdu` is a toy implementation of a subset of `du` functionality.

The main purpose is to play with sync and async IO.

Different version of `du -hs` are in the following branches:
- `sync` - fully blocking standard APIs.
- `async-seq` - async API but sequential processing.
- `async-par` - async API and maximally concurrent processing.