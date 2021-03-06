[![Build Status](https://travis-ci.org/bootandy/dust.svg?branch=master)](https://travis-ci.org/bootandy/dust)


# Dust
du + rust = dust. A rust alternative to du

Unlike du, dust is meant to give you an instant overview of which directories are using disk space without requiring sort or head. Dust does not count file system blocks; it uses file sizes instead. Dust will print a maximum of 1 'Did not have permissions message'.


Dust will list the 15 biggest sub directories and will smartly recurse down the tree to find the larger ones. There is no need for a '-d' flag or a '-h' flag. The largest sub directory will have its size shown in red

```
Usage: dust <dir>
Usage: dust -n 30  <dir>  (Shows 30 directories not 15)
```


```
djin:git/dust> dust
  65M  .
  65M └─┬ ./target
  49M   ├─┬ ./target/debug
  26M   │ ├─┬ ./target/debug/deps
  21M   │ │ └── ./target/debug/deps/libclap-9e6625ac8ff074ad.rlib
  13M   │ ├── ./target/debug/dust
 8.9M   │ └─┬ ./target/debug/incremental
 6.7M   │   ├─┬ ./target/debug/incremental/dust-2748eiei2tcnp
 6.7M   │   │ └─┬ ./target/debug/incremental/dust-2748eiei2tcnp/s-ezd6jnik5u-163pyem-1aab9ncf5glum
 3.0M   │   │   └── ./target/debug/incremental/dust-2748eiei2tcnp/s-ezd6jnik5u-163pyem-1aab9ncf5glum/dep-graph.bin
 2.2M   │   └─┬ ./target/debug/incremental/dust-1dlon65p8m3vl
 2.2M   │     └── ./target/debug/incremental/dust-1dlon65p8m3vl/s-ezd6jncecv-1xsnfd0-4dw9l1r2th2t
  15M   └─┬ ./target/release
 9.2M     ├─┬ ./target/release/deps
 6.7M     │ └── ./target/release/deps/libclap-87bc2534ea57f044.rlib
 5.9M     └── ./target/release/dust
```
Performance: dust is currently about 4 times slower than du.
