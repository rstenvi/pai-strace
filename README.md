# pai-strace

A strace-like tool created using [pai](https://github.com/rstenvi/pai)

## Install

~~~{.bash}
cargo install --force pai-strace
~~~

To check if a new version is available:

~~~{.bash}
pai-strace --check-update
~~~

To use on target where `cargo` is not installed, please see
[releases](https://github.com/rstenvi/pai-strace/releases).

## Development status

In development, expect some bugs.

## Compile

[cargo-make](https://github.com/sagiegurari/cargo-make) is used to control the
build process. [cross](https://github.com/cross-rs/cross) is used to support
cross-compilation. To simplify the build process, `cross` is used even when
compiling for host target.

The command to build targets are:

~~~{.bash}
cargo make [build|release] [target]
~~~

The output will be placed in `output/<target>/<debug|release>/pai-strace`

If you don't specify any target it will be under `output/<debug|release>/pai-strace`

### Cross-compile

Cross-compilation is sometimes as easy as described above, like this example for
Android:

~~~{.bash}
$ cargo make release aarch64-linux-android
$ ls output/aarch64-linux-android/release/pai-strace
output/aarch64-linux-android/release/pai-strace
~~~

Not all targets are supported in `cross` in those cases, we need to find an
appropriate linker.

## How to use

See `--help` for more commands, but below are some examples, each one simply
spawns the `true` command.

Most basic command is to just print all system calls to stdout.

~~~{.bash}
pai-strace true
~~~

Below is an example which writes both raw format and json format to files,
`calls.txt` and `calls.json`, respectively.

~~~{.bash}
pai-strace --format json --format raw --output calls true
~~~

In the following previous example you might have seen, something like:

~~~
[594079]: openat(fd=0xffffff9c, file=0x7ff1d2a7121b, flags=0x80000, mode=0x0) = 0x3
~~~

This doesn't give much information about how the file is opened. To provide some more contexts, we can provide the `--enrich` argument.

~~~{.bash}
pai-strace --enrich basic true
~~~

Line now becomes.

~~~
[594137]: openat(fd=fd(AT_FDCWD), file=ptr(7f363802421b), flags=flags(O_CLOEXEC), mode=flags()) = fd(3)
~~~

Now we can see that the file descriptor is a constant and the names of the flags passed.

We still don't know the filename however, to read pointers we can pass the `--enrich full` argument

~~~{.bash}
pai-strace --enrich full true
~~~

Output now becomes:

~~~
[594176]: openat(fd=fd(AT_FDCWD), file="/etc/ld.so.cache", flags=flags(O_CLOEXEC), mode=flags()) = fd(3)
~~~

The only difference is that the filename has been resolved. The `raw` format does omit some information, the full info of the above command in `json` is included below. It includes the real values used to derive flags, strings, etc. and also includes direction of the argument. In this case, all arguments are input.

~~~{.json}
{
 "tid": 594246,
 "sysno": 257,
 "name": "openat",
 "args": [
 {
  "name": "fd",
  "value": {
  "raw_value": 4294967196,
  "parsed": {
   "FdConst": {
    "value": -100,
    "name": "AT_FDCWD"
   }
  }
 },
  "dir": "In"
 },
 {
  "name": "file",
  "value": {
   "raw_value": 139771623875099,
   "parsed": {
    "String": {
    "string": "/etc/ld.so.cache"
   }
  }
 },
  "dir": "In"
 },
 {
  "name": "flags",
  "value": {
   "raw_value": 524288,
   "parsed": {
   "Flag": {
    "set": [
     "O_CLOEXEC"
    ]
   }
  }
 },
  "dir": "In"
 },
 {
  "name": "mode",
  "value": {
  "raw_value": 0,
  "parsed": {
   "Flag": {
    "set": []
   }
  }
 },
  "dir": "In"
 }],
 "output": {
  "raw_value": 3,
  "parsed": {
   "Fd": {
    "fd": 3
   }
  }
 }
}
~~~
