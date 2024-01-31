# pai-strace

A strace-like tool created using [pai](https://github.com/rstenvi/pai)

## Install

~~~
cargo install --force pai-strace
~~~

To check if a new version is available:
~~~
pai-strace --check-update
~~~

## Development status

In development, expect some bugs.

## Cross-compile

Cross-compilation depends on having a host linker for the given architecture.
Some linkers for some architectures have been set up in `.cargo/config.toml`.
Depending on you configuration, you might need to make some changes.

Below are some targets that can be built, and notes about building them:

**x86_64-linux-gnu**

Should work out of the box

~~~
cargo build --target=x86_64-unknown-linux-gnu
~~~

**aarch64-linux-gnu**

Need to install aarch64 cross-compiler, this can be installed from `apt` if you're on Ubuntu (`gcc-aarch64-linux-gnu`).

~~~
cargo build --target=aarch64-unknown-linux-gnu
~~~

**aarch64-linux-android**

You need to first install Android Native Development Kit (NDK). This will give you a `clang` compiler for various API levels, `.carg/config.toml` tries to use level 34.

`cargo` also looks for `ar` at `aarch64-linux-android-ar`. This is not included in NDK, but can be faked with:

~~~
ln -s /path/to/ndk/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar /dir/in/path/aarch64-linux-android-ar
~~~

Final build command:

~~~
cargo build --target=aarch64-linux-android
~~~

**x86_64-linux-android**

See notes for `aarch64-linux-android`

~~~
cargo build --target=x86_64-linux-android
~~~

## How to use

See `--help` for more commands, but below are some examples, each one simply
spawns the `true` command.

Most basic command is to just print all system calls to stdout.

~~~{.bash}
pai-strace true
~~~

Below is an example which writes both raw format and json format to files,
`calls.txt` and `calls.json`, respectively.

~~~
pai-strace --format json --format raw --output calls true
~~~

In the following previous example you might have seen, something like:

~~~
[594079]: openat(fd=0xffffff9c, file=0x7ff1d2a7121b, flags=0x80000, mode=0x0) = 0x3
~~~

This doesn't give much information about how the file is opened. To provide some more contexts, we can provide the `--enrich` argument.

~~~
pai-strace --enrich basic true
~~~

Line now becomes.

~~~
[594137]: openat(fd=fd(AT_FDCWD), file=ptr(7f363802421b), flags=flags(O_CLOEXEC), mode=flags()) = fd(3)
~~~

Now we can see that the file descriptor is a constant and the names of the flags passed.

We still don't know the filename however, to read pointers we can pass the `--enrich full` argument

~~~
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