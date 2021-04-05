# rdu

`rdu` is a toy implementation of a subset of `du` functionality.
The main purpose of this is to play with sync and async IO.

There are 3 different versions of `du` implemented:
- `rdu-sync` - fully blocking standard APIs.


- `rdu-async-seq` - async API but sequential processing. Basically, here we just
  replace `std::fs` with `tokio::fs` and insert `.await` when needed.

- `rdu-async-par` - async API and maximally concurrent processing. Instead of
  waiting sequentially, we use `FuturesUnordered` from `futures` crate to do
  as much awaiting concurently as we can.
  
## Benchmarks

Hardware:
```
$ inxi -bm
System:    Kernel: 5.11.0-7612-generic x86_64 bits: 64
Memory:    RAM: total: 62.78 GiB used: 7.60 GiB (12.1%)
CPU:       Info: 6-Core AMD Ryzen 5 3600 [MT MCP] speed: 2195 MHz min/max: 2200/3600 MHz
Drives:    Local Storage:
           ID-1: /dev/nvme0n1 vendor: Samsung model: SSD 970 EVO Plus 1TB size: 931.51 GiB
           ID-2: /dev/nvme1n1 vendor: Samsung model: SSD 970 EVO Plus 1TB size: 931.51 GiB
```

Contents of the test folder:
```
for d in $(ls); do \
  cd $d; \
  echo "$(git remote get-url origin) ($(git rev-parse --short HEAD))"; \
  cd ..; \
done
```

```
git@github.com:lampepfl/dotty.git (a288432bee)
git@github.com:emacs-mirror/emacs.git (a789d8a3a0)
git@github.com:openjdk/jdk.git (7f9ece23dc6)
git@github.com:torvalds/linux.git (1678e493d530)
git@github.com:rust-analyzer/rust-analyzer.git (ea8feca31)
```

### Native Linux

**With warm disk cache:**

```bash
$ hyperfine \
  -L exe du,target/release/rdu-sync,target/release/rdu-async-seq,target/release/rdu-async-par \
  '{exe} -hs ~/dev/rdu-test/'

Benchmark #1: du -hs ~/dev/rdu-test/
  Time (mean ± σ):     261.6 ms ±   2.6 ms    [User: 35.3 ms, System: 223.7 ms]
  Range (min … max):   255.1 ms … 264.5 ms    11 runs
 
Benchmark #2: target/release/rdu-sync -hs ~/dev/rdu-test/
  Time (mean ± σ):     345.3 ms ±   3.3 ms    [User: 62.3 ms, System: 280.0 ms]
  Range (min … max):   340.7 ms … 350.9 ms    10 runs
 
Benchmark #3: target/release/rdu-async-seq -hs ~/dev/rdu-test/
  Time (mean ± σ):      3.200 s ±  0.449 s    [User: 1.230 s, System: 2.459 s]
  Range (min … max):    2.539 s …  3.894 s    10 runs
 
Benchmark #4: target/release/rdu-async-par -hs ~/dev/rdu-test/
  Time (mean ± σ):      1.165 s ±  0.031 s    [User: 1.246 s, System: 1.657 s]
  Range (min … max):    1.118 s …  1.209 s    10 runs
 
Summary
  'du -hs ~/dev/rdu-test/' ran
    1.32 ± 0.02 times faster than 'target/release/rdu-sync -hs ~/dev/rdu-test/'
    4.45 ± 0.13 times faster than 'target/release/rdu-async-par -hs ~/dev/rdu-test/'
   12.23 ± 1.72 times faster than 'target/release/rdu-async-seq -hs ~/dev/rdu-test/'
```


We can see that `rdu-sync` (which is a **straightforward** blocking solution) is only about 35% slower
than `du -hs`, which is not too bad for 20 LOC!

The `async-seq` version is 12 times slower. Obviously, we couldn't expect speedup going from `sync` to `async-seq` - this is still the same sequential processing, but now with async overhead. However, 12x slower really extreme!

A more optimized version with real concurrent/parallel processing `async-par` runs much faster than `async-seq` (at the expense of much more complex code), but is still very slow compared to a blocking version.

**With cold disk cache:**

Now, let's run the benchmark again, but this time we try to flush disk caches
before each run (fingers crossed, `echo 3 | sudo tee
/proc/sys/vm/drop_caches` does what I think it does):

```bash
$ hyperfine \
  -L exe du,target/release/rdu-sync,target/release/rdu-async-seq,target/release/rdu-async-par \
  '{exe} -hs ~/dev/rdu-test/' \
  -p 'echo 3 | sudo tee /proc/sys/vm/drop_caches'

Benchmark #1: du -hs ~/dev/rdu-test/
  Time (mean ± σ):      2.504 s ±  0.013 s    [User: 69.0 ms, System: 579.5 ms]
  Range (min … max):    2.488 s …  2.532 s    10 runs
 
Benchmark #2: target/release/rdu-sync -hs ~/dev/rdu-test/
  Time (mean ± σ):      2.617 s ±  0.012 s    [User: 83.1 ms, System: 670.5 ms]
  Range (min … max):    2.600 s …  2.642 s    10 runs
 
Benchmark #3: target/release/rdu-async-seq -hs ~/dev/rdu-test/
  Time (mean ± σ):      5.043 s ±  0.427 s    [User: 1.029 s, System: 2.639 s]
  Range (min … max):    4.322 s …  5.566 s    10 runs
 
Benchmark #4: target/release/rdu-async-par -hs ~/dev/rdu-test/
  Time (mean ± σ):      1.206 s ±  0.015 s    [User: 1.302 s, System: 2.529 s]
  Range (min … max):    1.183 s …  1.233 s    10 runs
 
Summary
  'target/release/rdu-async-par -hs ~/dev/rdu-test/' ran
    2.08 ± 0.03 times faster than 'du -hs ~/dev/rdu-test/'
    2.17 ± 0.03 times faster than 'target/release/rdu-sync -hs ~/dev/rdu-test/'
    4.18 ± 0.36 times faster than 'target/release/rdu-async-seq -hs ~/dev/rdu-test/'
```

This time around `async-par` beats both `du` and `rdu-sync` by 2x. This makes
sense, since `async-par` tries to do many things concurrently, however I
expected something better than 2x.