# rdu

`rdu` is a toy implementation of a subset of `du` functionality.

The main purpose is to play with sync and async IO.

Here's the baseline numbers:
```bash
hyperfine 'du -hs ~/dev'
Benchmark #1: du -hs ~/dev
  Time (mean ± σ):     192.7 ms ±   1.0 ms    [User: 20.8 ms, System: 171.5 ms]
  Range (min … max):   191.8 ms … 195.5 ms    15 runs

# PERF
192.71 msec task-clock                #    0.996 CPUs utilized
     7      context-switches          #    0.036 K/sec
     0      cpu-migrations            #    0.000 K/sec
   567      page-faults               #    0.003 M/sec
```

Different version of `du -hs` are in the following branches:
- `sync` - fully blocking standard APIs.
  ```bash
  hyperfine -i 'target/release/rdu ~/dev'
  Benchmark #1: target/release/rdu ~/dev
    Time (mean ± σ):     259.7 ms ±   1.5 ms    [User: 51.2 ms, System: 207.9 ms]
    Range (min … max):   257.7 ms … 262.2 ms    11 runs
  
  # PERF
  258.91 msec task-clock                #    0.997 CPUs utilized
      10      context-switches          #    0.039 K/sec
       0      cpu-migrations            #    0.000 K/sec
     280      page-faults               #    0.001 M/sec
  ```

  This is about 35% slower than `du -hs` which is not too bad for a 10-20 LOC
  straightforward solution.

- `async-seq` - async API but sequential processing. Basically, here we just
  replace `std::fs` with `tokio::fs` and insert `.await` when needed.
  ```bash
  hyperfine -i 'target/release/rdu ~/dev'
  Benchmark #1: target/release/rdu ~/dev
    Time (mean ± σ):     16.552 s ±  0.092 s    [User: 3.769 s, System: 22.942 s]
    Range (min … max):   16.388 s … 16.676 s    10 runs

  # PERF
  14431.96 msec task-clock                #    0.817 CPUs utilized
    743625      context-switches          #    0.052 M/sec
       104      cpu-migrations            #    0.007 K/sec
       411      page-faults               #    0.028 K/sec
  ```

  Now this is just devastatingly slow, 64x slower than the blocking version. The number of context switches and cpu migrations is saddening.

- `async-par` - async API and maximally concurrent processing. Instead of
  waiting sequentially, we use `FuturesUnordered` from `futures` crate to do
  as much awaiting concurently as we can.
  
  ```bash
  hyperfine -i 'target/release/rdu ~/dev'
  Benchmark #1: target/release/rdu ~/dev
    Time (mean ± σ):      6.561 s ±  0.094 s    [User: 3.730 s, System: 13.271 s]
    Range (min … max):    6.450 s …  6.777 s    10 runs

  # PERF
  9419.39 msec task-clock                #    1.398 CPUs utilized
   409186      context-switches          #    0.043 M/sec
     1777      cpu-migrations            #    0.189 K/sec
     2704      page-faults               #    0.287 K/sec
  ```

  This one 2.5x faster than the previous one, but still unbearably slow.