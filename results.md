# results of running CI with different cache schemas

Disclaimer: Github actions public runners can fluctuate in performance so I did notice some runs take +-25% more or less time to complete without any changes to the code or action but it's still enough to see general tendencies

## Cache types explained

- `home` - only caching recommended folders from `$CARGO_HOME` without any caching of build artifacts (eliminates `crates.io` sync)
- `home_and_target` - caching recommended folders from `$CARGO_HOME` + whole `target` folder
- `full` - caching whole `$CARGO_HOME` + whole `target` folders
- `swatinem` - using `Swatinem/rust-cache@v2` as is (it uses the same set of folders as `home_and_target` but does additional cleanup to only cache dependencies)
- `sccache` - using `mozilla/sccache` and caching recommended folders from `$CARGO_HOME` (`$CARGO_HOME/git/db`, `$CARGO_HOME/registry/index`, `$CARGO_HOME/registry/cache`) + sccache cache folder (`$SCCACHE_DIR`) without caching build artifacts themselves

## sled

| cache type | cache size | toolchain + job setup time | cache sync time | first run (no cache) | second run (cache, no mtime restore) | third run (cache, mtime restore) |
| --- | --- | --- | --- | --- | --- | --- |
| home | 86 mb | 15s | 2s | 1m 50s | 1m 27s | 1m 27s |
| home_and_target| 250 mb | 15s | 8s | 2m 8s | 31s | 29s |
| full | 290 mb | 15s | 13s | 1m 56s | 38s | 31s |
| swatinem | 250 mb | 15s | 9s | 1m 49s | 34s | 27s |
| sccache | 170 mb | 25s | 5s | 2m 5s | 1m 1s | 1m 18s |

### cached build with no mtime restore

```sh
$ cargo test --locked --no-run
   Compiling linkmapper v0.1.0 (/home/runner/work/rust-ci-playground/rust-ci-playground)
    Finished test [unoptimized + debuginfo] target(s) in 4.09s
  Executable unittests src/main.rs (target/debug/deps/linkmapper-da17f15a9210ca29)
```

### cached build with mtime restore

```sh
$ cargo test --locked --no-run
    Finished test [unoptimized + debuginfo] target(s) in 0.62s
  Executable unittests src/main.rs (target/debug/deps/linkmapper-da17f15a9210ca29)
```

## rocksdb

| cache type | cache size | toolchain + job setup time | cache sync time | first run (no cache) | second run (cache, no mtime restore) | third run (cache, mtime restore) |
| --- | --- | --- | --- | --- | --- | --- |
| home | 100 mb | 15s | 2s | 8m 53s | 8m 26s | 8m 38s |
| home_and_target| 780 mb | 15s | 25s | 10m 11s | 10m 40s | 8m 28s |
| full | 830 mb | 15s | 26s | 9m 38s | 56s | 47s |
| swatinem | 730 mb | 15s | 23s | 9m 0s | 8m 26s | 10m 16s |
| sccache | 490 mb | 25s | 10s | 13m 58s | 2m 10s | 2m 50s |

### cached build with ~/.cargo/registry/src removed

```sh
$ cargo test --locked --no-run
   Compiling librocksdb-sys v0.8.0+7.4.4
   Compiling rocksdb v0.19.0
   Compiling linkmapper v0.1.0 (/home/runner/work/rust-ci-playground/rust-ci-playground)
    Finished test [unoptimized + debuginfo] target(s) in 8m 27s
  Executable unittests src/main.rs (target/debug/deps/linkmapper-6393465c78bf12f8)
```