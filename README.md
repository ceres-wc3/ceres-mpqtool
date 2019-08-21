# About

`mpqtool` is a command-line utility for reading and writing Blizzard's MPQ archive files.

It is built on top of the [`ceres-mpq`](https://crates.io/crates/ceres-mpq). Please refer to it for a detailed list of supported and unsupported MPQ features.

Roadmap:
- [x] Extracting archives to a directory on the disk, optionally filtered by a glob expression
- [x] Listing files contained within an archive, optionally filtered by a glob expression
- [x] Viewing a single file within an archive, emitting its contents to stdout (possibly useful for chaining with other command-line tools)
- [x] Creating a new archive from a directory's contents
- [ ] Preserving file headers when extracting and creating archives that are supposed to have them, e.g. WC3 maps
- [ ] Removing and adding file from/to existing archives

# Installation

NOTE: This tool does not have a GUI interface. If you need a visual MPQ editor, please refer to [MPQ Editor](http://www.zezula.net/en/mpq/download.html).

## Cargo

If you have `rustup` and `cargo` installed, simply run:
```
cargo install mpqtool
```

This is the recommended way to install the tool on Linux and Mac systems, as it will automatically add the tool to your `PATH` if you have Cargo installed via your package manager.

## Standalone

Download the latest release from the [releases section](https://github.com/ElusiveMori/ceres-mpqtool/releases). Optionally, add the tool to your `PATH` so that you can invoke it anywhere on your command line.

# Usage

`mpqtool` currently has 4 commands:

* `new` - creates a new archive
* `extract` - extracts the contents of an existing archive to a directory
* `view` - views (prints to stdout) the contents of a single file within the archive
* `list` - lists the files contained within an archive

Some commands also support specifying a `--filter`/`-f` argument to filter the output using a [glob pattern](https://en.wikipedia.org/wiki/Glob_(programming)).

To get usage help for a specific command, use `mpqtool <command> -h`. For example:

```
$ mpqtool extract --help
mpqtool-extract 0.1.0
extracts files from an archive

USAGE:
    mpqtool extract [OPTIONS] <archive>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --filter <pattern>    if specified, will only extract files which match the specified glob-pattern
    -o, --output <dir>        directory where to output extracted files [default: ./]

ARGS:
    <archive>    archive file to extract from
```

# Examples

Extracting an archive called `myarchive.mpq` to a directory `./out`:
```
$ mpqtool extract myarchive.mpq -o ./out
```

Viewing a file called `war3map.j` within an archive called `myarchive.mpq`:
```
$ mpqtool view myarchive.mpq war3map.j
```

Creating a new archive from a directory `mydir` called `myarchive.mpq`:
```
$ mpqtool new ./mydir myarhive.mpq 
```

Listing all `.mdx` files present in an archive:
```
$ mpqtool list myarchive.mpq -f "*.mdx"
```